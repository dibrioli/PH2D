//! Pointer + key dispatchers — translate raw shell events into
//! [`super::WidgetStore`] state mutations and [`super::WidgetEvent`]
//! emissions.
//!
//! Every dispatcher takes a `&'frame bumpalo::Bump` and returns a
//! `&'frame [WidgetEvent]` slice — the arena is the frame-local
//! event allocator. Caller drains the slice in the same frame and
//! resets the arena before the next frame.
//!
//! Phase 0 ships skeletons that only update the hover/focus
//! cursors. Click/drag/keyboard input wire up in Phases A-D as the
//! per-widget logic lands.

use super::{HitIndex, WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{KeyEvent, PointerEvent, PointerKind};

/// Entry point for pointer events. Updates [`WidgetStore::hot_id`]
/// based on the hit-test, then dispatches per-widget logic when the
/// per-widget wire-up exists (Phases A-D).
///
/// Returns the events emitted this dispatch as a slice in the
/// caller's arena. Slice is empty in Phase 0.
pub fn dispatch_pointer<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    let hit = hit_index.hit(event.x, event.y);

    match event.kind {
        PointerKind::Move => {
            // Hover tracking: hot_id follows the cursor regardless of
            // whether the widget under it has interactive logic wired
            // yet (Phase 0 baseline).
            if store.hot_id() != hit {
                store.set_hot(hit);
            }
        }
        PointerKind::Down | PointerKind::Up => {
            // Per-widget click/drag handling lands in Phases A-D.
        }
    }

    events.into_bump_slice()
}

/// Entry point for key events. Phase 0 baseline: routes Tab to
/// next-focus traversal; everything else stubs out until Phase A.
pub fn dispatch_key<'frame>(
    store: &mut WidgetStore,
    event: KeyEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let _ = (store, event); // wire-up in Phase A
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    events.into_bump_slice()
}

/// Character input from the IME / keyboard. Phase 0 stub; wired in
/// Phase C.
pub fn dispatch_text_input<'frame>(
    store: &mut WidgetStore,
    ch: char,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let _ = (store, ch);
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    events.into_bump_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractiveState;
    use crate::widget::ButtonState;
    use crate::zones::Rect;
    use ph2d_a11y::NodeId;
    use ph2d_host::PointerSource;

    fn move_event(x: f32, y: f32) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind: PointerKind::Move,
            source: PointerSource::Mouse,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn pointer_move_into_widget_sets_hot_id() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        let arena = Bump::new();

        let _ = dispatch_pointer(&mut store, &hits, move_event(50.0, 25.0), &arena);
        assert_eq!(store.hot_id(), Some(NodeId(7)));
    }

    #[test]
    fn pointer_move_outside_clears_hot_id() {
        let mut store = WidgetStore::with_capacity(4);
        store.set_hot(Some(NodeId(7)));
        let hits = HitIndex::new();
        let arena = Bump::new();

        let _ = dispatch_pointer(&mut store, &hits, move_event(500.0, 500.0), &arena);
        assert_eq!(store.hot_id(), None);
    }

    #[test]
    fn pointer_dispatch_returns_empty_slice_in_phase_0() {
        let mut store = WidgetStore::with_capacity(4);
        let hits = HitIndex::new();
        let arena = Bump::new();
        let evts = dispatch_pointer(&mut store, &hits, move_event(0.0, 0.0), &arena);
        assert!(evts.is_empty());
    }
}
