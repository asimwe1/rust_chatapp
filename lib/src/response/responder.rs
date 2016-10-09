use std::io::{Read, Write};
use std::fs::File;
use std::fmt;

use response::{Outcome, ResponseOutcome};
use http::mime::{Mime, TopLevel, SubLevel};
use http::hyper::{header, FreshHyperResponse, StatusCode};

// TODO: Have this return something saying whether every was okay. Need
// something like to be able to forward requests on when things don't work out.
// In particular, we want to try the next ranked route when when parsing
// parameters doesn't work out.
pub trait Responder {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> ResponseOutcome<'a>;
}

impl<'a> Responder for &'a str {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> ResponseOutcome<'b> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Plain, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        Outcome::of(res.send(self.as_bytes()))
    }
}

impl Responder for String {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Html, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        Outcome::of(res.send(self.as_bytes()))
    }
}

impl Responder for File {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        let size = match self.metadata() {
            Ok(md) => md.len(),
            Err(e) => {
                error_!("Failed to read file metadata: {:?}", e);
                return Outcome::Forward((StatusCode::InternalServerError, res));
            }
        };

        let mut v = Vec::new();
        if let Err(e) = self.read_to_end(&mut v) {
            error_!("Failed to read file: {:?}", e);
            return Outcome::Forward((StatusCode::InternalServerError, res));
        }

        res.headers_mut().set(header::ContentLength(size));
        Outcome::of(res.start().and_then(|mut stream| stream.write_all(&v)))
    }
}

impl<T: Responder> Responder for Option<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        if let Some(ref mut val) = *self {
            val.respond(res)
        } else {
            warn_!("Response was `None`.");
            Outcome::Forward((StatusCode::NotFound, res))
        }
    }
}

impl<T: Responder, E: fmt::Debug> Responder for Result<T, E> {
    // prepend with `default` when using impl specialization
    default fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        match *self {
            Ok(ref mut val) => val.respond(res),
            Err(ref e) => {
                error_!("{:?}", e);
                Outcome::Forward((StatusCode::InternalServerError, res))
            }
        }
    }
}

impl<T: Responder, E: Responder + fmt::Debug> Responder for Result<T, E> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        match *self {
            Ok(ref mut responder) => responder.respond(res),
            Err(ref mut responder) => responder.respond(res),
        }
    }
}
