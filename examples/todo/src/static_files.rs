use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

#[get("/<path..>", rank = 5)]
fn all(path: PathBuf) -> io::Result<File> {
    File::open(Path::new("static/").join(path))
}
