# Error Taxonomy Specification

This document defines the canonical error taxonomy for `transport-core`.

All implementations (native, WASM, and reimplementations such as Go) MUST map
internal errors to the categories defined in this document.

Error behavior is defined by category, not by language-specific error types.

---

## 1. Purpose

This specification ensures that all SDKs built on top of `transport-core`
expose consistent error semantics, regardless of programming language
or runtime environment.

The goal is behavioral consistency, not identical error objects.

---

## 2. Error Categories

All errors MUST be mapped to exactly one of the following categories.

### 2.1 NetworkError

Represents failures occurring at the network transport layer.

Includes:

- Connection reset
- Connection refused
- DNS resolution failure
- Broken pipe
- Network unreachable

Excludes:

- Timeouts (see TimeoutError)
- TLS validation failures (see FatalError)

---

### 2.2 TimeoutError

Represents failures caused by exceeding configured time limits.

Includes:

- Connection timeout
- Read timeout
- Write timeout
- Overall request timeout

TimeoutError MAY be retryable depending on retry rules.

---

### 2.3 AuthError

Represents authentication and authorization failures.

Includes:

- HTTP 401 responses
- Failed token refresh
- Missing or invalid authentication credentials

Excludes:

- HTTP 403 responses (see FatalError)

AuthError MAY trigger refresh behavior as defined in `auth.md`.

---

### 2.4 RateLimitError

Represents rate limiting by the remote service.

Includes:

- HTTP 429 responses
- Explicit rate limit signals from headers or metadata

RateLimitError MAY be retryable according to retry rules.

---

### 2.5 FatalError

Represents non-recoverable errors.

Includes:

- HTTP 400, 403, 404, 409, 422
- TLS certificate validation failure
- Invalid request construction
- Protocol violations
- Deserialization errors

FatalError MUST NOT be retried.

---

### 2.6 UnknownError

Represents errors that cannot be classified.

Includes:

- Unexpected internal failures
- Unrecognized error responses

UnknownError MUST be treated as FatalError unless explicitly overridden.

---

## 3. Error Mapping Rules

### 3.1 Single Category Rule

Each error MUST be mapped to exactly one error category.

Errors MUST NOT belong to multiple categories.

---

### 3.2 Language-Specific Errors

Implementations MAY expose language-specific error types, but:

- Each error MUST clearly map to one canonical category
- The canonical category MUST be discoverable programmatically

---

## 4. Interaction with Retry Semantics

Retry behavior is determined by error category:

| Error Category | Retry Allowed |
| -------------- | ------------- |
| NetworkError   | Yes           |
| TimeoutError   | Yes           |
| AuthError      | Conditional   |
| RateLimitError | Yes           |
| FatalError     | No            |
| UnknownError   | No            |

Retry rules defined in `retry.md` MUST still be enforced.

---

## 5. Interaction with Authentication

- AuthError MAY trigger refresh behavior as defined in `auth.md`
- FatalError MUST NOT trigger authentication refresh
- RateLimitError MUST NOT trigger authentication refresh

---

## 6. Error Propagation

When an error is returned to the caller:

- The canonical error category MUST be preserved
- The original error message MAY be included for debugging
- Sensitive information MUST NOT be exposed

---

## 7. Observability Requirements

Error events MUST emit observable metadata, including:

- Error category
- Retry decision (if any)
- Attempt number

Observability output MUST NOT include credentials or secrets.

---

## 8. Determinism Guarantee

Given the same error inputs, error categorization MUST be deterministic.

Non-deterministic or environment-dependent categorization is considered
a specification violation.

---

## 9. Compliance

An implementation is considered compliant if:

- All errors are mapped to a defined category
- Retry behavior matches the category rules
- Authentication interaction follows `auth.md`
- Identical inputs produce identical error categorization

The Rust implementation serves as a reference, but behavior is defined by this specification.
