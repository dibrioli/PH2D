//! Per-row blend-mode control for the layers panel: a compact custom chip
//! (registered as a `Dropdown` for the generic open/close dispatch) plus the
//! deferred popover that lists all 22 modes. Split out of `paint.rs` to keep
//! that file under the panel-crate file-LOC cap.

use crate::paint::register_button;
use crate::state;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::{
    PainterLayerWidget, painter_layer_blend_option_id, painter_layer_widget_id,
};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, DropdownState, paint_dropdown_popover_in_viewport,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BlendMode, MAX_BLEND_MODES};
use std::cell::RefCell;

const BLEND_POPOVER_W: f32 = 132.0; // LITERAL-PX-OK: open blend list width (extends left so long mode names fit one line)

thread_local! {
    /// Cache of the 22 blend-mode chip labels pre-fitted (ellipsized) to the
    /// current chip text-column width. Recomputed only when the width changes
    /// (panel resize) — so the per-frame cost is an index + a short clone, NOT
    /// per-chip text measurement. Replaces the old per-chip Vello `push_clip`
    /// (10 clip layers/frame at 10 layers — the layers-panel FPS sink).
    static FITTED_LABELS: RefCell<(f32, Vec<String>)> = const { RefCell::new((-1.0, Vec::new())) };
}

/// The blend-mode name for `mode`, truncated with an ellipsis to fit `max_w` at
/// `font` (single line, no Vello clip). Memoised per `max_w`.
fn fitted_label(ts: &mut TextSystem, mode: u8, font: f32, max_w: f32) -> String {
    FITTED_LABELS.with(|c| {
        let mut cache = c.borrow_mut();
        if (cache.0 - max_w).abs() > 0.5 || cache.1.len() != MAX_BLEND_MODES as usize {
            cache.1 = (0..MAX_BLEND_MODES)
                .map(|m| fit_one(ts, BlendMode::from_u8(m).name(), font, max_w))
                .collect();
            cache.0 = max_w;
        }
        cache.1[mode as usize].clone()
    })
}

/// Longest prefix of `name` (+ ellipsis when truncated) that fits `max_w`.
fn fit_one(ts: &mut TextSystem, name: &str, font: f32, max_w: f32) -> String {
    if ts.prefix_width(name, font) <= max_w {
        return name.to_string();
    }
    let budget = (max_w - ts.prefix_width("…", font)).max(0.0);
    let mut cut = 0;
    for (i, _) in name.char_indices().skip(1) {
        if ts.prefix_width(&name[..i], font) > budget {
            break;
        }
        cut = i;
    }
    format!("{}…", &name[..cut])
}

/// All 22 blend modes as `Dropdown` options for the layer with runtime id
/// `layer_u64` (value = wire discriminant, label = display name).
fn blend_options(layer_u64: u64) -> Vec<DropdownOption<u8>> {
    (0..MAX_BLEND_MODES)
        .map(|m| {
            DropdownOption::new(
                painter_layer_blend_option_id(layer_u64, m),
                m,
                BlendMode::from_u8(m).name(),
            )
        })
        .collect()
}

/// Paint a compact blend-mode chip (registered as a `Dropdown` for the generic
/// open/close dispatch) and, if open (single-open enforced), stash it for the
/// deferred popover pass. Custom-painted — smaller font + a hard single-line
/// clip so long names ("Color Burn", "Linear Light") truncate instead of
/// wrapping to two lines, which the canon `paint_dropdown_chip` (Base font +
/// wide padding) does in this narrow column.
pub(crate) fn paint_blend_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_u64: u64,
    cur_mode: u8,
    rect: Rect,
) {
    let id = painter_layer_widget_id(layer_u64, PainterLayerWidget::Blend);
    let store = ctx.host.store_mut();
    store.register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_mode as usize),
        },
    );
    let store_open = matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    // One popover at a time: only the first open dropdown (top→bottom) wins.
    let open = store_open && state::pending_blend_dd().is_none();
    if store_open
        && !open
        && let Some(InteractiveState::Dropdown { open: o, .. }) = store.get_mut(id)
    {
        *o = false;
    }

    let radius = Radius::Sm.px();
    fill_rounded_rect(ctx.scene, rect, radius, resolve(ColorToken::Bg1, theme));
    let border = if open {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        resolve(border, theme),
    );

    // Chevron (right), sized ~half the chip height.
    let chevron = Spacing::Md.px();
    let pad = Spacing::Sm.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
    );
    let icon = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        ctx.scene,
        icon,
        chevron_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    // Mode name — smaller font, single line, pre-truncated to the text column
    // (memoised ellipsis). No Vello clip: a per-chip `push_clip` is a per-frame
    // clip layer, and 10 of them (10 layers) was the panel's FPS sink.
    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    let label = fitted_label(ctx.text_system, cur_mode, font, text_w);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &label,
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );

    ctx.host.hit_index_mut().register(id, rect);
    if open {
        state::set_pending_blend_dd(Some((layer_u64, rect, cur_mode)));
    }
}

/// Deferred paint of the single open blend dropdown popover (on top of the
/// rows, clamped to the viewport so it stays on-screen). Registers each option
/// as a Button + its hit rect so option clicks dispatch.
pub(crate) fn paint_blend_popover(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_u64: u64,
    chip_rect: Rect,
    cur_mode: u8,
) {
    let dd = Dropdown::new(
        painter_layer_widget_id(layer_u64, PainterLayerWidget::Blend),
        "",
        blend_options(layer_u64),
    )
    .selected(cur_mode)
    .open(true);
    let viewport = ctx.viewport;
    // Wider than the chip so long mode names fit one line; right-aligned to the
    // chip right edge so it extends LEFT into the panel (stays on-screen).
    let pop_w = BLEND_POPOVER_W.max(chip_rect.w);
    let pop_chip = Rect::new(
        chip_rect.x + chip_rect.w - pop_w,
        chip_rect.y,
        pop_w,
        chip_rect.h,
    );
    let panel = dd.popover_rect_clamped(pop_chip, viewport);
    paint_dropdown_popover_in_viewport(
        &dd,
        pop_chip,
        Some(viewport),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    // Register option buttons (mutable store) then their hit rects (mutable
    // hit_index) in separate borrows — `store_and_hit_index_mut` hands back an
    // immutable store, which cannot `register_if_absent`.
    {
        let store = ctx.host.store_mut();
        for opt in dd.options.iter() {
            register_button(store, opt.id);
        }
    }
    let hit_index = ctx.host.hit_index_mut();
    for (i, opt) in dd.options.iter().enumerate() {
        hit_index.register(opt.id, dd.option_rect_in(pop_chip, panel, i));
    }
}
