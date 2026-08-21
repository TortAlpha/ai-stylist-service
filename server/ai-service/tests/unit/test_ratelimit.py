from uuid import uuid4

import pytest

from ai_service.ratelimit import RateLimiter, RejectReason


@pytest.mark.asyncio
async def test_consumes_message_tokens() -> None:
    rl = RateLimiter(messages_per_min=3, tokens_per_day=10_000)
    uid = uuid4()
    assert (await rl.check_message(uid)).allowed
    assert (await rl.check_message(uid)).allowed
    assert (await rl.check_message(uid)).allowed
    d = await rl.check_message(uid)
    assert d.allowed is False
    assert d.reason == RejectReason.MESSAGES_PER_MIN
    assert d.retry_after_seconds is not None and d.retry_after_seconds > 0


@pytest.mark.asyncio
async def test_independent_users() -> None:
    rl = RateLimiter(messages_per_min=1, tokens_per_day=10_000)
    a, b = uuid4(), uuid4()
    assert (await rl.check_message(a)).allowed
    assert (await rl.check_message(b)).allowed  # different user keeps full bucket


@pytest.mark.asyncio
async def test_daily_token_cap_blocks_next_message() -> None:
    rl = RateLimiter(messages_per_min=5, tokens_per_day=100)
    uid = uuid4()
    assert (await rl.check_message(uid)).allowed
    await rl.record_tokens(uid, 120)
    d = await rl.check_message(uid)
    assert d.allowed is False
    assert d.reason == RejectReason.TOKENS_PER_DAY


@pytest.mark.asyncio
async def test_record_tokens_under_cap_keeps_user_open() -> None:
    rl = RateLimiter(messages_per_min=5, tokens_per_day=1_000)
    uid = uuid4()
    await rl.record_tokens(uid, 200)
    assert await rl.tokens_used_today(uid) == 200
    assert (await rl.check_message(uid)).allowed


@pytest.mark.asyncio
async def test_zero_mpm_means_unlimited_messages() -> None:
    rl = RateLimiter(messages_per_min=0, tokens_per_day=10_000)
    uid = uuid4()
    # When mpm=0, the bucket check is bypassed.
    assert (await rl.check_message(uid)).allowed
    assert (await rl.check_message(uid)).allowed
