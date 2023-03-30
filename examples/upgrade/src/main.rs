#[macro_use] extern crate rocket;

use rocket::futures::{SinkExt, StreamExt};
use rocket::response::content::RawHtml;

mod ws;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(include_str!("../index.html"))
}

#[get("/echo")]
fn echo(ws: ws::WebSocket) -> ws::Channel {
    ws.channel(|mut stream| Box::pin(async move {
        while let Some(message) = stream.next().await {
            let _ = stream.send(message?).await;
        }

        Ok(())
    }))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, echo])
}
