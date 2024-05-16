use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use rocket::serde::Serialize;
use minijinja::{Environment, Error, ErrorKind, AutoEscape};

use crate::engine::Engine;

impl Engine for Environment<'static> {
    const EXT: &'static str = "j2";

    fn init<'a>(templates: impl Iterator<Item = (&'a str, &'a Path)>) -> Option<Self> {
        let _templates = Arc::new(templates
            .map(|(k, p)| (k.to_owned(), p.to_owned()))
            .collect::<HashMap<_, _>>());

        let templates = _templates.clone();
        let mut env = Environment::new();
        env.set_loader(move |name| {
            let Some(path) = templates.get(name) else {
                return Ok(None);
            };

            match std::fs::read_to_string(path) {
                Ok(result) => Ok(Some(result)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(
                    Error::new(ErrorKind::InvalidOperation, "template read failed").with_source(e)
                ),
            }
        });

        let templates = _templates.clone();
        env.set_auto_escape_callback(move |name| {
            templates.get(name)
                .and_then(|path| path.to_str())
                .map(minijinja::default_auto_escape_callback)
                .unwrap_or(AutoEscape::None)
        });

        Some(env)
    }

    fn render<C: Serialize>(&self, name: &str, context: C) -> Option<String> {
        let Ok(template) = self.get_template(name) else {
            error_!("Minijinja template '{name}' was not found.");
            return None;
        };

        template.render(context)
            .map_err(|e| error_!("Minijinja: {}", e))
            .ok()
    }
}
