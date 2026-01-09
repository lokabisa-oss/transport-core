# Request Lifecycle

1. Prepare request

   - Apply authentication headers
   - Attach idempotency key (if any)

2. Send request via transport adapter

3. Receive response or error

4. Evaluate result

   - Success → return
   - Retryable error → retry
   - Auth error → refresh token
   - Fatal error → fail

5. Terminate after success or max retry reached
