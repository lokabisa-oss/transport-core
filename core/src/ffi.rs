use crate::{
    auth::{AuthDecision, AuthState},
    decision::decide,
    model::{Decision, HttpMethod, Outcome, RequestContext},
};

#[repr(C)]
pub struct transport_core_client {
    auth_state: AuthState,
    last_decision: Option<Decision>,
}

#[no_mangle]
pub extern "C" fn tc_client_new() -> *mut transport_core_client {
    Box::into_raw(Box::new(transport_core_client {
        auth_state: AuthState::new(),
        last_decision: None,
    }))
}

#[no_mangle]
pub extern "C" fn tc_client_free(ptr: *mut transport_core_client) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/* ============================
 * tc_decide (ABI entrypoint)
 * ============================ */

 #[repr(C)]
 #[allow(non_camel_case_types)]
pub enum tc_decision_t {
    TC_DECISION_PROCEED = 0,
    TC_DECISION_RETRY = 1,
    TC_DECISION_REFRESH_AND_RETRY = 2,
    TC_DECISION_FAIL = 3,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum tc_auth_decision_t {
    TC_AUTH_REFRESH_AND_RETRY = 0,
    TC_AUTH_FAIL = 1,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum tc_http_method_t {
    TC_HTTP_GET = 0,
    TC_HTTP_POST,
    TC_HTTP_PUT,
    TC_HTTP_DELETE,
    TC_HTTP_HEAD,
    TC_HTTP_OPTIONS,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum tc_outcome_kind_t {
    TC_OUTCOME_NETWORK_ERROR = 0,
    TC_OUTCOME_TIMEOUT_ERROR,
    TC_OUTCOME_HTTP_STATUS,

    // NEW semantic outcomes
    TC_OUTCOME_RATE_LIMITED,
    TC_OUTCOME_BLOCKED,
    TC_OUTCOME_CAPTCHA,
}

#[repr(C)]
pub struct tc_request_context_t {
    pub method: tc_http_method_t,
    pub attempt: u8,
    pub max_attempts: u8,
    pub allow_non_idempotent_retry: bool,
    pub idempotency_key: *const std::os::raw::c_char,
}

#[repr(C)]
pub struct tc_outcome_t {
    pub kind: tc_outcome_kind_t,
    pub http_status: u16,
    pub retry_after_ms: u32,
}


#[no_mangle]
pub extern "C" fn tc_decide(
    client: *mut transport_core_client,
    ctx: *const tc_request_context_t,
    outcome: *const tc_outcome_t,
    auth_decision: tc_auth_decision_t,
    refresh_result: i8,
) -> tc_decision_t {
    // Basic safety
    if client.is_null() || ctx.is_null() || outcome.is_null() {
        return tc_decision_t::TC_DECISION_FAIL;
    }

    let client = unsafe { &mut *client };
    let ctx = unsafe { &*ctx };
    let outcome = unsafe { &*outcome };

    // Map HTTP method
    let method = match ctx.method {
        tc_http_method_t::TC_HTTP_GET => HttpMethod::GET,
        tc_http_method_t::TC_HTTP_POST => HttpMethod::POST,
        tc_http_method_t::TC_HTTP_PUT => HttpMethod::PUT,
        tc_http_method_t::TC_HTTP_DELETE => HttpMethod::DELETE,
        tc_http_method_t::TC_HTTP_HEAD => HttpMethod::HEAD,
        tc_http_method_t::TC_HTTP_OPTIONS => HttpMethod::OPTIONS,
    };

    let req_ctx = RequestContext {
        method,
        attempt: ctx.attempt,
        max_attempts: ctx.max_attempts,
        idempotency_key: None,
        allow_non_idempotent_retry: ctx.allow_non_idempotent_retry,
    };

    let rust_outcome = match outcome.kind {
        tc_outcome_kind_t::TC_OUTCOME_NETWORK_ERROR => Outcome::NetworkError,
        tc_outcome_kind_t::TC_OUTCOME_TIMEOUT_ERROR => Outcome::TimeoutError,
        tc_outcome_kind_t::TC_OUTCOME_HTTP_STATUS => Outcome::HttpStatus(outcome.http_status),

        tc_outcome_kind_t::TC_OUTCOME_RATE_LIMITED => {
            let hint = if outcome.retry_after_ms == 0 {
                None
            } else {
                Some(outcome.retry_after_ms)
            };
            Outcome::RateLimited { retry_after_ms: hint }
        }
        tc_outcome_kind_t::TC_OUTCOME_BLOCKED => Outcome::Blocked,
        tc_outcome_kind_t::TC_OUTCOME_CAPTCHA => Outcome::Captcha,
    };

    let auth_decision = match auth_decision {
        tc_auth_decision_t::TC_AUTH_REFRESH_AND_RETRY => Some(AuthDecision::RefreshAndRetry),
        tc_auth_decision_t::TC_AUTH_FAIL => None,
    };

    let refresh_result = match refresh_result {
        1 => Some(true),
        0 => Some(false),
        _ => None,
    };

    let decision = decide(
        &req_ctx,
        rust_outcome,
        auth_decision,
        &mut client.auth_state,
        refresh_result,
    );

    client.last_decision = Some(decision.clone());

    match decision {
        Decision::Proceed => tc_decision_t::TC_DECISION_PROCEED,
    
        Decision::Retry { .. } =>
            tc_decision_t::TC_DECISION_RETRY,
    
        Decision::RefreshAndRetry { .. } =>
            tc_decision_t::TC_DECISION_REFRESH_AND_RETRY,
    
        Decision::Fail { .. } =>
            tc_decision_t::TC_DECISION_FAIL,
    }    
}

#[no_mangle]
pub extern "C" fn tc_last_retry_after_ms(
    client: *const transport_core_client,
) -> u32 {
    if client.is_null() {
        return 0;
    }

    let client = unsafe { &*client };

    match &client.last_decision {
        Some(Decision::Retry { after_ms, .. }) => *after_ms,
        Some(Decision::RefreshAndRetry { after_ms }) => *after_ms,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn tc_last_retry_reason(
    client: *const transport_core_client,
) -> u8 {
    if client.is_null() {
        return 0;
    }

    let client = unsafe { &*client };

    match &client.last_decision {
        Some(Decision::Retry { reason, .. }) => *reason as u8,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn tc_last_fail_reason(
    client: *const transport_core_client,
) -> u8 {
    if client.is_null() {
        return 0;
    }

    let client = unsafe { &*client };

    match &client.last_decision {
        Some(Decision::Fail { reason, .. }) => *reason as u8,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn tc_last_fail_retryable(
    client: *const transport_core_client,
) -> bool {
    if client.is_null() {
        return false;
    }

    let client = unsafe { &*client };

    match &client.last_decision {
        Some(Decision::Fail { retryable, .. }) => *retryable,
        _ => false,
    }
}
