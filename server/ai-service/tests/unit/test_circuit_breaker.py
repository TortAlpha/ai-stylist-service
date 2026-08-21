import pytest

from ai_service.circuit_breaker import CircuitBreaker, CircuitOpen, State


@pytest.mark.asyncio
async def test_starts_closed_and_allows_calls() -> None:
    cb = CircuitBreaker("test", min_samples=2)
    await cb.check()
    assert cb.state == State.CLOSED


@pytest.mark.asyncio
async def test_opens_after_failure_rate_above_threshold() -> None:
    cb = CircuitBreaker("test", failure_ratio_threshold=0.5, min_samples=4)
    # 1 success + 3 failures → 75% > 50%
    await cb.record_success()
    for _ in range(3):
        await cb.record_failure()
    with pytest.raises(CircuitOpen):
        await cb.check()


@pytest.mark.asyncio
async def test_does_not_open_below_min_samples() -> None:
    cb = CircuitBreaker("test", failure_ratio_threshold=0.5, min_samples=10)
    for _ in range(3):
        await cb.record_failure()
    # Only 3 samples — not enough to trip.
    await cb.check()


@pytest.mark.asyncio
async def test_cooldown_transitions_to_half_open_then_closed() -> None:
    import asyncio

    cb = CircuitBreaker(
        "test",
        failure_ratio_threshold=0.0,
        min_samples=1,
        cooldown_seconds=0.05,
    )
    await cb.record_failure()
    with pytest.raises(CircuitOpen):
        await cb.check()
    await asyncio.sleep(0.06)
    # After cooldown the next check flips us to HALF_OPEN and allows the call.
    await cb.check()
    assert cb.state == State.HALF_OPEN
    await cb.record_success()
    assert cb.state == State.CLOSED


@pytest.mark.asyncio
async def test_failure_in_half_open_reopens() -> None:
    import asyncio

    cb = CircuitBreaker(
        "test",
        failure_ratio_threshold=0.0,
        min_samples=1,
        cooldown_seconds=0.05,
    )
    await cb.record_failure()
    await asyncio.sleep(0.06)
    await cb.check()
    assert cb.state == State.HALF_OPEN
    # Failure in HALF_OPEN must drop us back to OPEN immediately.
    await cb.record_failure()
    assert cb.state == State.OPEN
