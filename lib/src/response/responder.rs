use std::io::{Read, Write};
use std::fs::File;
use std::fmt;

use http::mime::{Mime, TopLevel, SubLevel};
use http::hyper::{header, FreshHyperResponse, StatusCode};
use outcome::{self, IntoOutcome};
use outcome::Outcome::*;

/// Type alias for the `Outcome` of a `Responder`.
pub type Outcome<'a> = outcome::Outcome<(), (), (StatusCode, FreshHyperResponse<'a>)>;

impl<'a, T, E> IntoOutcome<(), (), (StatusCode, FreshHyperResponse<'a>)> for Result<T, E> {
    fn into_outcome(self) -> Outcome<'a> {
        match self {
            Ok(_) => Success(()),
            Err(_) => Failure(())
        }
    }
}

pub trait Responder {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a>;
}

impl<'a> Responder for &'a str {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Plain, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        res.send(self.as_bytes()).into_outcome()
    }
}

impl Responder for String {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Html, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        res.send(self.as_bytes()).into_outcome()
    }
}

impl Responder for File {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        let size = match self.metadata() {
            Ok(md) => md.len(),
            Err(e) => {
                error_!("Failed to read file metadata: {:?}", e);
                return Forward((StatusCode::InternalServerError, res));
            }
        };

        let mut v = Vec::new();
        if let Err(e) = self.read_to_end(&mut v) {
            error_!("Failed to read file: {:?}", e);
            return Forward((StatusCode::InternalServerError, res));
        }

        res.headers_mut().set(header::ContentLength(size));
        res.start().and_then(|mut stream| stream.write_all(&v)).into_outcome()
    }
}

impl<T: Responder> Responder for Option<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if let Some(ref mut val) = *self {
            val.respond(res)
        } else {
            warn_!("Response was `None`.");
            Forward((StatusCode::NotFound, res))
        }
    }
}

impl<T: Responder, E: fmt::Debug> Responder for Result<T, E> {
    // prepend with `default` when using impl specialization
    default fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        match *self {
            Ok(ref mut val) => val.respond(res),
            Err(ref e) => {
                error_!("{:?}", e);
                Forward((StatusCode::InternalServerError, res))
            }
        }
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
