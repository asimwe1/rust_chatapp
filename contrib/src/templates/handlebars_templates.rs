extern crate handlebars;

use std::sync::RwLock;

use super::serde::Serialize;
use super::TemplateInfo;

use self::handlebars::Handlebars;

lazy_static! {
    static ref HANDLEBARS: RwLock<Handlebars> = RwLock::new(Handlebars::new());
}

pub const EXT: &'static str = "hbs";

pub fn render<T>(name: &str, info: &TemplateInfo, context: &T) -> Option<String>
    where T: Serialize
{
    // FIXME: Expose a callback to register each template at launch => no lock.
    if HANDLEBARS.read().unwrap().get_template(name).is_none() {
        let p = &info.full_path;
        if let Err(e) = HANDLEBARS.write().unwrap().register_template_file(name, p) {
            error_!("Handlebars template '{}' failed registry: {:?}", name, e);
            return None;
        }
    }

    match HANDLEBARS.read().unwrap().render(name, context) {
        Ok(string) => Some(string),
        Err(e) => {
            error_!("Error rendering Handlebars template '{}': {}", name, e);
            None
        }
    }
}
