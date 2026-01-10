# Request Lifecycle — Host and Core Interaction

This document describes the **recommended request lifecycle** when integrating
a host environment (SDK, runtime, or adapter) with `transport-core`.

⚠️ This document is **informative**, not normative.  
Canonical behavior is defined in [`spec/README.md`](./README.md).

---

## 1. Request Preparation (Host Responsibility)

Before sending a request, the host SHOULD:

- Apply authentication data (headers, tokens, etc.)
- Attach an idempotency key if applicable
- Initialize request context:
  - method
  - attempt
  - max_attempts
  - retry configuration

No network activity occurs in `transport-core`.

---

## 2. Request Execution (Host Responsibility)

The host sends the request using its chosen transport layer.

Examples:

- HTTP client
- WASM fetch
- gRPC
- custom adapter

---

## 3. Result Collection (Host Responsibility)

After execution, the host receives either:

- a successful response, or
- a runtime / transport error, or
- a protocol-level response (e.g. HTTP status)

---

## 4. Semantic Evaluation (Host Responsibility)

The host interprets the result and maps it into a semantic input for
`transport-core`.

Examples:

- Network failure → `Outcome::NetworkError`
- Timeout → `Outcome::TimeoutError`
- Rate limit → `Outcome::RateLimited`
- Auth failure → `Outcome::HttpStatus(401)` + `AuthDecision`
- Hard block → `Outcome::Blocked`

The host determines **what happened**.

---

## 5. Policy Decision (Core Responsibility)

The host invokes `transport-core` with:

- `RequestContext`
- `Outcome`
- `AuthDecision` (if applicable)
- refresh result (if applicable)

`transport-core` returns a `Decision`, such as:

- `Proceed`
- `Retry`
- `RefreshAndRetry`
- `Fail`

The core determines **what should happen next**.

---

## 6. Decision Handling (Host Responsibility)

Based on the returned decision, the host SHOULD:

### Proceed

- Return the response to the caller

### Retry

- Wait for the recommended delay (if any)
- Increment attempt counter
- Re-enter the lifecycle at step 2

### RefreshAndRetry

- Perform a single authentication refresh
- Report refresh result back to the core
- Retry or fail based on the next decision

### Fail

- Surface the failure to the caller
- Stop the lifecycle

---

## 7. Termination

The request lifecycle terminates when:

- the request succeeds, or
- the core returns `Fail`, or
- the maximum number of attempts is reached

---

## 8. Key Principle

> **The host executes requests.  
> The core executes policy.**

Clear separation of responsibility ensures:

- deterministic behavior
- portability across runtimes
- safe retry and auth coordination
