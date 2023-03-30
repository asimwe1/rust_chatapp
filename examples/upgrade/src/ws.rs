use std::io;

use rocket::futures::{StreamExt, SinkExt};
use rocket::futures::stream::SplitStream;
use rocket::{Request, response};
use rocket::data::{IoHandler, IoStream};
use rocket::request::{FromRequest, Outcome};
use rocket::response::{Responder, Response};
use rocket::futures::{self, future::BoxFuture};

use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::Role;

pub use tokio_tungstenite::tungstenite::error::{Result, Error};
pub use tokio_tungstenite::tungstenite::Message;

pub struct WebSocket(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebSocket {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use rocket::http::uncased::eq;

        let headers = req.headers();
        let is_upgrade = headers.get("Connection")
            .any(|h| h.split(',').any(|v| eq(v.trim(), "upgrade")));

        let is_ws = headers.get("Upgrade")
            .any(|h| h.split(',').any(|v| eq(v.trim(), "websocket")));

        let is_ws_13 = headers.get_one("Sec-WebSocket-Version").map_or(false, |v| v == "13");
        let key = headers.get_one("Sec-WebSocket-Key").map(|k| derive_accept_key(k.as_bytes()));
        match key {
            Some(key) if is_upgrade && is_ws && is_ws_13 => Outcome::Success(WebSocket(key)),
            Some(_) | None => Outcome::Forward(())
        }
    }
}

pub struct Channel<'r> {
    ws: WebSocket,
    handler: Box<dyn FnMut(WebSocketStream<IoStream>) -> BoxFuture<'r, Result<()>> + Send + 'r>,
}

pub struct MessageStream<'r, S> {
    ws: WebSocket,
    handler: Box<dyn FnMut(SplitStream<WebSocketStream<IoStream>>) -> S + Send + 'r>
}

impl WebSocket {
    pub fn channel<'r, F: Send + 'r>(self, handler: F) -> Channel<'r>
        where F: FnMut(WebSocketStream<IoStream>) -> BoxFuture<'r, Result<()>> + 'r
    {
        Channel { ws: self, handler: Box::new(handler), }
    }

    pub fn stream<'r, F, S>(self, stream: F) -> MessageStream<'r, S>
        where F: FnMut(SplitStream<WebSocketStream<IoStream>>) -> S + Send + 'r,
              S: futures::Stream<Item = Result<Message>> + Send + 'r
    {
        MessageStream { ws: self, handler: Box::new(stream), }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Channel<'o> {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        Response::build()
            .raw_header("Sec-Websocket-Version", "13")
            .raw_header("Sec-WebSocket-Accept", self.ws.0.clone())
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
            .raw_header("Sec-WebSocket-Accept", self.ws.0.clone())
            .upgrade("websocket", self)
            .ok()
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

#[rocket::async_trait]
impl IoHandler for Channel<'_> {
    async fn io(&mut self, io: IoStream) -> io::Result<()> {
        let stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        let result = (self.handler)(stream).await;
        handle_result(result).map(|_| ())
    }
}

#[rocket::async_trait]
impl<'r, S> IoHandler for MessageStream<'r, S>
    where S: futures::Stream<Item = Result<Message>> + Send + 'r
{
    async fn io(&mut self, io: IoStream) -> io::Result<()> {
        let stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        let (mut sink, stream) = stream.split();
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

#[macro_export]
macro_rules! Stream {
    ($l:lifetime) => (
        $crate::ws::MessageStream<$l, impl rocket::futures::Stream<
            Item = $crate::ws::Result<$crate::ws::Message>
        > + $l>
    )
}

#[macro_export]
macro_rules! stream {
    ($channel:ident => $($token:tt)*) => (
        let ws: $crate::ws::WebSocket = $channel;
        ws.stream(move |$channel| rocket::async_stream::try_stream! {
            $($token)*
        })
    )
}

pub use Stream as Stream;
pub use stream as stream;
