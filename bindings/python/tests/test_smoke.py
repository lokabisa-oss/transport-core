import pytest

from transport_core.transport_core import (
    Client,
    RequestContext,
    Outcome,
    HttpMethod,
    Decision,
    RetryReason,
    FailReason,
    AuthDecision,
)

# ------------------------------------------------------------
# Helper
# ------------------------------------------------------------

def default_ctx(attempt=1):
    return RequestContext(
        method=HttpMethod.GET,
        attempt=attempt,
        max_attempts=3,
        allow_non_idempotent_retry=False,
        idempotency_key=None,
    )

# ------------------------------------------------------------
# Tests
# ------------------------------------------------------------

def test_rate_limited_with_retry_after():
    """
    Smoke test:
    RateLimited outcome should trigger RETRY
    and preserve retry_after_ms hint.
    """
    with Client() as client:
        ctx = default_ctx(attempt=1)
        outcome = Outcome.rate_limited(3000)

        result = client.decide(ctx, outcome)

        assert result.decision == Decision.RETRY
        assert result.retry_after_ms == 3000
        assert result.retry_reason == RetryReason.RATE_LIMITED
        assert result.fail_retryable is False


def test_auth_401_refresh_and_retry():
    """
    Smoke test:
    401 + RefreshAndRetry should trigger REFRESH_AND_RETRY.
    """
    with Client() as client:
        ctx = default_ctx(attempt=1)
        outcome = Outcome.from_http_status(401)

        result = client.decide(
            ctx,
            outcome,
            auth_decision=AuthDecision.REFRESH_AND_RETRY,
            refresh_result=None,
        )

        assert result.decision == Decision.REFRESH_AND_RETRY
        assert result.fail_reason is None
