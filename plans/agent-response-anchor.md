# Anchor Agent Responses to User Messages

## Behavior

- Add an Agent setting that defaults to anchoring each response to its user message.
- Position every newly inserted user message at the top of the conversation viewport.
- Keep the viewport fixed while response entries are inserted or remeasured.
- Respect every manual scroll position, including the current bottom.
- Re-anchor only when another user message is inserted or the existing navigation control is used.
- Preserve Zed's current tail-following behavior when the setting is disabled.
- Preserve each thread's saved scroll position when switching threads.

## Implementation

- Add `agent.anchor_response_to_user_message` through the existing settings content, runtime,
  defaults, and Settings UI pipeline.
- Use `FollowMode::Normal` when anchoring is enabled and `FollowMode::Tail` otherwise.
- Skip the existing pre-send bottom scroll in anchored mode.
- Reuse `ThreadView::scroll_to_user_message_index` from the `AcpThreadEvent::NewEntry` path.
- Do not add a parallel scroll-state model or modify GPUI.

## Verification

- Test anchored streaming, a second user turn, manual positioning, stop/error behavior, thread
  restoration, and disabled legacy behavior.
- Run focused `agent_ui` tests, formatting, clippy, and `cargo run`.
