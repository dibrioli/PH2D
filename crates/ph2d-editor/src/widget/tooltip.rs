//! [`Tooltip`] — small floating popover with a single-line message.
//!
//! Lives in `Layer::Overlay`. v1 paints unconditionally — the host
//! decides when (hover dwell, focus + key) to render it. AccessKit
//! `Role::Tooltip` so screen readers can announce it as auxiliary
//! help text.

use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct Tooltip {
    pub id: NodeId,
    pub message: String,
}

impl Tooltip {
    pub fn new(id: NodeId, message: impl Into<String>) -> Self {
        Self {
            id,
            message: message.into(),
        }
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Tooltip)
            .label(&self.message)
            .bounds(x, y, w, h)
            .build()
    }
}

pub fn paint_tooltip(
    tip: &Tooltip,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg3, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    paint_text_centered(
        text_system,
        scene,
        &tip.message,
        rect,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = Tooltip::new(NodeId(1), "Save (Cmd+S)");
        assert_eq!(t.message, "Save (Cmd+S)");
    }

    #[test]
    fn a11y_role_is_tooltip() {
        let node = Tooltip::new(NodeId(1), "x").build_a11y(0.0, 0.0, 100.0, 24.0);
        assert_eq!(node.role(), Role::Tooltip);
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_tooltip(
            &Tooltip::new(NodeId(1), "Save (Cmd+S)"),
            Rect::new(0.0, 0.0, 120.0, 24.0),
            &mut scene,
            &mut text,
            Theme::ForgeSdf,
        );
    }
}
