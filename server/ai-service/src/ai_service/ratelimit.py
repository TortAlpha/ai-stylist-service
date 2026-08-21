"""Per-user rate limiting (in-memory).

Two limits, both keyed by `user_id`:

- **Messages/minute** — a sliding token bucket. Refills continuously at
  `messages_per_min / 60` tokens per second up to capacity. Consumed
  *before* hitting OpenAI so we never burn a chat-completion call for a
  user who's already over the limit.
- **Tokens/day** — a flat counter rolled over at UTC midnight. Updated
  *after* a successful OpenAI call using `usage.total_tokens` reported in
  the response, so the limit reflects real cost. A user past the daily
  cap is rejected on the *next* message — the in-flight stream still
  completes (otherwise we'd cut the user off mid-answer).
"""

from __future__ import annotations

import asyncio
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from enum import Enum
from uuid import UUID


class RejectReason(str, Enum):
    MESSAGES_PER_MIN = "messages_per_min"
    TOKENS_PER_DAY = "tokens_per_day"


@dataclass
class Decision:
    allowed: bool
    reason: RejectReason | None = None
    retry_after_seconds: float | None = None


@dataclass
class _Bucket:
    tokens: float
    last_refill: float


@dataclass
class _DayUsage:
    day: str
    used: int = 0


class RateLimiter:
    def __init__(self, *, messages_per_min: int, tokens_per_day: int) -> None:
        self._mpm = max(0, messages_per_min)
        self._tpd = max(0, tokens_per_day)
        self._buckets: dict[UUID, _Bucket] = {}
        self._day_usage: dict[UUID, _DayUsage] = {}
        self._lock = asyncio.Lock()

    @property
    def messages_per_min(self) -> int:
        return self._mpm

    @property
    def tokens_per_day(self) -> int:
        return self._tpd

    async def check_message(self, user_id: UUID) -> Decision:
        """Pre-flight check before starting a chat. Consumes 1 message token."""
        async with self._lock:
            # Daily token check first — if we know they're over, reject up front.
            usage = self._day_usage.get(user_id)
            today = _today_key()
            if usage is not None and usage.day == today and usage.used >= self._tpd:
                return Decision(allowed=False, reason=RejectReason.TOKENS_PER_DAY)

            # Sliding token bucket for messages/min.
            if self._mpm <= 0:
                return Decision(allowed=True)
            now = time.monotonic()
            bucket = self._buckets.get(user_id)
            if bucket is None:
                bucket = _Bucket(tokens=float(self._mpm), last_refill=now)
                self._buckets[user_id] = bucket
            elapsed = max(0.0, now - bucket.last_refill)
            refill_rate = self._mpm / 60.0
            bucket.tokens = min(float(self._mpm), bucket.tokens + elapsed * refill_rate)
            bucket.last_refill = now
            if bucket.tokens < 1.0:
                deficit = 1.0 - bucket.tokens
                retry_after = deficit / refill_rate if refill_rate > 0 else None
                return Decision(
                    allowed=False,
                    reason=RejectReason.MESSAGES_PER_MIN,
                    retry_after_seconds=retry_after,
                )
            bucket.tokens -= 1.0
            return Decision(allowed=True)

    async def record_tokens(self, user_id: UUID, tokens_used: int) -> None:
        """Account real OpenAI token usage against the daily cap."""
        if tokens_used <= 0 or self._tpd <= 0:
            return
        today = _today_key()
        async with self._lock:
            usage = self._day_usage.get(user_id)
            if usage is None or usage.day != today:
                usage = _DayUsage(day=today, used=0)
                self._day_usage[user_id] = usage
            usage.used += tokens_used

    async def tokens_used_today(self, user_id: UUID) -> int:
        today = _today_key()
        async with self._lock:
            usage = self._day_usage.get(user_id)
            if usage is None or usage.day != today:
                return 0
            return usage.used


def _today_key() -> str:
    return datetime.now(UTC).date().isoformat()
