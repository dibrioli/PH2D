//! Inline clip-rename field (W5 — clip selector).
//!
//! The pencil in the transport bar opens a single-line `TextInput` over the bar,
//! seeded with the active clip's name. Exactly the shape of the sibling
//! [`crate::marker_rename`] (which itself mirrors the hierarchy row rename): the
//! field text lives in the `WidgetStore` like every other text field, so the
//! global focus routing feeds it characters and — because a focused `TextInput`
//! trips the shell's `vector_text_field_focused` gate — the timeline's own
//! M / Delete / Ctrl+S/O shortcuts auto-suppress while typing, with no extra gating.
//!
//! Enter (or a click away) commits via `RenameClip`; Esc cancels. Those events
//! reach [`crate::event::apply_event`], which calls [`commit`] / [`cancel`].
//!
//! **Why a pencil and not a double-click** (the app's rename convention elsewhere):
//! the thing being renamed is a DROPDOWN chip, and the first click of a
//! double-click has already opened the list. A dedicated button has no such
//! ambiguity, and it is discoverable next to the `+` that made the clip.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken, Theme};

use crate::ids;
use crate::state::{self, TimelinePanelState};

/// Width of the inline field.
const FIELD_W: f32 = 140.0; // LITERAL-PX-OK: clip rename field width

/// Paint the open rename field (no-op when none is open). Called last in the panel
/// paint so the field overlays the transport bar it sits on.
pub(crate) fn paint(
    state: &mut TimelinePanelState,
    ctx: &mut PaintCtx,
    theme: Theme,
    body: Rect,
    snap: &TimelineViewSnapshot,
) {
    let Some(mut cr) = state.clip_rename else {
        return;
    };
    // The clip may have vanished (deleted, or an undo dropped it) — abandon the
    // rename rather than rename whatever slid into that index.
    let Some(name) = snap.clips.get(cr.index) else {
        state.clip_rename = None;
        return;
    };

    // Over the chip it renames, at the far left of the bar.
    let rect = Rect::new(body.x, body.y, FIELD_W.min(body.w), ROW_H_PX);

    // First frame the rename is open: seed the field with the current name, caret
    // at the end, and claim focus — ONCE (re-seeding every frame would stomp the
    // user's typing and reset the caret).
    if !cr.opened {
        ctx.host.store_mut().register(
            ids::TIMELINE_CLIP_RENAME_INPUT,
            InteractiveState::TextInput {
                state: TextInputState::Focused,
                text: name.clone(),
                caret: name.len(),
                selection_anchor: None,
            },
        );
        ctx.host
            .store_mut()
            .set_focus(Some(ids::TIMELINE_CLIP_RENAME_INPUT));
        cr.opened = true;
        state.clip_rename = Some(cr);
    }

    // A framed overlay so the field reads as an editor floating over the bar.
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
        match ctx.host.store().get(ids::TIMELINE_CLIP_RENAME_INPUT) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            _ => (TextInputState::Focused, String::new(), 0, None),
        };
    let input = TextInput::new(ids::TIMELINE_CLIP_RENAME_INPUT, "").state(ti_state);
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
        .register(ids::TIMELINE_CLIP_RENAME_INPUT, rect);
}

/// Open the rename field on the ACTIVE clip (the pencil button).
pub(crate) fn open(state: &mut TimelinePanelState, snap: &TimelineViewSnapshot) {
    state.clip_rename = Some(state::ClipRename {
        index: snap.active_clip,
        opened: false,
    });
}

/// Commit the open rename: push a `RenameClip` with the trimmed field text (an
/// empty name is ignored — the clip keeps its old one), and close the field. Fires
/// on Enter (Submit) and on click-away / focus loss (Blur); the `take` makes the
/// second of the Enter→Submit+Blur pair a no-op.
pub(crate) fn commit(state: &mut TimelinePanelState, store: &WidgetStore) {
    let Some(cr) = state.clip_rename.take() else {
        return;
    };
    if let Some(name) = field_text(store) {
        let name = name.trim();
        if !name.is_empty() {
            state::push_intent(TimelineIntent::RenameClip {
                index: cr.index,
                name: name.to_string(),
            });
        }
    }
}

/// Abandon the open rename without committing (Esc).
pub(crate) fn cancel(state: &mut TimelinePanelState) {
    state.clip_rename = None;
}

/// The live text of the rename field, if it is a `TextInput`.
fn field_text(store: &WidgetStore) -> Option<String> {
    match store.get(ids::TIMELINE_CLIP_RENAME_INPUT) {
        Some(InteractiveState::TextInput { text, .. }) => Some(text.clone()),
        _ => None,
    }
}
