//! Sanitized [`SemanticTokens`] → [`VectorNetwork`] (ADR-0061 §2.3): the parsed
//! shape lowers to a `ph2d-vector-doc` primitive (`spiral`, `polygon`, …) or an
//! explicit polyline. The output is a **standard, editable** network — the whole
//! point (vs an opaque SVG dump).
//!
//! Call only after [`crate::sanitizer::sanitize`] passes. As defense-in-depth
//! the primitives also self-clamp (`sides → [3, MAX_POLYGON_SIDES]`, etc.), so a
//! sanitizer bypass still cannot allocate unbounded geometry.

use glam::Vec2;
use ph2d_vector_doc::{primitives, Region, Segment, VectorNetwork, Vertex, WindingRule};

use crate::semantic_tokens::{SemanticTokens, Shape};

/// Lower sanitized tokens to a vector network.
pub fn tokens_to_network(tokens: &SemanticTokens) -> VectorNetwork {
    match &tokens.shape {
        Shape::Spiral {
            center,
            inner_radius,
            outer_radius,
            turns,
            samples_per_turn,
            rotation,
        } => primitives::spiral(
            *center,
            *inner_radius,
            *outer_radius,
            *turns,
            *samples_per_turn,
            *rotation,
        ),
        Shape::Polygon {
            center,
            radius,
            sides,
            rotation,
        } => primitives::polygon(*center, *radius, *sides, *rotation),
        Shape::Star {
            center,
            outer_radius,
            inner_radius,
            points,
            rotation,
        } => primitives::star(*center, *outer_radius, *inner_radius, *points, *rotation),
        Shape::Ellipse { center, radii } => primitives::ellipse(*center, *radii),
        Shape::Rect { corner_a, corner_b } => primitives::rect(*corner_a, *corner_b),
        Shape::Path { vertices, closed } => path_network(vertices, *closed),
    }
}

/// Build a network from explicit vertices: straight chain, optionally closed
/// with a `NonZero` region.
fn path_network(vertices: &[Vec2], closed: bool) -> VectorNetwork {
    let mut net = VectorNetwork::empty();
    for (i, v) in vertices.iter().enumerate() {
        net.vertices.push(Vertex::auto(i as u32, *v));
    }
    let n = vertices.len() as u32;
    if n < 2 {
        return net; // a point (or nothing) — no segments
    }
    for i in 0..n - 1 {
        net.segments.push(Segment::straight(i, i, i + 1));
    }
    if closed {
        net.segments.push(Segment::straight(n - 1, n - 1, 0));
        let mut region = Region::new(0, WindingRule::NonZero);
        region.segments = (0..n).map(|s| (s, true)).collect();
        net.regions.push(region);
    }
    net
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_tokens::{StyleTokens, SemanticTokens};

    fn toks(shape: Shape) -> SemanticTokens {
        SemanticTokens {
            shape,
            style: StyleTokens::default(),
        }
    }

    #[test]
    fn polygon_lowers_to_editable_region() {
        let net = tokens_to_network(&toks(Shape::Polygon {
            center: Vec2::ZERO,
            radius: 100.0,
            sides: 6,
            rotation: 0.0,
        }));
        assert!(net.validate().is_ok());
        assert_eq!(net.vertices.len(), 6, "hexagon — editable downstream");
        assert_eq!(net.regions.len(), 1);
    }

    #[test]
    fn spiral_lowers_to_open_path() {
        let net = tokens_to_network(&toks(Shape::Spiral {
            center: Vec2::ZERO,
            inner_radius: 0.0,
            outer_radius: 100.0,
            turns: 4.0,
            samples_per_turn: 16,
            rotation: 0.0,
        }));
        assert!(net.validate().is_ok());
        assert!(net.regions.is_empty(), "spiral is an open stroke");
        assert!(net.vertices.len() > 4);
    }

    #[test]
    fn closed_path_makes_a_region() {
        let net = tokens_to_network(&toks(Shape::Path {
            vertices: vec![Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(5.0, 8.0)],
            closed: true,
        }));
        assert!(net.validate().is_ok());
        assert_eq!(net.segments.len(), 3, "2 edges + closing");
        assert_eq!(net.regions.len(), 1);
    }

    #[test]
    fn open_path_has_no_region() {
        let net = tokens_to_network(&toks(Shape::Path {
            vertices: vec![Vec2::ZERO, Vec2::new(10.0, 0.0)],
            closed: false,
        }));
        assert!(net.validate().is_ok());
        assert_eq!(net.segments.len(), 1);
        assert!(net.regions.is_empty());
    }
}
