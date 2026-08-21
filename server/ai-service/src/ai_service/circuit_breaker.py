"""Tiny circuit breaker used to short-circuit calls to flapping upstreams.

State machine:

```
CLOSED ──fail-rate ≥ threshold over window──► OPEN
OPEN ──cooldown elapsed──► HALF_OPEN
HALF_OPEN ──success──► CLOSED
HALF_OPEN ──failure──► OPEN
```

Threshold and window are configurable; defaults match the values in
`DESIGN.md → Обработка ошибок`: >50% errors over 60s → break for 30s.
Decisions are made on a coarse sliding window of (success, failure)
samples — we keep one ring buffer per breaker, not per-second buckets.
"""

from __future__ import annotations

import asyncio
import time
from collections import deque
from enum import Enum


class State(str, Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class CircuitOpen(Exception):
    def __init__(self, name: str) -> None:
        super().__init__(f"circuit breaker '{name}' is open")
        self.name = name


class CircuitBreaker:
    def __init__(
        self,
        name: str,
        *,
        window_seconds: float = 60.0,
        failure_ratio_threshold: float = 0.5,
        min_samples: int = 10,
        cooldown_seconds: float = 30.0,
    ) -> None:
        self.name = name
        self._window = window_seconds
        self._threshold = failure_ratio_threshold
        self._min_samples = min_samples
        self._cooldown = cooldown_seconds
        self._lock = asyncio.Lock()
        self._state = State.CLOSED
        self._opened_at: float = 0.0
        # Each entry: (timestamp, is_failure)
        self._samples: deque[tuple[float, bool]] = deque()

    @property
    def state(self) -> State:
        return self._state

    async def check(self) -> None:
        """Raise `CircuitOpen` if the breaker won't currently allow calls."""
        async with self._lock:
            self._maybe_close_after_cooldown()
            if self._state == State.OPEN:
                raise CircuitOpen(self.name)

    async def record_success(self) -> None:
        async with self._lock:
            self._add_sample(False)
            if self._state == State.HALF_OPEN:
                self._state = State.CLOSED
                self._samples.clear()

    async def record_failure(self) -> None:
        async with self._lock:
            self._add_sample(True)
            if self._state == State.HALF_OPEN:
                self._state = State.OPEN
                self._opened_at = time.monotonic()
                return
            if self._state == State.CLOSED and self._should_open():
                self._state = State.OPEN
                self._opened_at = time.monotonic()

    def _add_sample(self, is_failure: bool) -> None:
        now = time.monotonic()
        self._samples.append((now, is_failure))
        cutoff = now - self._window
        while self._samples and self._samples[0][0] < cutoff:
            self._samples.popleft()

    def _should_open(self) -> bool:
        if len(self._samples) < self._min_samples:
            return False
        fails = sum(1 for _, f in self._samples if f)
        ratio = fails / len(self._samples)
        return ratio > self._threshold

    def _maybe_close_after_cooldown(self) -> None:
        if self._state != State.OPEN:
            return
        if time.monotonic() - self._opened_at >= self._cooldown:
            self._state = State.HALF_OPEN
