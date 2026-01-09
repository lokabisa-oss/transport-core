# Retry Semantics

## Retryable conditions

The client MUST retry when:

- Network error (connection reset, timeout)
- HTTP status: 5xx
- HTTP status: 429 (respect Retry-After header)
- HTTP status: 401 IF auth provider allows refresh

The client MUST NOT retry when:

- HTTP status: 400–403 (except 401)
- HTTP status: 404
- HTTP status: 422

## Retry limits

- Default max attempts: 3
- Attempts include the initial request

## Backoff

- Exponential backoff
- Formula:
  delay = min(base \* 2^attempt, max_delay)

## Idempotency

- Retries MUST only be performed for idempotent requests
  unless explicitly overridden.
