//! [`Spinner`] — indeterminate progress indicator.
//!
//! Always rotating in the live UI; the static frame is rendered via
//! `IconId::Spinner`. Rotation animation is shell-side (timeline +
//! per-frame Affine::rotate) — out of scope for paint.

use crate::icons::IconId;
use crate::paint::paint_icon;
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct Spinner {
    pub id: NodeId,
    pub label: String,
}

impl Spinner {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Build the AccessKit node. `Role::ProgressIndicator` without
    /// numeric value signals "indeterminate" to assistive tech.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::ProgressIndicator)
            .label(&self.label)
            .bounds(x, y, w, h)
            .build()
    }
}

/// Static spinner glyph at the rect, tinted with `Accent`. The shell
/// applies an `Affine::rotate` around the rect center each frame to
/// animate.
pub fn paint_spinner(spinner: &Spinner, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let _ = spinner;
    let color = crate::paint::resolve(ColorToken::Accent, theme);
    paint_icon(
        scene,
        IconId::Spinner,
        rect,
        color,
        StrokeToken::Default.px(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let s = Spinner::new(NodeId(1), "Loading");
        assert_eq!(s.id, NodeId(1));
        assert_eq!(s.label, "Loading");
    }

    #[test]
    fn a11y_role_is_progress_indicator() {
        let node = Spinner::new(NodeId(1), "Loading").build_a11y(0.0, 0.0, 24.0, 24.0);
        assert_eq!(node.role(), Role::ProgressIndicator);
        assert_eq!(node.label(), Some("Loading"));
    }

    #[test]
    fn paint_smoke() {
        let s = Spinner::new(NodeId(1), "Loading");
        let mut scene = VectorScene::new();
        paint_spinner(
            &s,
            Rect::new(0.0, 0.0, 24.0, 24.0),
            &mut scene,
            Theme::Forge,
        );
    }
}
