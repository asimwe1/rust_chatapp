use std::io;
use std::time::Duration;
use std::task::{Poll, Context};
use std::pin::Pin;

use tokio::time::{sleep, Sleep};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use futures::{StreamExt, future::{select, Either, Fuse, Future, FutureExt}};
use pin_project_lite::pin_project;

use crate::{config, Shutdown};
use crate::listener::{Listener, Connection, Certificates, Bounced, Endpoint};

// Rocket wraps all connections in a `CancellableIo` struct, an internal
// structure that gracefully closes I/O when it receives a signal. That signal
// is the `shutdown` future. When the future resolves, `CancellableIo` begins to
// terminate in grace, mercy, and finally force close phases. Since all
// connections are wrapped in `CancellableIo`, this eventually ends all I/O.
//
// At that point, unless a user spawned an infinite, stand-alone task that isn't
// monitoring `Shutdown`, all tasks should resolve. This means that all
// instances of the shared `Arc<Rocket>` are dropped and we can return the owned
// instance of `Rocket`.
//
// Unfortunately, the Hyper `server` future resolves as soon as it has finished
// processing requests without respect for ongoing responses. That is, `server`
// resolves even when there are running tasks that are generating a response.
// So, `server` resolving implies little to nothing about the state of
// connections. As a result, we depend on the timing of grace + mercy + some
// buffer to determine when all connections should be closed, thus all tasks
// should be complete, thus all references to `Arc<Rocket>` should be dropped
// and we can get a unique reference.
pin_project! {
    pub struct CancellableListener<F, L> {
        pub trigger: F,
        #[pin]
        pub listener: L,
        pub grace: Duration,
        pub mercy: Duration,
    }
}

pin_project! {
    /// I/O that can be cancelled when a future `F` resolves.
    #[must_use = "futures do nothing unless polled"]
    pub struct CancellableIo<F, I> {
        #[pin]
        io: Option<I>,
        #[pin]
        trigger: Fuse<F>,
        state: State,
        grace: Duration,
        mercy: Duration,
    }
}

enum State {
    /// I/O has not been cancelled. Proceed as normal.
    Active,
    /// I/O has been cancelled. See if we can finish before the timer expires.
    Grace(Pin<Box<Sleep>>),
    /// Grace period elapsed. Shutdown the connection, waiting for the timer
    /// until we force close.
    Mercy(Pin<Box<Sleep>>),
}

pub trait CancellableExt: Sized {
    fn cancellable(
        self,
        trigger: Shutdown,
        config: &config::Shutdown
    ) -> CancellableListener<Shutdown, Self> {
        if let Some(mut stream) = config.signal_stream() {
            let trigger = trigger.clone();
            tokio::spawn(async move {
                while let Some(sig) = stream.next().await {
                    if trigger.0.tripped() {
                        warn!("Received {}. Shutdown already in progress.", sig);
                    } else {
                        warn!("Received {}. Requesting shutdown.", sig);
                    }

                    trigger.0.trip();
                }
            });
        };

        CancellableListener {
            trigger,
            listener: self,
            grace: config.grace(),
            mercy: config.mercy(),
        }
    }
}

impl<L: Listener> CancellableExt for L { }

fn time_out() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "Shutdown grace timed out")
}

fn gone() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "IO driver has terminated")
}

impl<L, F> CancellableListener<F, Bounced<L>>
    where L: Listener + Sync,
          F: Future + Unpin + Clone + Send + Sync + 'static
{
    pub async fn accept_next(&self) -> Option<<Self as Listener>::Accept> {
        let next = std::pin::pin!(self.listener.accept_next());
        match select(next, self.trigger.clone()).await {
            Either::Left((next, _)) => Some(next),
            Either::Right(_) => None,
        }
    }
}

impl<L, F> CancellableListener<F, L>
    where L: Listener + Sync,
          F: Future + Clone + Send + Sync + 'static
{
    fn io<C>(&self, conn: C) -> CancellableIo<F, C> {
        CancellableIo {
            io: Some(conn),
            trigger: self.trigger.clone().fuse(),
            state: State::Active,
            grace: self.grace,
            mercy: self.mercy,
        }
    }
}

impl<L, F> Listener for CancellableListener<F, L>
    where L: Listener + Sync,
          F: Future + Clone + Send + Sync + Unpin + 'static
{
    type Accept = L::Accept;

    type Connection = CancellableIo<F, L::Connection>;

    async fn accept(&self) -> io::Result<Self::Accept> {
        let accept = std::pin::pin!(self.listener.accept());
        match select(accept, self.trigger.clone()).await {
            Either::Left((result, _)) => result,
            Either::Right(_) => Err(gone()),
        }
    }

    async fn connect(&self, accept: Self::Accept) -> io::Result<Self::Connection> {
        let conn = std::pin::pin!(self.listener.connect(accept));
        match select(conn, self.trigger.clone()).await {
            Either::Left((conn, _)) => Ok(self.io(conn?)),
            Either::Right(_) => Err(gone()),
        }
    }

    fn socket_addr(&self) -> io::Result<Endpoint> {
        self.listener.socket_addr()
    }
}

impl<F: Future, I: AsyncWrite> CancellableIo<F, I> {
    fn inner(&self) -> Option<&I> {
        self.io.as_ref()
    }

    /// Run `do_io` while connection processing should continue.
    fn poll_trigger_then<T>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        do_io: impl FnOnce(Pin<&mut I>, &mut Context<'_>) -> Poll<io::Result<T>>,
    ) -> Poll<io::Result<T>> {
        let mut me = self.as_mut().project();
        let io = match me.io.as_pin_mut() {
            Some(io) => io,
            None => return Poll::Ready(Err(gone())),
        };

        loop {
            match me.state {
                State::Active => {
                    if me.trigger.as_mut().poll(cx).is_ready() {
                        *me.state = State::Grace(Box::pin(sleep(*me.grace)));
                    } else {
                        return do_io(io, cx);
                    }
                }
                State::Grace(timer) => {
                    if timer.as_mut().poll(cx).is_ready() {
                        *me.state = State::Mercy(Box::pin(sleep(*me.mercy)));
                    } else {
                        return do_io(io, cx);
                    }
                }
                State::Mercy(timer) => {
                    if timer.as_mut().poll(cx).is_ready() {
                        self.project().io.set(None);
                        return Poll::Ready(Err(time_out()));
                    } else {
                        let result = futures::ready!(io.poll_shutdown(cx));
                        self.project().io.set(None);
                        return match result {
                            Err(e) => Poll::Ready(Err(e)),
                            Ok(()) => Poll::Ready(Err(gone()))
                        };
                    }
                },
            }
        }
    }
}

impl<F: Future, I: AsyncRead + AsyncWrite> AsyncRead for CancellableIo<F, I> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.as_mut().poll_trigger_then(cx, |io, cx| io.poll_read(cx, buf))
    }
}

impl<F: Future, I: AsyncWrite> AsyncWrite for CancellableIo<F, I> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.as_mut().poll_trigger_then(cx, |io, cx| io.poll_write(cx, buf))
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<io::Result<()>> {
        self.as_mut().poll_trigger_then(cx, |io, cx| io.poll_flush(cx))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<io::Result<()>> {
        self.as_mut().poll_trigger_then(cx, |io, cx| io.poll_shutdown(cx))
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        self.as_mut().poll_trigger_then(cx, |io, cx| io.poll_write_vectored(cx, bufs))
    }

    fn is_write_vectored(&self) -> bool {
        self.inner().map(|io| io.is_write_vectored()).unwrap_or(false)
    }
}

impl<F: Future, C: Connection> Connection for CancellableIo<F, C>
    where F: Unpin + Send + 'static
{
    fn peer_address(&self) -> io::Result<Endpoint> {
        self.inner()
            .ok_or_else(|| gone())
            .and_then(|io| io.peer_address())
    }

    fn peer_certificates(&self) -> Option<Certificates<'_>> {
        self.inner().and_then(|io| io.peer_certificates())
    }
}
