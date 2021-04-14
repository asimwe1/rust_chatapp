use rocket_contrib::msgpack::MsgPack;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Message<'r> {
    id: usize,
    contents: &'r str
}

#[get("/<id>", format = "msgpack")]
fn get(id: usize) -> MsgPack<Message<'static>> {
    MsgPack(Message { id, contents: "Hello, world!", })
}

#[post("/", data = "<data>", format = "msgpack")]
fn echo<'r>(data: MsgPack<Message<'r>>) -> &'r str {
    data.contents
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("MessagePack", |rocket| async {
        rocket.mount("/msgpack", routes![echo, get])
    })
}
