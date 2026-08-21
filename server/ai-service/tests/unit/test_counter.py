import pytest

from ai_service.embeddings.counter import QueryEmbedCounter


@pytest.mark.asyncio
async def test_consume_until_cap() -> None:
    c = QueryEmbedCounter(daily_cap=3)
    assert await c.try_consume() is True
    assert await c.try_consume() is True
    assert await c.try_consume() is True
    assert await c.try_consume() is False
    assert await c.used() == 3


@pytest.mark.asyncio
async def test_remaining_tracks_use() -> None:
    c = QueryEmbedCounter(daily_cap=5)
    await c.try_consume(2)
    assert await c.remaining() == 3


@pytest.mark.asyncio
async def test_zero_cap_always_rejects() -> None:
    c = QueryEmbedCounter(daily_cap=0)
    assert await c.try_consume() is False
