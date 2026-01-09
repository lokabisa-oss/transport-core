use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub method: HttpMethod,
    pub attempt: u8,
    pub max_attempts: u8,
    pub idempotency_key: Option<String>,
    pub allow_non_idempotent_retry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    NetworkError,
    TimeoutError,
    HttpStatus(u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Proceed,
    Retry,
    RefreshAndRetry,
    Fail,
}
