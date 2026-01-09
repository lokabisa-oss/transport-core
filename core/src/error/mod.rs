#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    NetworkError,
    TimeoutError,
    AuthError,
    RateLimitError,
    FatalError,
    UnknownError,
}

pub fn classify_http_status(status: u16) -> ErrorCategory {
    match status {
        401 => ErrorCategory::AuthError,
        429 => ErrorCategory::RateLimitError,
        500..=599 => ErrorCategory::NetworkError,
        400 | 403 | 404 | 409 | 422 => ErrorCategory::FatalError,
        _ => ErrorCategory::UnknownError,
    }
}
