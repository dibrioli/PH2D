//! [`Divider`] — 1 px line in `Border` color, used to break sections
//! inside a panel or list.
//!
//! AccessKit `Role::Splitter`: screen readers treat it as a non-
//! interactive separator. Has no state — visual is always the same.

use crate::paint::{rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum DividerOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct Divider {
    pub id: NodeId,
    pub orientation: DividerOrientation,
}

impl Divider {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            orientation: DividerOrientation::Horizontal,
        }
    }

    pub fn orientation(mut self, o: DividerOrientation) -> Self {
        self.orientation = o;
        self
    }

    /// Build the AccessKit node. `Role::Splitter` is the canonical
    /// "section break"; non-focusable.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        // Splitter is non-interactive in our usage — provide a
        // static label so screen readers can verbalize it.
        NodeBuilder::new(Role::Splitter)
            .label("separator")
            .bounds(x, y, w, h)
            .build()
    }
}

/// 1-px line centered along the divider's minor axis.
pub fn paint_divider(divider: &Divider, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let color = resolve(ColorToken::Border, theme);
    let line = match divider.orientation {
        DividerOrientation::Horizontal => Rect {
            x: rect.x,
            y: rect.y + (rect.h - 1.0) * 0.5,
            w: rect.w,
            h: 1.0,
        },
        DividerOrientation::Vertical => Rect {
            x: rect.x + (rect.w - 1.0) * 0.5,
            y: rect.y,
            w: 1.0,
            h: rect.h,
        },
    };
    scene.fill_rect(rect_to_vello(line), color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_horizontal() {
        let d = Divider::new(NodeId(1));
        assert_eq!(d.orientation, DividerOrientation::Horizontal);
    }

    #[test]
    fn orientation_chain_works() {
        let d = Divider::new(NodeId(1)).orientation(DividerOrientation::Vertical);
        assert_eq!(d.orientation, DividerOrientation::Vertical);
    }

    #[test]
    fn a11y_role_is_splitter() {
        let d = Divider::new(NodeId(1));
        let node = d.build_a11y(0.0, 0.0, 100.0, 1.0);
        assert_eq!(node.role(), Role::Splitter);
    }

    #[test]
    fn paint_smoke_horizontal() {
        let d = Divider::new(NodeId(1));
        let mut scene = VectorScene::new();
        paint_divider(
            &d,
            Rect::new(0.0, 0.0, 200.0, 8.0),
            &mut scene,
            Theme::Forge,
        );
    }

    #[test]
    fn paint_smoke_vertical() {
        let d = Divider::new(NodeId(1)).orientation(DividerOrientation::Vertical);
        let mut scene = VectorScene::new();
        paint_divider(
            &d,
            Rect::new(0.0, 0.0, 8.0, 200.0),
            &mut scene,
            Theme::Sunstone,
        );
    }
}
