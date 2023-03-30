#[macro_use] extern crate rocket;

use rocket::fs::{self, FileServer};
use rocket::futures::{SinkExt, StreamExt};

mod ws;

#[get("/echo/manual")]
fn echo_manual<'r>(ws: ws::WebSocket) -> ws::Channel<'r> {
    ws.channel(move |mut stream| Box::pin(async move {
        while let Some(message) = stream.next().await {
            let _ = stream.send(message?).await;
        }

        Ok(())
    }))
}

#[get("/echo")]
fn echo_stream<'r>(ws: ws::WebSocket) -> ws::Stream!['r] {
    ws::stream! { ws =>
        for await message in ws {
            yield message?;
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![echo_manual, echo_stream])
        .mount("/", FileServer::from(fs::relative!("static")))
}
