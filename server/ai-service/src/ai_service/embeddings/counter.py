"""Per-day, in-memory query embed counter.

Rolls over at UTC midnight. The semantic-search tool short-circuits with
a tool_error when the daily cap is reached; the LLM then falls back to
`search_products`. No Redis on MVP — counter is per-process.
"""

from __future__ import annotations

import asyncio
from datetime import UTC, datetime


class QueryEmbedCounter:
    def __init__(self, *, daily_cap: int) -> None:
        self._cap = daily_cap
        self._day: str = _today_key()
        self._used = 0
        self._lock = asyncio.Lock()

    @property
    def cap(self) -> int:
        return self._cap

    async def try_consume(self, n: int = 1) -> bool:
        async with self._lock:
            today = _today_key()
            if today != self._day:
                self._day = today
                self._used = 0
            if self._used + n > self._cap:
                return False
            self._used += n
            return True

    async def used(self) -> int:
        async with self._lock:
            today = _today_key()
            if today != self._day:
                self._day = today
                self._used = 0
            return self._used

    async def remaining(self) -> int:
        return max(0, self._cap - await self.used())


def _today_key() -> str:
    return datetime.now(UTC).date().isoformat()
