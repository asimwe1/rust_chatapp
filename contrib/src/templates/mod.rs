extern crate serde;
extern crate serde_json;
extern crate glob;

#[cfg(feature = "tera_templates")]
pub mod tera_templates;

#[cfg(feature = "handlebars_templates")]
pub mod handlebars_templates;

#[macro_use] mod macros;

use std::path::{Path, PathBuf};
use std::collections::HashMap;

use self::serde::Serialize;
use rocket::response::{data, Outcome, FreshHyperResponse, Responder};
use rocket::Rocket;
use self::glob::glob;

lazy_static! {
    static ref TEMPLATES: HashMap<String, TemplateInfo> = discover_templates();
    static ref TEMPLATE_DIR: String =
        Rocket::config("template_dir").unwrap_or("templates").to_string();
}

/// Removes the file path's extension or does nothing if there is none.
fn remove_extension<P: AsRef<Path>>(path: P) -> PathBuf {
    PathBuf::from(path.as_ref().file_stem().unwrap())
}

fn discover_templates() -> HashMap<String, TemplateInfo> {
    // Keep this set in-sync with the `render_set` invocation.
    let engines = engine_set![
        "tera_templates" => tera_templates,
        "handlebars_templates" => handlebars_templates
    ];

    let mut templates = HashMap::new();
    for ext in engines {
        let mut path: PathBuf = [&*TEMPLATE_DIR, "**", "*"].iter().collect();
        path.set_extension(ext);
        for p in glob(path.to_str().unwrap()).unwrap().filter_map(Result::ok) {
            let canonical_path = remove_extension(&p);
            let name = remove_extension(&canonical_path);
            let data_type = canonical_path.extension();
            templates.insert(name.to_string_lossy().into_owned(), TemplateInfo {
                full_path: p.to_path_buf(),
                path: p.strip_prefix(&*TEMPLATE_DIR).unwrap().to_path_buf(),
                canonical_path: canonical_path.clone(),
                extension: p.extension().unwrap().to_string_lossy().into_owned(),
                data_type: data_type.map(|d| d.to_string_lossy().into_owned())
            });
        }
    }

    templates
}

#[derive(Debug)]
pub struct Template(Option<String>, Option<String>);

#[derive(Debug)]
pub struct TemplateInfo {
    full_path: PathBuf,
    path: PathBuf,
    canonical_path: PathBuf,
    extension: String,
    data_type: Option<String>
}

impl Template {
    pub fn render<S, T>(name: S, context: T) -> Template
        where S: AsRef<str>, T: Serialize
{
        let name = name.as_ref();
        let template = TEMPLATES.get(name);
        if template.is_none() {
            error_!("Template '{}' does not exist.", name);
            return Template(None, None);
        }

        // Keep this set in-sync with the `engine_set` invocation.
        render_set!(name, template.unwrap(), context,
            "tera_templates" => tera_templates,
            "handlebars_templates" => handlebars_templates
        );

        unreachable!("A template extension was discovered but not rendered.")
    }
}

impl Responder for Template {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        match self.0 {
            // FIXME: Detect the data type using the extension in self.1.
            // Refactor response::named_file to use the extension map there.
            Some(ref render) => data::HTML(render.as_str()).respond(res),
            None => Outcome::Bad(res),
        }
    }
}
