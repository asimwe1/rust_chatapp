use std::{fmt, io};
use std::borrow::Cow;
use std::path::{Path, PathBuf, Component};

pub trait ReadExt: io::Read {
    fn read_max(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let start_len = buf.len();
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        Ok(start_len - buf.len())
    }
}

impl<T: io::Read> ReadExt for T {  }

pub struct NormalizedPath<'a>(&'a Path);

impl<'a> fmt::Display for NormalizedPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn to_str<'c>(c: &'c Component) -> Cow<'c, str> {
            c.as_os_str().to_string_lossy()
        }

        let mut components = self.0.components();
        match (components.next(), components.next()) {
            (Some(Component::RootDir), Some(c)) => write!(f, "/{}", to_str(&c))?,
            (Some(a), Some(b)) => write!(f, "{}/{}", to_str(&a), to_str(&b))?,
            (Some(c), None) => write!(f, "{}", to_str(&c))?,
            _ => return Ok(())
        };

        for c in components {
            write!(f, "/{}", to_str(&c))?;
        }

        Ok(())
    }
}

pub trait Normalize {
    fn normalized(&self) -> NormalizedPath;
}

impl<T: AsRef<Path>> Normalize for T {
    fn normalized(&self) -> NormalizedPath {
        NormalizedPath(self.as_ref())
    }
}
