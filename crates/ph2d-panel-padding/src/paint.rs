//! Padding panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot,
//!   [`paint_panel_title`], four labeled `NumberInput` rows (Top /
//!   Right / Bottom / Left), Cancel + Apply buttons. Every painter is
//!   the SHARED source-of-truth from `panel_chrome` / `widget` — no
//!   panel-local widget look.
//!
//! Field values seed from the per-frame [`crate::state::current_snapshot`]
//! the host publishes; the live stored buffer takes over while a field
//! is focused (so a keyboard edit shows instantly).
//!
//! No live preview in v1 — Apply bakes the resized canvas shell-side.

use crate::state::{self, PaddingPanelState, set_last_content_h, set_last_visible_h};
use crate::{PaddingPanel, ids};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title,
};
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, NumberInput, TextInputState, paint_button,
    paint_number_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Label column width for the edge rows. // LITERAL-PX-OK: panel grid metric
const LABEL_COL_W: f32 = 76.0;

pub(crate) fn paint(_state: &mut PaddingPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PaddingPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // PAD_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::PAD_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.padding;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::PAD_PANEL, rect);

    // Dark-glass surface + corner accent — identical chrome to the
    // Inspector / Bg Removal panels.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();

    // Canonical panel title (single source of truth).
    let title_size = paint_panel_title(rect, "Padding", 0.0, ctx.scene, ctx.text_system, theme);
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    // Disjoint borrows: store + hit_index from host; scene + text_system
    // are sibling fields on ctx; theme is Copy.
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // ── Four signed per-edge fields ────────────────────────────────
    // Each row: label column + a canonical `NumberInput` (keyboard +
    // drag-scrub via the canonical NumberInput dispatch). Positive = +px
    // transparent expand, negative = crop.
    for (label, id, fallback) in [
        ("Top", ids::PAD_TOP, snapshot.top),
        ("Right", ids::PAD_RIGHT, snapshot.right),
        ("Bottom", ids::PAD_BOTTOM, snapshot.bottom),
        ("Left", ids::PAD_LEFT, snapshot.left),
    ] {
        paint_edge_row(
            label,
            id,
            fallback,
            Rect::new(inner_x, y, inner_w, row_h),
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += row_h + row_gap;
    }

    y += row_gap;

    // ── Cancel (ghost) + Apply (accent CTA) row ────────────────────
    let btn_gap = Spacing::Sm.px();
    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store
        .button_state(ids::PAD_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::PAD_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::PAD_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::PAD_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::PAD_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::PAD_APPLY, apply_rect);
    y += row_h;

    // Body fits without scroll; publish height as both content + visible
    // so the orchestrator's scroll clamp is a no-op.
    let used_h = (y - rect.y + PANEL_HEAD_PAD).min(rect.h);
    set_last_content_h(used_h);
    set_last_visible_h(rect.h);
}

/// Paint one labeled `NumberInput` edge row. The displayed value seeds
/// from `fallback` (the per-frame snapshot) unless the field is focused,
/// in which case the live edit buffer takes over (so keyboard edits show
/// instantly). Mirrors `ph2d-panel-grid-snap::paint_number_row_from_state`.
#[allow(clippy::too_many_arguments)]
fn paint_edge_row(
    label: &str,
    id: NodeId,
    fallback: i32,
    row: Rect,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let (input_state, _stored_value, buffer, caret, anchor) = read_number_input(store, id);
    let label_font = TypeToken::Base.px();
    paint_text(
        text_system,
        scene,
        label,
        row.x,
        row.y + (row.h - label_font) * 0.5,
        label_font,
        LABEL_COL_W - Spacing::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    // Keep the live buffer for in-progress edits; otherwise display the
    // snapshot value (so reactivating / switching selection repaints the
    // tool's current spec).
    let buffer_arg = if input_state == TextInputState::Focused {
        Some(buffer)
    } else {
        None
    };
    let input_rect = Rect::new(row.x + LABEL_COL_W, row.y, row.w - LABEL_COL_W, row.h);
    let input = NumberInput::new(id, "", fallback as f64).state(input_state);
    paint_number_input_with_buffer(
        &input,
        buffer_arg,
        caret,
        anchor,
        input_rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, input_rect);
}
