#ifndef TRANSPORT_CORE_H
#define TRANSPORT_CORE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

/* ============================
 * ABI VERSION
 * ============================ */
#define TRANSPORT_CORE_ABI_VERSION 0

/* ============================
 * OPAQUE HANDLE
 * ============================ */
typedef struct transport_core_client transport_core_client_t;

/* ============================
 * ENUMS
 * ============================ */

/* HTTP Method */
typedef enum {
    TC_HTTP_GET = 0,
    TC_HTTP_POST,
    TC_HTTP_PUT,
    TC_HTTP_DELETE,
    TC_HTTP_HEAD,
    TC_HTTP_OPTIONS
} tc_http_method_t;

/* Outcome Type */
typedef enum {
    TC_OUTCOME_NETWORK_ERROR = 0,
    TC_OUTCOME_TIMEOUT_ERROR,
    TC_OUTCOME_HTTP_STATUS
} tc_outcome_kind_t;

/* Decision */
typedef enum {
    TC_DECISION_PROCEED = 0,
    TC_DECISION_RETRY,
    TC_DECISION_REFRESH_AND_RETRY,
    TC_DECISION_FAIL
} tc_decision_t;

/* Auth Decision (from host) */
typedef enum {
    TC_AUTH_REFRESH_AND_RETRY = 0,
    TC_AUTH_FAIL
} tc_auth_decision_t;

/* ============================
 * STRUCTS
 * ============================ */

/* Request Context */
typedef struct {
    tc_http_method_t method;
    uint8_t attempt;
    uint8_t max_attempts;
    bool allow_non_idempotent_retry;
    const char* idempotency_key; /* nullable */
} tc_request_context_t;

/* Outcome */
typedef struct {
    tc_outcome_kind_t kind;
    uint16_t http_status; /* only valid if kind == HTTP_STATUS */
} tc_outcome_t;

/* ============================
 * LIFECYCLE
 * ============================ */

/* Create client */
transport_core_client_t* tc_client_new(void);

/* Destroy client */
void tc_client_free(transport_core_client_t* client);

/* ============================
 * DECISION ENGINE
 * ============================ */

/*
 * Decide next action.
 *
 * refresh_result:
 *   -1 = refresh not attempted
 *    0 = refresh failed
 *    1 = refresh succeeded
 */
tc_decision_t tc_decide(
    transport_core_client_t* client,
    const tc_request_context_t* ctx,
    const tc_outcome_t* outcome,
    tc_auth_decision_t auth_decision,
    int8_t refresh_result
);

#ifdef __cplusplus
}
#endif

#endif /* TRANSPORT_CORE_H */
