use response::*;
use std::string::ToString;
use hyper::header::{SetCookie, CookiePair};

pub struct Cookied<R: Responder> {
    cookies: Option<Vec<CookiePair>>,
    responder: R
}

impl<R: Responder> Cookied<R> {
    pub fn new(responder: R) -> Cookied<R> {
        Cookied {
            cookies: None,
            responder: responder
        }
    }

    pub fn pairs(responder: R, pairs: &[(&ToString, &ToString)]) -> Cookied<R> {
        Cookied {
            cookies: Some(
                pairs.iter()
                .map(|p| CookiePair::new(p.0.to_string(), p.1.to_string()))
                .collect()
            ),
            responder: responder
        }
    }

    #[inline(always)]
    pub fn add<A: ToString, B: ToString>(mut self, a: A, b: B) -> Self {
        let new_pair = CookiePair::new(a.to_string(), b.to_string());
        match self.cookies {
            Some(ref mut pairs) => pairs.push(new_pair),
            None => self.cookies = Some(vec![new_pair])
        };

        self
    }
}

impl<R: Responder> Responder for Cookied<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        if let Some(pairs) = self.cookies.take() {
            res.headers_mut().set(SetCookie(pairs));
        }

        self.responder.respond(res)
    }
}

