# Authentication Behavior

## Auth application

- Auth provider MUST apply credentials before request is sent.

## Unauthorized response (401)

On receiving HTTP 401:

1. Client MUST call auth.refresh()
2. If refresh succeeds:
   - Retry the original request
3. If refresh fails:
   - Fail the request

## Refresh rules

- Only one refresh operation may run at a time
- Concurrent requests MUST wait for refresh result
