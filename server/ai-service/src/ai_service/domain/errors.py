from enum import StrEnum


class ErrorCode(StrEnum):
    RATE_LIMITED = "rate_limited"
    UPSTREAM_UNAVAILABLE = "upstream_unavailable"
    BAD_REQUEST = "bad_request"
    INTERNAL = "internal"


class AIServiceError(Exception):
    code: ErrorCode = ErrorCode.INTERNAL

    def __init__(self, message: str = "") -> None:
        super().__init__(message or self.code.value)
        self.message = message or self.code.value


class RateLimited(AIServiceError):
    code = ErrorCode.RATE_LIMITED


class UpstreamUnavailable(AIServiceError):
    code = ErrorCode.UPSTREAM_UNAVAILABLE


class BadRequest(AIServiceError):
    code = ErrorCode.BAD_REQUEST


class Internal(AIServiceError):
    code = ErrorCode.INTERNAL
