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

/// Push a clip layer matching the popover's interior so any content
/// painted on top can't leak past the rounded corners. Always pair
/// with [`pop_popover_clip`]. Common pitfall — sharp Kurbo rect fills
/// inside a rounded popover paint visible "ears" at the corners (see
/// `docs/UI_Bugs/README.md` §3.1).
pub fn push_popover_clip(rect: Rect, scene: &mut VectorScene) {
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
    );
    scene.push_clip(&clip);
}

/// Pop the clip layer pushed by [`push_popover_clip`].
pub fn pop_popover_clip(scene: &mut VectorScene) {
    scene.pop_layer();
}

/// Suggested anchor rect for a popover: place it `gap` px below the
/// trigger, clamp horizontally to the viewport. The popover is
/// centered on the trigger when there's room, else right-aligned to
/// the viewport edge. Returns the chosen rect.
pub fn anchor_below(trigger: Rect, content: (f32, f32), viewport: Rect, gap: f32) -> Rect {
    let (w, h) = content;
    let center_x = trigger.x + (trigger.w - w) * 0.5;
    let clamped_x = center_x.max(viewport.x).min(viewport.x + viewport.w - w);
    let y = trigger.y + trigger.h + gap;
    // Flip above the trigger if there isn't room below.
    let y = if y + h > viewport.y + viewport.h {
        (trigger.y - h - gap).max(viewport.y)
    } else {
        y
    };
    Rect::new(clamped_x, y, w, h)
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
    fn anchor_below_centers_when_room() {
        let trigger = Rect::new(100.0, 50.0, 80.0, 20.0);
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        let r = anchor_below(trigger, (120.0, 80.0), viewport, 4.0);
        let center_x = trigger.x + trigger.w * 0.5;
        let r_center_x = r.x + r.w * 0.5;
        assert!((r_center_x - center_x).abs() < 0.001);
        assert!(r.y > trigger.y + trigger.h);
    }

    #[test]
    fn anchor_below_flips_above_when_no_room() {
        let trigger = Rect::new(50.0, 550.0, 80.0, 20.0);
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        let r = anchor_below(trigger, (120.0, 80.0), viewport, 4.0);
        assert!(r.y + r.h <= trigger.y);
    }

    #[test]
    fn anchor_below_clamps_to_viewport_right() {
        let trigger = Rect::new(750.0, 50.0, 40.0, 20.0);
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        let r = anchor_below(trigger, (120.0, 80.0), viewport, 4.0);
        assert!(r.x + r.w <= viewport.x + viewport.w + 0.001);
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
