# Navigate Agent Messages with Alt-Scroll

## Behavior

- Alt-scroll up navigates to the previous user message.
- Alt-scroll down navigates to the next user message.
- Place the destination user message at the top of the conversation viewport.
- Consume recognized Alt-scroll input without also applying ordinary list scrolling.
- Leave ordinary scrolling unchanged when Alt is not held.
- Normalize line-based mouse-wheel input and pixel-based trackpad input.

## Implementation

- Reuse the existing `ScrollOutputToPreviousMessage` and `ScrollOutputToNextMessage` behavior.
- Keep wheel handling local to `ThreadView`.
- Add only the minimal local gesture state needed to normalize trackpad deltas.
- Do not add actions, keymap entries, settings, or GPUI behavior unless local event handling proves
  insufficient.

## Verification

- Test both directions, first/last-message boundaries, unmodified scrolling, trackpad accumulation,
  and interaction with streaming response anchoring.
- Verify behavior over normal conversation content and embedded scrollable entries.
- Run focused `agent_ui` tests, formatting, clippy, and `cargo run`.
