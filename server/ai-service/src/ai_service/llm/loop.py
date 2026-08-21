"""Tool-use loop driving the streaming chat handler.

Yields heterogeneous events:
- ("token", text) — assistant token deltas, forwarded as SSE `token`.
- ("products", [ids...]) — emitted when a tool returned product items.
- ("done", None) — emitted exactly once at the end of the loop.

Errors are raised as `AIServiceError` for the handler to convert into
`event: error`. The loop is bounded by `max_iterations` to defend against
runaway tool-use.
"""

from __future__ import annotations

import json
from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import Any, Literal

import structlog

from ..metrics import tool_calls_total
from .budget import ChatMessage, trim_history
from .client import AssistantTurn, LLMClient, TokenChunk
from .tools import TOOL_SCHEMAS, ToolContext, ToolResult, dispatch_tool

log = structlog.get_logger(__name__)

LoopEvent = tuple[Literal["token", "products"], object]


@dataclass(slots=True)
class LoopOutcome:
    text: str
    product_ids: list[str]
    iterations: int
    hit_cap: bool
    total_tokens: int = 0


def _tool_call_message(tcs) -> ChatMessage:
    return {  # type: ignore[typeddict-item]
        "role": "assistant",
        "content": "",
        "tool_calls": [
            {
                "id": tc.id,
                "type": "function",
                "function": {"name": tc.name, "arguments": tc.arguments or "{}"},
            }
            for tc in tcs
        ],
    }


def _tool_result_message(call_id: str, result: ToolResult) -> ChatMessage:
    return {  # type: ignore[typeddict-item]
        "role": "tool",
        "tool_call_id": call_id,
        "content": result.as_json(),
    }


def _final_assistant_message(text: str) -> ChatMessage:
    return {"role": "assistant", "content": text}  # type: ignore[typeddict-item]


async def run_loop(
    *,
    llm: LLMClient,
    tool_ctx: ToolContext,
    base_messages: list[ChatMessage],
    max_tool_iterations: int,
    history_budget: int,
    max_tokens_per_reply: int,
) -> AsyncIterator[LoopEvent | LoopOutcome]:
    messages: list[ChatMessage] = list(base_messages)
    product_ids: list[str] = []
    final_text_parts: list[str] = []
    iteration = 0
    hit_cap = False
    total_tokens = 0

    while True:
        iteration += 1
        force_no_tools = iteration > max_tool_iterations
        tools_arg = None if force_no_tools else TOOL_SCHEMAS
        tool_choice = "none" if force_no_tools else None
        if force_no_tools:
            hit_cap = True
            log.info("loop.tool_cap_hit", iteration=iteration)

        # Re-trim each iteration: tool_result blobs can be heavy.
        system_msg = messages[0]
        trimmed = trim_history(
            system_msg,
            messages[1:],
            model=llm.model,
            budget=history_budget,
        )
        if trimmed.dropped:
            log.info("loop.history_trimmed", dropped=len(trimmed.dropped))

        turn_text_parts: list[str] = []
        turn: AssistantTurn | None = None
        async for item in llm.chat_stream(
            trimmed.kept,
            max_tokens=max_tokens_per_reply,
            tools=tools_arg,
            tool_choice=tool_choice,
        ):
            if isinstance(item, TokenChunk):
                turn_text_parts.append(item.text)
                yield ("token", item.text)
            else:
                turn = item

        if turn is None:
            break

        total_tokens += turn.total_tokens

        if not turn.has_tool_calls or force_no_tools:
            final_text_parts.append(turn.text)
            break

        # Persist the assistant message that carried the tool_calls.
        messages.append(_tool_call_message(turn.tool_calls))

        for tc in turn.tool_calls:
            result = await dispatch_tool(tc.name, tc.arguments, ctx=tool_ctx)
            err_kind = result.payload.get("error") if isinstance(result.payload, dict) else None
            metric_result = err_kind if err_kind else "ok"
            tool_calls_total.labels(tool=tc.name, result=metric_result).inc()
            log.info(
                "loop.tool_result",
                tool=tc.name,
                items=len(result.product_ids),
                has_error=bool(err_kind),
            )
            messages.append(_tool_result_message(tc.id, result))
            if result.product_ids:
                product_ids = result.product_ids  # keep the latest tool's ids
                yield ("products", list(result.product_ids))

    final_text = "".join(final_text_parts).strip()
    if final_text:
        messages.append(_final_assistant_message(final_text))
    outcome = LoopOutcome(
        text=final_text,
        product_ids=product_ids,
        iterations=iteration,
        hit_cap=hit_cap,
        total_tokens=total_tokens,
    )
    # Yield outcome as the sentinel end-of-loop marker.
    yield outcome


def initial_history(
    system_prompt: str,
    history: list[ChatMessage],
) -> list[ChatMessage]:
    """Helper to assemble the initial message list for the loop."""
    system_msg: ChatMessage = {"role": "system", "content": system_prompt}  # type: ignore[typeddict-item]
    return [system_msg, *history]


def serialize_payload(payload: dict[str, Any]) -> str:
    return json.dumps(payload, ensure_ascii=False)
