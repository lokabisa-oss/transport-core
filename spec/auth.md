# Authentication Behavior Specification

This document defines the canonical authentication behavior for `transport-core`.

All implementations (native, WASM, and reimplementations such as Go) MUST follow
the rules defined in this document exactly.

Authentication behavior is defined as a deterministic state machine and MUST
integrate with retry semantics.

---

## 1. Scope

This specification defines:

- How authentication data is applied to requests
- How authentication failures are handled
- How token refresh is coordinated
- How concurrent requests interact with authentication state

This specification does NOT define:

- Credential formats
- Token storage format
- Specific authentication protocols (OAuth, API key, etc.)

---

## 2. Authentication Provider Contract

An Authentication Provider MUST expose the following behaviors:

- Apply authentication data to an outgoing request
- Decide whether a request is eligible for refresh on authentication failure
- Perform a refresh operation when requested

The transport layer MUST treat the provider as a black box and MUST NOT
inspect or modify credentials directly.

---

## 3. Authentication Application

Before a request is sent:

- The client MUST invoke the authentication provider to apply credentials
- Authentication data MUST be applied exactly once per attempt
- Authentication data MUST NOT be mutated during request transmission

Failure to apply authentication data MUST cause the request to fail immediately.

---

## 4. Unauthorized Response Handling (HTTP 401)

When an HTTP 401 response is received:

1. The client MUST consult the authentication provider
2. The provider MUST return one of the following decisions:
   - `RefreshAndRetry`
   - `Fail`

If the provider returns `Fail`, the request MUST fail immediately.

---

## 5. Token Refresh Flow

If the provider returns `RefreshAndRetry`:

1. The client MUST initiate a token refresh operation
2. The original request MUST be paused
3. If refresh succeeds:
   - The original request MUST be retried
4. If refresh fails:
   - The original request MUST fail

The refresh operation MUST complete before any dependent retries occur.

---

## 6. Refresh Concurrency Rules

At most **one** refresh operation MAY be in progress at any time.

If multiple requests encounter HTTP 401 concurrently:

- Only one request MAY perform the refresh
- All other requests MUST wait for the refresh result
- After refresh completes:
  - If successful, waiting requests MUST retry
  - If failed, waiting requests MUST fail

No request MAY initiate a second refresh while one is already in progress.

---

## 7. Refresh Attempt Limits

- A refresh operation MUST be attempted at most once per request lifecycle
- If a request receives HTTP 401 again after a successful refresh:
  - The request MUST fail
  - No further refresh attempts are allowed

This rule prevents infinite refresh loops.

---

## 8. Interaction with Retry Semantics

Authentication refresh is orthogonal to retry attempts.

Rules:

- A refresh-triggered retry DOES count as a retry attempt
- Retry limits defined in `retry.md` MUST still be enforced
- If retry limits are exhausted, the request MUST fail even if refresh succeeds

---

## 9. Non-401 Authentication Errors

The client MUST NOT attempt refresh for:

- HTTP 403 responses
- Malformed authentication data
- Explicit authentication failure responses

These errors MUST fail immediately.

---

## 10. State Isolation

- Authentication state MUST be shared across requests within the same client instance
- Authentication state MUST NOT be shared across different client instances
- Request-specific data MUST NOT leak into authentication state

---

## 11. Observability Requirements

Authentication events MUST emit observable metadata, including:

- Refresh start
- Refresh success
- Refresh failure
- Requests waiting on refresh

No sensitive credential data MAY be exposed in observability output.

---

## 12. Determinism Guarantee

Given the same sequence of requests and responses, authentication behavior
MUST be deterministic.

Race conditions, duplicate refreshes, and non-deterministic outcomes
are considered specification violations.

---

## 13. Compliance

An implementation is considered compliant if:

- It enforces single-flight refresh behavior
- It prevents infinite refresh loops
- It integrates correctly with retry limits
- It produces identical authentication decisions for identical inputs

The Rust implementation serves as a reference, but behavior is defined by this specification.
