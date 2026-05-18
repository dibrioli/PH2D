//! Hero screen 4-zone layout — chrome constants + `HeroLayout` struct.
//!
//! ADR-0029 Phase B.1: moved from `ph2d-editor::screens::hero::style`
//! (chrome consts) and `ph2d-editor::screens::hero` (HeroLayout
//! struct + ctor) to editor-core so `PaintCtx` (also in editor-core)
//! can hold `&HeroLayout` without referring back to `ph2d-editor`.
//!
//! Hero-screen-specific (TopBar + LeftRail + Hierarchy + Inspector +
//! BottomHud). Panel-specific chrome (paint_panel_surface, etc.) lives
//! in `widget::panel_chrome`.

use crate::zones::Rect;
use ph2d_tokens::{
    EDGE_PAD_PX, HERO_VIEWPORT_H_PX, HERO_VIEWPORT_W_PX, HIER_ROW_H_PX, HIERARCHY_W_PX,
    HUD_BOTTOM_PAD_PX, HUD_H_PX, INSPECTOR_W_PX, Spacing, TOPBAR_GAP_PX, TOPBAR_H_PX,
};

/// Default mockup viewport (iPad 12.9 landscape).
pub const HERO_VIEWPORT_W: f32 = HERO_VIEWPORT_W_PX;
pub const HERO_VIEWPORT_H: f32 = HERO_VIEWPORT_H_PX;

/// Padding from the screen edge to chrome (TopBar inset,
/// Hierarchy pinned-right inset, etc).
pub const EDGE_PAD: f32 = EDGE_PAD_PX;
pub const TOPBAR_H: f32 = TOPBAR_H_PX;
pub const TOPBAR_GAP: f32 = TOPBAR_GAP_PX;
/// Mirrors `crate::widget::TOOL_RAIL_WIDTH_PX`.
pub const RAIL_W: f32 = crate::widget::TOOL_RAIL_WIDTH_PX;
pub const INSPECTOR_W: f32 = INSPECTOR_W_PX;
pub const HIERARCHY_W: f32 = HIERARCHY_W_PX;
pub const HUD_H: f32 = HUD_H_PX;
pub const HUD_BOTTOM_PAD: f32 = HUD_BOTTOM_PAD_PX;
pub const HIER_ROW_H: f32 = HIER_ROW_H_PX;

/// Pre-computed sub-region rects for one frame. Built once per
/// frame from a viewport rect — cheap.
#[derive(Copy, Clone, Debug)]
pub struct HeroLayout {
    pub viewport: Rect,
    pub top_bar: Rect,
    pub left_rail: Rect,
    pub inspector: Rect,
    pub hierarchy: Rect,
    pub bottom_hud: Rect,
    /// Visible canvas region (between rail/inspector on the left
    /// and hierarchy on the right, between TopBar and HUD vertically).
    pub canvas: Rect,
}

impl HeroLayout {
    pub fn for_viewport(viewport: Rect) -> Self {
        Self::for_viewport_mirrored(viewport, false)
    }

    pub fn for_viewport_mirrored(viewport: Rect, mirrored: bool) -> Self {
        let top_bar = Rect::new(
            viewport.x + EDGE_PAD,
            viewport.y + EDGE_PAD,
            (viewport.w - EDGE_PAD * 2.0).max(0.0),
            TOPBAR_H,
        );
        let chrome_top = top_bar.y + top_bar.h + TOPBAR_GAP;
        let chrome_bot = viewport.y + viewport.h - HUD_BOTTOM_PAD - HUD_H - Spacing::Md.px();
        let chrome_h = (chrome_bot - chrome_top).max(0.0);
        let left_rail = Rect::new(viewport.x, chrome_top, RAIL_W, chrome_h);
        let (hierarchy_x, inspector_x) = if mirrored {
            (
                viewport.x + viewport.w - EDGE_PAD - HIERARCHY_W,
                viewport.x + RAIL_W + EDGE_PAD,
            )
        } else {
            (
                viewport.x + RAIL_W + EDGE_PAD,
                viewport.x + viewport.w - EDGE_PAD - INSPECTOR_W,
            )
        };
        let inspector = Rect::new(inspector_x, chrome_top, INSPECTOR_W, chrome_h.min(880.0)); // LITERAL-PX-OK: Inspector max height cap
        let hierarchy = Rect::new(hierarchy_x, chrome_top, HIERARCHY_W, chrome_h);
        let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h);
        let bottom_hud = Rect::new(
            viewport.x + (viewport.w - 480.0) * 0.5, // LITERAL-PX-OK: HUD strip width
            viewport.y + viewport.h - HUD_BOTTOM_PAD - HUD_H,
            480.0, // LITERAL-PX-OK: HUD strip width
            HUD_H,
        );
        Self {
            viewport,
            top_bar,
            left_rail,
            inspector,
            hierarchy,
            bottom_hud,
            canvas,
        }
    }
}
