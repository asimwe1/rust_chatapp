pub struct Response;

impl<'a> From<&'a str> for Response {
    fn from(_s: &'a str) -> Self {
        Response
    }
}

impl From<String> for Response {
    fn from(_s: String) -> Self {
        Response
    }
}

impl Response {
    pub fn error(number: usize) -> Response {
        println!("ERROR {}!", number);
        Response
    }
}
