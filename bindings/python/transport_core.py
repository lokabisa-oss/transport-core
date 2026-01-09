import ctypes
import os
from enum import IntEnum

# ===== Load shared library =====

_lib_path = os.path.join(
    os.path.dirname(__file__),
    "..", "..", "core", "target", "release", "libtransport_core.so"
)

_lib = ctypes.CDLL(_lib_path)


# ===== Enums (mirror C ABI) =====

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


# ===== Structs =====

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
    ]


# ===== Function signatures =====

_lib.tc_client_new.restype = ctypes.c_void_p

_lib.tc_client_free.argtypes = [ctypes.c_void_p]
_lib.tc_client_free.restype = None

_lib.tc_decide.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(RequestContext),
    ctypes.POINTER(Outcome),
    ctypes.c_int,   # auth decision
    ctypes.c_int8,  # refresh result
]
_lib.tc_decide.restype = ctypes.c_int


# ===== Python-friendly wrapper =====

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
        refresh_result: int | None = None,
    ) -> Decision:
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
        return Decision(decision)

    def close(self):
        if self._ptr:
            _lib.tc_client_free(self._ptr)
            self._ptr = None

    def __del__(self):
        self.close()
