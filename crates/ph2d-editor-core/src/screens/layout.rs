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

/// Split of the center region into the scene viewport and the Motion Nodes
/// graph, applied while the Motion tool is active (Motion Nodes M0.T4).
///
/// `t` is the fraction of the center band that the **scene** occupies — the top
/// slice in [`Self::Horizontal`], the left slice in [`Self::Vertical`] — clamped
/// to [`Self::T_MIN`]..=[`Self::T_MAX`]. `None` = no split (the scene fills the
/// whole center; the graph is hidden), the default for every non-Motion tool.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CenterSplit {
    /// No split — scene fills the center, graph hidden.
    None,
    /// Horizontal divider: scene on top, graph on the bottom (Cavalry layout).
    Horizontal { t: f32 },
    /// Vertical divider: scene on the left, graph on the right (TouchDesigner).
    Vertical { t: f32 },
}

impl CenterSplit {
    /// Divider clamp — the scene (and graph) always keep at least a quarter.
    pub const T_MIN: f32 = 0.25; // LITERAL-PX-OK: split ratio (fraction of center), not a design token.
    pub const T_MAX: f32 = 0.75; // LITERAL-PX-OK: split ratio (fraction of center), not a design token.
    /// Default split fraction — the scene gets 55 % of the center (plan §2.1).
    pub const T_DEFAULT: f32 = 0.55; // LITERAL-PX-OK: split ratio (fraction of center), not a design token.

    /// Clamp a raw fraction into the legal divider range. NaN-aware
    /// (`safe_clamp`): a divider drag that produced a NaN `t` collapses to the
    /// lower bound instead of poisoning the layout.
    pub fn clamp_t(t: f32) -> f32 {
        crate::math::safe_clamp(t, Self::T_MIN, Self::T_MAX)
    }

    /// `true` for a horizontal or vertical split (the graph is visible).
    pub fn is_split(self) -> bool {
        !matches!(self, Self::None)
    }

    /// `true` for a vertical split (side-by-side, divider drawn vertically) —
    /// the shell reads this to pick the divider's resize cursor (`EwResize` for
    /// a vertical divider, `NsResize` for a horizontal one).
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    /// The current divider fraction (or [`Self::T_DEFAULT`] when not split).
    pub fn t(self) -> f32 {
        match self {
            Self::Horizontal { t } | Self::Vertical { t } => t,
            Self::None => Self::T_DEFAULT,
        }
    }

    /// Same orientation, new (clamped) fraction. `None` stays `None`.
    pub fn with_t(self, t: f32) -> Self {
        let t = Self::clamp_t(t);
        match self {
            Self::Horizontal { .. } => Self::Horizontal { t },
            Self::Vertical { .. } => Self::Vertical { t },
            Self::None => Self::None,
        }
    }

    /// Switch to a horizontal split (scene on top), preserving the fraction.
    pub fn to_horizontal(self) -> Self {
        Self::Horizontal {
            t: Self::clamp_t(self.t()),
        }
    }

    /// Switch to a vertical split (scene on the left), preserving the fraction.
    pub fn to_vertical(self) -> Self {
        Self::Vertical {
            t: Self::clamp_t(self.t()),
        }
    }
}

/// Pre-computed sub-region rects for one frame. Built once per
/// frame from a viewport rect — cheap.
#[derive(Copy, Clone, Debug)]
pub struct HeroLayout {
    pub viewport: Rect,
    pub top_bar: Rect,
    pub left_rail: Rect,
    pub inspector: Rect,
    /// Background-Removal panel slot — shares the Inspector's right-dock
    /// geometry. Only painted while the `bgremoval` tool is active (the
    /// panel's own visibility gate keys off `panel_visible("bgremoval")`,
    /// which the shell drives from the active-tool id).
    pub bgremoval: Rect,
    /// Padding panel slot — shares the Inspector's right-dock geometry,
    /// like [`Self::bgremoval`]. Only painted while the `padding` tool is
    /// active (the panel's own visibility gate keys off
    /// `panel_visible("padding")`, which the shell drives from the
    /// active-tool id).
    pub padding: Rect,
    /// Painter sidebar slot — shares the Inspector's right-dock geometry
    /// (W2.T2.1 plan §5). Only painted while the `painter` tool is active
    /// (the panel's own visibility gate keys off
    /// `panel_visible("painter_sidebar")`, which the shell drives from
    /// the active-tool id).
    pub painter_sidebar: Rect,
    /// Painter layers panel slot — shares the Inspector's right-dock
    /// geometry too (W3.T3.4 plan §6, mirror do [`Self::painter_sidebar`]).
    /// Only painted while the `painter` tool is active (the panel's own
    /// visibility gate keys off `panel_visible("painter_layers")`, which
    /// the shell drives from the active-tool id).
    pub painter_layers: Rect,
    pub hierarchy: Rect,
    pub bottom_hud: Rect,
    /// Visible canvas region (between rail/inspector on the left
    /// and hierarchy on the right, between TopBar and HUD vertically).
    pub canvas: Rect,
    /// Scene viewport sub-rect (Motion Nodes M0.T4). Equals the full
    /// [`Self::canvas`] with no split; the top slice (`Horizontal`) or left
    /// slice (`Vertical`) of the center band when the Motion tool splits it.
    /// The scene is rendered into this rect via scissor + a sub-rect camera
    /// (M0.T13) when it differs from `canvas`.
    pub center_viewport: Rect,
    /// Motion Nodes graph sub-rect — the complement of [`Self::center_viewport`]
    /// in the center band. Zero-sized when the split is `None`.
    pub motion_graph: Rect,
    /// Zero-height reservation at the bottom edge of [`Self::motion_graph`] for
    /// the future timeline dock (plan §2.1; the timeline itself is deferred to
    /// its own module — this only holds the seam).
    pub motion_timeline_slot: Rect,
}

impl HeroLayout {
    pub fn for_viewport(viewport: Rect) -> Self {
        Self::for_viewport_mirrored(viewport, false)
    }

    pub fn for_viewport_mirrored(viewport: Rect, mirrored: bool) -> Self {
        Self::for_viewport_mirrored_with_rail_w(viewport, mirrored, RAIL_W)
    }

    /// Layout constructor with an explicit rail-column width. Used by
    /// the hero orchestrator when the user picks a non-default
    /// [`crate::widget::RailButtonSize`] preset in the Themes menu
    /// (2026-05-24) — the rail shrinks/grows and Inspector/Hierarchy
    /// x-positions follow.
    pub fn for_viewport_mirrored_with_rail_w(viewport: Rect, mirrored: bool, rail_w: f32) -> Self {
        Self::for_viewport_split(viewport, mirrored, rail_w, CenterSplit::None)
    }

    /// Layout constructor that also splits the center region for the Motion
    /// tool (M0.T4). With [`CenterSplit::None`] this is identical to
    /// [`Self::for_viewport_mirrored_with_rail_w`] — `center_viewport == canvas`,
    /// `motion_graph`/`motion_timeline_slot` zero-sized. Chrome (rail, panels,
    /// HUD) floats over the center exactly as before; only the scene/graph split
    /// is new.
    pub fn for_viewport_split(
        viewport: Rect,
        mirrored: bool,
        rail_w: f32,
        split: CenterSplit,
    ) -> Self {
        let top_bar = Rect::new(
            viewport.x + EDGE_PAD,
            viewport.y + EDGE_PAD,
            (viewport.w - EDGE_PAD * 2.0).max(0.0),
            TOPBAR_H,
        );
        let chrome_top = top_bar.y + top_bar.h + TOPBAR_GAP;
        let chrome_bot = viewport.y + viewport.h - HUD_BOTTOM_PAD - HUD_H - Spacing::Md.px();
        let chrome_h = (chrome_bot - chrome_top).max(0.0);
        let left_rail = Rect::new(viewport.x, chrome_top, rail_w, chrome_h);
        let (hierarchy_x, inspector_x) = if mirrored {
            (
                viewport.x + viewport.w - EDGE_PAD - HIERARCHY_W,
                viewport.x + rail_w + EDGE_PAD,
            )
        } else {
            (
                viewport.x + rail_w + EDGE_PAD,
                viewport.x + viewport.w - EDGE_PAD - INSPECTOR_W,
            )
        };
        let inspector = Rect::new(inspector_x, chrome_top, INSPECTOR_W, chrome_h.min(880.0)); // LITERAL-PX-OK: Inspector max height cap
        // Bg Removal panel shares the Inspector's right-dock x/width;
        // it replaces the Inspector visually while the tool is active.
        let bgremoval = inspector;
        // Padding panel shares the Inspector's right-dock geometry too.
        let padding = inspector;
        // Painter sidebar shares the Inspector's right-dock geometry —
        // takeover mode replaces Inspector visually (W2.T2.1 plan §5).
        let painter_sidebar = inspector;
        // Painter layers panel shares the Inspector's right-dock geometry
        // too (W3.T3.4 plan §6, mirror do painter_sidebar).
        let painter_layers = inspector;
        let hierarchy = Rect::new(hierarchy_x, chrome_top, HIERARCHY_W, chrome_h);
        let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h);
        // Center split (M0.T4): partition the chrome band (chrome_top..chrome_bot,
        // full width — panels float over it) into the scene sub-rect and the graph
        // sub-rect. `None` keeps the legacy full-bleed scene (== canvas).
        let (center_viewport, motion_graph) = match split {
            CenterSplit::None => (canvas, Rect::new(viewport.x, chrome_top, 0.0, 0.0)),
            CenterSplit::Horizontal { t } => {
                let top_h = chrome_h * CenterSplit::clamp_t(t);
                (
                    Rect::new(viewport.x, chrome_top, viewport.w, top_h),
                    Rect::new(viewport.x, chrome_top + top_h, viewport.w, chrome_h - top_h),
                )
            }
            CenterSplit::Vertical { t } => {
                let left_w = viewport.w * CenterSplit::clamp_t(t);
                (
                    Rect::new(viewport.x, chrome_top, left_w, chrome_h),
                    Rect::new(
                        viewport.x + left_w,
                        chrome_top,
                        viewport.w - left_w,
                        chrome_h,
                    ),
                )
            }
        };
        // Timeline dock seam: zero-height strip at the bottom of the graph.
        let motion_timeline_slot = Rect::new(
            motion_graph.x,
            motion_graph.y + motion_graph.h,
            motion_graph.w,
            0.0,
        );
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
            bgremoval,
            padding,
            painter_sidebar,
            painter_layers,
            hierarchy,
            bottom_hud,
            canvas,
            center_viewport,
            motion_graph,
            motion_timeline_slot,
        }
    }
}

#[cfg(test)]
mod split_tests {
    use super::*;

    fn vp() -> Rect {
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.5
    }

    #[test]
    fn no_split_scene_is_full_canvas_and_graph_is_empty() {
        let l = HeroLayout::for_viewport(vp());
        assert_eq!(l.center_viewport, l.canvas);
        assert_eq!(l.motion_graph.w, 0.0);
        assert_eq!(l.motion_graph.h, 0.0);
    }

    #[test]
    fn horizontal_split_scene_on_top_graph_below_partition_the_band() {
        let l = HeroLayout::for_viewport_split(
            vp(),
            false,
            RAIL_W,
            CenterSplit::Horizontal { t: 0.55 },
        );
        // Scene sits above the graph; both share the band's x/width.
        assert!(l.center_viewport.y < l.motion_graph.y);
        assert!(approx(l.center_viewport.x, l.motion_graph.x));
        assert!(approx(l.center_viewport.w, l.motion_graph.w));
        // They tile the band with no gap/overlap at the divider.
        assert!(approx(
            l.center_viewport.y + l.center_viewport.h,
            l.motion_graph.y
        ));
        // Scene gets ~55 % of the band height.
        let band = l.center_viewport.h + l.motion_graph.h;
        assert!(approx(l.center_viewport.h / band, 0.55));
    }

    #[test]
    fn vertical_split_scene_on_left_graph_on_right() {
        let l =
            HeroLayout::for_viewport_split(vp(), false, RAIL_W, CenterSplit::Vertical { t: 0.5 });
        assert!(l.center_viewport.x < l.motion_graph.x);
        assert!(approx(l.center_viewport.y, l.motion_graph.y));
        assert!(approx(l.center_viewport.h, l.motion_graph.h));
        assert!(approx(
            l.center_viewport.x + l.center_viewport.w,
            l.motion_graph.x
        ));
    }

    #[test]
    fn timeline_slot_is_zero_height_at_graph_bottom() {
        let l =
            HeroLayout::for_viewport_split(vp(), false, RAIL_W, CenterSplit::Horizontal { t: 0.6 });
        assert_eq!(l.motion_timeline_slot.h, 0.0);
        assert!(approx(
            l.motion_timeline_slot.y,
            l.motion_graph.y + l.motion_graph.h
        ));
    }

    #[test]
    fn divider_fraction_clamps() {
        assert_eq!(CenterSplit::clamp_t(0.9), CenterSplit::T_MAX);
        assert_eq!(CenterSplit::clamp_t(0.1), CenterSplit::T_MIN);
        // Orientation switch preserves the fraction.
        let h = CenterSplit::Horizontal { t: 0.4 };
        assert_eq!(h.to_vertical(), CenterSplit::Vertical { t: 0.4 });
        assert!((h.to_vertical().t() - 0.4).abs() < 1e-6);
    }
}
