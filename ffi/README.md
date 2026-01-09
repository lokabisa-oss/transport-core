# transport-core FFI (ABI v0)

This directory defines the **C ABI boundary** for `transport-core`.

The goal of this ABI is to expose the **core transport decision engine**
(retry + auth state machine) to multiple host languages
(Python, PHP, Ruby, etc.) **without duplicating logic**.

Rust is treated as the **core engine**, not as a language-specific extension.

---

## What This ABI Is

- A **stable C ABI** for transport decision logic
- Stateful (auth refresh state lives inside the client handle)
- Synchronous and deterministic
- Language-agnostic

This ABI is intended to be the **single source of truth**
for retry and authentication behavior.

---

## What This ABI Is NOT

- Not an HTTP client
- Not async / event-loop aware
- Not WASM (WASM has a separate interface)
- Not a Python-only or JS-only binding

The host language is responsible for:

- Performing HTTP requests
- Executing token refresh
- Managing async / concurrency
- Calling back into this engine with outcomes

---

## Core Concepts

### Opaque Client

```c
typedef struct transport_core_client transport_core_client_t;
```

The client handle is opaque to the host language.

Internally, it stores:

- Authentication state
- Refresh attempt history

The host must not inspect or modify its contents.

---

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

- Evaluates retry rules
- Applies authentication rules
- Updates internal auth state if needed
- Returns the next action to take

---

### Ownership Rules (IMPORTANT)

- tc_client_new() allocates memory
- tc_client_free() must be called by the host
- The host must not free the client manually

### Rule of thumb:

> The side that allocates, frees.

---

### ABI Stability

This ABI is versioned as v0.

Guarantees:

- Enum values will not change
- Struct field order will not change
- Function signatures will not change

Any breaking change will require ABI v1.

---

### Testing

The ABI implementation is validated via:

- Rust unit tests using JSON test vectors
- A C smoke test that links against the shared library

This ensures:

- Correct symbol export
- Correct memory layout
- Correct decision behavior

---

Typical Integration Flow

1. Host creates a client via tc_client_new
2. Host performs an HTTP request
3. Host reports the outcome via tc_decide
4. Host acts on the returned decision
5. Host repeats until PROCEED or FAIL
6. Host frees the client

---

### Future Work

- Higher-level language bindings (Python, PHP, Ruby)
- WASM interface (separate from C ABI)
- Optional metadata (retry delay, backoff hints)

---

### Summary

This ABI exists to make transport behavior:

- Consistent
- Testable
- Reusable
- Language-independent

It intentionally keeps the surface area minimal.
