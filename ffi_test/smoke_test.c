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

    // 3. Build outcome (HTTP 401)
    tc_outcome_t outcome = {
        .kind = TC_OUTCOME_HTTP_STATUS,
        .http_status = 401
    };

    // 4. Call decision engine
    tc_decision_t decision = tc_decide(
        client,
        &ctx,
        &outcome,
        TC_AUTH_REFRESH_AND_RETRY,
        -1  // refresh not attempted
    );

    // 5. Print decision
    printf("decision = %d\n", decision);

    // 6. Cleanup
    tc_client_free(client);

    return 0;
}
