//! [`Tooltip`] — small floating popover with a single-line message.
//!
//! Lives in `Layer::Overlay`. v1 paints unconditionally — the host
//! decides when (hover dwell, focus + key) to render it. AccessKit
//! `Role::Tooltip` so screen readers can announce it as auxiliary
//! help text.

use super::lista_flutuante::Superficie;
use crate::paint::{paint_text_centered, paint_text_elided};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
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
    let superficie = Superficie::de(theme);
    superficie.moldura_da_dica(scene, rect);
    if superficie.tinta().is_some() {
        // O `paint_text_centered` reserva respiro e cortava «Add entity» em «Add ent…» (foto m1_dica).
        let px = TypeToken::Sm.px();
        let w = text_system.prefix_width(&tip.message, px).min(rect.w);
        return paint_text_elided(
            text_system,
            scene,
            &tip.message,
            rect.x + (rect.w - w) * 0.5,
            rect.y + (rect.h - px) * 0.5,
            px,
            rect.w,
            superficie.cor(ColorToken::Text1),
        );
    }
    paint_text_centered(
        text_system,
        scene,
        &tip.message,
        rect,
        TypeToken::Sm.px(),
        superficie.cor(ColorToken::Text1),
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
        let mut text = TextSystem::without_system_fonts();
        paint_tooltip(
            &Tooltip::new(NodeId(1), "Save (Cmd+S)"),
            Rect::new(0.0, 0.0, 120.0, 24.0),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
    }
}
