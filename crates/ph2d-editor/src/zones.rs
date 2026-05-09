//! 4-zone layout engine (ADR-0023 §3).
//!
//! Computes pixel rects for each [`Zone`] given the window size +
//! sidebar position (which is user-configurable: vertical offset
//! plus left/right mirroring). Center always 100 % of the leftover.
//!
//! Z-order (back to front):
//!   1. Center (canvas)
//!   2. Top-left + Top-right + Sidebar (translucent, drawn last)
//!   3. FloatingPanels (above zones, below toasts)
//!   4. Toasts (always on top)

// ph2d_tokens::Spacing referenced only in tests (token math
// invariant) — declared inline there.

/// Pixel rect (left, top, width, height). f32 for direct flow into
/// vello/wgpu math.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn area(&self) -> f32 {
        self.w * self.h
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Zone {
    /// Top-left (Editing — Gallery, Actions, Scene tree, Adjustments,
    /// Selections, Transform).
    TopLeft,
    /// Top-right (Creating — Tile painter, Sprite placer, Asset
    /// insertion, Brush, Layer, Color).
    TopRight,
    /// Sidebar (modulators — brush size/opacity, snap, undo/redo,
    /// eyedropper, Modify button).
    Sidebar,
    /// Center (canvas — 100 % of leftover).
    Center,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SidebarSide {
    Right,
    Left,
}

impl SidebarSide {
    pub fn mirror(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
}

/// Layout configuration. Mutable: user can drag sidebar / mirror /
/// pin top-zone heights.
#[derive(Copy, Clone, Debug)]
pub struct Layout {
    pub window_w: f32,
    pub window_h: f32,
    /// Pixel height of the top-left + top-right toolbar strip.
    pub toolbar_h: f32,
    /// Sidebar width.
    pub sidebar_w: f32,
    /// Sidebar vertical center as fraction of window height
    /// (0.0 = top, 1.0 = bottom). Drag handle on Modify button moves
    /// this; persisted per-user.
    pub sidebar_v_center: f32,
    /// Sidebar visible height (in px). The rest of the column is
    /// transparent so canvas peeks through.
    pub sidebar_h: f32,
    pub sidebar_side: SidebarSide,
    /// When true, only the canvas is rendered; all zones suppressed.
    pub zen: bool,
}

impl Layout {
    pub fn new(window_w: f32, window_h: f32) -> Self {
        Self {
            window_w,
            window_h,
            toolbar_h: 56.0, // 32 px icon + Spacing::Sm padding (12 px) × 2
            sidebar_w: 64.0, // single column of 32 px icons + Spacing::Md padding × 2
            sidebar_v_center: 0.5,
            sidebar_h: 320.0,
            sidebar_side: SidebarSide::Right,
            zen: false,
        }
    }

    /// Compute the pixel rect for a zone.
    pub fn rect(&self, zone: Zone) -> Rect {
        if self.zen {
            // In Zen mode, only Center matters — others collapse to
            // zero-size rects so callers naturally skip drawing.
            return match zone {
                Zone::Center => Rect::new(0.0, 0.0, self.window_w, self.window_h),
                _ => Rect::new(0.0, 0.0, 0.0, 0.0),
            };
        }

        // Vertical split: toolbar strip on top + canvas region below.
        let canvas_y = self.toolbar_h;
        let canvas_h = self.window_h - self.toolbar_h;

        match zone {
            Zone::TopLeft => {
                // Half of toolbar width on the left.
                let half = self.window_w / 2.0;
                Rect::new(0.0, 0.0, half, self.toolbar_h)
            }
            Zone::TopRight => {
                let half = self.window_w / 2.0;
                Rect::new(half, 0.0, half, self.toolbar_h)
            }
            Zone::Sidebar => {
                let center_y = canvas_y + canvas_h * self.sidebar_v_center;
                let top = (center_y - self.sidebar_h / 2.0).max(canvas_y);
                let x = match self.sidebar_side {
                    SidebarSide::Right => self.window_w - self.sidebar_w,
                    SidebarSide::Left => 0.0,
                };
                Rect::new(x, top, self.sidebar_w, self.sidebar_h)
            }
            Zone::Center => {
                // Canvas is the FULL area below toolbar; sidebar
                // overlays it (translucent), doesn't shrink it.
                Rect::new(0.0, canvas_y, self.window_w, canvas_h)
            }
        }
    }

    pub fn toggle_zen(&mut self) {
        self.zen = !self.zen;
    }

    pub fn mirror_sidebar(&mut self) {
        self.sidebar_side = self.sidebar_side.mirror();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_zones_dont_overlap_canvas_horizontally_in_normal_mode() {
        let layout = Layout::new(1024.0, 768.0);
        let top_left = layout.rect(Zone::TopLeft);
        let top_right = layout.rect(Zone::TopRight);
        let center = layout.rect(Zone::Center);

        // Top zones cover full toolbar height; center starts below it.
        assert_eq!(top_left.y + top_left.h, center.y);
        assert_eq!(top_right.y + top_right.h, center.y);
        // Top-left + top-right cover full window width.
        assert_eq!(top_left.x + top_left.w, top_right.x);
        assert_eq!(top_left.x + top_left.w + top_right.w, layout.window_w);
    }

    #[test]
    fn sidebar_anchored_right_by_default() {
        let layout = Layout::new(1024.0, 768.0);
        let sidebar = layout.rect(Zone::Sidebar);
        assert_eq!(sidebar.x + sidebar.w, layout.window_w);
    }

    #[test]
    fn sidebar_mirror_swaps_to_left_edge() {
        let mut layout = Layout::new(1024.0, 768.0);
        layout.mirror_sidebar();
        let sidebar = layout.rect(Zone::Sidebar);
        assert_eq!(sidebar.x, 0.0);
    }

    #[test]
    fn zen_collapses_zones_to_canvas() {
        let mut layout = Layout::new(1024.0, 768.0);
        layout.toggle_zen();
        let canvas = layout.rect(Zone::Center);
        assert_eq!(canvas, Rect::new(0.0, 0.0, 1024.0, 768.0));
        // Other zones become zero-size.
        for zone in [Zone::TopLeft, Zone::TopRight, Zone::Sidebar] {
            let r = layout.rect(zone);
            assert_eq!(r.area(), 0.0, "{zone:?} must be zero-area in zen");
        }
    }

    #[test]
    fn zen_toggle_round_trips() {
        let mut layout = Layout::new(1024.0, 768.0);
        layout.toggle_zen();
        layout.toggle_zen();
        assert!(!layout.zen);
    }

    #[test]
    fn rect_contains_works() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(50.0, 40.0));
        assert!(r.contains(10.0, 20.0)); // top-left corner inclusive
        assert!(r.contains(110.0, 70.0)); // bottom-right corner inclusive
        assert!(!r.contains(9.99, 40.0));
        assert!(!r.contains(50.0, 71.0));
    }

    #[test]
    fn sidebar_drag_v_center_repositions() {
        let mut layout = Layout::new(1024.0, 768.0);
        layout.sidebar_v_center = 0.2;
        let s_top = layout.rect(Zone::Sidebar);
        layout.sidebar_v_center = 0.8;
        let s_bottom = layout.rect(Zone::Sidebar);
        assert!(s_bottom.y > s_top.y);
    }

    /// Smoke test: spacing tokens used in defaults are reasonable.
    /// Toolbar height = 32 px icon + Spacing::Sm padding × 2 = 56 px.
    #[test]
    fn default_toolbar_height_matches_token_math() {
        let expected = 32.0 + ph2d_tokens::Spacing::Sm.px() * 2.0;
        let layout = Layout::new(1024.0, 768.0);
        assert_eq!(layout.toolbar_h, expected);
    }
}
