//! The "+ Adjustment" kind-picker popover (W4 T4.15): a deferred dropdown that
//! lists every [`AdjustmentKind`] so the user creates any of the 24 kinds, not
//! just HSB. Split out of `paint.rs` to keep that file under the panel-crate
//! file-LOC cap. Mirrors [`crate::blend`]: the `+ Adj` toolbar button is
//! registered as a `Dropdown` (generic open/close dispatch), and this paints the
//! open list on top of everything in `paint`'s deferred pass.
//!
//! Picking an option forwards `SelectOption(PAINTER_LAYERS_ADD_ADJUSTMENT, idx)`
//! (idx = position in [`AdjustmentKind::ALL`]); the tool maps it back to the kind
//! and creates the layer. See `event.rs` / `tool.rs`.

use ph2d_editor_core::ids::painter_adjustment_kind_option_id;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::AdjustmentKind;

/// All 24 adjustment kinds as `Dropdown` options (value = index into
/// [`AdjustmentKind::ALL`] = the wire value forwarded to the tool, label =
/// English `display_name`).
fn kind_options() -> Vec<DropdownOption<usize>> {
    AdjustmentKind::ALL
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            DropdownOption::new(
                painter_adjustment_kind_option_id(i as u8),
                i,
                kind.display_name(),
            )
        })
        .collect()
}

/// Deferred paint of the open "+ Adjustment" kind menu (on top of the rows,
/// clamped to the viewport so the 24-row list stays on-screen — it flips above
/// the chip if it would overflow below). `menu_chip` is the synthetic full-width
/// chip the toolbar stashed (panel content width, at the action-toolbar row), so
/// the list drops straight down the panel and every kind name fits one line.
/// Registers each option as a Button + its hit rect so option clicks dispatch.
pub(crate) fn paint_adjustment_menu_popover(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    menu_chip: Rect,
) {
    // Delegate to the shared SCROLLABLE popover (24 adjustment kinds overflow the dock). The menu has
    // no current selection, so pass an out-of-range value — no row is highlighted.
    crate::paint_brush::paint_dropdown_popover(
        ctx,
        theme,
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        kind_options(),
        menu_chip,
        usize::MAX,
    );
}
