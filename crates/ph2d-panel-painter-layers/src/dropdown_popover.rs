//! The panel's shared **scrollable** dropdown-popover renderer. Split from `paint_brush.rs` for the
//! panel-crate file-LOC cap. Every brush/texture/blend/adjustment menu in this panel defers to it, so
//! long option lists (texture Kind = 26, Blend = 24, Adjustment = 24…) clamp to the viewport and then
//! SCROLL — wheel over the list, or drag the bar (Enio: "menus com scroll devem ser o padrão").

use ph2d_editor_core::panel::{PaintCtx, paint_dropdown_list};
use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;

/// Deferred paint of an open dropdown popover that **scrolls** when the option list overflows the
/// viewport-clamped popover — by the foundation door. Generic over the option value type so the
/// u8-valued chips (Blend / Falloff / Texture / Ramp) and the usize-valued Adjustment menu share it.
pub(crate) fn paint_dropdown_popover<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    options: Vec<DropdownOption<T>>,
    chip_rect: Rect,
    cur: T,
) {
    let dd = Dropdown::new(id, "", options).selected(cur).open(true);
    let viewport = ctx.viewport;
    paint_dropdown_list(ctx, &dd, chip_rect, viewport, theme, None);
}

#[cfg(test)]
#[path = "dropdown_popover_tests.rs"]
mod tests;
