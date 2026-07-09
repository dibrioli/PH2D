//! Hit registration that respects the panel's scroll clip.
//!
//! The scrolled body is clipped **visually** (`VectorScene::push_clip`), but the
//! [`HitIndex`] is a flat, global list of rects with no notion of layers. A widget
//! scrolled up under the title bar — or down past the panel's foot — keeps its
//! registered rect and stays clickable while invisible. Every body widget therefore
//! registers through [`ClippedHits`] instead of the raw index.
//!
//! The scrollbar thumb does NOT: it lives on the rail, outside the clipped body.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::zones::Rect;

/// A [`HitIndex`] that only accepts widgets lying wholly inside `clip`.
pub(crate) struct ClippedHits<'a> {
    hit_index: &'a mut HitIndex,
    clip: Rect,
}

impl<'a> ClippedHits<'a> {
    pub(crate) fn new(hit_index: &'a mut HitIndex, clip: Rect) -> Self {
        Self { hit_index, clip }
    }

    /// Register `rect` iff it lies wholly within the visible body. A widget straddling
    /// the edge is dropped rather than left half-clickable: a slider you can only grab
    /// by its bottom half is worse than one you must scroll into view first.
    pub(crate) fn register(&mut self, id: NodeId, rect: Rect) {
        if contains_rect(self.clip, rect) {
            self.hit_index.register(id, rect);
        }
    }
}

/// Whether `inner` lies wholly within `outer`.
fn contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.w <= outer.x + outer.w
        && inner.y + inner.h <= outer.y + outer.h
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: Rect = Rect {
        x: 0.0,
        y: 100.0,
        w: 240.0,
        h: 400.0,
    };

    fn id(n: u64) -> NodeId {
        NodeId(n)
    }

    /// A widget inside the body registers and is hittable.
    #[test]
    fn a_visible_widget_registers() {
        let mut index = HitIndex::new();
        ClippedHits::new(&mut index, BODY).register(id(1), Rect::new(10.0, 200.0, 100.0, 20.0));
        assert_eq!(index.hit(50.0, 210.0), Some(id(1)));
    }

    /// THE point of this type: a widget scrolled up under the title bar is invisible,
    /// so it must also be un-clickable. Registering it would leave a live button in
    /// the panel's header — painted over, still armed.
    #[test]
    fn a_widget_scrolled_above_the_body_is_not_clickable() {
        let mut index = HitIndex::new();
        let above = Rect::new(10.0, 40.0, 100.0, 20.0); // header strip, y < BODY.y
        ClippedHits::new(&mut index, BODY).register(id(2), above);
        assert_eq!(index.hit(50.0, 50.0), None, "ghost button in the header");
    }

    /// Same below the foot — a widget scrolled past the bottom would otherwise stay
    /// clickable over whatever the panel sits on.
    #[test]
    fn a_widget_scrolled_below_the_body_is_not_clickable() {
        let mut index = HitIndex::new();
        let below = Rect::new(10.0, 600.0, 100.0, 20.0);
        ClippedHits::new(&mut index, BODY).register(id(3), below);
        assert_eq!(index.hit(50.0, 610.0), None);
    }

    /// A widget straddling the edge is dropped whole. Half a slider is not a slider.
    #[test]
    fn a_widget_straddling_the_edge_is_dropped_whole() {
        let mut index = HitIndex::new();
        let straddling = Rect::new(10.0, 90.0, 100.0, 20.0); // crosses BODY.y = 100
        ClippedHits::new(&mut index, BODY).register(id(4), straddling);
        assert_eq!(index.hit(50.0, 105.0), None, "half-grabbable widget");
    }

    /// Exactly flush with the body's bounds still counts as inside — otherwise the
    /// first and last rows of a full-height body would be dead.
    #[test]
    fn a_widget_flush_with_the_bounds_is_inside() {
        let mut index = HitIndex::new();
        let flush = Rect::new(BODY.x, BODY.y, BODY.w, BODY.h);
        ClippedHits::new(&mut index, BODY).register(id(5), flush);
        assert_eq!(index.hit(120.0, 300.0), Some(id(5)));
    }
}
