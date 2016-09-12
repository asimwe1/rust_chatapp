use rocket::response::NamedFile;
use std::io;
use std::path::{Path, PathBuf};

#[get("/<path..>", rank = 5)]
fn all(path: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("static/").join(path))
}
