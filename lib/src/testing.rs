use std::io::Cursor;
use outcome::Outcome::*;
use http::{hyper, Method};
use request::{Request, Data};
use Rocket;

pub struct MockRequest {
    request: Request,
    data: Data
}

impl MockRequest {
    pub fn new<S: AsRef<str>>(method: Method, uri: S) -> Self {
        MockRequest {
            request: Request::mock(method, uri.as_ref()),
            data: Data::new(vec![])
        }
    }

    pub fn headers<'h, H: AsRef<[(&'h str, &'h str)]>>(mut self, headers: H) -> Self {
        let mut hyp_headers = hyper::HyperHeaders::new();

        for &(name, fields) in headers.as_ref() {
            let mut vec_fields = vec![];
            for field in fields.split(";") {
                vec_fields.push(field.as_bytes().to_vec());
            }

            hyp_headers.set_raw(name.to_string(), vec_fields);
        }

        self.request.set_headers(hyp_headers);
        self
    }

    pub fn body<S: AsRef<str>>(mut self, body: S) -> Self {
        self.data = Data::new(body.as_ref().as_bytes().into());
        self
    }

    pub fn dispatch_with(mut self, rocket: &Rocket) -> Option<String> {
        let request = self.request;
        let data = ::std::mem::replace(&mut self.data, Data::new(vec![]));

        let mut response = Cursor::new(vec![]);

        // Create a new scope so we can get the inner from response later.
        let ok = {
            let mut h_h = hyper::HyperHeaders::new();
            let res = hyper::FreshHyperResponse::new(&mut response, &mut h_h);
            match rocket.dispatch(&request, data) {
                Ok(mut responder) => {
                    match responder.respond(res) {
                        Success(_) => true,
                        Failure(_) => false,
                        Forward((code, r)) => rocket.handle_error(code, &request, r)
                    }
                }
                Err(code) => rocket.handle_error(code, &request, res)
            }
        };

        if !ok {
            return None;
        }

        match String::from_utf8(response.into_inner()) {
            Ok(string) => {
                // TODO: Expose the full response (with headers) somewhow.
                string.find("\r\n\r\n").map(|i| {
                    string[(i + 4)..].to_string()
                })
            }
            Err(e) => {
                error_!("Could not create string from response: {:?}", e);
                None
            }
        }
    }
}
