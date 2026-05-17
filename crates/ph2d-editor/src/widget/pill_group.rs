//! [`PillGroup`] — horizontal `BgElev` chip grouping N icon-only
//! Buttons (and similar children) in a single rounded surface.
//!
//! Used 5x in the editor TopBar (theme cluster, save chip, project
//! chip, theme/play cluster, right cluster). Pure layout primitive:
//! caller paints children into the slot rects we expose via
//! [`PillGroup::child_rect`].

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, PILL_PADDING_PX as CHROME_PILL_PADDING, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Default padding inside the pill body. Children sit inside this
/// inset; total width grows linearly with `child_count`.
/// Per tokens.json `chrome.pill-padding`.
pub const PILL_PADDING_PX: f32 = CHROME_PILL_PADDING;

#[derive(Clone, Debug)]
pub struct PillGroup {
    pub id: NodeId,
    pub label: String,
    /// Number of slots reserved for children. Used to compute
    /// [`PillGroup::preferred_width`] and [`PillGroup::child_rect`].
    pub child_count: usize,
    /// Edge length of each child slot (square). 32 px matches the
    /// design system "icon button" size.
    pub child_size: f32,
}

impl PillGroup {
    pub fn new(id: NodeId, label: impl Into<String>, child_count: usize) -> Self {
        Self {
            id,
            label: label.into(),
            child_count,
            child_size: Spacing::Xl3.px(),
        }
    }

    pub fn child_size(mut self, size: f32) -> Self {
        self.child_size = size;
        self
    }

    /// Total width needed to fit every child slot + padding.
    pub fn preferred_width(&self) -> f32 {
        let count = self.child_count.max(1) as f32;
        count * self.child_size + PILL_PADDING_PX * 2.0
    }

    /// Total height = child_size + 2*padding.
    pub fn preferred_height(&self) -> f32 {
        self.child_size + PILL_PADDING_PX * 2.0
    }

    /// Slot rect for child `index` inside the pill at `host`.
    pub fn child_rect(&self, host: Rect, index: usize) -> Rect {
        let pad = PILL_PADDING_PX;
        let x = host.x + pad + self.child_size * index as f32;
        let y = host.y + (host.h - self.child_size) * 0.5;
        Rect::new(x, y, self.child_size, self.child_size)
    }

    /// Build the AccessKit container node. `Role::Toolbar` matches
    /// the "row of related actions" semantic used in the design
    /// system audit.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Toolbar)
            .label(&self.label)
            .bounds(x, y, w, h)
            .build()
    }
}

/// Paint the pill body. Children are NOT painted here — call sites
/// invoke their own `paint_button`/`paint_icon` into the slot rects.
pub fn paint_pill_group(_group: &PillGroup, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Xl.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PillGroup {
        PillGroup::new(NodeId(1), "TopBar group", 3)
    }

    #[test]
    fn defaults_match_spec() {
        let p = fixture();
        assert_eq!(p.child_count, 3);
        assert_eq!(p.child_size, 32.0);
    }

    #[test]
    fn preferred_width_scales_with_count() {
        let p = fixture();
        let w1 = PillGroup::new(NodeId(1), "x", 1).preferred_width();
        let w3 = p.preferred_width();
        assert!(w3 > w1);
        assert!((w3 - (32.0 * 3.0 + PILL_PADDING_PX * 2.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn child_rect_indices_step_by_child_size() {
        let p = fixture();
        let host = Rect::new(0.0, 0.0, p.preferred_width(), p.preferred_height());
        let r0 = p.child_rect(host, 0);
        let r1 = p.child_rect(host, 1);
        assert!((r1.x - r0.x - p.child_size).abs() < f32::EPSILON);
    }

    #[test]
    fn child_rect_centered_vertically() {
        let p = fixture();
        let host = Rect::new(0.0, 0.0, p.preferred_width(), 60.0);
        let r0 = p.child_rect(host, 0);
        assert!((r0.y - (host.y + (host.h - p.child_size) * 0.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn a11y_role_is_toolbar() {
        let node = fixture().build_a11y(0.0, 0.0, 120.0, 40.0);
        assert_eq!(node.role(), Role::Toolbar);
    }

    fn smoke(p: PillGroup, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_pill_group(
            &p,
            Rect::new(0.0, 0.0, p.preferred_width(), p.preferred_height()),
            &mut scene,
            theme,
        );
    }

    #[test]
    fn paint_smoke_default() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_single_slot() {
        smoke(PillGroup::new(NodeId(1), "x", 1), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_large_slots() {
        smoke(
            PillGroup::new(NodeId(1), "x", 4).child_size(44.0),
            Theme::Blueprint,
        );
    }
}
