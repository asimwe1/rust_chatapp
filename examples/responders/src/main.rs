#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

/***************************** `Stream` Responder *****************************/

use std::{io, env};

use rocket::tokio::fs::{self, File};
use rocket::tokio::io::{repeat, AsyncRead, AsyncReadExt};
use rocket::response::{content, Stream};
use rocket::data::{Capped, TempFile};

// Upload your `big_file.dat` by POSTing it to /upload.
// try `curl --data-binary @file.txt http://127.0.0.1:8000/stream/file`
const FILENAME: &str = "big_file.dat";

#[get("/stream/a")]
fn many_as() -> content::Plain<Stream<impl AsyncRead>> {
    content::Plain(Stream::from(repeat('a' as u8).take(25000)))
}

#[get("/stream/file")]
async fn file() -> Option<Stream<File>> {
    // NOTE: Rocket _always_ streams data from an `AsyncRead`, even when
    // `Stream` isn't used. By using `Stream`, however, the data is sent using
    // chunked-encoding in HTTP 1.1. DATA frames are sent in HTTP/2.
    File::open(env::temp_dir().join(FILENAME)).await.map(Stream::from).ok()
}

#[post("/stream/file", data = "<file>")]
async fn upload(mut file: Capped<TempFile<'_>>) -> io::Result<String> {
    file.persist_to(env::temp_dir().join(FILENAME)).await?;
    Ok(format!("{} bytes at {}", file.n.written, file.path().unwrap().display()))
}

#[delete("/stream/file")]
async fn delete() -> Option<()> {
    fs::remove_file(env::temp_dir().join(FILENAME)).await.ok()
}

/***************************** `Redirect` Responder ***************************/

use rocket::response::Redirect;

#[get("/redir")]
fn redir_root() -> Redirect {
    Redirect::to(uri!(redir_login))
}

#[get("/redir/login")]
fn redir_login() -> &'static str {
    "Hi! Please log in before continuing."
}

#[get("/redir/<name>")]
fn maybe_redir(name: &str) -> Result<&'static str, Redirect> {
    match name {
        "Sergio" => Ok("Hello, Sergio!"),
        _ => Err(Redirect::to(uri!(redir_login))),
    }
}

/***************************** `content` Responders ***************************/

use rocket::Request;

// NOTE: This example explicitly uses the `Json` type from `response::content`
// for demonstration purposes. In a real application, _always_ prefer to use
// `rocket_contrib::json::Json` instead!

// In a `GET` request and all other non-payload supporting request types, the
// preferred media type in the Accept header is matched against the `format` in
// the route attribute. Because the client can use non-specific media types like
// `*/*` in `Accept`, these first two routes would collide without `rank`.
#[get("/content", format = "xml", rank = 1)]
fn xml() -> content::Xml<&'static str> {
    content::Xml("<payload>I'm here</payload>")
}

#[get("/content", format = "json", rank = 2)]
fn json() -> content::Json<&'static str> {
    content::Json(r#"{ "payload": "I'm here" }"#)
}

#[catch(404)]
fn not_found(request: &Request<'_>) -> content::Html<String> {
    let html = match request.format() {
        Some(ref mt) if !(mt.is_xml() || mt.is_html()) => {
            format!("<p>'{}' requests are not supported.</p>", mt)
        }
        _ => format!("<p>Sorry, '{}' is an invalid path! Try \
                 /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
                 request.uri())
    };

    content::Html(html)
}

/******************************* `Either` Responder ***************************/

use rocket::Either;
use rocket::response::content::{Json, MsgPack};
use rocket::http::uncased::AsUncased;

// NOTE: In a real application, we'd use `Json` and `MsgPack` from
// `rocket_contrib`, which perform automatic serialization of responses and
// automatically set the `Content-Type`.
#[get("/content/<kind>")]
fn json_or_msgpack(kind: &str) -> Either<Json<&'static str>, MsgPack<&'static [u8]>> {
    if kind.as_uncased() == "msgpack" {
        Either::Right(MsgPack(&[162, 104, 105]))
    } else {
        Either::Left(Json("\"hi\""))
    }
}

/******************************* Custom Responder *****************************/

use std::borrow::Cow;

use rocket::response::NamedFile;
use rocket::response::content::Html;

#[derive(Responder)]
enum StoredData {
    File(Option<NamedFile>),
    String(Cow<'static, str>),
    Bytes(Vec<u8>),
    #[response(status = 401)]
    NotAuthorized(Html<&'static str>),
}

#[derive(FromFormField, UriDisplayQuery)]
enum Kind {
    File,
    String,
    Bytes
}

#[get("/custom?<kind>")]
async fn custom(kind: Option<Kind>) -> StoredData {
    match kind {
        Some(Kind::File) => {
            let path = env::temp_dir().join(FILENAME);
            StoredData::File(NamedFile::open(path).await.ok())
        },
        Some(Kind::String) => StoredData::String("Hey, I'm some data.".into()),
        Some(Kind::Bytes) => StoredData::Bytes(vec![72, 105]),
        None => StoredData::NotAuthorized(Html("No no no!"))
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![many_as, file, upload, delete])
        .mount("/", routes![redir_root, redir_login, maybe_redir])
        .mount("/", routes![xml, json, json_or_msgpack])
        .mount("/", routes![custom])
        .register("/", catchers![not_found])
}
