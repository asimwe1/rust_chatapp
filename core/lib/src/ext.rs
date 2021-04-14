use std::io;
use std::pin::Pin;
use std::task::{Poll, Context};

use bytes::BytesMut;
use tokio::io::{AsyncRead, ReadBuf};
use pin_project_lite::pin_project;
use futures::{ready, stream::Stream};

use crate::http::hyper::Bytes;

pin_project! {
    pub struct ReaderStream<R> {
        #[pin]
        reader: Option<R>,
        buf: BytesMut,
        cap: usize,
    }
}

impl<R: AsyncRead> Stream for ReaderStream<R> {
    type Item = std::io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use tokio_util::io::poll_read_buf;

        let mut this = self.as_mut().project();

        let reader = match this.reader.as_pin_mut() {
            Some(r) => r,
            None => return Poll::Ready(None),
        };

        if this.buf.capacity() == 0 {
            this.buf.reserve(*this.cap);
        }

        match poll_read_buf(reader, cx, &mut this.buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => {
                self.project().reader.set(None);
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(Ok(0)) => {
                self.project().reader.set(None);
                Poll::Ready(None)
            }
            Poll::Ready(Ok(_)) => {
                let chunk = this.buf.split();
                Poll::Ready(Some(Ok(chunk.freeze())))
            }
        }
    }
}

pub trait AsyncReadExt: AsyncRead + Sized {
    fn into_bytes_stream(self, cap: usize) -> ReaderStream<Self> {
        ReaderStream { reader: Some(self), cap, buf: BytesMut::with_capacity(cap) }
    }
}

impl<T: AsyncRead> AsyncReadExt for T { }

pub trait PollExt<T, E> {
    fn map_err_ext<U, F>(self, f: F) -> Poll<Option<Result<T, U>>>
        where F: FnOnce(E) -> U;
}

impl<T, E> PollExt<T, E> for Poll<Option<Result<T, E>>> {
    /// Changes the error value of this `Poll` with the closure provided.
    fn map_err_ext<U, F>(self, f: F) -> Poll<Option<Result<T, U>>>
        where F: FnOnce(E) -> U
    {
        match self {
            Poll::Ready(Some(Ok(t))) => Poll::Ready(Some(Ok(t))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(f(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pin_project! {
    /// Stream for the [`chain`](super::AsyncReadExt::chain) method.
    #[must_use = "streams do nothing unless polled"]
    pub struct Chain<T, U> {
        #[pin]
        first: T,
        #[pin]
        second: U,
        done_first: bool,
    }
}

impl<T: AsyncRead, U: AsyncRead> Chain<T, U> {
    pub(crate) fn new(first: T, second: U) -> Self {
        Self { first, second, done_first: false }
    }
}

impl<T: AsyncRead, U: AsyncRead> Chain<T, U> {
    /// Gets references to the underlying readers in this `Chain`.
    pub fn get_ref(&self) -> (&T, &U) {
        (&self.first, &self.second)
    }
}

impl<T: AsyncRead, U: AsyncRead> AsyncRead for Chain<T, U> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let me = self.project();

        if !*me.done_first {
            let init_rem = buf.remaining();
            ready!(me.first.poll_read(cx, buf))?;
            if buf.remaining() == init_rem {
                *me.done_first = true;
            } else {
                return Poll::Ready(Ok(()));
            }
        }
        me.second.poll_read(cx, buf)
    }
}
