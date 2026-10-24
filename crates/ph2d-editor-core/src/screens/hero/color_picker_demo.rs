//! Floating [`crate::widget::BlenderColorPicker`] anchored to the
//! bottom-right of the canvas.
//!
//! The picker is intentionally **not** parented to the Inspector
//! panel: it lives on top of the canvas so dragging it around (via
//! its drag handle) feels like a real floating tool palette. State —
//! retained color, palette, drag offset, channel/interpolation modes
//! — lives on the [`WidgetStore`] under [`ids::INSP_BLENDER_PICKER`]
//! (the legacy id name is preserved to minimize churn; the picker is
//! NOT actually rendered inside the Inspector).

use super::HeroLayout;
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::BlenderColorPicker;
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

const PICKER_W: f32 = 280.0; // LITERAL-PX-OK: Blender color picker fixed width (chrome-specific)
const PICKER_H: f32 = 620.0; // LITERAL-PX-OK: Blender color picker fixed height (chrome-specific; +60 for the Color Harmonies section)
const VIEWPORT_MIN_MARGIN: f32 = 40.0; // LITERAL-PX-OK: min viewport margin to host picker (chrome-specific)

/// Compute the picker rect for the current viewport + drag offset.
/// Returns `None` when the canvas is too small to host it — the
/// painter early-outs on `None` so a degenerate viewport doesn't
/// strand the picker off-screen.
pub fn current_picker_rect(layout: &HeroLayout, store: &WidgetStore) -> Option<Rect> {
    if layout.canvas.w < PICKER_W + VIEWPORT_MIN_MARGIN
        || layout.canvas.h < PICKER_H + VIEWPORT_MIN_MARGIN
    {
        return None;
    }
    // ⭐ O desvio arrastado é relativo ao encaixe AO LADO do Inspector (dono 24/10: abria por
    //    cima dele) — numa janela redimensionada o seletor acompanha a coluna.
    let (dx, dy) = store.blender_picker_offset(ids::INSP_BLENDER_PICKER);
    let (base_x, base_y) =
        picker_anchor_beside(layout.canvas, layout.inspector, PICKER_W, PICKER_H);
    let min_x = layout.canvas.x + Spacing::Md.px();
    let max_x = layout.canvas.x + layout.canvas.w - PICKER_W - Spacing::Md.px();
    // Drag handle (top of panel) must stay inside the canvas; the
    // panel may overflow the canvas bottom — user can drag it back
    // up via the handle.
    let min_y = layout.canvas.y;
    let max_y = layout.canvas.y + layout.canvas.h - 60.0; // LITERAL-PX-OK: drag handle min height inside canvas (chrome-specific)
    Some(Rect::new(
        crate::math::safe_clamp(base_x + dx, min_x, max_x),
        crate::math::safe_clamp(base_y + dy, min_y, max_y),
        PICKER_W,
        PICKER_H,
    ))
}

/// ⭐⭐ **Onde o seletor nasce: AO LADO do Inspector, nunca por cima** — o canto de cima do rect
/// `(w, h)`, encostado à coluna pelo lado de dentro da janela (coluna à direita ⇒ à esquerda
/// dela; coluna à esquerda, sob espelho ⇒ à direita), alinhado ao topo dela e preso à janela.
#[must_use]
pub fn picker_anchor_beside(window: Rect, inspector: Rect, w: f32, h: f32) -> (f32, f32) {
    let gap = Spacing::Md.px();
    let coluna_a_direita = inspector.x + inspector.w * 0.5 >= window.x + window.w * 0.5; // LITERAL-PX-OK: o meio, não um token
    let x = if coluna_a_direita {
        inspector.x - gap - w
    } else {
        inspector.x + inspector.w + gap
    };
    let y = inspector.y + gap;
    (
        crate::math::safe_clamp(x, window.x + gap, window.x + window.w - w - gap),
        crate::math::safe_clamp(y, window.y, window.y + window.h - h),
    )
}

pub fn paint_blender_picker_demo(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    // Hidden by default — only painted when some widget explicitly
    // opens it via `store.set_picker_target(...)`. The picker is
    // the editor's single color-picking surface; clicking outside
    // it (and outside another color target) closes it.
    if store.picker_target().is_none() {
        return;
    }
    let Some(rect) = current_picker_rect(layout, store) else {
        return;
    };
    let cp = BlenderColorPicker::new(ids::INSP_BLENDER_PICKER, tr("chrome.color.color"));
    let sub_ids = crate::widget::BlenderSubIds {
        parent: ids::INSP_BLENDER_PICKER,
        wheel: ids::BLENDER_WHEEL,
        value_slider: ids::BLENDER_VALUE_SLIDER,
        interp_linear: ids::BLENDER_INTERP_LINEAR,
        interp_perceptual: ids::BLENDER_INTERP_PERCEPTUAL,
        channel_rgb: ids::BLENDER_CHANNEL_RGB,
        channel_hsv: ids::BLENDER_CHANNEL_HSV,
        channel_oklch: ids::BLENDER_CHANNEL_OKLCH,
        channels: [
            ids::BLENDER_CHANNEL_0,
            ids::BLENDER_CHANNEL_1,
            ids::BLENDER_CHANNEL_2,
            ids::BLENDER_CHANNEL_3,
        ],
        channels_num: [
            ids::BLENDER_NUM_0,
            ids::BLENDER_NUM_1,
            ids::BLENDER_NUM_2,
            ids::BLENDER_NUM_3,
        ],
        hex: ids::BLENDER_HEX,
        add_swatch: ids::BLENDER_ADD_SWATCH,
        import_palette: ids::BLENDER_IMPORT_PALETTE,
        export_palette: ids::BLENDER_EXPORT_PALETTE,
        palette_dropdown: ids::BLENDER_PALETTE_DROPDOWN,
        palette_tabs: ids::BLENDER_PALETTE_TABS,
        new_palette: ids::BLENDER_NEW_PALETTE,
        rename_palette: ids::BLENDER_RENAME_PALETTE,
        delete_palette: ids::BLENDER_DELETE_PALETTE,
        palette_name: ids::BLENDER_PALETTE_NAME,
        eyedropper: ids::BLENDER_EYEDROPPER,
        drag_handle: ids::BLENDER_DRAG_HANDLE,
        close: ids::BLENDER_CLOSE,
        pin: ids::BLENDER_PIN,
        swatches: [
            ids::BLENDER_SWATCH_0,
            ids::BLENDER_SWATCH_1,
            ids::BLENDER_SWATCH_2,
            ids::BLENDER_SWATCH_3,
            ids::BLENDER_SWATCH_4,
            ids::BLENDER_SWATCH_5,
            ids::BLENDER_SWATCH_6,
            ids::BLENDER_SWATCH_7,
            ids::BLENDER_SWATCH_8,
            ids::BLENDER_SWATCH_9,
            ids::BLENDER_SWATCH_10,
            ids::BLENDER_SWATCH_11,
            ids::BLENDER_SWATCH_12,
            ids::BLENDER_SWATCH_13,
            ids::BLENDER_SWATCH_14,
            ids::BLENDER_SWATCH_15,
            ids::BLENDER_SWATCH_16,
            ids::BLENDER_SWATCH_17,
            ids::BLENDER_SWATCH_18,
            ids::BLENDER_SWATCH_19,
            ids::BLENDER_SWATCH_20,
            ids::BLENDER_SWATCH_21,
            ids::BLENDER_SWATCH_22,
            ids::BLENDER_SWATCH_23,
            ids::BLENDER_SWATCH_24,
            ids::BLENDER_SWATCH_25,
            ids::BLENDER_SWATCH_26,
        ],
        harmony_schemes: ids::BLENDER_HARMONY_SCHEMES,
        harmony_swatches: ids::BLENDER_HARMONY_SWATCHES,
        harmony_add: ids::BLENDER_HARMONY_ADD,
    };
    crate::widget::paint_blender_color_picker_with_store(
        &cp,
        rect,
        &sub_ids,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}

#[cfg(test)]
#[path = "color_picker_demo_tests.rs"]
mod tests;
