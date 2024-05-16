use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::error::Error;

use crate::engine::Engines;
use crate::template::TemplateInfo;

use rocket::http::ContentType;
use normpath::PathExt;

pub(crate) type Callback =
    Box<dyn Fn(&mut Engines) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>;

pub(crate) struct Context {
    /// The root of the template directory.
    pub root: PathBuf,
    /// Mapping from template name to its information.
    pub templates: HashMap<String, TemplateInfo>,
    /// Loaded template engines
    pub engines: Engines,
}

pub(crate) use self::manager::ContextManager;

impl Context {
    /// Load all of the templates at `root`, initialize them using the relevant
    /// template engine, and store all of the initialized state in a `Context`
    /// structure, which is returned if all goes well.
    pub fn initialize(root: &Path, callback: &Callback) -> Option<Context> {
        fn is_file_with_ext(entry: &walkdir::DirEntry, ext: &str) -> bool {
            let is_file = entry.file_type().is_file();
            let has_ext = entry.path().extension().map_or(false, |e| e == ext);
            is_file && has_ext
        }

        let root = match root.normalize() {
            Ok(root) => root.into_path_buf(),
            Err(e) => {
                error!("Invalid template directory '{}': {}.", root.display(), e);
                return None;
            }
        };

        let mut templates: HashMap<String, TemplateInfo> = HashMap::new();
        for &ext in Engines::ENABLED_EXTENSIONS {
            for entry in walkdir::WalkDir::new(&root).follow_links(true) {
                let entry = match entry {
                    Ok(entry) if is_file_with_ext(&entry, ext) => entry,
                    Ok(_) | Err(_) => continue,
                };

                let (name, data_type_str) = split_path(&root, entry.path());
                if let Some(info) = templates.get(&*name) {
                    warn_!("Template name '{}' does not have a unique source.", name);
                    match info.path {
                        Some(ref path) => info_!("Existing path: {:?}", path),
                        None => info_!("Existing Content-Type: {}", info.data_type),
                    }

                    info_!("Additional path: {:?}", entry.path());
                    warn_!("Keeping existing template '{}'.", name);
                    continue;
                }

                let data_type = data_type_str.as_ref()
                    .and_then(|ext| ContentType::from_extension(ext))
                    .unwrap_or(ContentType::Text);

                templates.insert(name, TemplateInfo {
                    path: Some(entry.into_path()),
                    engine_ext: ext,
                    data_type,
                });
            }
        }

        let mut engines = Engines::init(&templates)?;
        if let Err(e) = callback(&mut engines) {
            error_!("Template customization callback failed.");
            error_!("{}", e);
            return None;
        }

        for (name, engine_ext) in engines.templates() {
            if !templates.contains_key(name) {
                let data_type = Path::new(name).extension()
                    .and_then(|osstr| osstr.to_str())
                    .and_then(ContentType::from_extension)
                    .unwrap_or(ContentType::Text);

                let info = TemplateInfo { path: None, engine_ext, data_type };
                templates.insert(name.to_string(), info);
            }
        }

        Some(Context { root, templates, engines })
    }
}

#[cfg(not(debug_assertions))]
mod manager {
    use std::ops::Deref;
    use super::Context;

    /// Wraps a Context. With `cfg(debug_assertions)` active, this structure
    /// additionally provides a method to reload the context at runtime.
    pub(crate) struct ContextManager(Context);

    impl ContextManager {
        pub fn new(ctxt: Context) -> ContextManager {
            ContextManager(ctxt)
        }

        pub fn context<'a>(&'a self) -> impl Deref<Target=Context> + 'a {
            &self.0
        }

        pub fn is_reloading(&self) -> bool {
            false
        }
    }
}

#[cfg(debug_assertions)]
mod manager {
    use std::ops::{Deref, DerefMut};
    use std::sync::{RwLock, Mutex};
    use std::sync::mpsc::{channel, Receiver};

    use notify::{recommended_watcher, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};

    use super::{Callback, Context};

    /// Wraps a Context. With `cfg(debug_assertions)` active, this structure
    /// additionally provides a method to reload the context at runtime.
    pub(crate) struct ContextManager {
        /// The current template context, inside an RwLock so it can be updated.
        context: RwLock<Context>,
        /// A filesystem watcher and the receive queue for its events.
        watcher: Option<(RecommendedWatcher, Mutex<Receiver<Result<Event, Error>>>)>,
    }

    impl ContextManager {
        pub fn new(ctxt: Context) -> ContextManager {
            let (tx, rx) = channel();
            let watcher = recommended_watcher(tx).and_then(|mut watcher| {
                watcher.watch(&ctxt.root.canonicalize()?, RecursiveMode::Recursive)?;
                Ok(watcher)
            });

            let watcher = match watcher {
                Ok(watcher) => Some((watcher, Mutex::new(rx))),
                Err(e) => {
                    warn!("Failed to enable live template reloading: {}", e);
                    debug_!("Reload error: {:?}", e);
                    warn_!("Live template reloading is unavailable.");
                    None
                }
            };

            ContextManager { watcher, context: RwLock::new(ctxt), }
        }

        pub fn context(&self) -> impl Deref<Target=Context> + '_ {
            self.context.read().unwrap()
        }

        pub fn is_reloading(&self) -> bool {
            self.watcher.is_some()
        }

        fn context_mut(&self) -> impl DerefMut<Target=Context> + '_ {
            self.context.write().unwrap()
        }

        /// Checks whether any template files have changed on disk. If there
        /// have been changes since the last reload, all templates are
        /// reinitialized from disk and the user's customization callback is run
        /// again.
        pub fn reload_if_needed(&self, callback: &Callback) {
            let templates_changes = self.watcher.as_ref()
                .map(|(_, rx)| rx.lock().expect("fsevents lock").try_iter().count() > 0);

            if let Some(true) = templates_changes {
                info_!("Change detected: reloading templates.");
                let root = self.context().root.clone();
                if let Some(new_ctxt) = Context::initialize(&root, callback) {
                    *self.context_mut() = new_ctxt;
                } else {
                    warn_!("An error occurred while reloading templates.");
                    warn_!("Existing templates will remain active.");
                };
            }
        }
    }
}

/// Removes the file path's extension or does nothing if there is none.
fn remove_extension(path: &Path) -> PathBuf {
    let stem = match path.file_stem() {
        Some(stem) => stem,
        None => return path.to_path_buf()
    };

    match path.parent() {
        Some(parent) => parent.join(stem),
        None => PathBuf::from(stem)
    }
}

/// Splits a path into a name that may be used to identify the template, and the
/// template's data type, if any.
fn split_path(root: &Path, path: &Path) -> (String, Option<String>) {
    let rel_path = path.strip_prefix(root).unwrap().to_path_buf();
    let path_no_ext = remove_extension(&rel_path);
    let data_type = path_no_ext.extension();
    let mut name = remove_extension(&path_no_ext).to_string_lossy().into_owned();

    // Ensure template name consistency on Windows systems
    if cfg!(windows) {
        name = name.replace('\\', "/");
    }

    (name, data_type.map(|d| d.to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_path_index_html() {
        for root in &["/", "/a/b/c/", "/a/b/c/d/", "/a/"] {
            for filename in &["index.html.hbs", "index.html.tera"] {
                let path = Path::new(root).join(filename);
                let (name, data_type) = split_path(Path::new(root), &path);

                assert_eq!(name, "index");
                assert_eq!(data_type, Some("html".into()));
            }
        }
    }

    #[test]
    fn template_path_subdir_index_html() {
        for root in &["/", "/a/b/c/", "/a/b/c/d/", "/a/"] {
            for sub in &["a/", "a/b/", "a/b/c/", "a/b/c/d/"] {
                for filename in &["index.html.hbs", "index.html.tera"] {
                    let path = Path::new(root).join(sub).join(filename);
                    let (name, data_type) = split_path(Path::new(root), &path);

                    let expected_name = format!("{}index", sub);
                    assert_eq!(name, expected_name.as_str());
                    assert_eq!(data_type, Some("html".into()));
                }
            }
        }
    }

    #[test]
    fn template_path_doc_examples() {
        fn name_for(path: &str) -> String {
            split_path(Path::new("templates/"), &Path::new("templates/").join(path)).0
        }

        assert_eq!(name_for("index.html.hbs"), "index");
        assert_eq!(name_for("index.tera"), "index");
        assert_eq!(name_for("index.hbs"), "index");
        assert_eq!(name_for("dir/index.hbs"), "dir/index");
        assert_eq!(name_for("dir/index.html.tera"), "dir/index");
        assert_eq!(name_for("index.template.html.hbs"), "index.template");
        assert_eq!(name_for("subdir/index.template.html.hbs"), "subdir/index.template");
    }
}
