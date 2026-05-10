//! [`HitIndex`] — `(NodeId, Rect)` slots indexed by paint order.
//!
//! Paint pass calls [`HitIndex::register`] as it emits each
//! interactive widget's geometry. Pointer dispatch consults
//! [`HitIndex::hit`], which walks back-to-front so the most-recently
//! painted (top-most z) widget wins for overlapping rects.
//!
//! Backing store is [`smallvec::SmallVec`] inline up to 128 entries
//! (covers the ~31 widgets in the current hero with room for several
//! more screens). Beyond 128, it spills to heap — visible in
//! profiler. [`HitIndex::clear_for_frame`] drops length to 0 without
//! freeing capacity.

use crate::zones::Rect;
use ph2d_a11y::NodeId;
use smallvec::SmallVec;

const INLINE_CAPACITY: usize = 128;

#[derive(Debug, Default)]
pub struct HitIndex {
    /// Painted in z-order (back→front). Iteration for `hit` reverses
    /// so first match is the top-most widget.
    rects: SmallVec<[(NodeId, Rect); INLINE_CAPACITY]>,
}

impl HitIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop registered rects without freeing capacity. Called once
    /// per frame before the paint pass re-registers everything.
    pub fn clear_for_frame(&mut self) {
        self.rects.clear();
    }

    /// Register a widget's rect. Paint pass calls this for each
    /// interactive widget as it emits geometry. Multiple registrations
    /// for the same `NodeId` are allowed — the latest wins for hit
    /// (used when a popover / tooltip overlays an existing widget).
    pub fn register(&mut self, id: NodeId, rect: Rect) {
        self.rects.push((id, rect));
    }

    /// Find the topmost widget under the cursor. Returns `None` if
    /// the cursor is outside every registered rect.
    pub fn hit(&self, x: f32, y: f32) -> Option<NodeId> {
        for (id, rect) in self.rects.iter().rev() {
            if rect.contains(x, y) {
                return Some(*id);
            }
        }
        None
    }

    /// Number of registered rects this frame.
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    /// Inline capacity — used by HR-3 tests to assert no heap spill.
    pub fn inline_capacity(&self) -> usize {
        INLINE_CAPACITY
    }

    /// True iff the rect storage is still inline (no heap spill).
    pub fn is_inline(&self) -> bool {
        !self.rects.spilled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(x, y, w, h)
    }

    #[test]
    fn empty_index_no_hit() {
        let idx = HitIndex::new();
        assert!(idx.hit(50.0, 50.0).is_none());
    }

    #[test]
    fn hit_returns_topmost_overlap() {
        let mut idx = HitIndex::new();
        idx.register(NodeId(1), r(0.0, 0.0, 100.0, 100.0));
        idx.register(NodeId(2), r(20.0, 20.0, 60.0, 60.0));
        // (50, 50) sits inside both — newer registration wins.
        assert_eq!(idx.hit(50.0, 50.0), Some(NodeId(2)));
        // (10, 10) only inside the first.
        assert_eq!(idx.hit(10.0, 10.0), Some(NodeId(1)));
    }

    #[test]
    fn hit_outside_all_rects_returns_none() {
        let mut idx = HitIndex::new();
        idx.register(NodeId(1), r(0.0, 0.0, 50.0, 50.0));
        assert!(idx.hit(100.0, 100.0).is_none());
    }

    #[test]
    fn clear_for_frame_keeps_capacity() {
        let mut idx = HitIndex::new();
        for i in 0..30 {
            idx.register(NodeId(i as u64), r(i as f32 * 10.0, 0.0, 5.0, 5.0));
        }
        let cap_before = idx.rects.capacity();
        idx.clear_for_frame();
        assert_eq!(idx.len(), 0);
        assert!(idx.rects.capacity() >= cap_before);
    }

    #[test]
    fn stays_inline_under_capacity() {
        let mut idx = HitIndex::new();
        for i in 0..INLINE_CAPACITY {
            idx.register(NodeId(i as u64), r(0.0, 0.0, 1.0, 1.0));
        }
        assert!(
            idx.is_inline(),
            "should not spill at exactly INLINE_CAPACITY"
        );
    }

    #[test]
    fn spills_past_inline_capacity() {
        let mut idx = HitIndex::new();
        for i in 0..(INLINE_CAPACITY + 1) {
            idx.register(NodeId(i as u64), r(0.0, 0.0, 1.0, 1.0));
        }
        assert!(!idx.is_inline());
    }
}
