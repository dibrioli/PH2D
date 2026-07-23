//! Inline **name** field — the clip selector's (W5) and the Containers list's.
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
//!
//! The Containers list arrived at the same pencil from the other direction: there the
//! double-click is taken — it ENTERS the container (Enio, 2026-07-21) — so a rename could
//! not be one. [`crate::state::RenameKind`] is what lets both use this one field instead of
//! growing a second one that would drift on the next fix.

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
    chip: Rect,
    snap: &TimelineViewSnapshot,
) {
    let Some(mut cr) = state.clip_rename else {
        return;
    };
    // The clip (or container) may have vanished — deleted, or an undo dropped it. Abandon the
    // rename rather than rename whatever slid into that index.
    let Some(name) = current_name(snap, cr) else {
        state.clip_rename = None;
        return;
    };

    // **Over the chip it renames** — same left edge, same row, at least as wide as
    // the chip so it covers it rather than sitting beside it. A rename field that
    // floats somewhere else is a field with nothing to say what it is renaming.
    let rect = Rect::new(chip.x, chip.y, FIELD_W.max(chip.w), ROW_H_PX);

    // First frame the rename is open: seed the field with the current name, caret
    // at the end, and claim focus — ONCE (re-seeding every frame would stomp the
    // user's typing and reset the caret).
    if !cr.opened {
        ctx.host.store_mut().register(
            ids::TIMELINE_CLIP_RENAME_INPUT,
            InteractiveState::TextInput {
                state: TextInputState::Focused,
                text: name.to_string(),
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

/// The name the field seeds from — the ONE place that maps a [`state::RenameKind`] to a list,
/// so the seeding and the commit cannot disagree about what is being renamed.
fn current_name(snap: &TimelineViewSnapshot, cr: state::ClipRename) -> Option<&str> {
    match cr.kind {
        state::RenameKind::Clip => snap.clips.get(cr.index).map(String::as_str),
        state::RenameKind::Container => snap.containers.get(cr.index).map(|c| c.name.as_str()),
        state::RenameKind::Lane => snap.lanes.get(cr.index).map(|l| l.name.as_str()),
    }
}

/// Open the rename field on the ACTIVE clip (the pencil button in the transport bar).
pub(crate) fn open(state: &mut TimelinePanelState, snap: &TimelineViewSnapshot) {
    state.clip_rename = Some(state::ClipRename {
        kind: state::RenameKind::Clip,
        index: snap.active_clip,
        opened: false,
    });
}

/// Open the rename field on container `index` (a pencil in the Containers list).
pub(crate) fn open_container(state: &mut TimelinePanelState, index: usize) {
    state.clip_rename = Some(state::ClipRename {
        kind: state::RenameKind::Container,
        index,
        opened: false,
    });
}

/// Open the rename field on lane `index` (Rename Lane in the lane's right-click menu).
/// `index` is the lane's position within the OPEN host's stack — the snapshot's `lanes`.
pub(crate) fn open_lane(state: &mut TimelinePanelState, index: usize) {
    state.clip_rename = Some(state::ClipRename {
        kind: state::RenameKind::Lane,
        index,
        opened: false,
    });
}

/// **Where a lane's rename field floats** — over the label column of its own row, the
/// sibling of [`crate::container_list::rename_anchor`]. `None` when no lane rename is open,
/// the view is not a lanes view, or the row is scrolled out of the band.
pub(crate) fn lane_rename_anchor(
    g: &crate::geom::Geom,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) -> Option<ph2d_editor_core::zones::Rect> {
    let cr = state
        .clip_rename
        .filter(|c| c.kind == state::RenameKind::Lane)?;
    // A lanes view (Arrange, or inside a container) — never the Keys tab or the list.
    if crate::tab::rows(state.tab, snap) != crate::tab::Rows::Lanes {
        return None;
    }
    let region = g.rows;
    let (_, y, h) = crate::geom::stack_bands(snap, state.tab, region.y, state.scroll_y)
        .find(|(i, _, _)| *i == cr.index)?;
    (y >= region.y && y + h <= region.y + region.h)
        .then(|| ph2d_editor_core::zones::Rect::new(region.x, y, g.label_w, h))
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
            // The kind picks the intent, and it is the same kind the seed read from — one
            // enum, so typing into a container's field can never rename a clip.
            state::push_intent(match cr.kind {
                state::RenameKind::Clip => TimelineIntent::RenameClip {
                    index: cr.index,
                    name: name.to_string(),
                },
                state::RenameKind::Container => TimelineIntent::RenameContainer {
                    index: cr.index,
                    name: name.to_string(),
                },
                state::RenameKind::Lane => TimelineIntent::RenameLane {
                    lane: cr.index,
                    name: name.to_string(),
                },
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
