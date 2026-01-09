use crate::model::{HttpMethod, RequestContext};

pub fn is_idempotent(method: &HttpMethod) -> bool {
    matches!(
        method,
        HttpMethod::GET
            | HttpMethod::HEAD
            | HttpMethod::PUT
            | HttpMethod::DELETE
            | HttpMethod::OPTIONS
    )
}

pub fn can_retry(ctx: &RequestContext) -> bool {
    if ctx.attempt >= ctx.max_attempts {
        return false;
    }

    if is_idempotent(&ctx.method) {
        return true;
    }

    ctx.idempotency_key.is_some() && ctx.allow_non_idempotent_retry
}
