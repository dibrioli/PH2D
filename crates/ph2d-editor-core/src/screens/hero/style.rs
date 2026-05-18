//! Hero-screen-specific layout constants + small style helpers.
//!
//! Wave 8 Phase 2.A — panel-chrome paint helpers (paint_panel_surface,
//! paint_panel_corner_dot, panel_drag/resize_handle_rect,
//! clamp_panel_rect, PANEL_RADIUS, PANEL_HEAD_PAD, PANEL_RESIZE_HANDLE_SIZE)
//! moved to `ph2d-editor-core::widget::panel_chrome` so panel crates
//! (`ph2d-panel-*`) consume them without depending on `ph2d-editor`.
//! Re-exported from here for backwards compatibility — existing
//! call sites inside `ph2d-editor` keep working unchanged.
//!
//! Hero-layout constants (EDGE_PAD, TOPBAR_H, RAIL_W, HIERARCHY_W,
//! INSPECTOR_W, HUD_H, HUD_BOTTOM_PAD, HIER_ROW_H, TOPBAR_GAP,
//! HERO_VIEWPORT_*) stay here — they describe the hero orchestrator's
//! 4-zone layout, not the per-panel chrome.

use crate::widget::ButtonState;
use ph2d_tokens::{
    ColorToken, EDGE_PAD_PX, HERO_VIEWPORT_H_PX, HERO_VIEWPORT_W_PX, HIER_ROW_H_PX, HIERARCHY_W_PX,
    HUD_BOTTOM_PAD_PX, HUD_H_PX, INSPECTOR_W_PX, TOPBAR_GAP_PX, TOPBAR_H_PX,
};

// Wave 8 Phase 2.A re-exports — panel-chrome surface helpers + constants
// live in editor-core. Existing consumers (`use super::style::*`)
// keep working.
pub use crate::widget::panel_chrome::{
    HIGHLIGHTER_RGBA, PANEL_HEAD_PAD, PANEL_RADIUS, PANEL_RESIZE_HANDLE_SIZE, clamp_panel_rect,
    paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect, panel_resize_handle_rect,
};

/// Default mockup viewport (iPad 12.9 landscape). Public so callers
/// like `shells/desktop` and tests can size their windows to match.
/// Per tokens.json `chrome.hero-viewport-w`.
pub const HERO_VIEWPORT_W: f32 = HERO_VIEWPORT_W_PX;
/// Per tokens.json `chrome.hero-viewport-h`.
pub const HERO_VIEWPORT_H: f32 = HERO_VIEWPORT_H_PX;

/// Padding from the screen edge to chrome (TopBar inset, Hierarchy
/// pinned-right inset, etc).
pub const EDGE_PAD: f32 = EDGE_PAD_PX;
pub const TOPBAR_H: f32 = TOPBAR_H_PX;
pub const TOPBAR_GAP: f32 = TOPBAR_GAP_PX;
/// Mirrors `crate::widget::TOOL_RAIL_WIDTH_PX`. The hero layout uses
/// this for the rail's outer rect; the widget reuses it as a sizing
/// hint. Keep them in lockstep.
pub const RAIL_W: f32 = crate::widget::TOOL_RAIL_WIDTH_PX;
pub const INSPECTOR_W: f32 = INSPECTOR_W_PX;
pub const HIERARCHY_W: f32 = HIERARCHY_W_PX;
pub const HUD_H: f32 = HUD_H_PX;
pub const HUD_BOTTOM_PAD: f32 = HUD_BOTTOM_PAD_PX;

/// Hierarchy row height — used by the hero orchestrator + the
/// hierarchy panel chrome layout. Stays here because it's specific
/// to the live Hierarchy panel, not a per-panel chrome primitive.
pub const HIER_ROW_H: f32 = HIER_ROW_H_PX;

/// Pick a chrome icon's foreground tint based on its interactive
/// state. Used by TopBar single-icon clusters and the LeftRail tools.
pub(super) fn icon_button_fg(state: ButtonState) -> ColorToken {
    match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Disabled => ColorToken::TextDisabled,
        ButtonState::Normal | ButtonState::Loading => ColorToken::Text2,
    }
}
