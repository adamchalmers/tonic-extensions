use crate::{request_context::RequestContext, Name};
use hyper::Body;
use slog::{info, Logger};
use std::{
    task::{Context, Poll},
    time::Instant,
};
use tonic::body::BoxBody;
use tower::{Layer, Service};

// -----
// Begin tower boilerplate that I don't really understand
// -----
#[derive(Clone)]
pub struct LoggingLayer {
    pub log: Logger,
}

impl<S> Layer<S> for LoggingLayer {
    type Service = Logging<S>;

    fn layer(&self, service: S) -> Self::Service {
        Logging {
            inner: service,
            log: self.log.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Logging<S> {
    inner: S,
    log: Logger,
}

impl<S> Service<hyper::Request<Body>> for Logging<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    // -----
    // End tower boilerplate that I don't really understand,
    // Begin my logging logic.
    // -----

    fn call(&mut self, mut req: hyper::Request<Body>) -> Self::Future {
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let req_ctx = RequestContext::from(&req);
        let logger = req_ctx.into_logger(self.log.clone());
        req.extensions_mut().insert(logger.clone());
        Box::pin(async move {
            // call the service
            let started = Instant::now();
            let response: Self::Response = inner.call(req).await?;
            let elapsed = started.elapsed();
            let name = &response.extensions().get::<Name>().unwrap().0;
            info!(logger, "handled a request";
                "microseconds" => elapsed.as_micros(),
                "http.status" => %response.status(),
                "name" => name,
            );
            Ok(response)
        })
    }
}
