//! The exact-topology boolean engine: maps each frozen [`BooleanOp`] (9
//! variants) onto Linesweeper's 4 set ops via the [`ph2d_vector_kurbo`] bridge,
//! composing where Illustrator's Pathfinder vocabulary is richer than raw set
//! algebra.
//!
//! This is the **reconcile** (canonical) half of the draft+reconcile pipeline
//! (ADR-0059 §2.4 + ADR-0065): exact planar topology via the Linesweeper
//! sweep-line, run on the CPU as the node's [`crate::MANIFEST`] `eval` output.
//! The SDF GPU *draft* (silhouette, ≤ 0.5 ms) is the renderer-side companion
//! (`crates/ph2d-vector/shaders/boolean_sdf.wgsl`); it is **not** topology and
//! never feeds this path (ADR-0065 §2.3).
//!
//! The `VectorNetwork` ⇄ `kurbo::BezPath` conversion, the Q16.16 determinism
//! snap, and the raw Linesweeper call all live in `ph2d-vector-kurbo` (shared
//! with `vector.offset` and the rest of the fan-out); this module owns only the
//! op vocabulary.

use ph2d_vector_doc::{BooleanOp, VectorNetwork};
use ph2d_vector_kurbo::{
    BinaryOp, boolean_paths, contours_to_network, fill_rule_of, network_to_bezpath,
};

/// Compute `op` over the filled regions of `a` and `b`, returning a fresh
/// network whose regions are the exact result faces.
///
/// Empty inputs (a network with no fillable regions — e.g. an open spiral) are
/// treated as the empty set: `Union` then yields the other operand, `Subtract`
/// yields `a`, `Intersect` yields nothing — all of which fall out naturally from
/// an empty `BezPath`.
#[must_use]
pub fn boolean(a: &VectorNetwork, b: &VectorNetwork, op: BooleanOp) -> VectorNetwork {
    let bez_a = network_to_bezpath(a);
    let bez_b = network_to_bezpath(b);
    let fill = fill_rule_of(a);
    let deterministic = a.deterministic && b.deterministic;

    // A∘B and B∘A in Linesweeper terms (fill rule is shared by both operands).
    let ab = |o| boolean_paths(&bez_a, &bez_b, fill, o);
    let ba = |o| boolean_paths(&bez_b, &bez_a, fill, o);

    let contours = match op {
        // Direct set ops.
        BooleanOp::Union => ab(BinaryOp::Union),
        BooleanOp::Subtract => ab(BinaryOp::Difference),
        BooleanOp::Intersect => ab(BinaryOp::Intersection),
        BooleanOp::Exclude => ab(BinaryOp::Xor),

        // Single-fill equivalences (exact for the W3 single-style carrier; the
        // Pathfinder distinction is colour-/operand-bias, not geometry):
        //  - Merge ≡ Union of co-incident faces (no operand bias).
        //  - Crop  ≡ clip A to B's area = Intersection.
        BooleanOp::Merge => ab(BinaryOp::Union),
        BooleanOp::Crop => ab(BinaryOp::Intersection),

        // Divide: cut both shapes along their mutual boundary into every face —
        // {A∖B} ∪ {A∩B} ∪ {B∖A}, each kept as separate regions.
        BooleanOp::Divide => {
            let mut v = ab(BinaryOp::Difference);
            v.extend(ab(BinaryOp::Intersection));
            v.extend(ba(BinaryOp::Difference));
            v
        }

        // Trim: keep parts of A outside B and parts of B outside A, as separate
        // abutting regions (the overlap is removed; unlike Exclude the two
        // pieces stay distinct rather than merging into one even-odd face).
        BooleanOp::Trim => {
            let mut v = ab(BinaryOp::Difference);
            v.extend(ba(BinaryOp::Difference));
            v
        }

        // Outline: the merged boundary. W3 emits the Union geometry; true
        // width-expansion stroke-outlining is `vector-outline-stroke` (T3.4+).
        BooleanOp::Outline => ab(BinaryOp::Union),
    };

    contours_to_network(&contours, deterministic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use ph2d_vector_doc::deterministic::snap;
    use ph2d_vector_doc::primitives;

    /// An axis-aligned square `[lo,hi]²` as a deterministic single-region
    /// network (the `vector.source` rect generator + the determinism flag).
    fn square(lo: f32, hi: f32) -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(lo, lo), Vec2::new(hi, hi));
        net.deterministic = true;
        net
    }

    /// Axis-aligned bounding box of all vertex positions.
    fn bbox(net: &VectorNetwork) -> (Vec2, Vec2) {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for v in &net.vertices {
            min = min.min(v.pos);
            max = max.max(v.pos);
        }
        (min, max)
    }

    fn approx(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 0.01
    }

    /// Every coordinate lies exactly on the Q16.16 grid — the cross-OS golden
    /// property.
    fn grid_snapped(net: &VectorNetwork) -> bool {
        let on = |v: Vec2| v == Vec2::new(snap(v.x), snap(v.y));
        net.vertices.iter().all(|v| on(v.pos))
            && net
                .segments
                .iter()
                .all(|s| on(s.out_at_start) && on(s.in_at_end))
    }

    const ALL_OPS: [BooleanOp; 9] = [
        BooleanOp::Union,
        BooleanOp::Subtract,
        BooleanOp::Intersect,
        BooleanOp::Exclude,
        BooleanOp::Divide,
        BooleanOp::Trim,
        BooleanOp::Merge,
        BooleanOp::Crop,
        BooleanOp::Outline,
    ];

    #[test]
    fn union_of_overlapping_squares_is_one_region() {
        // [0,2]² ∪ [1,3]² → one connected staircase, spanning [0,3]².
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(3.0, 3.0)));
    }

    #[test]
    fn intersect_of_overlapping_squares_is_the_overlap() {
        // [0,2]² ∩ [1,3]² = exactly [1,2]².
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Intersect);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(
            approx(min, Vec2::new(1.0, 1.0)) && approx(max, Vec2::new(2.0, 2.0)),
            "intersection bbox was {min:?}..{max:?}, expected [1,1]..[2,2]"
        );
    }

    #[test]
    fn subtract_is_a_minus_b() {
        // [0,2]² ∖ [1,3]² = an L-shape still bounded by [0,2]².
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Subtract);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(2.0, 2.0)));
    }

    #[test]
    fn intersect_of_disjoint_squares_is_empty() {
        let out = boolean(&square(0.0, 1.0), &square(2.0, 3.0), BooleanOp::Intersect);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty(), "disjoint intersection is empty");
    }

    #[test]
    fn union_of_disjoint_squares_is_two_regions() {
        let out = boolean(&square(0.0, 1.0), &square(2.0, 3.0), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 2, "disjoint union keeps two regions");
    }

    #[test]
    fn divide_of_overlap_yields_three_faces() {
        // {A∖B} ∪ {A∩B} ∪ {B∖A} — three distinct faces.
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Divide);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 3, "divide cuts into three faces");
    }

    #[test]
    fn every_op_produces_a_valid_network() {
        // DoD: all 9 variants run through Linesweeper without an invalid network
        // or a panic — both for overlapping and disjoint operands.
        for op in ALL_OPS {
            for (a, b) in [
                (square(0.0, 2.0), square(1.0, 3.0)), // overlapping
                (square(0.0, 1.0), square(2.0, 3.0)), // disjoint
            ] {
                let out = boolean(&a, &b, op);
                assert!(
                    out.validate().is_ok(),
                    "op {op:?} produced an invalid network"
                );
            }
        }
    }

    #[test]
    fn deterministic_inputs_snap_and_reproduce() {
        let a = square(0.0, 2.0);
        let b = square(1.0, 3.0);
        let out = boolean(&a, &b, BooleanOp::Union);
        assert!(out.deterministic, "both inputs deterministic → output too");
        assert!(grid_snapped(&out), "output must be Q16.16-snapped");
        assert_eq!(out, boolean(&a, &b, BooleanOp::Union));
    }

    #[test]
    fn non_deterministic_input_leaves_output_unflagged() {
        let mut a = square(0.0, 2.0);
        a.deterministic = false;
        let out = boolean(&a, &square(1.0, 3.0), BooleanOp::Union);
        assert!(!out.deterministic);
    }

    #[test]
    fn empty_inputs_do_not_panic() {
        let out = boolean(
            &VectorNetwork::empty(),
            &VectorNetwork::empty(),
            BooleanOp::Union,
        );
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }

    #[test]
    fn union_with_empty_returns_the_other_operand() {
        // A ∪ ∅ = A (one region, bounded by A's box).
        let out = boolean(&square(0.0, 2.0), &VectorNetwork::empty(), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(2.0, 2.0)));
    }
}
