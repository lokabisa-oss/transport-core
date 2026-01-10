# Retry Semantics — Host-Level Guidance

⚠️ **Non-normative document**

This document provides **host-side guidance** for retry behavior when using
`transport-core`.

The **canonical and normative specification** for retry behavior is defined in
[`spec/README.md`](./README.md).

This document explains **how a host SHOULD map runtime or transport conditions**
into semantic `Outcome` values consumed by `transport-core`.

---

## 1. Scope and Responsibility

### What This Document Is

- Guidance for **host implementations**
- Examples of mapping errors to `Outcome`
- Recommended retry strategies
- Non-binding recommendations

### What This Document Is NOT

- ❌ It is NOT the core specification
- ❌ It does NOT override `spec/README.md`
- ❌ It does NOT define ABI or policy guarantees

---

## 2. Core Design Recap

`transport-core` is:

- semantic-first
- policy-only
- deterministic
- IO-free

Therefore:

> **The host decides the meaning of an error.  
> The core decides what action to take.**

---

## 3. Semantic Outcome Mapping (Host Responsibility)

Hosts SHOULD translate runtime or transport errors into semantic `Outcome` values.

### 3.1 Network-Level Errors

The following conditions SHOULD be mapped to:

```text
Outcome::NetworkError
```

Examples:

- Connection reset
- Connection refused
- DNS resolution failure
- Unexpected socket close

### 3.2 Timeout Conditions

The following conditions SHOULD be mapped to:

```text
Outcome::TimeoutError
```

Examples:

- Read timeout
- Write timeout
- Request deadline exceeded

### 3.3 Rate Limiting

When a service indicates throttling or rate limiting, the host SHOULD map to:

```text
Outcome::RateLimited { retry_after_ms?: u32 }
```

Sources:

- HTTP 429 responses
- API-specific rate limit signals
- Gateway throttling responses

**Retry-After Hint**

If available, the host MAY extract a retry delay hint and pass it as:

```text
retry_after_ms
```

Notes:

- This value is a hint, not a command
- The core may clamp or ignore it
- Header parsing is strictly host responsibility

### 3.4 Blocking and Challenges

The following conditions SHOULD be mapped to hard failure outcomes:

```text
Outcome::Blocked
Outcome::Captcha
```

Examples:

- IP blocked
- WAF rejection
- CAPTCHA or bot challenge detected

These outcomes are never retryable.

### 3.5 Legacy HTTP Status Mapping

If semantic mapping is not possible, hosts MAY fall back to:

```text
Outcome::HttpStatus(u16)
```

However:

- Retry is NOT guaranteed for raw HTTP status
- Hosts SHOULD avoid relying on this path for retry logic

---

## 4. Authentication Interaction (Host Perspective)

When receiving authentication failures (e.g. HTTP 401):

1. The host consults its authentication provider
2. The provider returns:

- `RefreshAndRetry`
- or `Fail`

3. The host passes this decision into `transport-core`

Only one refresh attempt SHOULD be performed per request lifecycle.

---

## 5. Idempotency Considerations

### 5.1 Idempotency Is Not Sufficient

Providing an idempotency key or enabling non-idempotent retry:

- does NOT guarantee retry
- does NOT override semantic outcome rules

Retry still requires:

1. A retryable `Outcome`
2. Approval by core retry policy

---

### 5.2 Recommended Host Behavior

Hosts SHOULD:

- Avoid retrying non-idempotent requests by default
- Enable retries only with:
  - explicit idempotency key
  - explicit configuration
  - retryable semantic outcome

---

## 6. Retry Limits and Attempts

- `attempt` and `max_attempts` are host-managed
- The core does NOT increment attempts
- Hosts MUST stop retrying when:
  - the core returns `Fail`
  - `attempt >= max_attempts`

---

## 7. Backoff Strategy (Host-Level Policy)

Backoff behavior is not part of the core contract.

Hosts MAY implement:

- exponential backoff
- fixed delay
- jitter
- adaptive strategies

The core returns:

- recommended retry delay (after_ms)
- retry reason (for observability)

Hosts are responsible for:

- sleeping
- scheduling
- concurrency control

---

## 8. Observability Recommendations

Hosts SHOULD log or emit:

- decision type
- retry reason
- retry delay
- attempt number

These values are available via core decision output or FFI getters.

---

## 9. Determinism Guidelines

To preserve deterministic behavior:

- Avoid random jitter unless explicitly configured
- Use consistent outcome mapping
- Ensure retry policy inputs are stable

---

## 10. Summary

- `transport-core` defines what to do
- The host defines what happened
- Retry behavior is driven by semantic outcomes, not transport details

> Semantic in, decision out.
> HTTP and runtime details stay outside the core.
