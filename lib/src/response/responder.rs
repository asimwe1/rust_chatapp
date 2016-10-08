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

        res.send(self.as_bytes()).unwrap();
        Outcome::Success
    }
}

impl Responder for String {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Html, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }
        res.send(self.as_bytes()).unwrap();
        Outcome::Success
    }
}

impl Responder for File {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        let size = self.metadata().unwrap().len();

        res.headers_mut().set(header::ContentLength(size));
        *(res.status_mut()) = StatusCode::Ok;

        let mut v = Vec::new();
        self.read_to_end(&mut v).unwrap();

        let mut stream = res.start().unwrap();
        stream.write_all(&v).unwrap();
        Outcome::Success
    }
}

impl<T: Responder> Responder for Option<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        if self.is_none() {
            warn_!("response was `None`");
            return Outcome::Forward((StatusCode::NotFound, res));
        }

        self.as_mut().unwrap().respond(res)
    }
}

impl<T: Responder, E: fmt::Debug> Responder for Result<T, E> {
    // prepend with `default` when using impl specialization
    default fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        if self.is_err() {
            error_!("{:?}", self.as_ref().err().unwrap());
            return Outcome::Forward((StatusCode::InternalServerError, res));
        }

        self.as_mut().unwrap().respond(res)
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
