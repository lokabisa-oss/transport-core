#include <stdio.h>
#include "../ffi/transport_core.h"

int main() {
    // 1. Create client
    transport_core_client_t* client = tc_client_new();
    if (!client) {
        printf("failed to create client\n");
        return 1;
    }

    // 2. Build request context
    tc_request_context_t ctx = {
        .method = TC_HTTP_GET,
        .attempt = 1,
        .max_attempts = 3,
        .allow_non_idempotent_retry = false,
        .idempotency_key = NULL
    };

    // 3. Build outcome (Rate Limited with retry-after hint)
    tc_outcome_t outcome = {
        .kind = TC_OUTCOME_RATE_LIMITED,
        .http_status = 0,        // unused
        .retry_after_ms = 3000   // 3 seconds
    };

    // 4. Call decision engine
    tc_decision_t decision = tc_decide(
        client,
        &ctx,
        &outcome,
        TC_AUTH_FAIL,  // auth not involved
        -1
    );

    // 5. Print decision
    printf("decision = %d\n", decision);

    // 6. Inspect decision details
    printf("retry_after_ms = %u\n",
           tc_last_retry_after_ms(client));
    printf("retry_reason = %u\n",
           tc_last_retry_reason(client));
    printf("fail_reason = %u\n",
           tc_last_fail_reason(client));
    printf("fail_retryable = %d\n",
           tc_last_fail_retryable(client));

    // 7. Cleanup
    tc_client_free(client);

    return 0;
}
