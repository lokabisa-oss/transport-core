# Retry Semantics Specification

This document defines the canonical retry behavior for `transport-core`.

All implementations (native, WASM, and reimplementations such as Go) MUST follow the rules defined in this document exactly.

Retry behavior is defined by outcome and context, not by transport library.

---

## 1. Definitions

### Attempt

An **attempt** is a single transmission of a request over the network.

- The initial request counts as attempt #1
- Each retry increments the attempt counter

### Retryable

A **retryable outcome** is a response or error condition that allows the request to be attempted again.

### Idempotent Request

A request is **idempotent** if repeating it produces the same effect.

The following HTTP methods are considered idempotent by default:

- GET
- HEAD
- PUT
- DELETE
- OPTIONS

All other methods are **non-idempotent** unless explicitly marked otherwise.

---

## 2. Retry Eligibility Rules

### 2.1 Retryable Network Errors

The client MUST retry on the following network-level errors:

- Connection reset
- Connection refused
- DNS resolution failure
- Read timeout
- Write timeout

The client MUST NOT retry on:

- Invalid request construction
- TLS certificate validation failure

---

### 2.2 Retryable HTTP Status Codes

| Status Code | Behavior                                           |
| ----------- | -------------------------------------------------- |
| 500–599     | Retry                                              |
| 429         | Retry (respect `Retry-After`)                      |
| 401         | Conditional retry (see Authentication Interaction) |

---

### 2.3 Non-Retryable HTTP Status Codes

The client MUST NOT retry on:

- 400
- 403
- 404
- 409
- 422

These responses MUST fail immediately.

---

## 3. Idempotency Rules

### 3.1 Default Behavior

Retries MUST only be performed for idempotent requests.

---

### 3.2 Override for Non-Idempotent Requests

A non-idempotent request MAY be retried only if ALL of the following are true:

- An explicit idempotency key is provided
- The transport configuration explicitly allows retries

If these conditions are not met, the request MUST fail immediately.

---

## 4. Retry Limits

### 4.1 Maximum Attempts

- Default maximum attempts: **3**
- This count includes the initial attempt

---

### 4.2 Termination Conditions

The client MUST stop retrying when:

- The maximum number of attempts is reached, OR
- A non-retryable condition is encountered

---

## 5. Backoff Strategy

### 5.1 Backoff Algorithm

The client MUST use exponential backoff.

Formula:

delay = min(base_delay \* (2 ^ attempt), max_delay)

Where:

- `attempt` starts at 1 for the first retry
- `base_delay` default: 100 milliseconds
- `max_delay` default: 2000 milliseconds

---

### 5.2 Retry-After Handling

For HTTP 429 responses:

- If a `Retry-After` header is present:
  - The client MUST wait for the specified duration
- If the header is not present:
  - The client MUST fall back to exponential backoff

---

## 6. Authentication Interaction

Retry behavior MUST integrate with authentication rules.

On receiving HTTP 401:

1. The client MUST consult the authentication provider
2. If the provider returns `RefreshAndRetry`:
   - A token refresh MUST be attempted
   - On success, the original request MUST be retried
3. If refresh fails:
   - The request MUST fail

Only one token refresh operation MAY be in progress at a time.

---

## 7. Concurrency Rules

- Retry counters are scoped per request
- Concurrent requests MUST NOT share retry state
- Authentication refresh state MAY be shared across concurrent requests

---

## 8. Observability Requirements

Each retry MUST emit metadata including:

- Attempt number
- Retry reason
- Backoff duration

These signals MUST be observable by the host runtime.

---

## 9. Determinism Guarantee

Given the same inputs (request, responses, timing), retry behavior MUST be deterministic.

Random jitter MUST NOT be used unless explicitly enabled by configuration.

---

## 10. Compliance

An implementation is considered compliant if:

- It follows all retry eligibility rules
- It respects idempotency constraints
- It produces identical decisions for identical inputs

The Rust implementation serves as a reference, but behavior is defined by this specification.
