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
    fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, DropdownState, paint_dropdown_popover_in_viewport,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BlendMode, MAX_BLEND_MODES};

const BLEND_POPOVER_W: f32 = 132.0; // LITERAL-PX-OK: open blend list width (extends left so long mode names fit one line)
/// Width below which paint_text never wraps — the custom blend chip lays the
/// mode name on a single line and the chip clip truncates any overflow.
const CHIP_TEXT_NOWRAP_W: f32 = 4096.0; // LITERAL-PX-OK: layout width, not a design value

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

    // Mode name — smaller font, single line, clipped to the text column.
    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    let text_clip = Rect::new(text_x, rect.y, text_w, rect.h);
    ctx.scene.push_clip(&rect_to_vello(text_clip));
    paint_text(
        ctx.text_system,
        ctx.scene,
        BlendMode::from_u8(cur_mode).name(),
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        CHIP_TEXT_NOWRAP_W,
        resolve(ColorToken::Text1, theme),
    );
    ctx.scene.pop_layer();

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
