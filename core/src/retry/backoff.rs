use crate::model::{RetryReason, RequestContext};

pub fn retry_delay_ms(
    _ctx: &RequestContext,
    reason: RetryReason,
) -> u32 {
    match reason {
        RetryReason::NetworkError => 200,
        RetryReason::Timeout => 500,
        RetryReason::RateLimited => 1500,
        RetryReason::AuthExpired => 0,
    }
}

const MAX_RETRY_AFTER_MS: u32 = 120_000;
pub fn clamp_retry_after(ms: u32) -> u32 {
    ms.min(MAX_RETRY_AFTER_MS)
}