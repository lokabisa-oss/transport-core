# transport-core

[![CI](https://github.com/lokabisa-oss/transport-core/actions/workflows/ci.yml/badge.svg)](https://github.com/lokabisa-oss/transport-core/actions/workflows/ci.yml)

A domain-agnostic transport policy engine for building reliable SDKs.

`transport-core` defines a **deterministic, spec-driven core**
for handling retries, authentication coordination, and request lifecycle
across multiple programming languages.

It is designed to be **embedded**, not used directly.

---

## What this is

`transport-core` provides a **single source of truth** for transport behavior:

- Retry eligibility and backoff decisions
- Authentication failure handling and refresh coordination
- Deterministic request lifecycle decisions
- Language-independent transport policy

The core logic is implemented in Rust and exposed via a **stable C ABI**,
allowing it to be reused by SDKs in different runtimes
without duplicating complex logic.

---

## What this is NOT

- ❌ Not an API SDK
- ❌ Not a domain-specific client
- ❌ Not a high-level HTTP library
- ❌ Not responsible for performing network I/O

All HTTP requests, async handling, and credential management
are performed by the host runtime.

---

## Why this exists

Most SDKs reimplement retry and authentication logic independently,
leading to:

- Inconsistent behavior across languages
- Subtle bugs in edge cases (retry storms, auth loops)
- Difficult-to-test transport behavior

`transport-core` solves this by:

- Defining behavior in a **canonical specification**
- Implementing that behavior once
- Reusing it everywhere via a stable ABI

---

## Architecture Overview

- **Core**: deterministic policy engine (this repository)
- **Host**: performs HTTP requests and authentication refresh
- **Interface**: stable C ABI or WASM boundary

> The host executes requests.  
> The core executes policy.

---

## Specifications

Behavior is defined by specification, not implementation.

See the `spec/` directory for:

- Retry semantics
- Authentication coordination
- Error taxonomy
- Request lifecycle rules

All implementations MUST conform to these specifications.

---

## Language Strategy

`transport-core` is language-agnostic by design.

Planned integration approaches:

- **Native bindings**: Python, PHP, Ruby (via C ABI)
- **WASM interface**: JavaScript runtimes
- **Spec-driven reimplementations**: Go and others

Bindings live outside this repository.

---

## Stability

- Core behavior is frozen in **SPEC v1**
- C ABI is frozen in **ABI v1**
- Breaking changes require explicit version bumps

This makes `transport-core` suitable as a long-term foundation
for production SDKs.

---

## License

MIT
