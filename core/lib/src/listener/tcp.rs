use std::io;

#[doc(inline)]
pub use tokio::net::{TcpListener, TcpStream};

use crate::listener::{Listener, Bindable, Connection, Endpoint};

impl Bindable for std::net::SocketAddr {
    type Listener = TcpListener;

    type Error = io::Error;

    async fn bind(self) -> Result<Self::Listener, Self::Error> {
        TcpListener::bind(self).await
    }

    fn bind_endpoint(&self) -> io::Result<Endpoint> {
        Ok(Endpoint::Tcp(*self))
    }
}

impl Listener for TcpListener {
    type Accept = Self::Connection;

    type Connection = TcpStream;

    async fn accept(&self) -> io::Result<Self::Accept> {
        let conn = self.accept().await?.0;
        let _ = conn.set_nodelay(true);
        let _ = conn.set_linger(None);
        Ok(conn)
    }

    async fn connect(&self, conn: Self::Connection) -> io::Result<Self::Connection> {
        Ok(conn)
    }

    fn endpoint(&self) -> io::Result<Endpoint> {
        self.local_addr().map(Endpoint::Tcp)
    }
}

impl Connection for TcpStream {
    fn endpoint(&self) -> io::Result<Endpoint> {
        self.peer_addr().map(Endpoint::Tcp)
    }
}
