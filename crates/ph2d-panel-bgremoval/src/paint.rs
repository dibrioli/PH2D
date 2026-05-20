//! Background-Removal panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect taken from `ctx.layout.bgremoval` (Inspector slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface, [`paint_panel_title`],
//!   [`paint_segmented_button`] (Mode), [`paint_slider_with_chip_layout`]
//!   (Tolerance / Feather / Refine), Cancel + Apply buttons. Every
//!   painter is the SHARED source-of-truth from `panel_chrome` /
//!   `widget` — no panel-local widget look.
//! - `content_h` / `visible_h` publish for scroll bounds.
//!
//! The Mode highlight reads the per-frame [`crate::state::current_snapshot`]
//! the host publishes; slider track positions read the live stored value
//! (so a drag updates instantly), falling back to the snapshot.
//!
//! The live preview is NOT painted here — it renders on the canvas
//! (shell-side overlay), in place of the real sprite image.

use crate::state::{self, BgRemovalPanelState, set_last_content_h, set_last_visible_h};
use crate::{BgRemovalPanel, ids};
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::tools::bgremoval::BgRemovalMode;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title, paint_segmented_group,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, paint_button, paint_slider_with_chip_layout,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// Label column width for slider rows (passed to the canonical
/// `paint_slider_with_chip_layout`). // LITERAL-PX-OK: panel grid metric
const LABEL_COL_W: f32 = 76.0;

pub(crate) fn paint(_state: &mut BgRemovalPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(BgRemovalPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // BGR_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::BGR_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.bgremoval;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::BGR_PANEL, rect);

    // Dark-glass surface + corner accent — identical chrome to the
    // Inspector / Grid Settings panels.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();

    // Canonical panel title (single source of truth).
    let title_size = paint_panel_title(rect, "Bg Removal", 0.0, ctx.scene, ctx.text_system, theme);
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    // Disjoint borrows: store + hit_index from host; scene + text_system
    // are sibling fields on ctx; theme is a Copy.
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // ── Mode segmented control (Chroma | Smart Cut) ────────────────
    // Canonical segmented GROUP (central layout + gap, outlined).
    paint_segmented_group(
        Rect::new(inner_x, y, inner_w, row_h),
        &[
            (
                "Chroma",
                snapshot.mode == BgRemovalMode::Chroma,
                ids::BGR_MODE_CHROMA,
            ),
            (
                "Smart Cut",
                snapshot.mode == BgRemovalMode::GrabCut,
                ids::BGR_MODE_GRABCUT,
            ),
        ],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + row_gap * 2.0;

    // ── Sliders ────────────────────────────────────────────────────
    // CANONICAL `paint_slider_with_chip_layout` (rectangular track +
    // numeric chip) — the exact painter the Widget Gallery showcase
    // renders. `chip_id = NodeId(0)` ⇒ read-only value readout.
    let chip_w = Spacing::Xl.px() * 2.0;
    for (label, id, fallback) in [
        ("Tolerance", ids::BGR_TOLERANCE, snapshot.tolerance01),
        ("Feather", ids::BGR_FEATHER, snapshot.feather01),
        ("Refine", ids::BGR_REFINE, snapshot.refine01),
    ] {
        let value = store.slider(id).map(|(_, v)| v).unwrap_or(fallback);
        paint_slider_with_chip_layout(
            Rect::new(inner_x, y, inner_w, row_h),
            label,
            value,
            value as f64,
            None,
            id,
            NodeId(0),
            LABEL_COL_W,
            chip_w,
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
        .button_state(ids::BGR_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::BGR_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::BGR_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::BGR_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_APPLY, apply_rect);
    y += row_h;

    // Body fits without scroll; publish height as both content +
    // visible so the orchestrator's scroll clamp is a no-op.
    let used_h = (y - rect.y + PANEL_HEAD_PAD).min(rect.h);
    set_last_content_h(used_h);
    set_last_visible_h(rect.h);
}
