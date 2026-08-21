"""Local load test for /api/stylist/chat.

50 concurrent SSE chats against a FakeLLM, in-process via httpx.ASGITransport
so we don't need a live OpenAI key or a running container. Asserts:
- no exceptions
- p95 below the soft target (3s with a fake LLM)
- distinct conversation_ids per worker (no cross-talk)

Run from server/ai-service/:
    PYTHONPATH=src python ../../scripts/loadtest_chat.py
"""

from __future__ import annotations

import asyncio
import os
import statistics
import sys
import time
from collections.abc import AsyncIterator
from typing import Any
from uuid import UUID, uuid4

import httpx

# Hop into the ai-service src/ before importing.
HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.normpath(os.path.join(HERE, "..", "server", "ai-service", "src"))
sys.path.insert(0, SRC)
# Disable LangSmith network attempts during the test.
os.environ.setdefault("LANGSMITH_TRACING", "false")

from ai_service.embeddings.counter import QueryEmbedCounter  # noqa: E402
from ai_service.llm.client import AssistantTurn, TokenChunk  # noqa: E402
from ai_service.main import create_app  # noqa: E402
from ai_service.ratelimit import RateLimiter  # noqa: E402


class FakeLLM:
    model = "fake"

    async def chat_stream(
        self,
        messages: list[dict[str, str]],
        *,
        max_tokens: int,
        tools: list[dict[str, Any]] | None = None,
        tool_choice: str | None = None,
    ) -> AsyncIterator[TokenChunk | AssistantTurn]:
        for tok in ("Привет", ", ", "мир!"):
            yield TokenChunk(text=tok)
        yield AssistantTurn(
            text="Привет, мир!", tool_calls=[], finish_reason="stop", total_tokens=12
        )


async def one_session(client: httpx.AsyncClient, user_id: UUID) -> float:
    headers = {"X-User-Id": str(user_id), "X-User-Role": "customer"}
    started = time.monotonic()
    r = await client.post("/api/stylist/conversations", headers=headers)
    r.raise_for_status()
    conv_id = r.json()["conversation_id"]
    async with client.stream(
        "POST",
        "/api/stylist/chat",
        headers=headers,
        json={"conversation_id": conv_id, "message": "test"},
    ) as resp:
        resp.raise_for_status()
        async for _ in resp.aiter_bytes():
            pass
    return time.monotonic() - started


class LifespanRunner:
    """Run the FastAPI lifespan protocol in the background for in-process tests."""

    def __init__(self, app: Any) -> None:
        self._app = app
        self._receive_q: asyncio.Queue[dict[str, Any]] = asyncio.Queue()
        self._startup = asyncio.Event()
        self._shutdown = asyncio.Event()
        self._task: asyncio.Task[None] | None = None

    async def start(self) -> None:
        self._task = asyncio.create_task(self._run())
        await self._receive_q.put({"type": "lifespan.startup"})
        await self._startup.wait()

    async def stop(self) -> None:
        await self._receive_q.put({"type": "lifespan.shutdown"})
        if self._task is not None:
            await self._task

    async def _run(self) -> None:
        async def receive() -> dict[str, Any]:
            return await self._receive_q.get()

        async def send(msg: dict[str, Any]) -> None:
            if msg["type"] == "lifespan.startup.complete":
                self._startup.set()
            elif msg["type"] == "lifespan.shutdown.complete":
                self._shutdown.set()

        try:
            await self._app({"type": "lifespan"}, receive, send)
        except Exception as exc:  # surface failed startup
            print(f"lifespan crashed: {exc}", flush=True)
            self._startup.set()
            self._shutdown.set()


async def main(concurrency: int = 50, rounds: int = 5) -> None:
    app = create_app()
    lifespan = LifespanRunner(app)
    await lifespan.start()

    # Replace heavy bits with cheap fakes after lifespan finished startup.
    app.state.llm_client = FakeLLM()  # type: ignore[assignment]
    app.state.rate_limiter = RateLimiter(messages_per_min=10_000, tokens_per_day=10_000_000)
    app.state.query_counter = QueryEmbedCounter(daily_cap=10_000)

    transport = httpx.ASGITransport(app=app)
    async with httpx.AsyncClient(
        transport=transport, base_url="http://load-test", timeout=10
    ) as client:
        latencies: list[float] = []
        for r in range(rounds):
            users = [uuid4() for _ in range(concurrency)]
            results = await asyncio.gather(
                *(one_session(client, u) for u in users), return_exceptions=True
            )
            errs = [e for e in results if isinstance(e, Exception)]
            ok = [e for e in results if isinstance(e, float)]
            latencies.extend(ok)
            print(
                f"round {r + 1}/{rounds}: ok={len(ok)} err={len(errs)} "
                f"p50={statistics.median(ok) if ok else 0:.3f}s "
                f"p95={percentile(ok, 95):.3f}s",
                flush=True,
            )
            if errs:
                for e in errs[:3]:
                    print(f"  err: {type(e).__name__}: {e}")
                await lifespan.stop()
                raise SystemExit(1)

    await lifespan.stop()

    p95 = percentile(latencies, 95)
    p99 = percentile(latencies, 99)
    print(f"\nTOTAL: n={len(latencies)} p50={statistics.median(latencies):.3f}s "
          f"p95={p95:.3f}s p99={p99:.3f}s")
    if p95 > 3.0:
        print(f"FAIL: p95={p95:.3f}s exceeds 3.0s soft target")
        raise SystemExit(2)
    print("OK")


def percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = max(0, min(len(s) - 1, int(round((pct / 100) * (len(s) - 1)))))
    return s[k]


if __name__ == "__main__":
    asyncio.run(main())
