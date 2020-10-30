use std::io;
use std::pin::Pin;
use std::task::{Poll, Context};

use futures::{ready, stream::Stream};
use tokio::io::{AsyncRead, ReadBuf};
use pin_project_lite::pin_project;

use crate::http::hyper::Bytes;

pub struct IntoBytesStream<R> {
    inner: R,
    buf_size: usize,
    buffer: Vec<u8>,
}

impl<R> Stream for IntoBytesStream<R>
    where R: AsyncRead + Unpin
{
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>{
        debug_assert!(self.buffer.len() == self.buf_size);

        let Self { ref mut inner, ref mut buffer, buf_size } = *self;

        let mut buf = ReadBuf::new(&mut buffer[..]);
        match Pin::new(inner).poll_read(cx, &mut buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Ok(())) if buf.filled().is_empty() => Poll::Ready(None),
            Poll::Ready(Ok(())) => {
                let n = buf.filled().len();
                // FIXME(perf).
                let mut next = std::mem::replace(buffer, vec![0; buf_size]);
                next.truncate(n);
                Poll::Ready(Some(Ok(Bytes::from(next))))
            }
        }
    }
}

pub trait AsyncReadExt: AsyncRead + Sized {
    fn into_bytes_stream(self, buf_size: usize) -> IntoBytesStream<Self> {
        IntoBytesStream { inner: self, buf_size, buffer: vec![0; buf_size] }
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
