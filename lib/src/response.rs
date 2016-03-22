pub use hyper::server::Response as HypResponse;
pub use hyper::net::Fresh as HypFresh;

use hyper::status::StatusCode;
use hyper::header;
use std::io::{Read, Write};
use std::fs::File;

pub struct Response<'a> {
    pub body: Box<Responder + 'a>
}

impl<'a> Response<'a> {
    pub fn new<T: Responder + 'a>(body: T) -> Response<'a> {
        Response {
            body: Box::new(body)
        }
    }

    pub fn empty() -> Response<'a> {
        Response {
            body: Box::new(Empty::new(StatusCode::Ok))
        }
    }

    pub fn not_found() -> Response<'a> {
        Response {
            body: Box::new(Empty::new(StatusCode::NotFound))
        }
    }

    pub fn server_error() -> Response<'a> {
        Response {
            body: Box::new(Empty::new(StatusCode::InternalServerError))
        }
    }
}

pub trait Responder {
    fn respond<'a>(&mut self, mut res: HypResponse<'a, HypFresh>);
}

pub struct Empty {
    status: StatusCode
}

impl Empty {
    fn new(status: StatusCode) -> Empty {
        Empty {
            status: status
        }
    }
}

impl Responder for Empty {
    fn respond<'a>(&mut self, mut res: HypResponse<'a, HypFresh>) {
        res.headers_mut().set(header::ContentLength(0));
        *(res.status_mut()) = self.status;

        let mut stream = res.start().unwrap();
        stream.write_all(b"").unwrap();
    }
}

impl<'a> Responder for &'a str {
    fn respond<'b>(&mut self, res: HypResponse<'b, HypFresh>) {
        res.send(self.as_bytes()).unwrap();
    }
}

impl Responder for String {
    fn respond<'b>(&mut self, res: HypResponse<'b, HypFresh>) {
        res.send(self.as_bytes()).unwrap();
    }
}

impl Responder for File {
    fn respond<'b>(&mut self, mut res: HypResponse<'b, HypFresh>) {
        let size = self.metadata().unwrap().len();

        res.headers_mut().set(header::ContentLength(size));
        *(res.status_mut()) = StatusCode::Ok;

        let mut v = Vec::new();
        self.read_to_end(&mut v).unwrap();

        let mut stream = res.start().unwrap();
        stream.write_all(&v).unwrap();
    }
}

// TODO: Allow streamed responses.
// const CHUNK_SIZE: u32 = 4096;
// pub struct Stream<T: Read>(T);
// impl<T> Responder for Stream<T> {
//     fn respond<'a>(&self, mut r: HypResponse<'a, HypFresh>) {
//         r.headers_mut().set(header::TransferEncoding(vec![Encoding::Chunked]));
//         *(r.status_mut()) = StatusCode::Ok;
//         let mut stream = r.start();

//         r.write()
//         Response {
//             status: StatusCode::Ok,
//             headers: headers,
//             body: Body::Stream(r)
//         }
//     }
// }
