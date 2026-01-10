use crate::{
    auth::{AuthDecision, AuthState},
    error::{classify_http_status, ErrorCategory},
    model::{
        Decision,
        Outcome,
        RequestContext,
        RetryReason,
        FailReason,
    },
    retry::{
        can_retry,
        retry_delay_ms,
        clamp_retry_after,
    },
};

pub fn decide(
    ctx: &RequestContext,
    outcome: Outcome,
    auth_decision: Option<AuthDecision>,
    auth_state: &mut AuthState,
    refresh_result: Option<bool>,
) -> Decision {
    match outcome {
        Outcome::RateLimited { retry_after_ms } => {
            if can_retry(ctx) {
                let base = retry_delay_ms(ctx, RetryReason::RateLimited);
                let after_ms = match retry_after_ms {
                    Some(ms) => clamp_retry_after(ms),
                    None => base,
                };

                Decision::Retry {
                    after_ms,
                    reason: RetryReason::RateLimited,
                }
            } else {
                Decision::Fail {
                    reason: FailReason::MaxAttemptsExceeded,
                    retryable: false,
                }
            }
        }
    
        Outcome::Blocked | Outcome::Captcha => {
            Decision::Fail {
                reason: FailReason::HardBlocked,
                retryable: false,
            }
        }

        Outcome::NetworkError => {
            if can_retry(ctx) {
                Decision::Retry {
                    after_ms: retry_delay_ms(ctx, RetryReason::NetworkError),
                    reason: RetryReason::NetworkError,
                }
            } else {
                Decision::Fail {
                    reason: FailReason::MaxAttemptsExceeded,
                    retryable: false,
                }
            }
        }

        Outcome::TimeoutError => {
            if can_retry(ctx) {
                Decision::Retry {
                    after_ms: retry_delay_ms(ctx, RetryReason::Timeout),
                    reason: RetryReason::Timeout,
                }
            } else {
                Decision::Fail {
                    reason: FailReason::MaxAttemptsExceeded,
                    retryable: false,
                }
            }
        }

        // NOTE:
        // HttpStatus is a legacy fallback.
        // Prefer semantic Outcome (RateLimited, Blocked, Captcha)
        // from host environments when possible.
        Outcome::HttpStatus(status) => match classify_http_status(status) {
            ErrorCategory::AuthError => {
                if auth_state.refresh_attempted {
                    return Decision::Fail {
                        reason: FailReason::AuthFailed,
                        retryable: false,
                    };
                }

                match auth_decision {
                    Some(AuthDecision::RefreshAndRetry) => {
                        auth_state.refresh_attempted = true;
                        auth_state.refresh_in_progress = true;

                        match refresh_result {
                            Some(true) => {
                                auth_state.refresh_in_progress = false;
                                if can_retry(ctx) {
                                    Decision::RefreshAndRetry {
                                        after_ms: retry_delay_ms(ctx, RetryReason::AuthExpired),
                                    }
                                } else {
                                    Decision::Fail {
                                        reason: FailReason::MaxAttemptsExceeded,
                                        retryable: false,
                                    }
                                }
                            }
                            Some(false) => {
                                auth_state.refresh_in_progress = false;
                                Decision::Fail {
                                    reason: FailReason::AuthFailed,
                                    retryable: false,
                                }
                            }
                            None => Decision::RefreshAndRetry {
                                after_ms: 0,
                            },
                        }
                    }
                    _ => Decision::Fail {
                        reason: FailReason::AuthFailed,
                        retryable: false,
                    },
                }
            }

            ErrorCategory::RateLimitError => {
                if can_retry(ctx) {
                    Decision::Retry {
                        after_ms: retry_delay_ms(ctx, RetryReason::RateLimited),
                        reason: RetryReason::RateLimited,
                    }
                } else {
                    Decision::Fail {
                        reason: FailReason::MaxAttemptsExceeded,
                        retryable: false,
                    }
                }
            }

            _ => Decision::Fail {
                reason: FailReason::Unknown,
                retryable: false,
            },
        },
    }
}
