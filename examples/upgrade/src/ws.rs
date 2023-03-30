use std::io;

use rocket::{Request, response};
use rocket::data::{IoHandler, IoStream};
use rocket::request::{FromRequest, Outcome};
use rocket::response::{Responder, Response};
use rocket::futures::future::BoxFuture;

use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::tungstenite::error::{Result, Error};

pub struct WebSocket(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebSocket {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use rocket::http::uncased::eq;

        let headers = req.headers();
        let is_upgrade = headers.get_one("Connection").map_or(false, |c| eq(c, "upgrade"));
        let is_ws = headers.get("Upgrade").any(|p| eq(p, "websocket"));
        let is_ws_13 = headers.get_one("Sec-WebSocket-Version").map_or(false, |v| v == "13");
        let key = headers.get_one("Sec-WebSocket-Key").map(|k| derive_accept_key(k.as_bytes()));
        match key {
            Some(key) if is_upgrade && is_ws && is_ws_13 => Outcome::Success(WebSocket(key)),
            Some(_) | None => Outcome::Forward(())
        }
    }
}

pub struct Channel {
    ws: WebSocket,
    handler: Box<dyn FnMut(WebSocketStream<IoStream>) -> BoxFuture<'static, Result<()>> + Send>,
}

impl WebSocket {
    pub fn channel<F: Send + 'static>(self, handler: F) -> Channel
        where F: FnMut(WebSocketStream<IoStream>) -> BoxFuture<'static, Result<()>>
    {
        Channel { ws: self, handler: Box::new(handler), }
    }
}

impl<'r> Responder<'r, 'static> for Channel {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .raw_header("Sec-Websocket-Version", "13")
            .raw_header("Sec-WebSocket-Accept", self.ws.0.clone())
            .upgrade("websocket", self)
            .ok()
    }
}

#[rocket::async_trait]
impl IoHandler for Channel {
    async fn io(&mut self, io: IoStream) -> io::Result<()> {
        let stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        (self.handler)(stream).await.map_err(|e| match e {
            Error::Io(e) => e,
            other => io::Error::new(io::ErrorKind::Other, other)
        })
    }
}
