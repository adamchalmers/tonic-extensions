use hyper::{Body, Request, Uri};
use slog::{o, Logger};
use uuid::Uuid;

/// Information extracted from each request
pub struct RequestContext {
    pub user_agent: String,
    pub endpoint: Uri,
    pub request_id: Uuid,
}

impl RequestContext {
    /// Move the context fields from this value into the logger.
    pub fn into_logger(self, log: Logger) -> Logger {
        log.new(o!(
            "http.user_agent" => self.user_agent,
            "uri" => self.endpoint.to_string(),
            "request_id" => self.request_id.to_string(),
        ))
    }
}

/// You can create a RequestContext by examining a Hyper HTTP request.
impl From<&hyper::Request<Body>> for RequestContext {
    fn from(req: &Request<Body>) -> Self {
        let user_agent = req
            .headers()
            .get(hyper::header::USER_AGENT)
            .and_then(|ua| ua.to_str().ok())
            .unwrap_or("unknown")
            .to_owned();

        Self {
            user_agent,
            endpoint: req.uri().clone(),
            request_id: Uuid::new_v4(),
        }
    }
}
