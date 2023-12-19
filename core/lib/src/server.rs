use std::io;
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder;
use futures::{Future, TryFutureExt, future::{select, Either::*}};
use tokio::time::sleep;

use crate::{Request, Rocket, Orbit, Data, Ignite};
use crate::request::ConnectionMeta;
use crate::erased::{ErasedRequest, ErasedResponse, ErasedIoHandler};
use crate::listener::{Listener, CancellableExt, BouncedExt};
use crate::error::{Error, ErrorKind};
use crate::data::IoStream;
use crate::util::ReaderStream;
use crate::http::Status;

impl Rocket<Orbit> {
    async fn service(
        self: Arc<Self>,
        mut req: hyper::Request<hyper::body::Incoming>,
        connection: ConnectionMeta,
    ) -> Result<hyper::Response<ReaderStream<ErasedResponse>>, http::Error> {
        let upgrade = hyper::upgrade::on(&mut req);
        let (parts, incoming) = req.into_parts();
        let request = ErasedRequest::new(self, parts, |rocket, parts| {
            Request::from_hyp(rocket, parts, connection).unwrap_or_else(|e| e)
        });

        let mut response = request.into_response(
            incoming,
            |incoming| Data::from(incoming),
            |rocket, request, data| Box::pin(rocket.preprocess(request, data)),
            |token, rocket, request, data| Box::pin(async move {
                if !request.errors.is_empty() {
                    return rocket.dispatch_error(Status::BadRequest, request).await;
                }

                let mut response = rocket.dispatch(token, request, data).await;
                response.body_mut().size().await;
                response
            })
        ).await;

        let io_handler = response.to_io_handler(Rocket::extract_io_handler);
        if let Some(handler) = io_handler {
            let upgrade = upgrade.map_ok(IoStream::from).map_err(io::Error::other);
            tokio::task::spawn(io_handler_task(upgrade, handler));
        }

        let mut builder = hyper::Response::builder();
        builder = builder.status(response.inner().status().code);
        for header in response.inner().headers().iter() {
            builder = builder.header(header.name().as_str(), header.value());
        }

        if let Some(size) = response.inner().body().preset_size() {
            builder = builder.header("Content-Length", size);
        }

        let chunk_size = response.inner().body().max_chunk_size();
        builder.body(ReaderStream::with_capacity(response, chunk_size))
    }
}

async fn io_handler_task<S>(stream: S, mut handler: ErasedIoHandler)
    where S: Future<Output = io::Result<IoStream>>
{
    let stream = match stream.await {
        Ok(stream) => stream,
        Err(e) => return warn_!("Upgrade failed: {e}"),
    };

    info_!("Upgrade succeeded.");
    if let Err(e) = handler.take().io(stream).await {
        match e.kind() {
            io::ErrorKind::BrokenPipe => warn!("Upgrade I/O handler was closed."),
            e => error!("Upgrade I/O handler failed: {e}"),
        }
    }
}

impl Rocket<Ignite> {
    pub(crate) async fn serve<L>(self, listener: L) -> Result<Self, crate::Error>
        where L: Listener + 'static
    {
        let mut builder = Builder::new(TokioExecutor::new());
        let keep_alive = Duration::from_secs(self.config.keep_alive.into());
        builder.http1()
            .half_close(true)
            .timer(TokioTimer::new())
            .keep_alive(keep_alive > Duration::ZERO)
            .preserve_header_case(true)
            .header_read_timeout(Duration::from_secs(15));

        #[cfg(feature = "http2")] {
            builder.http2().timer(TokioTimer::new());
            if keep_alive > Duration::ZERO {
                builder.http2()
                    .timer(TokioTimer::new())
                    .keep_alive_interval(keep_alive / 4)
                    .keep_alive_timeout(keep_alive);
            }
        }

        let listener = listener.bounced().cancellable(self.shutdown(), &self.config.shutdown);
        let rocket = Arc::new(self.into_orbit(listener.socket_addr()?));
        let _ = tokio::spawn(Rocket::liftoff(rocket.clone())).await;

        let (server, listener) = (Arc::new(builder), Arc::new(listener));
        while let Some(accept) = listener.accept_next().await {
            let (listener, rocket, server) = (listener.clone(), rocket.clone(), server.clone());
            tokio::spawn({
                let result = async move {
                    let conn = TokioIo::new(listener.connect(accept).await?);
                    let meta = ConnectionMeta::from(conn.inner());
                    let service = service_fn(|req| rocket.clone().service(req, meta.clone()));
                    let serve = pin!(server.serve_connection_with_upgrades(conn, service));
                    match select(serve, rocket.shutdown()).await {
                        Left((result, _)) => result,
                        Right((_, mut conn)) => {
                            conn.as_mut().graceful_shutdown();
                            conn.await
                        }
                    }
                };

                result.inspect_err(crate::error::log_server_error)
            });
        }

        // Rocket wraps all connections in a `CancellableIo` struct, an internal
        // structure that gracefully closes I/O when it receives a signal. That
        // signal is the `shutdown` future. When the future resolves,
        // `CancellableIo` begins to terminate in grace, mercy, and finally
        // force close phases. Since all connections are wrapped in
        // `CancellableIo`, this eventually ends all I/O.
        //
        // At that point, unless a user spawned an infinite, stand-alone task
        // that isn't monitoring `Shutdown`, all tasks should resolve. This
        // means that all instances of the shared `Arc<Rocket>` are dropped and
        // we can return the owned instance of `Rocket`.
        //
        // Unfortunately, the Hyper `server` future resolves as soon as it has
        // finished processing requests without respect for ongoing responses.
        // That is, `server` resolves even when there are running tasks that are
        // generating a response. So, `server` resolving implies little to
        // nothing about the state of connections. As a result, we depend on the
        // timing of grace + mercy + some buffer to determine when all
        // connections should be closed, thus all tasks should be complete, thus
        // all references to `Arc<Rocket>` should be dropped and we can get back
        // a unique reference.
        info!("Shutting down. Waiting for shutdown fairings and pending I/O...");
        tokio::spawn({
            let rocket = rocket.clone();
            async move { rocket.fairings.handle_shutdown(&*rocket).await }
        });

        let config = &rocket.config.shutdown;
        let wait = Duration::from_micros(250);
        for period in [wait, config.grace(), wait, config.mercy(), wait * 4] {
            if Arc::strong_count(&rocket) == 1 { break }
            sleep(period).await;
        }

        match Arc::try_unwrap(rocket) {
            Ok(rocket) => {
                info!("Graceful shutdown completed successfully.");
                Ok(rocket.into_ignite())
            }
            Err(rocket) => {
                warn!("Shutdown failed: outstanding background I/O.");
                Err(Error::new(ErrorKind::Shutdown(rocket)))
            }
        }
    }
}
