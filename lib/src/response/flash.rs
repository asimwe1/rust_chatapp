use response::*;
use std::convert::AsRef;
use hyper::header::{SetCookie, CookiePair};
use request::{Request, FromRequest};

pub struct Flash<R> {
    name: String,
    message: String,
    responder: R
}

impl<R: Responder> Flash<R> {
    pub fn new<N: AsRef<str>, M: AsRef<str>>(res: R, name: N, msg: M) -> Flash<R> {
        Flash {
            name: name.as_ref().to_string(),
            message: msg.as_ref().to_string(),
            responder: res,
        }
    }

    pub fn warning<S: AsRef<str>>(responder: R, msg: S) -> Flash<R> {
        Flash::new(responder, "warning", msg)
    }

    pub fn success<S: AsRef<str>>(responder: R, msg: S) -> Flash<R> {
        Flash::new(responder, "success", msg)
    }

    pub fn error<S: AsRef<str>>(responder: R, msg: S) -> Flash<R> {
        Flash::new(responder, "error", msg)
    }

    pub fn cookie_pair(&self) -> CookiePair {
        let content = format!("{}{}{}", self.name.len(), self.name, self.message);
        let mut pair = CookiePair::new("flash".to_string(), content);
        pair.path = Some("/".to_string());
        pair.max_age = Some(300);
        pair
    }
}

impl<R: Responder> Responder for Flash<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        trace_!("Flash: setting message: {}:{}", self.name, self.message);
        res.headers_mut().set(SetCookie(vec![self.cookie_pair()]));
        self.responder.respond(res)
    }
}

impl Flash<()> {
    fn named(name: &str, msg: &str) -> Flash<()> {
        Flash {
            name: name.to_string(),
            message: msg.to_string(),
            responder: (),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn msg(&self) -> &str {
        self.message.as_str()
    }
}

// TODO: Using Flash<()> is ugly. Either create a type FlashMessage = Flash<()>
// or create a Flash under request that does this.
// TODO: Consider not removing the 'flash' cookie until after this thing is
// dropped. This is because, at the moment, if Flash is including as a
// from_request param, and some other param fails, then the flash message will
// be dropped needlessly. This may or may not be the intended behavior.
// Alternatively, provide a guarantee about the order that from_request params
// will be evaluated and recommend that Flash is last.
impl<'r, 'c> FromRequest<'r, 'c> for Flash<()> {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        trace_!("Flash: attemping to retrieve message.");
        request.cookies().find("flash").ok_or(()).and_then(|cookie| {
            // Clear the flash message.
            trace_!("Flash: retrieving message: {:?}", cookie);
            request.cookies().remove("flash");

            // Parse the flash.
            let content = cookie.pair().1;
            let (len_str, rest) = match content.find(|c: char| !c.is_digit(10)) {
                Some(i) => (&content[..i], &content[i..]),
                None => (content, "")
            };

            let name_len: usize = len_str.parse().map_err(|_| ())?;
            let (name, msg) = (&rest[..name_len], &rest[name_len..]);
            Ok(Flash::named(name, msg))
        })
    }
}

