use std::io;

use rocket::data::{IoHandler, IoStream};
use rocket::futures::{self, StreamExt, SinkExt, future::BoxFuture, stream::SplitStream};
use rocket::response::{self, Responder, Response};
use rocket::request::{FromRequest, Outcome};
use rocket::request::Request;

use crate::{Config, Message};
use crate::stream::DuplexStream;
use crate::result::{Result, Error};

/// A request guard that identifies WebSocket requests. Converts into a
/// [`Channel`] or [`MessageStream`].
pub struct WebSocket {
    config: Config,
    key: String,
}

impl WebSocket {
    fn new(key: String) -> WebSocket {
        WebSocket { config: Config::default(), key }
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn channel<'r, F: Send + 'r>(self, handler: F) -> Channel<'r>
        where F: FnMut(DuplexStream) -> BoxFuture<'r, Result<()>> + 'r
    {
        Channel { ws: self, handler: Box::new(handler), }
    }

    pub fn stream<'r, F, S>(self, stream: F) -> MessageStream<'r, S>
        where F: FnMut(SplitStream<DuplexStream>) -> S + Send + 'r,
              S: futures::Stream<Item = Result<Message>> + Send + 'r
    {
        MessageStream { ws: self, handler: Box::new(stream), }
    }
}

/// A streaming channel, returned by [`WebSocket::channel()`].
pub struct Channel<'r> {
    ws: WebSocket,
    handler: Box<dyn FnMut(DuplexStream) -> BoxFuture<'r, Result<()>> + Send + 'r>,
}

/// A [`Stream`](futures::Stream) of [`Message`]s, returned by
/// [`WebSocket::stream()`], used via [`Stream!`].
///
/// This type is not typically used directly. Instead, it is used via the
/// [`Stream!`] macro, which expands to both the type itself and an expression
/// which evaluates to this type.
// TODO: Get rid of this or `Channel` via a single `enum`.
pub struct MessageStream<'r, S> {
    ws: WebSocket,
    handler: Box<dyn FnMut(SplitStream<DuplexStream>) -> S + Send + 'r>
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebSocket {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use crate::tungstenite::handshake::derive_accept_key;
        use rocket::http::uncased::eq;

        let headers = req.headers();
        let is_upgrade = headers.get("Connection")
            .any(|h| h.split(',').any(|v| eq(v.trim(), "upgrade")));

        let is_ws = headers.get("Upgrade")
            .any(|h| h.split(',').any(|v| eq(v.trim(), "websocket")));

        let is_13 = headers.get_one("Sec-WebSocket-Version").map_or(false, |v| v == "13");
        let key = headers.get_one("Sec-WebSocket-Key").map(|k| derive_accept_key(k.as_bytes()));
        match key {
            Some(key) if is_upgrade && is_ws && is_13 => Outcome::Success(WebSocket::new(key)),
            Some(_) | None => Outcome::Forward(())
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Channel<'o> {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        Response::build()
            .raw_header("Sec-Websocket-Version", "13")
            .raw_header("Sec-WebSocket-Accept", self.ws.key.clone())
            .upgrade("websocket", self)
            .ok()
    }
}

impl<'r, 'o: 'r, S> Responder<'r, 'o> for MessageStream<'o, S>
    where S: futures::Stream<Item = Result<Message>> + Send + 'o
{
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        Response::build()
            .raw_header("Sec-Websocket-Version", "13")
            .raw_header("Sec-WebSocket-Accept", self.ws.key.clone())
            .upgrade("websocket", self)
            .ok()
    }
}

#[rocket::async_trait]
impl IoHandler for Channel<'_> {
    async fn io(&mut self, io: IoStream) -> io::Result<()> {
        let result = (self.handler)(DuplexStream::new(io, self.ws.config).await).await;
        handle_result(result).map(|_| ())
    }
}

#[rocket::async_trait]
impl<'r, S> IoHandler for MessageStream<'r, S>
    where S: futures::Stream<Item = Result<Message>> + Send + 'r
{
    async fn io(&mut self, io: IoStream) -> io::Result<()> {
        let (mut sink, stream) = DuplexStream::new(io, self.ws.config).await.split();
        let mut stream = std::pin::pin!((self.handler)(stream));
        while let Some(msg) = stream.next().await {
            let result = match msg {
                Ok(msg) => sink.send(msg).await,
                Err(e) => Err(e)
            };

            if !handle_result(result)? {
                return Ok(());
            }
        }

        Ok(())
    }
}

/// Returns `Ok(true)` if processing should continue, `Ok(false)` if processing
/// has terminated without error, and `Err(e)` if an error has occurred.
fn handle_result(result: Result<()>) -> io::Result<bool> {
    match result {
        Ok(_) => Ok(true),
        Err(Error::ConnectionClosed) => Ok(false),
        Err(Error::Io(e)) => Err(e),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e))
    }
}
