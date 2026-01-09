use crate::{
    auth::AuthDecision,
    auth::AuthState,
    error::{classify_http_status, ErrorCategory},
    model::{Decision, Outcome, RequestContext},
    retry::can_retry,
};

pub fn decide(
    ctx: &RequestContext,
    outcome: Outcome,
    auth_decision: Option<AuthDecision>,
    auth_state: &mut AuthState,
    refresh_result: Option<bool>,
) -> Decision {
    match outcome {
        Outcome::NetworkError | Outcome::TimeoutError => {
            if can_retry(ctx) {
                Decision::Retry
            } else {
                Decision::Fail
            }
        }

        Outcome::HttpStatus(status) => match classify_http_status(status) {
            ErrorCategory::AuthError => {
                // Refresh sudah pernah dicoba → fail (no infinite loop)
                if auth_state.refresh_attempted {
                    return Decision::Fail;
                }
            
                match auth_decision {
                    Some(AuthDecision::RefreshAndRetry) => {
                        // Tandai bahwa refresh sudah dicoba
                        auth_state.refresh_attempted = true;
                        auth_state.refresh_in_progress = true;
            
                        match refresh_result {
                            // Refresh sukses → retry
                            Some(true) => {
                                auth_state.refresh_in_progress = false;
                                if can_retry(ctx) {
                                    Decision::RefreshAndRetry
                                } else {
                                    Decision::Fail
                                }
                            }
            
                            // Refresh gagal → fail
                            Some(false) => {
                                auth_state.refresh_in_progress = false;
                                Decision::Fail
                            }
            
                            // Refresh belum dieksekusi (signal untuk host)
                            None => Decision::RefreshAndRetry,
                        }
                    }
            
                    // Provider tidak mengizinkan refresh
                    _ => Decision::Fail,
                }
            },            

            ErrorCategory::RateLimitError | ErrorCategory::NetworkError => {
                if can_retry(ctx) {
                    Decision::Retry
                } else {
                    Decision::Fail
                }
            }

            _ => Decision::Fail,
        },
    }
}
