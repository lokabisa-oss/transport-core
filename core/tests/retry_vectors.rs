use serde::Deserialize;
use std::fs;


use transport_core::{
    auth::AuthDecision,
    auth::AuthState,
    decision::decide,
    model::{Decision, HttpMethod, Outcome, RequestContext},
};

#[derive(Debug, Deserialize)]
struct RetryTestFile {
    cases: Vec<RetryTestCase>,
}

#[derive(Debug, Deserialize)]
struct RetryTestCase {
    name: String,
    input: RetryInput,
    expected: RetryExpected,
}

#[derive(Debug, Deserialize)]
struct RetryInput {
    method: String,
    attempt: u8,
    status: Option<u16>,
    error: Option<String>,
    retry_after_ms: Option<u32>,
    idempotency_key: Option<String>,
    allow_non_idempotent_retry: Option<bool>,
    auth_decision: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RetryExpected {
    action: String,
    next_attempt: Option<u8>,
}

fn parse_method(m: &str) -> HttpMethod {
    match m {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "DELETE" => HttpMethod::DELETE,
        "HEAD" => HttpMethod::HEAD,
        "OPTIONS" => HttpMethod::OPTIONS,
        _ => panic!("unknown method: {}", m),
    }
}

fn parse_auth_decision(d: &Option<String>) -> Option<AuthDecision> {
    match d.as_deref() {
        Some("RefreshAndRetry") => Some(AuthDecision::RefreshAndRetry),
        Some("Fail") => None,
        None => None,
        Some(v) => panic!("unknown auth decision: {}", v),
    }
}

fn parse_outcome(input: &RetryInput) -> Outcome {
    if let Some(err) = &input.error {
        return match err.as_str() {
            "NetworkError" => Outcome::NetworkError,
            "TimeoutError" => Outcome::TimeoutError,
            "RateLimited" => Outcome::RateLimited {
                retry_after_ms: input.retry_after_ms,
            },
            "Blocked" => Outcome::Blocked,
            "Captcha" => Outcome::Captcha,
            _ => panic!("unknown error: {}", err),
        };
    }

    if let Some(status) = input.status {
        return Outcome::HttpStatus(status);
    }

    panic!("invalid test vector input");
}

#[test]
fn retry_vectors_should_match_spec() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../spec/test-vectors/retry.json");

    let raw = fs::read_to_string(&path)
        .expect("failed to read retry.json");

    let vectors: RetryTestFile =
        serde_json::from_str(&raw).expect("invalid retry.json format");

    for case in vectors.cases {
        let ctx = RequestContext {
            method: parse_method(&case.input.method),
            attempt: case.input.attempt,
            max_attempts: 3,
            idempotency_key: case.input.idempotency_key.clone(),
            allow_non_idempotent_retry: case
                .input
                .allow_non_idempotent_retry
                .unwrap_or(false),
        };

        let outcome = parse_outcome(&case.input);
        let auth_decision = parse_auth_decision(&case.input.auth_decision);

        let mut auth_state = AuthState::new();
        
        let decision = decide(
            &ctx,
            outcome,
            auth_decision,
            &mut auth_state,
            None,
        );

        let actual_action = decision_action(&decision);

        assert_eq!(
            actual_action,
            case.expected.action,
            "retry test failed: {}",
            case.name
        );

        if case.expected.action == "FAIL" {
            if let Decision::Fail { retryable, .. } = decision {
                assert!(!retryable, "FAIL must not be retryable");
            }
        }
    }
}


fn decision_action(decision: &Decision) -> &'static str {
    match decision {
        Decision::Proceed => "PROCEED",
        Decision::Retry { .. } => "RETRY",
        Decision::RefreshAndRetry { .. } => "REFRESH_AND_RETRY",
        Decision::Fail { .. } => "FAIL",
    }
}