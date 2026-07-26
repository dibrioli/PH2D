//! Inline **property-expression** field (ADR-0144) — the formula that drives a
//! property, authored in text.
//!
//! Opened from a track menu's "Expression\u{2026}" row, it floats at the click
//! position (so it needs no row geometry) as a single-line `TextInput`, seeded
//! with the binding's current formula. It is the EXACT shape of [`crate::clip_rename`]:
//! the field text lives in the `WidgetStore`, so the shell's global focus routing
//! feeds it characters and the timeline's own M / Delete / Ctrl+S shortcuts
//! auto-suppress while typing (a focused `TextInput` trips the shell gate).
//!
//! Enter (or click-away) commits via [`ph2d_timeline::TimelineIntent::SetBindingExpr`];
//! Esc cancels. An EMPTY field clears the expression (back to keyframes) — the
//! intent handler normalizes whitespace-only text to `None`.

use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, PanelHostInternal};
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{AnimTarget, TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken, Theme};

use crate::ids;
use crate::state::{self, TimelinePanelState};

/// Width of the inline formula field.
const FIELD_W: f32 = 200.0; // LITERAL-PX-OK: expression field width

/// Open the field for `target`'s binding at the click position `(x, y)`.
pub(crate) fn open(state: &mut TimelinePanelState, target: u64, x: f32, y: f32) {
    state.expr_edit = Some(state::ExprEdit {
        target,
        x,
        y,
        opened: false,
    });
}

/// Abandon the open edit without committing (Esc).
pub(crate) fn cancel(state: &mut TimelinePanelState) {
    state.expr_edit = None;
}

/// Everything the expression cluster answers: the menu row that OPENS the field,
/// and the field's own Submit/Blur/Cancel. `None` = not ours (the caller falls
/// through). One arm in `apply_event`, owned like the marker menu, so opening the
/// field (which mutates `state`) and committing it live in one place.
pub(crate) fn route(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: &WidgetEvent,
) -> Option<EventOutcome> {
    match *ev {
        WidgetEvent::Click(id) if id == ids::CTX_MENU_TL_EXPR => {
            Some(crate::event_track_menu::open_expr(state, host))
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::TIMELINE_TRACK_EXPR_INPUT => {
            commit(state, host.store());
            Some(EventOutcome::Consumed)
        }
        WidgetEvent::Cancel(id) if id == ids::TIMELINE_TRACK_EXPR_INPUT => {
            cancel(state);
            Some(EventOutcome::Consumed)
        }
        _ => None,
    }
}

/// The current formula for `target` in the snapshot — the seed. `""` when the
/// binding has no expression yet (a fresh authoring).
fn current_expr(snap: &TimelineViewSnapshot, target: u64) -> String {
    snap.tracks
        .iter()
        .find(|t| t.target.get() == target)
        .and_then(|t| t.expr.clone())
        .unwrap_or_default()
}

/// Paint the open field (no-op when none is open). Called last in the panel paint
/// so it overlays the sheet.
pub(crate) fn paint(
    state: &mut TimelinePanelState,
    ctx: &mut PaintCtx,
    theme: Theme,
    snap: &TimelineViewSnapshot,
) {
    let Some(mut ee) = state.expr_edit else {
        return;
    };
    // The track may have vanished (deleted / undo). Abandon rather than author a
    // formula onto whatever slid into its place.
    if !snap.tracks.iter().any(|t| t.target.get() == ee.target) {
        state.expr_edit = None;
        return;
    }

    let rect = Rect::new(ee.x, ee.y, FIELD_W, ROW_H_PX);

    // First frame: seed with the current formula, caret at the end, claim focus —
    // ONCE (re-seeding every frame would stomp the user's typing).
    if !ee.opened {
        let seed = current_expr(snap, ee.target);
        let caret = seed.len();
        ctx.host.store_mut().register(
            ids::TIMELINE_TRACK_EXPR_INPUT,
            InteractiveState::TextInput {
                state: TextInputState::Focused,
                text: seed,
                caret,
                selection_anchor: None,
            },
        );
        ctx.host
            .store_mut()
            .set_focus(Some(ids::TIMELINE_TRACK_EXPR_INPUT));
        ee.opened = true;
        state.expr_edit = Some(ee);
    }

    // A framed overlay so the field reads as an editor floating over the sheet.
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

    let (ti_state, text, caret, anchor) = match ctx.host.store().get(ids::TIMELINE_TRACK_EXPR_INPUT)
    {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Focused, String::new(), 0, None),
    };
    let input = TextInput::new(ids::TIMELINE_TRACK_EXPR_INPUT, "").state(ti_state);
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
        .register(ids::TIMELINE_TRACK_EXPR_INPUT, rect);
}

/// Commit the open edit: push `SetBindingExpr` with the field text (empty ->
/// `None`, back to keyframes), and close. Fires on Enter (Submit) and click-away
/// (Blur); the `take` makes the Enter->Submit+Blur pair idempotent.
pub(crate) fn commit(state: &mut TimelinePanelState, store: &WidgetStore) {
    let Some(ee) = state.expr_edit.take() else {
        return;
    };
    let text = field_text(store).unwrap_or_default();
    let trimmed = text.trim();
    state::push_intent(TimelineIntent::SetBindingExpr {
        target: AnimTarget::new(ee.target),
        expr: (!trimmed.is_empty()).then(|| trimmed.to_string()),
    });
}

/// The live text of the field, if it is a `TextInput`.
fn field_text(store: &WidgetStore) -> Option<String> {
    match store.get(ids::TIMELINE_TRACK_EXPR_INPUT) {
        Some(InteractiveState::TextInput { text, .. }) => Some(text.clone()),
        _ => None,
    }
}
