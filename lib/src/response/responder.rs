use response::*;
use std::io::{Read, Write};
use std::fs::File;
use std::fmt;

// TODO: Have this return something saying whether every was okay. Need
// something like to be able to forward requests on when things don't work out.
// In particular, we want to try the next ranked route when when parsing
// parameters doesn't work out.
pub trait Responder {
    fn respond<'a>(&mut self, mut res: HyperResponse<'a, HyperFresh>);
}

impl<'a> Responder for &'a str {
    fn respond<'b>(&mut self, res: HyperResponse<'b, HyperFresh>) {
        res.send(self.as_bytes()).unwrap();
    }
}

impl Responder for String {
    fn respond<'b>(&mut self, res: HyperResponse<'b, HyperFresh>) {
        res.send(self.as_bytes()).unwrap();
    }
}

// FIXME: Should we set a content-type here? Safari needs text/html to render.
impl Responder for File {
    fn respond<'b>(&mut self, mut res: HyperResponse<'b, HyperFresh>) {
        let size = self.metadata().unwrap().len();

        res.headers_mut().set(header::ContentLength(size));
        *(res.status_mut()) = StatusCode::Ok;

        let mut v = Vec::new();
        self.read_to_end(&mut v).unwrap();

        let mut stream = res.start().unwrap();
        stream.write_all(&v).unwrap();
    }
}

impl<T: Responder> Responder for Option<T> {
    fn respond<'b>(&mut self, res: HyperResponse<'b, HyperFresh>) {
        if self.is_none() {
            println!("Option is none.");
            // TODO: Should this be a 404 or 500?
            return Empty::new(StatusCode::NotFound).respond(res)
        }

        self.as_mut().unwrap().respond(res);
    }
}

impl<T: Responder, E: fmt::Debug> Responder for Result<T, E> {
    // prepend with `default` when using impl specialization
    default fn respond<'b>(&mut self, res: HyperResponse<'b, HyperFresh>) {
        if self.is_err() {
            println!("Error: {:?}", self.as_ref().err().unwrap());
            // TODO: Should this be a 404 or 500?
            return Empty::new(StatusCode::NotFound).respond(res)
        }

        self.as_mut().unwrap().respond(res);
    }
}

impl<T: Responder, E: Responder + fmt::Debug> Responder for Result<T, E> {
    fn respond<'b>(&mut self, res: HyperResponse<'b, HyperFresh>) {
        match self {
            &mut Ok(ref mut responder) => responder.respond(res),
            &mut Err(ref mut responder) => responder.respond(res)
        }
    }
}
