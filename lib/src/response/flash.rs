use std::convert::AsRef;

use response::{ResponseOutcome, Responder};
use request::{Request, FromRequest, RequestOutcome};
use http::hyper::{HyperSetCookie, HyperCookiePair, FreshHyperResponse};

const FLASH_COOKIE_NAME: &'static str = "_flash";

pub struct Flash<R> {
    name: String,
    message: String,
    responder: R,
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

    fn cookie_pair(&self) -> HyperCookiePair {
        let content = format!("{}{}{}", self.name.len(), self.name, self.message);
        let mut pair = HyperCookiePair::new(FLASH_COOKIE_NAME.to_string(), content);
        pair.path = Some("/".to_string());
        pair.max_age = Some(300);
        pair
    }
}

impl<R: Responder> Responder for Flash<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> ResponseOutcome<'b> {
        trace_!("Flash: setting message: {}:{}", self.name, self.message);
        res.headers_mut().set(HyperSetCookie(vec![self.cookie_pair()]));
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
impl<'r> FromRequest<'r> for Flash<()> {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        trace_!("Flash: attemping to retrieve message.");
        let r = request.cookies().find(FLASH_COOKIE_NAME).ok_or(()).and_then(|cookie| {
            // Clear the flash message.
            trace_!("Flash: retrieving message: {:?}", cookie);
            request.cookies().remove(FLASH_COOKIE_NAME);

            // Parse the flash.
            let content = cookie.pair().1;
            let (len_str, rest) = match content.find(|c: char| !c.is_digit(10)) {
                Some(i) => (&content[..i], &content[i..]),
                None => (content, ""),
            };

            let name_len: usize = len_str.parse().map_err(|_| ())?;
            let (name, msg) = (&rest[..name_len], &rest[name_len..]);
            Ok(Flash::named(name, msg))
        });

        RequestOutcome::of(r)
    }
}
