//! The parallel/contour offset engine (ADR-0058 §2.2.3, plan T3.4).
//!
//! CPU offset of closed regions via **stroke band + boolean** (the spec's kurbo
//! `Offset` CPU fallback; the GPU Euler-spiral path is a later optimization):
//! stroke the region boundary into a band of width `2·|distance|` with the
//! chosen join, then `Union` it with the region (outward, `distance > 0`) or
//! `Difference` it (inward, `distance < 0`). The stroke reaches `|distance|` to
//! each side of the boundary, so the union grows / the difference shrinks the
//! filled area by exactly `distance` — with the join style shaping the corners.
//!
//! Reuses the shared [`ph2d_vector_kurbo`] conversion + Linesweeper boolean.
//! **Open-path → region** offset is `vector.outline-stroke`'s job (§2.2.4), not
//! this node — a region-less input offsets to nothing.

use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_kurbo::{
    BinaryOp, Cap, Join, Stroke, StrokeOpts, boolean_paths, contours_to_network, fill_rule_of,
    network_to_bezpath, stroke,
};

/// Curve-flattening tolerance for the stroke band (network-local px). Fixed (no
/// adaptive zoom) so the offset is reproducible cross-OS once snapped to Q16.16
/// (ADR-0065 §2.4).
const STROKE_TOLERANCE: f64 = 0.05;

/// Offset every filled region of `input` by `distance` (negative = inset),
/// joining corners with `join` (miter capped by `miter_limit`).
#[must_use]
pub fn offset(input: &VectorNetwork, distance: f32, join: Join, miter_limit: f32) -> VectorNetwork {
    // A zero (or non-finite) offset is the identity — skip the lossy
    // stroke+boolean round trip and preserve the input verbatim.
    if distance == 0.0 || !distance.is_finite() {
        return input.clone();
    }
    let bez = network_to_bezpath(input);
    if bez.is_empty() {
        // No fillable region → nothing to offset (open-path offset is
        // vector.outline-stroke's domain).
        return VectorNetwork::empty();
    }
    let style = Stroke::new(2.0 * f64::from(distance.abs()))
        .with_join(join)
        .with_miter_limit(f64::from(miter_limit.max(1.0)))
        .with_caps(Cap::Butt);
    let band = stroke(bez.iter(), &style, &StrokeOpts::default(), STROKE_TOLERANCE);
    let fill = fill_rule_of(input);
    let op = if distance > 0.0 {
        BinaryOp::Union
    } else {
        BinaryOp::Difference
    };
    let contours = boolean_paths(&bez, &band, fill, op);
    contours_to_network(&contours, input.deterministic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use ph2d_vector_doc::deterministic::snap;
    use ph2d_vector_doc::primitives;

    fn square(lo: f32, hi: f32) -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(lo, lo), Vec2::new(hi, hi));
        net.deterministic = true;
        net
    }

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
        (a - b).length() < 0.05
    }

    fn grid_snapped(net: &VectorNetwork) -> bool {
        let on = |v: Vec2| v == Vec2::new(snap(v.x), snap(v.y));
        net.vertices.iter().all(|v| on(v.pos))
            && net
                .segments
                .iter()
                .all(|s| on(s.out_at_start) && on(s.in_at_end))
    }

    #[test]
    fn outward_offset_grows_the_box_by_distance() {
        // [0,2]² offset +1 → the straight edges move out to [-1,3]² (corners
        // rounded inside that box; the bbox is set by the edges).
        let out = offset(&square(0.0, 2.0), 1.0, Join::Round, 4.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(
            approx(min, Vec2::new(-1.0, -1.0)) && approx(max, Vec2::new(3.0, 3.0)),
            "outward offset bbox was {min:?}..{max:?}, expected [-1,-1]..[3,3]"
        );
    }

    #[test]
    fn inward_offset_shrinks_the_box_by_distance() {
        // [0,2]² offset -0.5 → [0.5,1.5]².
        let out = offset(&square(0.0, 2.0), -0.5, Join::Miter, 4.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(
            approx(min, Vec2::new(0.5, 0.5)) && approx(max, Vec2::new(1.5, 1.5)),
            "inward offset bbox was {min:?}..{max:?}, expected [0.5,0.5]..[1.5,1.5]"
        );
    }

    #[test]
    fn over_erosion_yields_empty() {
        // A 2-wide square inset by 1.5 erodes past nothing → empty.
        let out = offset(&square(0.0, 2.0), -1.5, Join::Round, 4.0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty(), "over-inset must vanish");
    }

    #[test]
    fn zero_offset_is_the_identity() {
        let input = square(0.0, 2.0);
        let out = offset(&input, 0.0, Join::Round, 4.0);
        assert_eq!(out, input, "zero offset returns the input verbatim");
    }

    #[test]
    fn deterministic_input_snaps_and_reproduces() {
        let input = square(0.0, 2.0);
        let out = offset(&input, 1.0, Join::Round, 4.0);
        assert!(out.deterministic);
        assert!(grid_snapped(&out), "output must be Q16.16-snapped");
        assert_eq!(out, offset(&input, 1.0, Join::Round, 4.0), "reproducible");
    }

    #[test]
    fn region_less_input_offsets_to_nothing() {
        let out = offset(&VectorNetwork::empty(), 5.0, Join::Round, 4.0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }

    #[test]
    fn round_and_bevel_joins_both_produce_valid_offsets() {
        for join in [Join::Round, Join::Bevel, Join::Miter] {
            let out = offset(&square(0.0, 2.0), 1.0, join, 4.0);
            assert!(out.validate().is_ok(), "join {join:?} produced invalid offset");
            assert_eq!(out.regions.len(), 1);
        }
    }
}
