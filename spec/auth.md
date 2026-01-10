# Authentication Semantics — Host-Level Guidance

⚠️ **Non-normative document**

This document provides **host-side guidance** for integrating authentication
flows with `transport-core`.

The **canonical and normative behavior** of authentication decisions is defined
in [`spec/README.md`](./README.md).

This document explains how a host SHOULD coordinate authentication providers,
token refresh, and concurrent requests when interacting with `transport-core`.

---

## 1. Scope

This document describes:

- Recommended authentication flow coordination
- Token refresh handling
- Concurrency considerations
- Mapping authentication outcomes into core inputs

This document does NOT define:

- Credential formats
- Token storage mechanisms
- Authentication protocols (OAuth, API keys, etc.)
- Network request execution

---

## 2. Core Design Recap

`transport-core`:

- does NOT perform authentication
- does NOT execute refresh operations
- does NOT manage request lifecycles

Instead:

> **The host executes authentication.  
> The core only signals when refresh is required or forbidden.**

---

## 3. Authentication Provider Contract (Host Perspective)

Hosts SHOULD provide an authentication provider capable of:

- Applying credentials to outgoing requests
- Deciding whether a request is eligible for refresh
- Performing a refresh operation when requested

The provider is treated as a **black box** by the host.

---

## 4. Authentication Failure Handling

When a request fails due to authentication issues (e.g. HTTP 401):

1. The host determines this is an authentication failure
2. The host consults the authentication provider
3. The provider returns one of:
   - `RefreshAndRetry`
   - `Fail`
4. The host passes this decision into `transport-core`

The core then decides whether refresh is allowed based on its internal state.

---

## 5. Refresh Signaling and Execution

When `transport-core` returns:

```text
Decision::RefreshAndRetry
```

The host SHOULD:

1. Perform a single refresh operation
2. Report the result (`success` or `failure`) back to the core
3. Retry or fail the request based on the next decision

The core itself never performs refresh.

---

## 6. Refresh Concurrency (Single-Flight)

Recommended behavior:

- At most one refresh operation SHOULD be in progress per client instance
- Concurrent requests encountering auth failure SHOULD:
  - wait for the refresh result
  - reuse the result of the in-flight refresh

This behavior aligns with the core’s internal `AuthState`.

---

## 7. Refresh Attempt Limits

- A refresh SHOULD be attempted at most once per request lifecycle
- If a request encounters auth failure again after a successful refresh:
  - The host SHOULD fail the request
  - No further refresh attempts SHOULD be made

This prevents infinite refresh loops.

---

## 8. Interaction with Retry Policy

Authentication refresh is orthogonal to retry semantics.

Notes:

- A refresh-triggered retry MAY count as an attempt (host-defined)
- Retry limits are enforced by host-provided `RequestContext`
- The core only evaluates whether retry is allowed

---

## 9. Non-Refreshable Authentication Errors

Hosts SHOULD NOT attempt refresh for:

- Authorization failures (e.g. HTTP 403)
- Explicit credential rejection
- Malformed authentication data

Such failures SHOULD be mapped to non-retryable outcomes.

---

## 10. State Scope and Isolation

- Authentication state SHOULD be scoped to a single client instance
- Authentication state SHOULD NOT leak across clients
- Request-specific data MUST NOT be stored in auth state

---

## 11. Observability Recommendations

Hosts SHOULD emit observable signals for:

- Refresh started
- Refresh succeeded
- Refresh failed
- Requests waiting on refresh

Sensitive credential data MUST NOT be logged.

---

## 12. Determinism Guidelines

To preserve deterministic behavior:

- Ensure refresh decisions are consistent
- Avoid duplicate or parallel refresh execution
- Use stable mappings between auth failures and core inputs

---

## 13. Summary

- Authentication execution is a host responsibility
- `transport-core` only coordinates decisions
- Refresh is signaled, not performed, by the core

> Auth happens outside.
> Coordination happens inside.
