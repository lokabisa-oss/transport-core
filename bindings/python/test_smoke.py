from transport_core import (
    Client,
    RequestContext,
    Outcome,
    HttpMethod,
    OutcomeKind,
    AuthDecision
)

client = Client()

ctx = RequestContext(
    method=HttpMethod.GET,
    attempt=1,
    max_attempts=3,
    allow_non_idempotent_retry=False,
    idempotency_key=None,
)

outcome = Outcome(
    kind=OutcomeKind.HTTP_STATUS,
    http_status=401,
)

decision = client.decide(
    ctx,
    outcome,
    auth_decision=AuthDecision.REFRESH_AND_RETRY,
)
print("decision:", decision)
