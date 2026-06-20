//! Pointer-event dispatcher (entry point + arm fan-out).
//!
//! The 893-LOC `dispatch_pointer_with_text` god-function was split by event
//! kind into sibling modules (blindagem Fase 3.2): `pointer_move`,
//! `pointer_down` (+ `pointer_down_menus`), `pointer_up`. This file keeps only
//! the two public entry points and the `match event.kind` fan-out. The arm
//! modules use the exact same `super::` paths, so the move is behaviour-neutral
//! (covered by `dispatch::tests`).

use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{PointerEvent, PointerKind};
use ph2d_text::TextSystem;

/// Entry point for pointer events. Updates [`WidgetStore`] hover /
/// active / focus cursors based on the hit-test, transitions the
/// per-widget interactive state, and emits widget events into the
/// caller's frame-local arena.
///
/// Returns the events emitted for this single dispatch call. Caller
/// drains synchronously; after the frame ends, caller resets the
/// arena (deallocates events for the next frame).
///
/// Approximate click→byte mapping (no real glyph measurement). For
/// pixel-accurate caret placement on text widgets, prefer
/// [`dispatch_pointer_with_text`] and pass a live [`TextSystem`].
pub fn dispatch_pointer<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    dispatch_pointer_with_text(store, hit_index, event, None, arena)
}

/// Like [`dispatch_pointer`] but takes an optional `TextSystem`. When
/// `Some`, the click→byte mapping uses real glyph layout (binary
/// search the nearest glyph boundary) so the caret lands exactly
/// where the user clicked. When `None`, falls back to the
/// `font_size * APPROX_ADVANCE_RATIO` heuristic — adequate for
/// tests, but visibly off on long lines or proportional content.
pub fn dispatch_pointer_with_text<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    text_system: Option<&mut TextSystem>,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);

    // Each event is exactly one kind; route to the matching arm. `text_system`
    // is consumed by whichever arm needs real glyph layout (Move/Down); Up
    // never touches it.
    match event.kind {
        PointerKind::Move => {
            super::pointer_move::dispatch_move(store, hit_index, event, text_system, &mut events)
        }
        PointerKind::Down => {
            super::pointer_down::dispatch_down(store, hit_index, event, text_system, &mut events)
        }
        PointerKind::Up => super::pointer_up::dispatch_up(store, hit_index, event, &mut events),
    }

    events.into_bump_slice()
}
