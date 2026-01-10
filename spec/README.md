# transport-core — Specification v1

This directory defines the **canonical behavior** of `transport-core`.

All implementations (Rust, WASM, Python bindings, or reimplementations)
**MUST follow the rules defined here**.

The Rust implementation is the reference implementation,  
but **behavior is defined by this specification — not by code**.

---

## 1. Purpose

`transport-core` is a **policy engine** that decides **what should happen next**
for a request, **without performing IO** and **without interpreting HTTP directly**.

It is designed to be:

- deterministic
- language-agnostic
- WASM-safe
- suitable for retry, auth-refresh, and failure decisions

---

## 2. Core Principles (Frozen)

1. **Semantic-first**
   - Retry decisions are based on **semantic outcomes**, not raw HTTP status codes.
2. **Policy-only**
   - No HTTP requests, no sleeping, no timers, no threads.
3. **Host responsibility**
   - The host maps runtime errors or HTTP responses into semantic `Outcome`.
4. **Deterministic**
   - Same input always yields the same decision.
5. **ABI-safe**
   - All decision details must be retrievable via FFI getters.

---

## 3. Input Model

### 3.1 RequestContext

```text
RequestContext {
  method: HttpMethod
  attempt: u8
  max_attempts: u8
  idempotency_key: Option<String>
  allow_non_idempotent_retry: bool
}
```

Notes:

- attempt is not incremented by the core
- Attempt management is the host’s responsibility

### 3.2 Outcome (Primary Input)

```text
Outcome =
  NetworkError
  TimeoutError
  RateLimited { retry_after_ms?: u32 }
  Blocked
  Captcha
  HttpStatus(u16)   // legacy fallback
```

Rules:

- Outcome::HttpStatus is legacy fallback only
- Retry is NOT guaranteed for raw HTTP status
- Hosts SHOULD prefer semantic outcomes

---

## 4. Output Model

### 4.1 Decision

```text
Decision =
  Proceed
  Retry { after_ms: u32, reason: RetryReason }
  RefreshAndRetry { after_ms: u32 }
  Fail { reason: FailReason, retryable: bool }
```

---

## 5. Decision Rules

### 5.1 Retryable Outcomes

| Outcome      | Decision                            |
| ------------ | ----------------------------------- |
| NetworkError | Retry                               |
| TimeoutError | Retry                               |
| RateLimited  | Retry (may use retry_after_ms hint) |

Retry only occurs if:

- `attempt < max_attempts`
- retry policy allows it

### 5.2 Non-Retryable Outcomes

| Outcome | Decision           |
| ------- | ------------------ |
| Blocked | Fail (HardBlocked) |
| Captcha | Fail (HardBlocked) |

### 5.3 Auth (Legacy via HttpStatus)

- 401:
  - if AuthDecision::RefreshAndRetry:
    - refresh not yet attempted → RefreshAndRetry
    - refresh failed or already attempted → Fail(AuthFailed)
- 403 → Fail
- other statuses → Fail

---

## 6. HTTP Status Semantics

| HTTP Status | Retry Guaranteed?                    |
| ----------- | ------------------------------------ |
| 500         | ❌ No                                |
| 429         | ❌ No (unless mapped to RateLimited) |
| 401         | ❌ (auth flow only)                  |

> Raw HTTP status codes do NOT determine retry behavior.

---

## 7. Retry-After Hint

- retry_after_ms:

  - only exists on Outcome::RateLimited
  - is a hint, not a command

- core may:
  - use it
  - clamp it
  - ignore it

---

## 8. Non-Idempotent Requests

- `allow_non_idempotent_retry = true` does not guarantee retry
- Retry still requires:
  1. A retryable semantic Outcome
  2. Retry policy approval

---

## 9. FFI Contract (Summary)

- tc_decide() returns a coarse decision enum
- Detailed decision data is accessed via getters:
  - retry delay
  - retry reason
  - fail reason
  - retryable flag

Decision payloads are never returned directly.

---

## 10. Explicit Non-Goals

`transport-core` does NOT:

- parse HTTP headers
- parse Retry-After headers
- perform sleeps or delays
- increment attempts
- manage per-domain state
- perform logging

---

## 11. Test Vector Rules

- Retry test vectors MUST use:
  - `NetworkError`
  - `TimeoutError`
  - `RateLimited`
- `HttpStatus` is only for:
  - auth flow
  - hard failures
- Attempt progression is not tested in core vectors

---

## 12. Versioning

- _Specification version_: v1 (frozen)
- Breaking changes require v2
- Additive extensions are allowed in minor versions

---

## 13. Summary Philosophy

> Retry is about semantics, not status codes.
> The core decides what; the host decides how.
