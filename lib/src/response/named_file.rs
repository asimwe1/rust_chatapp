use response::*;
use content_type::ContentType;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct NamedFile(PathBuf, File);

impl NamedFile {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<NamedFile> {
        let file = File::open(path.as_ref())?;
        Ok(NamedFile(path.as_ref().to_path_buf(), file))
    }

    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

impl Responder for NamedFile {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if let Some(ext) = self.path().extension() {
            let ext_string = ext.to_string_lossy().to_lowercase();
            let content_type = ContentType::from_extension(&ext_string);
            if !content_type.is_any() {
                res.headers_mut().set(header::ContentType(content_type.into()));
            }
        }

        self.1.respond(res)
    }
}

impl Deref for NamedFile {
    type Target = File;

    fn deref(&self) -> &File {
        &self.1
    }
}

impl DerefMut for NamedFile {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.1
    }
}

