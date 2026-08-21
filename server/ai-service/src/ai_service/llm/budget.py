from dataclasses import dataclass
from typing import TypedDict, cast

import tiktoken


class ChatMessage(TypedDict, total=False):
    role: str
    content: str
    name: str
    tool_call_id: str


def _encoding_for(model: str) -> tiktoken.Encoding:
    try:
        return tiktoken.encoding_for_model(model)
    except KeyError:
        return tiktoken.get_encoding("cl100k_base")


_PER_MESSAGE_OVERHEAD = 4
_PER_REPLY_OVERHEAD = 2


def count_tokens(messages: list[ChatMessage], *, model: str) -> int:
    enc = _encoding_for(model)
    total = 0
    for msg in messages:
        total += _PER_MESSAGE_OVERHEAD
        for value in msg.values():
            if isinstance(value, str):
                total += len(enc.encode(value))
    total += _PER_REPLY_OVERHEAD
    return total


@dataclass(slots=True)
class TrimResult:
    kept: list[ChatMessage]
    dropped: list[ChatMessage]


def trim_history(
    system_message: ChatMessage,
    history: list[ChatMessage],
    *,
    model: str,
    budget: int,
) -> TrimResult:
    """Greedily drop oldest history messages until total fits the budget.

    `system_message` is always kept. Returns the kept tail (with the system
    prefix prepended) and the dropped prefix.
    """
    enc = _encoding_for(model)

    def _tokens(msg: ChatMessage) -> int:
        n = _PER_MESSAGE_OVERHEAD
        for v in msg.values():
            if isinstance(v, str):
                n += len(enc.encode(v))
        return n

    system_tokens = _tokens(system_message) + _PER_REPLY_OVERHEAD
    available = max(0, budget - system_tokens)

    tail: list[ChatMessage] = []
    used = 0
    for msg in reversed(history):
        cost = _tokens(msg)
        if used + cost > available:
            break
        tail.append(msg)
        used += cost
    tail.reverse()

    drop_count = len(history) - len(tail)
    dropped = history[:drop_count]
    return TrimResult(kept=[system_message, *tail], dropped=dropped)


def summary_message(summary_text: str) -> ChatMessage:
    return cast(
        ChatMessage,
        {"role": "system", "content": f"[summary] {summary_text}"},
    )
