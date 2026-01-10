import ctypes
import os
import sys
from enum import IntEnum
from typing import Optional

# ============================================================
# Shared library loader (portable)
# ============================================================

def _load_library():
    base = os.path.dirname(__file__)

    if sys.platform.startswith("linux"):
        name = "libtransport_core.so"
    elif sys.platform == "darwin":
        name = "libtransport_core.dylib"
    elif sys.platform == "win32":
        name = "transport_core.dll"
    else:
        raise RuntimeError(f"unsupported platform: {sys.platform}")

    path = os.path.join(base, name)
    return ctypes.CDLL(path)


_lib = _load_library()

# ============================================================
# Enums (MUST mirror C ABI exactly)
# ============================================================

class Decision(IntEnum):
    PROCEED = 0
    RETRY = 1
    REFRESH_AND_RETRY = 2
    FAIL = 3


class AuthDecision(IntEnum):
    REFRESH_AND_RETRY = 0
    FAIL = 1


class HttpMethod(IntEnum):
    GET = 0
    POST = 1
    PUT = 2
    DELETE = 3
    HEAD = 4
    OPTIONS = 5


class OutcomeKind(IntEnum):
    NETWORK_ERROR = 0
    TIMEOUT_ERROR = 1
    HTTP_STATUS = 2
    RATE_LIMITED = 3
    BLOCKED = 4
    CAPTCHA = 5


class RetryReason(IntEnum):
    NETWORK = 1
    TIMEOUT = 2
    RATE_LIMITED = 3
    AUTH_EXPIRED = 4


class FailReason(IntEnum):
    MAX_ATTEMPTS_EXCEEDED = 1
    AUTH_FAILED = 2
    HARD_BLOCKED = 3
    UNKNOWN = 255


# ============================================================
# Structs (ABI v1 layout)
# ============================================================

class RequestContext(ctypes.Structure):
    _fields_ = [
        ("method", ctypes.c_int),
        ("attempt", ctypes.c_uint8),
        ("max_attempts", ctypes.c_uint8),
        ("allow_non_idempotent_retry", ctypes.c_bool),
        ("idempotency_key", ctypes.c_char_p),
    ]


class Outcome(ctypes.Structure):
    _fields_ = [
        ("kind", ctypes.c_int),
        ("http_status", ctypes.c_uint16),
        ("retry_after_ms", ctypes.c_uint32),
    ]

    # -------- helpers (FACTORY METHODS) --------

    @staticmethod
    def network_error():
        return Outcome(
            OutcomeKind.NETWORK_ERROR,
            0,
            0,
        )

    @staticmethod
    def timeout():
        return Outcome(
            OutcomeKind.TIMEOUT_ERROR,
            0,
            0,
        )

    @staticmethod
    def from_http_status(code: int):
        """
        Create Outcome from HTTP status code.
        This avoids name collision with `http_status` field.
        """
        return Outcome(
            OutcomeKind.HTTP_STATUS,
            code,
            0,
        )

    @staticmethod
    def rate_limited(retry_after_ms: int):
        return Outcome(
            OutcomeKind.RATE_LIMITED,
            0,
            retry_after_ms,
        )

    @staticmethod
    def blocked():
        return Outcome(
            OutcomeKind.BLOCKED,
            0,
            0,
        )

    @staticmethod
    def captcha():
        return Outcome(
            OutcomeKind.CAPTCHA,
            0,
            0,
        )


# ============================================================
# FFI signatures
# ============================================================

_lib.tc_client_new.restype = ctypes.c_void_p

_lib.tc_client_free.argtypes = [ctypes.c_void_p]
_lib.tc_client_free.restype = None

_lib.tc_decide.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(RequestContext),
    ctypes.POINTER(Outcome),
    ctypes.c_int,
    ctypes.c_int8,
]
_lib.tc_decide.restype = ctypes.c_int

_lib.tc_last_retry_after_ms.argtypes = [ctypes.c_void_p]
_lib.tc_last_retry_after_ms.restype = ctypes.c_uint32

_lib.tc_last_retry_reason.argtypes = [ctypes.c_void_p]
_lib.tc_last_retry_reason.restype = ctypes.c_uint8

_lib.tc_last_fail_reason.argtypes = [ctypes.c_void_p]
_lib.tc_last_fail_reason.restype = ctypes.c_uint8

_lib.tc_last_fail_retryable.argtypes = [ctypes.c_void_p]
_lib.tc_last_fail_retryable.restype = ctypes.c_bool


# ============================================================
# Python-friendly result object
# ============================================================

class DecisionResult:
    def __init__(self, decision: int, client_ptr):
        self.decision = Decision(decision)
        self.retry_after_ms = _lib.tc_last_retry_after_ms(client_ptr)

        _raw_retry_reason = _lib.tc_last_retry_reason(client_ptr)

        if self.decision == Decision.RETRY:
            if _raw_retry_reason == 0:
                raise RuntimeError("RETRY decision must have retry_reason")
            self.retry_reason = RetryReason(_raw_retry_reason)
        else:
            self.retry_reason = None

        _raw_fail = _lib.tc_last_fail_reason(client_ptr)

        if _raw_fail == 0:
            self.fail_reason = None
        else:
            self.fail_reason = FailReason(_raw_fail)

        self.fail_retryable = bool(
            _lib.tc_last_fail_retryable(client_ptr)
        )

    def __repr__(self):
        return (
            f"DecisionResult("
            f"decision={self.decision.name}, "
            f"retry_after_ms={self.retry_after_ms}, "
            f"retry_reason={self.retry_reason.name}, "
            f"fail_reason={self.fail_reason.name}, "
            f"fail_retryable={self.fail_retryable})"
        )


# ============================================================
# Client wrapper
# ============================================================

class Client:
    def __init__(self):
        self._ptr = _lib.tc_client_new()
        if not self._ptr:
            raise RuntimeError("failed to create transport-core client")

    def decide(
        self,
        ctx: RequestContext,
        outcome: Outcome,
        auth_decision: AuthDecision = AuthDecision.FAIL,
        refresh_result: Optional[bool] = None,
    ) -> DecisionResult:
        if refresh_result is None:
            rr = -1
        else:
            rr = 1 if refresh_result else 0

        decision = _lib.tc_decide(
            self._ptr,
            ctypes.byref(ctx),
            ctypes.byref(outcome),
            int(auth_decision),
            rr,
        )

        return DecisionResult(decision, self._ptr)

    def close(self):
        if self._ptr:
            _lib.tc_client_free(self._ptr)
            self._ptr = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        self.close()

    def __del__(self):
        self.close()
