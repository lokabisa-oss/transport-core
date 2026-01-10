use serde::Deserialize;
use std::fs;
use std::path::Path;

use transport_core::{
    auth::AuthDecision,
    auth::AuthState,
    decision::decide,
    model::{Decision, FailReason, HttpMethod, Outcome, RequestContext},
};

#[derive(Debug, Deserialize)]
struct AuthTestFile {
    cases: Vec<AuthTestCase>,
}

#[derive(Debug, Deserialize)]
struct AuthTestCase {
    name: String,
    input: AuthInput,
    expected: AuthExpected,
}

#[derive(Debug, Deserialize)]
struct AuthInput {
    attempt: Option<u8>,
    status: Option<u16>,
    auth_decision: Option<String>,
    refresh_result: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthExpected {
    action: Option<String>,
    error_category: Option<String>,
}

fn parse_auth_decision(d: &Option<String>) -> Option<AuthDecision> {
    match d.as_deref() {
        Some("RefreshAndRetry") => Some(AuthDecision::RefreshAndRetry),
        Some("Fail") => None,
        None => None,
        Some(v) => panic!("unknown auth decision: {}", v),
    }
}

#[test]
fn auth_vectors_should_match_spec() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/test-vectors/auth.json");

    let raw = fs::read_to_string(&path).expect("failed to read auth.json");

    let vectors: AuthTestFile = serde_json::from_str(&raw).expect("invalid auth.json format");

    for case in vectors.cases {
        // NOTE:
        // Untuk tahap ini, kita fokus pada auth-triggered decisions.
        // RequestContext disederhanakan.
        let ctx = RequestContext {
            method: HttpMethod::GET,
            attempt: case.input.attempt.unwrap_or(1),
            max_attempts: 3,
            idempotency_key: None,
            allow_non_idempotent_retry: false,
        };

        let outcome = match case.input.status {
            Some(status) => Outcome::HttpStatus(status),
            None => continue, // cases like apply_auth_before_request (handled later)
        };

        let auth_decision = parse_auth_decision(&case.input.auth_decision);

        let mut auth_state = AuthState::new();

        let decision = decide(
            &ctx,
            outcome,
            auth_decision,
            &mut auth_state,
            match case.input.refresh_result.as_deref() {
                Some("SUCCESS") => Some(true),
                Some("FAILURE") => Some(false),
                _ => None,
            },
        );

        if let Some(action) = &case.expected.action {
            let actual_action = decision_action(&decision);

            assert_eq!(actual_action, action, "auth test failed: {}", case.name);
        }

        // Optional: validate error category if provided
        if let Some(cat) = &case.expected.error_category {
            assert!(
                decision_action(&decision) == "FAIL",
                "expected FAIL for error category {} in case {}",
                cat,
                case.name
            );
        }

        if let Some(cat) = &case.expected.error_category {
            if let Decision::Fail { reason, .. } = &decision {
                match cat.as_str() {
                    "AuthError" => {
                        assert!(
                            matches!(reason, FailReason::AuthFailed),
                            "expected AuthFailed in case {}",
                            case.name
                        );
                    }
                    _ => {}
                }
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
