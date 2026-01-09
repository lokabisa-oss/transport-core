# transport-core

A domain-agnostic transport foundation for building reliable SDKs.

## What this is

`transport-core` provides a deterministic core for network communication,
including retry semantics, authentication handling, and state management.

It is designed to be embedded into SDKs across multiple programming languages.

## What this is NOT

- Not an API SDK
- Not domain-specific
- Not a high-level HTTP client

## Design principles

- Single source of truth for transport behavior
- Stable ABI boundary
- Language bindings live outside this repository
- Core logic is runtime-agnostic

## Language strategy

- Native binding: Python, PHP, Ruby
- WASM binding: JavaScript (default)
- Reimplementation: Go (spec-driven)

## License

MIT
