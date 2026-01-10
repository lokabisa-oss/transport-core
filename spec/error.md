# Error Mapping — Host-Level Guidance

⚠️ **Non-normative document**

This document provides **host-side guidance** for mapping language-specific,
runtime, or transport errors into semantic `Outcome` values consumed by
`transport-core`.

The **canonical and normative behavior** of error handling and retry decisions
is defined in [`spec/README.md`](./README.md).

---

## 1. Purpose

This document aims to help host implementations achieve:

- consistent semantic error mapping
- deterministic retry behavior
- cross-language behavioral parity

It does NOT define a core error taxonomy.

---

## 2. Core Design Recap

`transport-core`:

- does NOT receive error objects
- does NOT classify language-specific errors
- does NOT understand protocols, TLS, or parsing errors

Instead:

> **Hosts interpret errors.  
> The core interprets semantics.**

---

## 3. Semantic Outcomes (Core Input)

Hosts SHOULD map errors into one of the following semantic outcomes:

```text
Outcome =
  NetworkError
  TimeoutError
  RateLimited { retry_after_ms?: u32 }
  Blocked
  Captcha
  HttpStatus(u16)   // legacy fallback
```

## 4. Recommended Error Mapping

### 4.1 Network-Level Failures

The following SHOULD be mapped to:

```text
Outcome::NetworkError
```

Examples:

- Connection reset
- Connection refused
- DNS resolution failure
- Broken pipe
- Network unreachable

### 4.2 Timeout Failures

The following SHOULD be mapped to:

```text
Outcome::TimeoutError
```

Examples:

- Connection timeout
- Read timeout
- Write timeout
- Request deadline exceeded

### 4.3 Rate Limiting

Rate limiting signals SHOULD be mapped to:

```text
Outcome::RateLimited { retry_after_ms?: u32 }
```

Examples:

- HTTP 429 responses
- Explicit quota exceeded signals
- Gateway throttling

Retry delay hints MAY be extracted by the host.

### 4.4 Blocking and Challenges

The following SHOULD be mapped to hard failure outcomes:

```text
Outcome::Blocked
Outcome::Captcha
```

Examples:

- IP blocking
- WAF rejection
- Bot challenges

These outcomes are never retryable.

### 4.5 Legacy HTTP Status Fallback

If semantic mapping is not possible, hosts MAY use:

```text
Outcome::HttpStatus(u16)
```

Notes:

- Retry is NOT guaranteed for this outcome
- Hosts SHOULD avoid relying on this path

---

## 5. Authentication-Related Errors

Authentication failures SHOULD be mapped based on host logic:

- Authentication expired → `HttpStatus(401)` + `AuthDecision`
- Authorization denied → `HttpStatus(403)`
- Refresh failure → host reports refresh failure to core

The core does NOT inspect authentication errors directly.

## 6. Determinism Guidelines

To ensure deterministic behavior:

- Error-to-outcome mapping MUST be consistent
- The same runtime error MUST map to the same `Outcome`
- Mapping SHOULD NOT depend on timing or environment variance

---

## 7. Observability Recommendations

Hosts SHOULD emit observable metadata including:

- Mapped `Outcome`
- Core `Decision`
- Retry reason (if any)
- Attempt number

Sensitive data MUST NOT be logged.

---

## 8. Explicit Non-Goals

This document does NOT define:

- Language-specific error types
- Error object structures
- Stack traces or debugging formats
- Transport library behavior

---

## 9. Summary

- Error taxonomy lives in the host
- Semantic outcome lives in the core
- Consistent mapping ensures consistent behavior

> **Errors are interpreted outside.  
> Decisions are made inside.**
