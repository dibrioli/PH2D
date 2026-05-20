//! [`Avatar`] — square or circle preview chip with placeholder
//! initial when no image data is supplied.
//!
//! Image loading is the shell's job (paths through `ph2d-asset`); the
//! avatar widget only knows the *initial* glyph for the placeholder
//! variant. M14+ wires real image bitmaps via `peniko::Image`.

use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum AvatarShape {
    #[default]
    Circle,
    Square,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum AvatarState {
    #[default]
    Normal,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Avatar {
    pub id: NodeId,
    pub label: String,
    /// Single-glyph placeholder (the leading letter of a name, etc).
    /// Used until the real image lands in M14+.
    pub initial: char,
    pub shape: AvatarShape,
    pub state: AvatarState,
}

impl Avatar {
    pub fn new(id: NodeId, label: impl Into<String>, initial: char) -> Self {
        Self {
            id,
            label: label.into(),
            initial,
            shape: AvatarShape::Circle,
            state: AvatarState::Normal,
        }
    }

    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn state(mut self, state: AvatarState) -> Self {
        self.state = state;
        self
    }

    /// Build the AccessKit node. `Role::Image` with the avatar's
    /// label as the alt text.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Image)
            .label(&self.label)
            .bounds(x, y, w, h)
            .build()
    }
}

/// Filled circle/square + centered initial glyph. Uses `Bg2` for the
/// fill so it reads as a chip on top of `Bg1`/`BgElev` panels.
pub fn paint_avatar(
    avatar: &Avatar,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = match avatar.shape {
        AvatarShape::Circle => Radius::Full.px(),
        AvatarShape::Square => Radius::Md.px(),
    };
    let bg_token = if avatar.state == AvatarState::Disabled {
        ColorToken::Border
    } else {
        ColorToken::Bg2
    };
    fill_rounded_rect(scene, rect, radius, resolve(bg_token, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let glyph = avatar
        .initial
        .to_uppercase()
        .next()
        .unwrap_or('?')
        .to_string();
    let fg = if avatar.state == AvatarState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    let size = (rect.h * 0.5).clamp(TypeToken::Sm.px(), TypeToken::Lg.px());
    paint_text_centered(text_system, scene, &glyph, rect, size, resolve(fg, theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let a = Avatar::new(NodeId(1), "Enio", 'E');
        assert_eq!(a.shape, AvatarShape::Circle);
        assert_eq!(a.state, AvatarState::Normal);
        assert_eq!(a.initial, 'E');
    }

    #[test]
    fn a11y_role_is_image() {
        let node = Avatar::new(NodeId(1), "Avatar", 'A').build_a11y(0.0, 0.0, 32.0, 32.0);
        assert_eq!(node.role(), Role::Image);
        assert_eq!(node.label(), Some("Avatar"));
    }

    fn smoke(av: Avatar, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_avatar(
            &av,
            Rect::new(0.0, 0.0, 32.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_circle() {
        smoke(Avatar::new(NodeId(1), "x", 'X'), Theme::Forge);
    }

    #[test]
    fn paint_smoke_square() {
        smoke(
            Avatar::new(NodeId(1), "x", 'X').shape(AvatarShape::Square),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            Avatar::new(NodeId(1), "x", 'X').state(AvatarState::Disabled),
            Theme::Blueprint,
        );
    }
}
