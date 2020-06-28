use std::io::{self, Cursor};
use std::pin::Pin;
use std::task::{Poll, Context};

use futures::{ready, Future, future::BoxFuture, stream::Stream};
use tokio::io::{AsyncRead, AsyncReadExt as _};

use crate::http::hyper;
use hyper::{Bytes, HttpBody};

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

        match Pin::new(inner).poll_read(cx, &mut buffer[..]) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Ok(n)) if n == 0 => Poll::Ready(None),
            Poll::Ready(Ok(n)) => {
                let mut next = std::mem::replace(buffer, vec![0; buf_size]);
                next.truncate(n);
                Poll::Ready(Some(Ok(Bytes::from(next))))
            }
        }
    }
}

pub trait AsyncReadExt: AsyncRead {
    fn into_bytes_stream(self, buf_size: usize) -> IntoBytesStream<Self> where Self: Sized {
        IntoBytesStream { inner: self, buf_size, buffer: vec![0; buf_size] }
    }

    fn read_max<'a>(&'a mut self, mut buf: &'a mut [u8]) -> BoxFuture<'_, io::Result<usize>>
        where Self: Send + Unpin
    {
        Box::pin(async move {
            let start_len = buf.len();
            while !buf.is_empty() {
                match self.read(buf).await {
                    Ok(0) => break,
                    Ok(n) => buf = &mut buf[n..],
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }

            Ok(start_len - buf.len())
        })
    }
}

impl<T: AsyncRead> AsyncReadExt for T { }

pub struct AsyncReadBody {
    inner: hyper::Body,
    state: AsyncReadBodyState,
}

enum AsyncReadBodyState {
    Pending,
    Partial(Cursor<Bytes>),
    Done,
}

impl AsyncReadBody {
    pub fn empty() -> Self {
        Self { inner: hyper::Body::empty(), state: AsyncReadBodyState::Done }
    }
}

impl From<hyper::Body> for AsyncReadBody {
    fn from(body: hyper::Body) -> Self {
        Self { inner: body, state: AsyncReadBodyState::Pending }
    }
}

impl AsyncRead for AsyncReadBody {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        loop {
            match self.state {
                AsyncReadBodyState::Pending => {
                    match ready!(Pin::new(&mut self.inner).poll_data(cx)) {
                        Some(Ok(bytes)) => self.state = AsyncReadBodyState::Partial(Cursor::new(bytes)),
                        Some(Err(e)) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
                        None => self.state = AsyncReadBodyState::Done,
                    }
                },
                AsyncReadBodyState::Partial(ref mut cursor) => {
                    match ready!(Pin::new(cursor).poll_read(cx, buf)) {
                        Ok(n) if n == 0 => {
                            self.state = AsyncReadBodyState::Pending;
                        }
                        Ok(n) => return Poll::Ready(Ok(n)),
                        Err(e) => return Poll::Ready(Err(e)),
                    }
                }
                AsyncReadBodyState::Done => return Poll::Ready(Ok(0)),
            }
        }
    }
}

// The code below was adapted from the `replace_with` crate and reproduced here
// under the rights granted by the MIT license. The code is copyright the
// `replace_with` developers. See LICENSE-MIT for the full text.

struct OnUnwind<F: FnOnce()>(std::mem::ManuallyDrop<F>);

impl<F: FnOnce()> Drop for OnUnwind<F> {
    #[inline(always)]
    fn drop(&mut self) {
        (unsafe { std::ptr::read(&*self.0) })();
    }
}

#[inline(always)]
pub async fn async_on_unwind<F, Fut, T, P: FnOnce()>(f: F, p: P) -> T
    where F: FnOnce() -> Fut, Fut: Future<Output = T>,
{
    let x = OnUnwind(std::mem::ManuallyDrop::new(p));
    let t = f().await;
    let _ = unsafe { std::ptr::read(&*x.0) };
    std::mem::forget(x);
    t
}

#[inline]
pub async fn async_replace_with_or_else<T, Fut, F, D>(dest: &mut T, d: D, f: F)
    where Fut: Future<Output = T>,
          F: FnOnce(T) -> Fut,
          D: FnOnce() -> T,
{
    unsafe {
        let old = std::ptr::read(dest);
        let new = async_on_unwind(
            || async move { f(old).await },
            || std::ptr::write(dest, d()),
        ).await;

        std::ptr::write(dest, new);
    }
}


#[inline]
pub async fn async_replace_with<T, Fut, F>(dest: &mut T, f: F)
    where Fut: Future<Output = T>,
          F: FnOnce(T) -> Fut,
{
    async_replace_with_or_else(dest, || std::process::abort(), f).await
}
