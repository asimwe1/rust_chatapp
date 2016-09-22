extern crate tera;

use std::path::PathBuf;
use self::tera::Renderer;

use super::serde::Serialize;
use super::serde_json;
use super::{TemplateInfo, TEMPLATE_DIR};

lazy_static! {
    static ref TERA: tera::Tera = {
        let path: PathBuf = [&*TEMPLATE_DIR, "**", "*.tera"].iter().collect();
        tera::Tera::new(path.to_str().unwrap())
    };
}

pub const EXT: &'static str = "tera";

pub fn render<T>(name: &str, info: &TemplateInfo, context: &T) -> Option<String>
    where T: Serialize
{
    let template = match TERA.get_template(&info.path.to_string_lossy()) {
        Ok(template) => template,
        Err(_) => {
            error_!("Tera template '{}' does not exist.", name);
            return None;
        }
    };

    let value = serde_json::to_value(&context);
    let mut renderer = Renderer::new_with_json(template, &TERA, value);
    match renderer.render() {
        Ok(string) => Some(string),
        Err(e) => {
            error_!("Error rendering Tera template '{}': {}", name, e);
            None
        }
    }
}
