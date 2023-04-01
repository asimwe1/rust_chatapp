#[macro_use] extern crate rocket;

use rocket::fs::{self, FileServer};
use rocket::futures::{SinkExt, StreamExt};

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
fn echo_stream(ws: ws::WebSocket) -> ws::Stream!['static] {
    let ws = ws.config(ws::Config { max_send_queue: Some(5), ..Default::default() });
    ws::Stream! { ws =>
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
