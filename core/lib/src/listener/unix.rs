use std::io;
use std::path::PathBuf;

use tokio::time::{sleep, Duration};

use crate::fs::NamedFile;
use crate::listener::{Listener, Bindable, Connection, Endpoint};
use crate::util::unix;

pub use tokio::net::UnixStream;

#[derive(Debug, Clone)]
pub struct UdsConfig {
    /// Socket address.
    pub path: PathBuf,
    /// Recreate a socket that already exists.
    pub reuse: Option<bool>,
}

pub struct UdsListener {
    path: PathBuf,
    lock: Option<NamedFile>,
    listener: tokio::net::UnixListener,
}

impl Bindable for UdsConfig {
    type Listener = UdsListener;

    type Error = io::Error;

    async fn bind(self) -> Result<Self::Listener, Self::Error> {
        let lock = if self.reuse.unwrap_or(true) {
            let lock_ext = match self.path.extension().and_then(|s| s.to_str()) {
                Some(ext) if !ext.is_empty() => format!("{}.lock", ext),
                _ => "lock".to_string()
            };

            let mut opts = tokio::fs::File::options();
            opts.create(true).write(true);
            let lock_path = self.path.with_extension(lock_ext);
            let lock_file = NamedFile::open_with(lock_path, &opts).await?;

            unix::lock_exclusive_nonblocking(lock_file.file())?;
            if self.path.exists() {
                tokio::fs::remove_file(&self.path).await?;
            }

            Some(lock_file)
        } else {
            None
        };

        // Sometimes, we get `AddrInUse`, even though we've tried deleting the
        // socket. If all is well, eventually the socket will _really_ be gone,
        // and this will succeed. So let's try a few times.
        let mut retries = 5;
        let listener = loop {
            match tokio::net::UnixListener::bind(&self.path) {
                Ok(listener) => break listener,
                Err(e) if self.path.exists() && lock.is_none() => return Err(e),
                Err(_) if retries > 0 => {
                    retries -= 1;
                    sleep(Duration::from_millis(100)).await;
                },
                Err(e) => return Err(e),
            }
        };

        Ok(UdsListener { lock, listener, path: self.path, })
    }

    fn candidate_endpoint(&self) -> io::Result<Endpoint> {
        Ok(Endpoint::Unix(self.path.clone()))
    }
}

impl Listener for UdsListener {
    type Accept = UnixStream;

    type Connection = Self::Accept;

    async fn accept(&self) -> io::Result<Self::Accept> {
        Ok(self.listener.accept().await?.0)
    }

    async fn connect(&self, accept:Self::Accept) -> io::Result<Self::Connection> {
        Ok(accept)
    }

    fn endpoint(&self) -> io::Result<Endpoint> {
        self.listener.local_addr()?.try_into()
    }
}

impl Connection for UnixStream {
    fn endpoint(&self) -> io::Result<Endpoint> {
        self.local_addr()?.try_into()
    }
}

impl Drop for UdsListener {
    fn drop(&mut self) {
        if let Some(lock) = &self.lock {
            let _ = std::fs::remove_file(&self.path);
            let _ = std::fs::remove_file(lock.path());
            let _ = unix::unlock_nonblocking(lock.file());
        } else {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
