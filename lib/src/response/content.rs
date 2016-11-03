use response::{Responder, Outcome};
use http::hyper::{header, FreshHyperResponse};
use http::mime::{Mime, TopLevel, SubLevel};
use http::ContentType;

#[derive(Debug)]
pub struct Content<T: Responder>(pub ContentType, pub T);

impl<T: Responder> Responder for Content<T> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        res.headers_mut().set(header::ContentType(self.0.clone().into()));
        self.1.respond(res)
    }
}

macro_rules! impl_data_type_responder {
    ($name:ident: $top:ident/$sub:ident) => (
    #[derive(Debug)]
    pub struct $name<T: Responder>(pub T);

    impl<T: Responder> Responder for $name<T> {
        fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
            let mime = Mime(TopLevel::$top, SubLevel::$sub, vec![]);
            res.headers_mut().set(header::ContentType(mime));
            self.0.respond(res)
        }
    })
}

impl_data_type_responder!(JSON: Application/Json);
impl_data_type_responder!(XML: Application/Xml);
impl_data_type_responder!(HTML: Text/Html);
impl_data_type_responder!(Plain: Text/Plain);
impl_data_type_responder!(CSS: Text/Css);
impl_data_type_responder!(JavaScript: Application/Javascript);
