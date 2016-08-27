use response::{header, Responder, FreshHyperResponse, Outcome};
use response::mime::{Mime, TopLevel, SubLevel};

macro_rules! impl_data_type_responder {
    ($name:ident: $top:ident/$sub:ident) => (
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
