use response::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use response::mime::{Mime, TopLevel, SubLevel};
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
            let (top_level, sub_level) = match ext_string.as_str() {
                "txt" => (TopLevel::Text, SubLevel::Plain),
                "html" => (TopLevel::Text, SubLevel::Html),
                "xml" => (TopLevel::Application, SubLevel::Xml),
                "js" => (TopLevel::Application, SubLevel::Javascript),
                "css" => (TopLevel::Text, SubLevel::Css),
                "json" => (TopLevel::Application, SubLevel::Json),
                "png" => (TopLevel::Image, SubLevel::Png),
                "gif" => (TopLevel::Image, SubLevel::Gif),
                "bmp" => (TopLevel::Image, SubLevel::Bmp),
                "jpeg" => (TopLevel::Image, SubLevel::Jpeg),
                "jpg" => (TopLevel::Image, SubLevel::Jpeg),
                _ => (TopLevel::Star, SubLevel::Star),
            };

            if top_level != TopLevel::Star && sub_level != SubLevel::Star {
                let mime = Mime(top_level, sub_level, vec![]);
                res.headers_mut().set(header::ContentType(mime));
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

