//! Equalize Sizes panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot, panel title,
//!   then sections:
//!     1. **Target** — 3 mode buttons (accent the active one); when
//!        Fixed → W and H chips; when GridUnit → slider + chip.
//!     2. **Upscale** — Upscale-if-smaller toggle; when on → 3
//!        algorithm buttons (accent active).
//!     3. **Rasterize** — single toggle.
//!     4. **Actions** — Cancel + Apply.
//! - Every painter is the SHARED source-of-truth from
//!   `panel_chrome` / `widget` — no panel-local widget look.

use crate::state::{self, EqualizeSizesPanelState, set_last_content_h, set_last_visible_h};
use crate::{EqualizeSizesPanel, ids};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, paint_button, paint_slider_with_chip_layout,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_equalize_sizes::params::{TargetMode, UpscaleAlgorithm, grid_unit_to_slider};
use ph2d_vector::VectorScene;

/// Label column width for slider rows. // LITERAL-PX-OK: panel grid metric
const LABEL_COL_W: f32 = 72.0;

pub(crate) fn paint(_state: &mut EqualizeSizesPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(EqualizeSizesPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // EQS_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::EQS_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.padding;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::EQS_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();

    let title_size = paint_panel_title(
        rect,
        "Equalize Sizes",
        0.0,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // ── Section: Target mode (3-way radio) ──────────────────────────
    let mode_row = Rect::new(inner_x, y, inner_w, row_h);
    paint_radio_row(
        mode_row,
        &[
            (
                "Max",
                ids::EQS_MODE_MAX,
                snapshot.target_mode == TargetMode::MaxOfSelection,
            ),
            (
                "Fixed",
                ids::EQS_MODE_FIXED,
                snapshot.target_mode == TargetMode::Fixed,
            ),
            (
                "Grid",
                ids::EQS_MODE_GRID,
                snapshot.target_mode == TargetMode::GridUnit,
            ),
        ],
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += row_h + row_gap;

    // ── Mode-conditional rows ───────────────────────────────────────
    match snapshot.target_mode {
        TargetMode::Fixed => {
            let chip_gap = Spacing::Sm.px();
            let half = ((inner_w - chip_gap) * 0.5).max(0.0);
            // Manually paint a simple labeled chip pair (W=…, H=…). The
            // editor's shared `paint_slider_with_chip_layout` expects a
            // slider, so we inline a small chip row painter here.
            paint_labeled_chip(
                Rect::new(inner_x, y, half, row_h),
                "W",
                ids::EQS_FIXED_W,
                snapshot.fixed_w as f64,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            paint_labeled_chip(
                Rect::new(inner_x + half + chip_gap, y, half, row_h),
                "H",
                ids::EQS_FIXED_H,
                snapshot.fixed_h as f64,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            y += row_h + row_gap;
        }
        TargetMode::GridUnit => {
            let chip_w = Spacing::Xl.px() * 2.0;
            let track = store
                .slider(ids::EQS_GRID_UNIT)
                .map(|(_, v)| v)
                .unwrap_or_else(|| grid_unit_to_slider(snapshot.grid_unit));
            let px = store
                .number_value(ids::EQS_GRID_UNIT_NUM)
                .unwrap_or(snapshot.grid_unit as f64)
                .round() as i64;
            let px_display = px.to_string();
            paint_slider_with_chip_layout(
                Rect::new(inner_x, y, inner_w, row_h),
                "Grid",
                track,
                px as f64,
                Some(&px_display),
                ids::EQS_GRID_UNIT,
                ids::EQS_GRID_UNIT_NUM,
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
        TargetMode::MaxOfSelection => {
            // No extra row — preview text (Final size: …) is optional v2.
        }
    }

    y += row_gap;

    // ── Section: Upscale if smaller (accent toggle) ─────────────────
    let upscale_on = snapshot.upscale_if_smaller;
    paint_toggle_button(
        Rect::new(inner_x, y, inner_w, row_h),
        "Upscale if smaller",
        ids::EQS_UPSCALE_IF_SMALLER,
        upscale_on,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += row_h + row_gap;

    if upscale_on {
        let alg_row = Rect::new(inner_x, y, inner_w, row_h);
        paint_radio_row(
            alg_row,
            &[
                (
                    "Lanczos",
                    ids::EQS_ALG_LANCZOS,
                    snapshot.upscale_algorithm == UpscaleAlgorithm::Lanczos3,
                ),
                (
                    "Nearest",
                    ids::EQS_ALG_NEAREST,
                    snapshot.upscale_algorithm == UpscaleAlgorithm::Nearest,
                ),
                (
                    "xBR",
                    ids::EQS_ALG_XBR,
                    snapshot.upscale_algorithm == UpscaleAlgorithm::Xbr,
                ),
            ],
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += row_h + row_gap;
    }

    y += row_gap;

    // ── Section: Rasterize after (accent toggle) ────────────────────
    paint_toggle_button(
        Rect::new(inner_x, y, inner_w, row_h),
        "Rasterize after",
        ids::EQS_RASTERIZE_AFTER,
        snapshot.rasterize_after,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += row_h + row_gap;

    y += row_gap;

    // ── Cancel + Apply row ─────────────────────────────────────────
    let btn_gap = Spacing::Sm.px();
    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store
        .button_state(ids::EQS_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::EQS_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::EQS_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::EQS_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::EQS_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::EQS_APPLY, apply_rect);
    y += row_h;

    let used_h = (y - rect.y + PANEL_HEAD_PAD).min(rect.h);
    set_last_content_h(used_h);
    set_last_visible_h(rect.h);
}

/// Paint a horizontal row of N equal-width buttons that behave as a
/// radio group (active one is `ButtonKind::Accent`, others
/// `ButtonKind::Default`). The Tool's `handle_panel_event` does the
/// actual selection — this is paint-only.
#[allow(clippy::too_many_arguments)]
fn paint_radio_row(
    rect: Rect,
    items: &[(&str, NodeId, bool)],
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if items.is_empty() {
        return;
    }
    let gap = Spacing::Sm.px();
    let n = items.len() as f32;
    let item_w = ((rect.w - gap * (n - 1.0)) / n).max(0.0);
    for (i, (label, id, active)) in items.iter().enumerate() {
        let item_rect = Rect::new(rect.x + (item_w + gap) * i as f32, rect.y, item_w, rect.h);
        let kind = if *active {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let btn_state = if *active {
            ButtonState::Pressed
        } else {
            store.button_state(*id).unwrap_or(ButtonState::Normal)
        };
        let b = Button::new(*id, *label).kind(kind).state(btn_state);
        paint_button(&b, item_rect, scene, text_system, theme);
        hit_index.register(*id, item_rect);
    }
}

/// Paint a single accent-when-on toggle button.
#[allow(clippy::too_many_arguments)]
fn paint_toggle_button(
    rect: Rect,
    label: &str,
    id: NodeId,
    on: bool,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let kind = if on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let btn_state = if on {
        ButtonState::Pressed
    } else {
        store.button_state(id).unwrap_or(ButtonState::Normal)
    };
    let b = Button::new(id, label).kind(kind).state(btn_state);
    paint_button(&b, rect, scene, text_system, theme);
    hit_index.register(id, rect);
}

/// Paint a label + NumberInput chip pair on one row (no slider). The
/// chip uses the stored number_value (already mirrored by the host on
/// snapshot push). Layout: `[ label : chip ]`.
#[allow(clippy::too_many_arguments)]
fn paint_labeled_chip(
    rect: Rect,
    label: &str,
    chip_id: NodeId,
    value: f64,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // Reuse `paint_slider_with_chip_layout` with a degenerate
    // (zero-width) slider track — keeps a single source of truth for
    // chip paint and label baseline; the slider region collapses behind
    // the label column.
    let chip_w = (rect.w * 0.55).max(Spacing::Xl.px() * 2.0);
    let label_col = (rect.w - chip_w - Spacing::Sm.px()).max(Spacing::Md.px());
    let display = value.round().to_string();
    paint_slider_with_chip_layout(
        rect,
        label,
        0.0, // slider track value (hidden by zero-width)
        value,
        Some(&display),
        // Re-use the chip_id as the "slider" id; the dispatcher will see
        // it as a number-only input because `register_only_chip` below
        // doesn't put a slider InteractiveState behind it. We don't
        // emit ValueChanged on the slider id (chip events are how the
        // tool gets updates).
        NodeId(0),
        chip_id,
        label_col,
        chip_w,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}
