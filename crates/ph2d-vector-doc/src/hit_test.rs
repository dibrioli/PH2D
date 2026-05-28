//! Hit testing helpers for [`crate::VectorNetwork`].
//!
//! Per the W1 plan §2.1 module map + R4 audit Lens-G MED-G5:
//! Pen tool close-path detection, Select tool marquee, Direct Select
//! tool tangent grab — all need point-to-vertex distance lookup.
//!
//! W1 ships **only** the brute-force linear scan `nearest_vertex`
//! (O(N)) — sufficient for typical-document interactive use. W2+
//! lands a BVH spatial index for large networks via amendment ADR-0056
//! (or a sibling module here) when the Pencil / Direct Select tools
//! exercise it on dense networks.

use glam::Vec2;

use crate::cubic::VertexId;
use crate::network::VectorNetwork;

impl VectorNetwork {
    /// Return the [`VertexId`] of the vertex closest to `p` within
    /// `tolerance` pixels (Euclidean distance), or `None` if every
    /// vertex is farther than the tolerance.
    ///
    /// **W1 close-path detection helper** (R4 audit Lens-G MED-G5).
    /// Linear scan O(N_vertices); cache the result across frames if
    /// calling per pointer event.
    ///
    /// Ties broken by lowest [`VertexId`] (deterministic for replay).
    #[must_use]
    pub fn nearest_vertex(&self, p: Vec2, tolerance: f32) -> Option<VertexId> {
        let tol_sq = tolerance * tolerance;
        let mut best: Option<(VertexId, f32)> = None;
        for v in &self.vertices {
            let d_sq = (v.pos - p).length_squared();
            if d_sq > tol_sq {
                continue;
            }
            best = match best {
                None => Some((v.id, d_sq)),
                Some((_, prev_d_sq)) if d_sq < prev_d_sq => Some((v.id, d_sq)),
                // Tie-break: keep the lower id for deterministic replay.
                Some((prev_id, prev_d_sq)) if d_sq == prev_d_sq && v.id < prev_id => {
                    Some((v.id, d_sq))
                }
                other => other,
            };
        }
        best.map(|(id, _)| id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cubic::Vertex;

    fn make_net_with_3_vertices() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        net.vertices.push(Vertex::auto(2, Vec2::new(50.0, 86.6)));
        net
    }

    #[test]
    fn nearest_vertex_returns_closest_within_tolerance() {
        let net = make_net_with_3_vertices();
        assert_eq!(net.nearest_vertex(Vec2::new(1.0, 1.0), 5.0), Some(0));
        assert_eq!(net.nearest_vertex(Vec2::new(99.0, 1.0), 5.0), Some(1));
    }

    #[test]
    fn nearest_vertex_returns_none_when_all_out_of_tolerance() {
        let net = make_net_with_3_vertices();
        assert_eq!(net.nearest_vertex(Vec2::new(500.0, 500.0), 5.0), None);
    }

    #[test]
    fn nearest_vertex_handles_empty_network() {
        let net = VectorNetwork::empty();
        assert_eq!(net.nearest_vertex(Vec2::ZERO, 100.0), None);
    }

    #[test]
    fn nearest_vertex_tie_break_favors_lower_id() {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(5, Vec2::ZERO));
        net.vertices.push(Vertex::auto(2, Vec2::ZERO));
        net.vertices.push(Vertex::auto(7, Vec2::ZERO));
        assert_eq!(net.nearest_vertex(Vec2::ZERO, 1.0), Some(2));
    }
}
