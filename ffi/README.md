# transport-core FFI (ABI v1)

This directory defines the **C ABI boundary** for `transport-core`.

The goal of this ABI is to expose the **core transport decision engine**
(retry + authentication coordination) to multiple host languages
(Python, PHP, Ruby, etc.) **without duplicating logic**.

Rust is treated as the **core engine**, not as a language-specific extension.

---

## What This ABI Is

- A **stable C ABI** for transport decision logic
- Stateful (authentication refresh state lives inside the client handle)
- Synchronous and deterministic
- Language-agnostic

This ABI is intended to be the **single source of truth**
for retry and authentication decisions across languages.

---

## What This ABI Is NOT

- Not an HTTP client
- Not async / event-loop aware
- Not responsible for sleeping or scheduling
- Not responsible for executing token refresh
- Not a Python-only or JS-only binding

The host language is responsible for:

- Performing HTTP requests
- Executing authentication refresh
- Managing async / concurrency
- Mapping runtime errors into semantic outcomes
- Calling back into this engine with results

---

## Core Concepts

### Opaque Client Handle

```c
typedef struct transport_core_client transport_core_client_t;
```

The client handle is opaque to the host language.

Internally, it stores:

- Authentication coordination state
- Refresh attempt tracking

The host MUST NOT inspect or modify its contents.

### Decision Engine

The primary entrypoint is:

```c
tc_decision_t tc_decide(
    transport_core_client_t* client,
    const tc_request_context_t* ctx,
    const tc_outcome_t* outcome,
    tc_auth_decision_t auth_decision,
    int8_t refresh_result
);
```

This function:

- Evaluates retry eligibility
- Applies authentication coordination rules
- Updates internal auth state
- Returns the next action to take

It does not perform any IO.

### Decision Details (Getters)

Detailed decision metadata is accessed via getters
after calling `tc_decide`:

- `tc_last_retry_after_ms`
- `tc_last_retry_reason`
- `tc_last_fail_reason`
- `tc_last_fail_retryable`

This design keeps the primary ABI surface small
while allowing richer decision introspection.

### Ownership Rules (IMPORTANT)

- `tc_client_new()` allocates the client
- `tc_client_free()` MUST be called by the host
- The host MUST NOT free or copy the client manually

Rule of thumb:

> **The side that allocates, frees.**

### ABI Stability

This ABI is versioned as v1.

Guarantees for ABI v1:

- Enum numeric values will not change
- Struct field order will not change
- Function signatures will not change

Any breaking change requires a new ABI version.

### Typical Integration Flow

1. Host creates a client via tc_client_new
2. Host performs a request using its transport
3. Host maps the result into a semantic outcome
4. Host calls tc_decide
5. Host acts on the returned decision
6. Host queries decision details via getters if needed
7. Host repeats until PROCEED or FAIL
8. Host frees the client

### Testing

The ABI is validated via:

- Rust unit tests using canonical JSON test vectors
- C-level smoke tests linking against the shared library

This ensures:

- Correct symbol export
- Correct memory layout
- Correct and deterministic behavior

### Summary

This ABI exists to make transport behavior:

- Consistent
- Deterministic
- Testable
- Reusable
- Language-independent

It intentionally keeps the ABI surface minimal,
while allowing rich policy decisions.

> **The host executes requests.  
> The core executes policy.**
