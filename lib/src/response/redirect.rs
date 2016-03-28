use response::*;

#[derive(Debug)]
pub struct Redirect(StatusCode, String);

impl Redirect {
    pub fn to(uri: &str) -> Redirect {
        Redirect(StatusCode::TemporaryRedirect, String::from(uri))
    }

    pub fn created(uri: &str) -> Redirect {
        Redirect(StatusCode::Created, String::from(uri))
    }

    pub fn other(uri: &str) -> Redirect {
        Redirect(StatusCode::SeeOther, String::from(uri))
    }

    pub fn permanent(uri: &str) -> Redirect {
        Redirect(StatusCode::PermanentRedirect, String::from(uri))
    }
}

impl<'a> Responder for Redirect {
    fn respond<'b>(&mut self, mut res: HypResponse<'b, HypFresh>) {
        res.headers_mut().set(header::ContentLength(0));
        res.headers_mut().set(header::Location(self.1.clone()));
        *(res.status_mut()) = self.0;
        res.send(b"").unwrap();
    }
}

