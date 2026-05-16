//! Quadtree — adaptive spatial subdivision for point queries.
//!
//! Cells are not uniform, so [`Quadtree`] does **not** implement
//! [`crate::GridMath`]; it exposes its own insert / query / leaf-
//! enumeration APIs.
//!
//! # Subdivision policy
//!
//! A leaf splits when it would hold more than `max_points_per_leaf`
//! points **and** its depth is below `max_depth`. The four children
//! divide the leaf's AABB into SW / SE / NW / NE quadrants of equal
//! size. Subdivision is purely numeric (no float jitter): each child
//! AABB is computed by halving the parent's extents around the
//! parent's center.
//!
//! # Determinism (HR-5)
//!
//! Insertion order matters for the contents of overlapping AABB
//! queries — output order matches insertion order within each leaf.
//! No `HashMap`; children stored in fixed `[Box<Node>; 4]`.

use crate::Vec2;

/// Axis-aligned bounding box `(min, max)` in world coords. `max` is
/// exclusive on the high side so adjacent cell AABBs partition the
/// plane without double-counting boundary points.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Width (x-extent).
    pub fn width(&self) -> f32 {
        self.max[0] - self.min[0]
    }

    /// Height (y-extent).
    pub fn height(&self) -> f32 {
        self.max[1] - self.min[1]
    }

    /// AABB center.
    pub fn center(&self) -> Vec2 {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }

    /// True iff `point` lies inside `self` (`[min, max)` on each axis).
    pub fn contains_point(&self, p: Vec2) -> bool {
        p[0] >= self.min[0] && p[0] < self.max[0] && p[1] >= self.min[1] && p[1] < self.max[1]
    }

    /// True iff `self` and `other` overlap.
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min[0] < other.max[0]
            && self.max[0] > other.min[0]
            && self.min[1] < other.max[1]
            && self.max[1] > other.min[1]
    }
}

/// Quadrant index — values match the order children are stored.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Quadrant {
    SW = 0,
    SE = 1,
    NW = 2,
    NE = 3,
}

/// Point + payload entry.
#[derive(Clone, Debug)]
pub struct Entry<T> {
    pub point: Vec2,
    pub data: T,
}

/// Single quadtree node — either a leaf with points or an internal
/// node with 4 children.
enum Node<T> {
    Leaf {
        bounds: AABB,
        depth: u32,
        entries: Vec<Entry<T>>,
    },
    Internal {
        bounds: AABB,
        children: Box<[Node<T>; 4]>,
    },
}

impl<T> Node<T> {
    fn bounds(&self) -> AABB {
        match self {
            Node::Leaf { bounds, .. } | Node::Internal { bounds, .. } => *bounds,
        }
    }
}

/// Quadtree spatial index.
pub struct Quadtree<T> {
    root: Node<T>,
    max_points_per_leaf: usize,
    max_depth: u32,
}

impl<T: Clone> Quadtree<T> {
    /// New empty quadtree spanning `bounds`. `max_points_per_leaf`
    /// triggers split when exceeded; `max_depth` caps subdivision
    /// (insertion past max_depth just appends to the leaf — accept
    /// the overflow rather than infinite-recurse).
    pub fn new(bounds: AABB, max_points_per_leaf: usize, max_depth: u32) -> Self {
        debug_assert!(max_points_per_leaf > 0);
        Self {
            root: Node::Leaf {
                bounds,
                depth: 0,
                entries: Vec::new(),
            },
            max_points_per_leaf,
            max_depth,
        }
    }

    /// Insert a point with attached `data`. Returns `false` if the
    /// point is outside the quadtree bounds (silently dropped).
    pub fn insert(&mut self, point: Vec2, data: T) -> bool {
        if !self.root.bounds().contains_point(point) {
            return false;
        }
        Self::insert_into(
            &mut self.root,
            Entry { point, data },
            self.max_points_per_leaf,
            self.max_depth,
        );
        true
    }

    fn insert_into(node: &mut Node<T>, entry: Entry<T>, max_per_leaf: usize, max_depth: u32) {
        match node {
            Node::Leaf {
                entries,
                depth,
                bounds,
            } => {
                entries.push(entry);
                if entries.len() > max_per_leaf && *depth < max_depth {
                    Self::split(node, max_per_leaf, max_depth);
                } else {
                    // No-op (cap reached or under threshold).
                    let _ = bounds;
                }
            }
            Node::Internal {
                children, bounds, ..
            } => {
                let q = quadrant_of(*bounds, entry.point);
                Self::insert_into(&mut children[q as usize], entry, max_per_leaf, max_depth);
            }
        }
    }

    fn split(node: &mut Node<T>, max_per_leaf: usize, max_depth: u32) {
        // Take ownership of leaf state, replace with Internal.
        let (bounds, depth, old_entries) = match node {
            Node::Leaf {
                bounds,
                depth,
                entries,
            } => (*bounds, *depth, core::mem::take(entries)),
            Node::Internal { .. } => return, // already split
        };
        let center = bounds.center();
        let make_child = |min: Vec2, max: Vec2| Node::Leaf {
            bounds: AABB { min, max },
            depth: depth + 1,
            entries: Vec::new(),
        };
        let mut children: [Node<T>; 4] = [
            make_child(bounds.min, center), // SW
            make_child([center[0], bounds.min[1]], [bounds.max[0], center[1]]), // SE
            make_child([bounds.min[0], center[1]], [center[0], bounds.max[1]]), // NW
            make_child(center, bounds.max), // NE
        ];
        for entry in old_entries {
            let q = quadrant_of(bounds, entry.point);
            Self::insert_into(&mut children[q as usize], entry, max_per_leaf, max_depth);
        }
        *node = Node::Internal {
            bounds,
            children: Box::new(children),
        };
    }

    /// All entries whose point lies inside `rect`. Clears + pushes.
    pub fn query_rect(&self, rect: AABB, out: &mut Vec<Entry<T>>) {
        out.clear();
        Self::query_rect_into(&self.root, rect, out);
    }

    fn query_rect_into(node: &Node<T>, rect: AABB, out: &mut Vec<Entry<T>>) {
        if !node.bounds().intersects(&rect) {
            return;
        }
        match node {
            Node::Leaf { entries, .. } => {
                for e in entries {
                    if rect.contains_point(e.point) {
                        out.push(e.clone());
                    }
                }
            }
            Node::Internal { children, .. } => {
                for c in children.iter() {
                    Self::query_rect_into(c, rect, out);
                }
            }
        }
    }

    /// All leaf AABBs, in deterministic depth-first order. Clears +
    /// pushes. Used by the editor render adapter to draw the
    /// subdivision tree.
    pub fn iter_leaf_bounds(&self, out: &mut Vec<AABB>) {
        out.clear();
        Self::collect_leaf_bounds(&self.root, out);
    }

    fn collect_leaf_bounds(node: &Node<T>, out: &mut Vec<AABB>) {
        match node {
            Node::Leaf { bounds, .. } => out.push(*bounds),
            Node::Internal { children, .. } => {
                for c in children.iter() {
                    Self::collect_leaf_bounds(c, out);
                }
            }
        }
    }

    /// Total entry count across all leaves. O(N) walk; cheap to
    /// re-derive on demand (no cached count to keep invalidation
    /// trivial).
    pub fn len(&self) -> usize {
        Self::count(&self.root)
    }

    /// Returns true when no points have been inserted (or all were
    /// outside bounds).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn count(node: &Node<T>) -> usize {
        match node {
            Node::Leaf { entries, .. } => entries.len(),
            Node::Internal { children, .. } => children.iter().map(Self::count).sum(),
        }
    }
}

fn quadrant_of(bounds: AABB, p: Vec2) -> Quadrant {
    let center = bounds.center();
    let east = p[0] >= center[0];
    let north = p[1] >= center[1];
    match (east, north) {
        (false, false) => Quadrant::SW,
        (true, false) => Quadrant::SE,
        (false, true) => Quadrant::NW,
        (true, true) => Quadrant::NE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_bounds() -> AABB {
        AABB::new([0.0, 0.0], [10.0, 10.0])
    }

    #[test]
    fn aabb_contains_excludes_max_edge() {
        let b = AABB::new([0.0, 0.0], [1.0, 1.0]);
        assert!(b.contains_point([0.0, 0.0]));
        assert!(b.contains_point([0.5, 0.5]));
        assert!(!b.contains_point([1.0, 0.5])); // max-x excluded
        assert!(!b.contains_point([0.5, 1.0])); // max-y excluded
    }

    #[test]
    fn quadrant_of_picks_correct_quadrant() {
        let b = AABB::new([0.0, 0.0], [10.0, 10.0]);
        assert_eq!(quadrant_of(b, [2.0, 2.0]), Quadrant::SW);
        assert_eq!(quadrant_of(b, [7.0, 2.0]), Quadrant::SE);
        assert_eq!(quadrant_of(b, [2.0, 7.0]), Quadrant::NW);
        assert_eq!(quadrant_of(b, [7.0, 7.0]), Quadrant::NE);
    }

    #[test]
    fn insert_outside_bounds_returns_false() {
        let mut qt: Quadtree<()> = Quadtree::new(unit_bounds(), 4, 6);
        assert!(!qt.insert([-1.0, 5.0], ()));
        assert!(!qt.insert([20.0, 5.0], ()));
        assert_eq!(qt.len(), 0);
    }

    #[test]
    fn insert_and_count() {
        let mut qt: Quadtree<u32> = Quadtree::new(unit_bounds(), 4, 6);
        for i in 0..10 {
            qt.insert([i as f32, i as f32], i);
        }
        assert_eq!(qt.len(), 10);
        assert!(!qt.is_empty());
    }

    #[test]
    fn leaf_splits_when_max_points_exceeded() {
        let mut qt: Quadtree<u32> = Quadtree::new(unit_bounds(), 2, 6);
        for i in 0..5 {
            qt.insert([1.0 + i as f32 * 0.1, 1.0 + i as f32 * 0.1], i);
        }
        // All points in SW quadrant; quadtree must split at least
        // once to bring per-leaf count to ≤ 2.
        let mut leaves = Vec::new();
        qt.iter_leaf_bounds(&mut leaves);
        assert!(
            leaves.len() > 1,
            "expected splits; got {} leaves",
            leaves.len()
        );
    }

    #[test]
    fn query_rect_finds_inserted_points() {
        let mut qt: Quadtree<u32> = Quadtree::new(unit_bounds(), 2, 6);
        for i in 0..10 {
            qt.insert([i as f32 + 0.5, i as f32 + 0.5], i);
        }
        // Query rect covering [3, 7) on both axes — should find
        // points 3, 4, 5, 6.
        let mut out = Vec::new();
        qt.query_rect(AABB::new([3.0, 3.0], [7.0, 7.0]), &mut out);
        let found: Vec<u32> = out.iter().map(|e| e.data).collect();
        for expected in [3, 4, 5, 6] {
            assert!(
                found.contains(&expected),
                "missing {expected}; found {found:?}"
            );
        }
        assert_eq!(found.len(), 4);
    }

    #[test]
    fn max_depth_caps_subdivision() {
        // Tiny max_depth=1; insert many points in one quadrant. Tree
        // must NOT subdivide past depth 1.
        let mut qt: Quadtree<u32> = Quadtree::new(unit_bounds(), 1, 1);
        for i in 0..20 {
            qt.insert([0.5, 0.5 + i as f32 * 0.01], i);
        }
        let mut leaves = Vec::new();
        qt.iter_leaf_bounds(&mut leaves);
        // 4 leaves at depth 1 (root split once).
        assert_eq!(leaves.len(), 4);
    }

    #[test]
    fn empty_quadtree_reports_empty() {
        let qt: Quadtree<u32> = Quadtree::new(unit_bounds(), 2, 6);
        assert!(qt.is_empty());
        assert_eq!(qt.len(), 0);
    }
}
