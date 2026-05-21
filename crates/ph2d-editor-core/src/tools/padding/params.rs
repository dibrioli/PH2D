//! Parameters + UI projection for the stateful Padding tool.
//!
//! The Padding tool condenses the legacy *Image Padding* + *Directional
//! Expand* tools into one: four SIGNED per-edge values (top / right /
//! bottom / left), where a positive value expands that edge with
//! transparent pixels and a negative value crops it. The pure resize
//! lives in `ph2d-tool-padding::add_padding`; this module is just the
//! editor-side state shape — editor-core deliberately does NOT depend on
//! `ph2d-tool-padding` (the spec is four `i32`s; the shell converts them
//! to `ph2d_tool_padding::PaddingSpec` at bake time).

/// Normalized projection of the tool's per-edge state for the typed
/// `ph2d-panel-padding` to paint. All four fields are signed pixel
/// counts (positive = expand, negative = crop). The host publishes a
/// fresh snapshot each frame via
/// `ph2d_panel_padding::set_current_padding_snapshot`.
///
/// Unlike the Bg-Removal snapshot these are NOT normalized to `0..1` —
/// the panel paints four `NumberInput` fields whose displayed value IS
/// the pixel count, so the snapshot carries the raw `i32`s.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PaddingUiSnapshot {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

/// One panel-originated edit, routed editor-core → shell over
/// `EditorAction::PaddingUiEdit`. The shell drains it and calls
/// [`PaddingTool::apply_ui_edit`](super::tool::PaddingTool::apply_ui_edit)
/// against the active tool instance. Inverse of
/// [`PaddingTool::ui_snapshot`](super::tool::PaddingTool::ui_snapshot).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PaddingUiEdit {
    /// Top edge field edited (signed px).
    Top(i32),
    /// Right edge field edited (signed px).
    Right(i32),
    /// Bottom edge field edited (signed px).
    Bottom(i32),
    /// Left edge field edited (signed px).
    Left(i32),
    /// Apply pressed — bake the resized canvas at full resolution.
    Apply,
}
