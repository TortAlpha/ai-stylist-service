from typing import cast

from ai_service.llm.budget import ChatMessage, count_tokens, trim_history


def _m(role: str, content: str) -> ChatMessage:
    return cast(ChatMessage, {"role": role, "content": content})


def test_count_tokens_grows_with_content() -> None:
    short = count_tokens([_m("user", "hi")], model="gpt-4o-mini")
    long = count_tokens([_m("user", "hi " * 200)], model="gpt-4o-mini")
    assert long > short > 0


def test_trim_history_fits_under_budget() -> None:
    system = _m("system", "you are a stylist")
    history = [_m("user" if i % 2 == 0 else "assistant", f"msg {i} " * 50) for i in range(20)]
    result = trim_history(system, history, model="gpt-4o-mini", budget=400)

    assert result.kept[0] == system
    assert len(result.dropped) > 0
    assert len(result.kept) - 1 + len(result.dropped) == len(history)
    assert count_tokens(result.kept, model="gpt-4o-mini") <= 400


def test_trim_history_keeps_everything_under_big_budget() -> None:
    system = _m("system", "sys")
    history = [_m("user", "tiny") for _ in range(3)]
    result = trim_history(system, history, model="gpt-4o-mini", budget=10_000)
    assert result.dropped == []
    assert len(result.kept) == 4
