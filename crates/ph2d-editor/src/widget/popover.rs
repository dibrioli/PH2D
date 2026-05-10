//! [`Popover`] — generic floating container reused by Dropdown,
//! Tooltip, ContextMenu when they need a shadow-elevated surface.
//!
//! Pure paint primitive: caller supplies the rect, popover paints
//! body + border. AccessKit `Role::Group` so screen readers don't
//! announce this as anything specific — the *content* declares the
//! role (Menu / Listbox / Tooltip / etc).

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct Popover {
    pub id: NodeId,
}

impl Popover {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Group)
            .label("popover")
            .bounds(x, y, w, h)
            .build()
    }
}

pub fn paint_popover(_popover: &Popover, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a11y_role_is_group() {
        let node = Popover::new(NodeId(1)).build_a11y(0.0, 0.0, 100.0, 100.0);
        assert_eq!(node.role(), Role::Group);
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        paint_popover(
            &Popover::new(NodeId(1)),
            Rect::new(10.0, 10.0, 200.0, 100.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }
}
