use response::*;
use response::mime::{Mime, TopLevel, SubLevel};
use std::io::{Read, Write};
use std::fs::File;
use std::fmt;

// TODO: Have this return something saying whether every was okay. Need
// something like to be able to forward requests on when things don't work out.
// In particular, we want to try the next ranked route when when parsing
// parameters doesn't work out.
pub trait Responder {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a>;
}

impl<'a> Responder for &'a str {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Plain, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        res.send(self.as_bytes()).unwrap();
        Outcome::Complete
    }
}

impl Responder for String {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Html, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }
        res.send(self.as_bytes()).unwrap();
        Outcome::Complete
    }
}

// FIXME: Should we set a content-type here? Safari needs text/html to render.
// Unfortunately, the file name is gone at this point. Should fix this. There's
// a way to retrieve a file based on its fd, strangely enough. See...
// https://stackoverflow.com/questions/1188757/getting-filename-from-file-descriptor-in-c
impl Responder for File {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        let size = self.metadata().unwrap().len();

        res.headers_mut().set(header::ContentLength(size));
        *(res.status_mut()) = StatusCode::Ok;

        let mut v = Vec::new();
        self.read_to_end(&mut v).unwrap();

        let mut stream = res.start().unwrap();
        stream.write_all(&v).unwrap();
        Outcome::Complete
    }
}

impl<T: Responder> Responder for Option<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if self.is_none() {
            trace!("Option is none.");
            // TODO: Should this be a 404 or 500?
            return Outcome::FailForward(res);
        }

        self.as_mut().unwrap().respond(res)
    }
}

impl<T: Responder, E: fmt::Debug> Responder for Result<T, E> {
    // prepend with `default` when using impl specialization
    default fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if self.is_err() {
            error_!("{:?}", self.as_ref().err().unwrap());
            // TODO: Should this be a 404 or 500?
            return Outcome::FailForward(res);
        }

        self.as_mut().unwrap().respond(res)
    }
}

impl<T: Responder, E: Responder + fmt::Debug> Responder for Result<T, E> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        match *self {
            Ok(ref mut responder) => responder.respond(res),
            Err(ref mut responder) => responder.respond(res),
        }
    }
}
