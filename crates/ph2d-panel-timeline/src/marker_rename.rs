//! Inline marker-rename field (W4.T3).
//!
//! Double-clicking a marker pennant opens a single-line `TextInput` just below
//! the ruler, seeded with the marker's current label. It mirrors the hierarchy
//! row rename (the uncoupled template): the field text lives in the `WidgetStore`
//! like every other text field, so the global focus routing feeds it characters
//! and — because a focused `TextInput` trips the shell's `vector_text_field_focused`
//! gate — the timeline's own M / Delete / Ctrl+S/O shortcuts auto-suppress while
//! typing, with no extra gating.
//!
//! Enter (or a click away) commits via `RenameMarker`; Esc cancels. Those events
//! reach [`crate::event::apply_event`], which calls [`commit`] / [`cancel`].

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken, Theme};

use crate::ids;
use crate::ruler::RULER_H;
use crate::state::{self, TimelinePanelState};

/// Width of the inline field.
const FIELD_W: f32 = 120.0; // LITERAL-PX-OK: marker rename field width

/// Paint the open rename field (no-op when none is open). Called last in the
/// panel paint so the field overlays the ruler + lanes. Anchored at the marker's
/// x, clamped to stay fully on-screen.
pub(crate) fn paint(
    state: &mut TimelinePanelState,
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    view_start: f64,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
    let Some(mut mr) = state.marker_rename else {
        return;
    };
    // The marker may have vanished (deleted, or an undo dropped it) — abandon the
    // rename rather than key a stale index.
    let Some((t, label)) = snap.markers.get(mr.index) else {
        state.marker_rename = None;
        return;
    };

    let x = region.x + ((t - view_start) * px_per_s) as f32;
    let hi = (region.x + region.w - FIELD_W).max(region.x);
    let field_x = safe_clamp(x, region.x, hi);
    let rect = Rect::new(field_x, region.y + RULER_H, FIELD_W, ROW_H_PX);

    // First frame the rename is open: seed the field with the current label,
    // caret at the end, and claim focus — ONCE (re-seeding every frame would
    // stomp the user's typing). Force-overwrite `register` (like the hierarchy)
    // resets only buffer/caret/state; the id owns no colour/scroll side tables.
    if !mr.opened {
        ctx.host.store_mut().register(
            ids::TIMELINE_MARKER_RENAME_INPUT,
            InteractiveState::TextInput {
                state: TextInputState::Focused,
                text: label.clone(),
                caret: label.len(),
                selection_anchor: None,
            },
        );
        ctx.host
            .store_mut()
            .set_focus(Some(ids::TIMELINE_MARKER_RENAME_INPUT));
        mr.opened = true;
        state.marker_rename = Some(mr);
    }

    // A framed overlay so the field reads as an editor floating over the lanes.
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::TimelinePlayhead, theme),
    );

    let (ti_state, text, caret, anchor) =
        match ctx.host.store().get(ids::TIMELINE_MARKER_RENAME_INPUT) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            _ => (TextInputState::Focused, String::new(), 0, None),
        };
    let input = TextInput::new(ids::TIMELINE_MARKER_RENAME_INPUT, "").state(ti_state);
    paint_text_input_with_buffer(
        &input,
        Some(text.as_str()),
        Some(caret),
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_MARKER_RENAME_INPUT, rect);
}

/// Commit the open rename: push a `RenameMarker` with the trimmed field text (an
/// empty label is ignored — the marker keeps its old name), and close the field.
/// Fires on Enter (Submit) and on click-away / focus loss (Blur); the `take`
/// makes the second of the Enter→Submit+Blur pair a no-op.
pub(crate) fn commit(state: &mut TimelinePanelState, store: &WidgetStore) {
    let Some(mr) = state.marker_rename.take() else {
        return;
    };
    if let Some(label) = field_text(store) {
        let label = label.trim();
        if !label.is_empty() {
            state::push_intent(TimelineIntent::RenameMarker {
                index: mr.index,
                label: label.to_string(),
            });
        }
    }
}

/// Abandon the open rename without committing (Esc).
pub(crate) fn cancel(state: &mut TimelinePanelState) {
    state.marker_rename = None;
}

/// The live text of the rename field, if it is a `TextInput`.
fn field_text(store: &WidgetStore) -> Option<String> {
    match store.get(ids::TIMELINE_MARKER_RENAME_INPUT) {
        Some(InteractiveState::TextInput { text, .. }) => Some(text.clone()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "marker_rename_tests.rs"]
mod tests;
