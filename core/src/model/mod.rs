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
    // Transport-level
    NetworkError,
    TimeoutError,

    // Semantic (preferred)
    RateLimited { retry_after_ms: Option<u32> },
    Blocked,
    Captcha,

    // Backward compatibility
    HttpStatus(u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Proceed,
    Retry { after_ms: u32, reason: RetryReason },

    RefreshAndRetry { after_ms: u32 },

    Fail { reason: FailReason, retryable: bool },
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryReason {
    NetworkError = 1,
    Timeout = 2,
    RateLimited = 3,
    AuthExpired = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailReason {
    MaxAttemptsExceeded = 1,
    AuthFailed = 2,
    HardBlocked = 3,
    Unknown = 255,
}
