//! Padding panel ⟷ tool bridge.
//!
//! Run once per frame BEFORE `paint_hero_screen` (sibling of
//! `bgremoval_preview.rs`, but far simpler — Padding has no live
//! preview / on-canvas overlay in v1). Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `padding` is the active
//!    tool, keyed "padding" to match `PaddingPanel::ID`).
//! 2. Drains the panel's `PaddingUiEdit`s into the active `PaddingTool`.
//! 3. Publishes the per-frame snapshot the panel paints next frame.
//! 4. Returns the selection to bake on Apply (the tool's
//!    `take_pending_apply` fired) so the caller can drain the actual
//!    canvas resize.

use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;

/// Returns `Some((entity_bits, spec, recenter_pivot))` iff Apply fired
/// this frame — the caller runs the full-resolution bake against that
/// selection with the captured per-edge spec + pivot mode and then tears
/// the tool down (deactivate + restore Inspector), exactly like the
/// Bg-Removal apply teardown. The spec + pivot flag are captured here
/// (while the tool is borrowed) so the bake site doesn't have to
/// re-borrow `tools`.
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    padding_ui_edits: Vec<ph2d_editor::tools::padding::PaddingUiEdit>,
) -> Option<(u64, ph2d_tool_padding::PaddingSpec, bool)> {
    let padding_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("padding"))
        .unwrap_or(false);
    // Visibility: shown iff padding is the active tool.
    hero.panel_visibility.insert("padding", padding_is_active);

    let mut apply: Option<(u64, ph2d_tool_padding::PaddingSpec, bool)> = None;
    if let Some(tool) = tools.active_mut()
        && let Some(pad) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_editor::tools::padding::PaddingTool>()
    {
        for edit in padding_ui_edits {
            pad.apply_ui_edit(edit);
        }
        if pad.take_pending_apply()
            && let Some(bits) = hero.gizmo.selection
        {
            let (top, right, bottom, left) = pad.spec();
            apply = Some((
                bits,
                ph2d_tool_padding::PaddingSpec {
                    top,
                    right,
                    bottom,
                    left,
                },
                pad.recenter_pivot(),
            ));
        }
        #[cfg(feature = "panel-padding")]
        ph2d_panel_padding::set_current_padding_snapshot(if padding_is_active {
            Some(pad.ui_snapshot())
        } else {
            None
        });
    }
    apply
}
