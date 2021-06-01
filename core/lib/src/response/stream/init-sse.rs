//! A Responder implementing a [Server-sent events] (SSE) stream.
//!
//! This module is intended for eventual inclusion in rocket_contrib.
//!
//! [Server-sent events]: https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events

use std::borrow::Cow;
use std::time::Duration;

use rocket::request::Request;
use rocket::response::{Responder, Response};

use rocket::futures::stream::Stream;

// Based on https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream
// (Reproduced here for quick reference. retrieved 2021-04-17)
//
//     stream        = [ bom ] *event
//     event         = *( comment / field ) end-of-line
//     comment       = colon *any-char end-of-line
//     field         = 1*name-char [ colon [ space ] *any-char ] end-of-line
//     end-of-line   = ( cr lf / cr / lf )
//
//     ; characters
//     lf            = %x000A ; U+000A LINE FEED (LF)
//     cr            = %x000D ; U+000D CARRIAGE RETURN (CR)
//     space         = %x0020 ; U+0020 SPACE
//     colon         = %x003A ; U+003A COLON (:)
//     bom           = %xFEFF ; U+FEFF BYTE ORDER MARK
//     name-char     = %x0000-0009 / %x000B-000C / %x000E-0039 / %x003B-10FFFF
//                     ; a scalar value other than U+000A LINE FEED (LF), U+000D CARRIAGE RETURN (CR), or U+003A COLON (:)
//     any-char      = %x0000-0009 / %x000B-000C / %x000E-10FFFF
//                     ; a scalar value other than U+000A LINE FEED (LF) or U+000D CARRIAGE RETURN (CR)/
//
// Notice that Multiple encodings are possible for the same data, especially in
// the choice of newline. This implementation always uses only "\n" (LF).

/// Low-level serialization of fields in text/event-stream format.
///
/// Corresponds to 'comment / field' above; there is no dedicated name for this concept.
///
/// Always use the public constructors [`FieldKind::comment`] and
/// [`FieldKind::field`], which validate inputs. Misuse of the public enum
/// variants cannot lead to memory unsafety, but it can break the event stream.
enum FieldKind<'a> {
    /// Serializes as ":{}\n". May contain any characters except CR or LF
    Comment(&'a str),

    /// Serializes as "{}\n" or "{}: {}\n".
    ///
    /// The name may contain any characters except CR or LF or ':' (colon).
    /// The value, if present, may contain any characters except CR or LF.
    Field(&'a str, Option<&'a str>),
}

impl<'a> FieldKind<'a> {
    /// Returns true if 'name' is a valid name for an SSE field.
    /// All characters are valid except for ':' (colon), CR, and LF.
    pub fn is_valid_name(name: &str) -> bool {
        !name.bytes().any(|b| b == b'\n' || b == b'\r' || b == b':')
    }

    /// Returns true if 'value' is a valid value for an SSE field.
    /// All characters are valid except for CR, and LF.
    pub fn is_valid_value(value: &str) -> bool {
        !value.bytes().any(|b| b == b'\n' || b == b'\r')
    }

    /// Creates a comment field.
    pub fn comment(comment: &'a str) -> Result<Self, ()> {
        let comment = comment.into();
        if Self::is_valid_value(comment) {
            Ok(Self::Comment(comment))
        } else {
            Err(())
        }
    }

    /// Creates a key/value field.
    pub fn field(name: &'a str, value: Option<&'a str>) -> Result<Self, ()> {
        let name = name.into();
        let value = value.map(|v| v.into());

        if Self::is_valid_name(name) && value.map_or(true, Self::is_valid_value) {
            Ok(Self::Field(name, value))
        } else {
            Err(())
        }
    }

    /// Serializes 'self' into 'out' in the text/event-stream format, including
    /// a trailing newline.
    pub fn serialize(&self, out: &mut Vec<u8>) {
        match self {
            FieldKind::Comment(comment) => {
                out.push(b':');
                out.extend_from_slice(comment.as_bytes());
            }
            FieldKind::Field(name, None) => {
                out.extend_from_slice(name.as_bytes());
            }
            FieldKind::Field(name, Some(value)) => {
                out.extend_from_slice(name.as_bytes());
                out.extend_from_slice(b": ");
                out.extend_from_slice(value.as_bytes());
            }
        }
        out.push(b'\n');
    }

    #[cfg(test)]
    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = vec![];
        self.serialize(&mut vec);
        vec
    }
}

#[cfg(test)]
mod field_tests {
    use super::FieldKind;

    #[test]
    pub fn test_field_serialization() {
        assert_eq!(
            FieldKind::comment("test comment").unwrap().to_vec(),
            b":test comment\n"
        );

        assert_eq!(
            FieldKind::field("magic", None).unwrap().to_vec(),
            b"magic\n"
        );

        assert_eq!(
            FieldKind::field("hello", Some("w:o:r:l:d"))
                .unwrap()
                .to_vec(),
            b"hello: w:o:r:l:d\n"
        );
    }

    #[test]
    pub fn test_disallowed_field_values() {
        assert!(FieldKind::comment("newlines\nbad").is_err());
        assert!(FieldKind::field("newlines\nbad", None).is_err());
        assert!(FieldKind::field("no:colon", None).is_err());
        assert!(FieldKind::field("x", Some("newlines\nbad")).is_err());
    }
}

/// A single event in an SSE stream, with optional `event`, `data`, `id`, and
/// `retry` fields.
///
/// Events are created with [`Event::new()`] or [`Event::message()`]. They can be
/// [`serialize`](Event::serialize())d or wrapped into an [`EventSource`].
///
/// See [Using server-sent events] for more information on the meaning of the
/// fields and how they are interpreted by user agents.
///
/// [Using server-sent events]:
/// https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#fields
#[derive(Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct Event {
    event: Option<String>,
    data: Option<String>,
    id: Option<String>,
    retry: Option<u32>,
}

impl Event {
    /// Creates a new empty `Event`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Event` with only a data field.
    ///
    /// Since the 'event' (event type) is left unspecified, the client will use
    /// the default event type of `message`.
    pub fn message<S: Into<String>>(data: S) -> Self {
        Self::new().with_data(data)
    }

    /// Sets the value of the 'event' (event type) field. It may not contain newlines.
    pub fn with_event<T: Into<String>>(mut self, event: T) -> Result<Self, ()> {
        let event = event.into();
        if !FieldKind::is_valid_value(&event) {
            return Err(());
        }
        self.event = Some(event);
        Ok(self)
    }

    /// Sets the value of the 'id' field. It may not contain newlines.
    pub fn with_id<T: Into<String>>(mut self, id: T) -> Result<Self, ()> {
        let id = id.into();
        if !FieldKind::is_valid_value(&id) {
            return Err(());
        }
        self.id = Some(id);
        Ok(self)
    }

    /// Sets the value of the 'data' field. It may contain newlines.
    pub fn with_data<T: Into<String>>(mut self, data: T) -> Self {
        let data = data.into();
        // No need to validate this: only newlines might be invalid, and for
        // 'data' they are handled separately during serialization
        self.data = Some(data);
        self
    }

    /// Sets the value of the 'retry' field, in milliseconds.
    pub fn with_retry(mut self, retry: u32) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Returns the 'event' (event type) for this `Event`
    pub fn event(&self) -> Option<&str> {
        self.event.as_deref()
    }

    /// Returns the 'data' for this `Event`
    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }

    /// Returns the 'id' for this `Event`
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns the retry time for this `Event`
    pub fn retry(&self) -> Option<u32> {
        self.retry
    }

    fn try_serialize(self) -> Result<Vec<u8>, ()> {
        let mut out = vec![];

        if let Some(event) = self.event {
            FieldKind::field("event", Some(&event))?.serialize(&mut out);
        }
        if let Some(id) = self.id {
            FieldKind::field("id", Some(&id))?.serialize(&mut out);
        }
        if let Some(data) = self.data {
            // "data" is treated specially: it can contain newlines, which are
            // encoded in multiple "data" fields
            for line in data.lines() {
                FieldKind::field("data", Some(&line))?.serialize(&mut out);
            }
        }

        if let Some(retry) = self.retry {
            FieldKind::field("retry", Some(&retry.to_string()))?.serialize(&mut out);
        }

        // extra blank line indicates "end of Event"
        out.push(b'\n');

        Ok(out)
    }

    /// Serializes `self` into a byte buffer.
    pub fn serialize(self) -> Vec<u8> {
        self.try_serialize()
            .expect("internal invariant broken: field contents should have already been validated")
    }
}

const EMPTY_COMMENT_EVENT: &[u8] = b":\n\n";

#[cfg(test)]
mod event_tests {
    use super::Event;

    #[test]
    pub fn test_event_serialization() {
        assert_eq!(
            Event::new()
                .with_event("test")
                .unwrap()
                .with_data("line 1\nline 2")
                .serialize(),
            b"event: test\ndata: line 1\ndata: line 2\n\n"
        );

        assert_eq!(
            Event::new().with_event("nodata").unwrap().serialize(),
            b"event: nodata\n\n"
        );

        assert_eq!(
            Event::new()
                .with_id("event1")
                .unwrap()
                .with_retry(5)
                .serialize(),
            b"id: event1\nretry: 5\n\n"
        );

        assert_eq!(Event::new().serialize(), b"\n");
    }

    #[test]
    pub fn test_disallowed_event_values() {
        assert!(Event::new().with_event("a\rb").is_err());
        assert!(Event::new().with_event("a\nb").is_err());
        assert!(Event::new().with_id("1\r2").is_err());
        assert!(Event::new().with_id("1\n2").is_err());
    }
}

/// A `Responder` representing an SSE stream.
///
/// See the [`EventSource::new()`] function for a usage example.
///
/// The `Last-Event-ID` header is not handled by this API; if you wish to
/// support the feature, send events with some kind of `id` and use the
/// value in the header (if provided) to decide which event to resume from.
pub struct EventSource<S> {
    stream: S,
    interval: Option<Duration>,
}

impl<S: Stream<Item = Event>> EventSource<S> {
    /// Creates an `EventSource` from a [`Stream`] of [`Event`]s.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::get;
    /// #
    /// use rocket_rooms::sse::{self, Event, EventSource};
    /// use rocket::futures::stream::Stream;
    /// use rocket::response::stream::stream;
    ///
    /// #[get("/events")]
    /// fn events() -> EventSource<impl Stream<Item = Event>> {
    ///     EventSource::new(stream! {
    ///         let mut i = 0;
    ///         while i <= 3 {
    ///             i += 1;
    ///             yield Event::message(format!("data {}", i));
    ///         }
    ///     })
    /// }
    /// ```
    pub fn new(stream: S) -> Self {
        EventSource {
            stream,
            interval: Some(Duration::from_secs(30)),
        }
    }

    /// Sets a "ping" interval for this `EventSource` to avoid connection
    /// timeouts when no data is being transferred. The default `interval` for a
    /// newly created `EventSource` is `None`, which disables this
    /// functionality.
    ///
    /// The ping is implemented by sending an empty comment to the client every
    /// `interval` seconds.
    ///
    /// # Example
    /// ```rust
    /// # use rocket::get;
    /// #
    /// use std::time::Duration;
    ///
    /// use rocket_rooms::sse::{self, Event, EventSource};
    /// use rocket::futures::stream::Stream;
    ///
    /// #[get("/events")]
    /// fn events() -> EventSource<impl Stream<Item = Event>> {
    ///     # let event_stream = rocket::futures::stream::pending();
    ///     // let event_stream = ...
    ///
    ///     // Set the ping interval to 15 seconds
    ///     EventSource::new(event_stream).with_ping_interval(Some(Duration::from_secs(15)))
    /// }
    /// ```
    pub fn with_ping_interval(mut self, interval: Option<Duration>) -> Self {
        self.interval = interval;
        self
    }
}

impl<'r, S: Stream<Item = Event> + Send + 'r> Responder<'r, 'r> for EventSource<S> {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'r> {
        use rocket::response::stream::ByteStream;
        use rocket::tokio::time::interval;
        use tokio_stream::{wrappers::IntervalStream, StreamExt};

        let serialized_events = self.stream.map(|e| Cow::Owned(e.serialize()));

        let response = if let Some(duration) = self.interval {
            let pings =
                IntervalStream::new(interval(duration)).map(|_| Cow::Borrowed(EMPTY_COMMENT_EVENT));
            ByteStream::from(pings.merge(serialized_events)).respond_to(req)?
        } else {
            ByteStream::from(serialized_events).respond_to(req)?
        };

        Response::build_from(response)
            .raw_header("Content-Type", "text/event-stream")
            .raw_header("Cache-Control", "no-cache")
            .raw_header("Expires", "0")
            .ok()
    }
}
