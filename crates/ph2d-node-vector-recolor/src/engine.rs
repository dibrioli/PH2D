//! The recolor engine: reassign every region's **fill ref** (geometry intact).
//!
//! ## Carrier scope (important)
//!
//! A `VectorNetwork` carries fill/stroke **refs** (`u32` indices), not colours —
//! the OKLCH values live in the asset-level `StyleTable` (ADR-0056 §2.3). So a
//! graph node can only retarget which ref every region points at; the colour
//! that ref resolves to is an asset/document concern. v1 paints **all** regions
//! with one `fill` ref. Per-ref remapping / stroke recolour and a graph-carried
//! style channel are follow-ups. Vertices/segments/topology are untouched, so
//! the determinism flag passes through unchanged.

use ph2d_vector_doc::VectorNetwork;

/// Set every region's fill to `fill_ref`. Geometry is unchanged.
#[must_use]
pub fn recolor(input: &VectorNetwork, fill_ref: u32) -> VectorNetwork {
    let mut out = input.clone();
    for r in out.regions.iter_mut() {
        r.fill = Some(fill_ref);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use ph2d_vector_doc::primitives;

    fn rect() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn sets_every_region_fill() {
        let out = recolor(&rect(), 3);
        assert!(out.validate().is_ok());
        assert!(out.regions.iter().all(|r| r.fill == Some(3)));
    }

    #[test]
    fn geometry_is_untouched() {
        let input = rect();
        let out = recolor(&input, 7);
        assert_eq!(out.vertices, input.vertices);
        assert_eq!(out.segments, input.segments);
        assert_eq!(out.deterministic, input.deterministic);
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = recolor(&rect(), 1);
        assert!(out.deterministic);
        assert_eq!(out, recolor(&rect(), 1));
    }

    #[test]
    fn empty_input_is_valid() {
        let out = recolor(&VectorNetwork::empty(), 0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }
}
