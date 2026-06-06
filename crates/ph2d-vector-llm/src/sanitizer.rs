//! Token-injection **sanitizer** (ADR-0061 §2.4, Antigravity L4F1 — CRITICAL).
//!
//! LLM output is **untrusted** (prompt injection or an adversarial response).
//! Every numeric param is bounds-checked *before* any geometry is allocated, so
//! `turns: 1_000_000` or `vertices: [10⁹ entries]` is **rejected**, never
//! materialized. The caps are the frozen [`ph2d_vector_doc`] values
//! (`MAX_SPIRAL_TURNS`, `MAX_POLYGON_SIDES`, `MAX_VERTICES_PER_LLM_GEN`) plus a
//! coordinate magnitude bound. Fuzzed daily (`vector_fuzz_llm_semantic_tokens`):
//! the contract is **no panic / no OOM on any input**, always a `Result`.

use glam::Vec2;
use ph2d_vector_doc::postcard_schema::{
    MAX_POLYGON_SIDES, MAX_SPIRAL_TURNS, MAX_VERTICES_PER_LLM_GEN,
};

use crate::semantic_tokens::{SemanticTokens, Shape};

/// Coordinate magnitude cap (ADR-0061 §2.4 / §2.8). Beyond this, reject — huge
/// coords are an adversarial / exponential-blowup signal, not a real shape.
pub const MAX_COORD: f32 = 1.0e6;

/// Why an LLM token set was rejected before allocation.
#[derive(Clone, Debug, PartialEq)]
pub enum SanitizerError {
    /// An integer param exceeds its adversarial cap.
    ExceedsBound {
        param: &'static str,
        limit: u32,
        got: u32,
    },
    /// A coordinate is non-finite or beyond [`MAX_COORD`].
    CoordOutOfBounds { param: &'static str, x: f32, y: f32 },
    /// A scalar param is NaN/∞.
    NonFinite { param: &'static str },
    /// The shape would allocate more than [`MAX_VERTICES_PER_LLM_GEN`] vertices.
    TooManyVertices { estimated: u64, limit: usize },
}

impl core::fmt::Display for SanitizerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SanitizerError::ExceedsBound { param, limit, got } => {
                write!(f, "param {param:?} = {got} exceeds cap {limit}")
            }
            SanitizerError::CoordOutOfBounds { param, x, y } => {
                write!(f, "coord {param:?} = ({x}, {y}) out of bounds (±{MAX_COORD})")
            }
            SanitizerError::NonFinite { param } => write!(f, "param {param:?} is not finite"),
            SanitizerError::TooManyVertices { estimated, limit } => {
                write!(f, "estimated {estimated} vertices exceeds cap {limit}")
            }
        }
    }
}

impl std::error::Error for SanitizerError {}

/// Bounds-check `tokens`. `Ok(())` means safe to materialize; `Err` means the
/// LLM gave out-of-bounds params (the caller surfaces it / re-prompts).
pub fn sanitize(tokens: &SemanticTokens) -> Result<(), SanitizerError> {
    match &tokens.shape {
        Shape::Spiral {
            center,
            inner_radius,
            outer_radius,
            turns,
            samples_per_turn,
            rotation,
        } => {
            finite("turns", *turns)?;
            finite("rotation", *rotation)?;
            scalar_in_range("inner_radius", *inner_radius)?;
            scalar_in_range("outer_radius", *outer_radius)?;
            coord("center", *center)?;
            if *turns > MAX_SPIRAL_TURNS as f32 {
                return Err(SanitizerError::ExceedsBound {
                    param: "turns",
                    limit: MAX_SPIRAL_TURNS,
                    got: turns.ceil().max(0.0).min(u32::MAX as f32) as u32,
                });
            }
            // est = turns·samples_per_turn + 1 (f64 — no overflow on huge spt).
            let est = (*turns as f64 * f64::from(*samples_per_turn)).ceil().max(0.0) as u64 + 1;
            cap_vertices(est)?;
        }
        Shape::Polygon {
            center,
            radius,
            sides,
            rotation,
        } => {
            scalar_in_range("radius", *radius)?;
            finite("rotation", *rotation)?;
            coord("center", *center)?;
            bound("sides", *sides, MAX_POLYGON_SIDES)?;
            cap_vertices(u64::from(*sides))?;
        }
        Shape::Star {
            center,
            outer_radius,
            inner_radius,
            points,
            rotation,
        } => {
            scalar_in_range("outer_radius", *outer_radius)?;
            scalar_in_range("inner_radius", *inner_radius)?;
            finite("rotation", *rotation)?;
            coord("center", *center)?;
            bound("points", *points, MAX_POLYGON_SIDES / 2)?;
            cap_vertices(u64::from(*points) * 2)?;
        }
        Shape::Ellipse { center, radii } => {
            coord("center", *center)?;
            coord("radii", *radii)?;
            cap_vertices(4)?;
        }
        Shape::Rect { corner_a, corner_b } => {
            coord("corner_a", *corner_a)?;
            coord("corner_b", *corner_b)?;
            cap_vertices(4)?;
        }
        Shape::Path { vertices, .. } => {
            // Check the count BEFORE iterating coords (the count is the OOM lever).
            cap_vertices(vertices.len() as u64)?;
            for v in vertices {
                coord("vertex", *v)?;
            }
        }
    }
    Ok(())
}

fn finite(param: &'static str, v: f32) -> Result<(), SanitizerError> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(SanitizerError::NonFinite { param })
    }
}

/// A scalar that is also within the coordinate magnitude (radii / lengths).
fn scalar_in_range(param: &'static str, v: f32) -> Result<(), SanitizerError> {
    finite(param, v)?;
    if v.abs() > MAX_COORD {
        return Err(SanitizerError::CoordOutOfBounds { param, x: v, y: 0.0 });
    }
    Ok(())
}

fn coord(param: &'static str, v: Vec2) -> Result<(), SanitizerError> {
    if !v.x.is_finite() || !v.y.is_finite() || v.x.abs() > MAX_COORD || v.y.abs() > MAX_COORD {
        return Err(SanitizerError::CoordOutOfBounds {
            param,
            x: v.x,
            y: v.y,
        });
    }
    Ok(())
}

fn bound(param: &'static str, got: u32, limit: u32) -> Result<(), SanitizerError> {
    if got > limit {
        Err(SanitizerError::ExceedsBound { param, limit, got })
    } else {
        Ok(())
    }
}

fn cap_vertices(estimated: u64) -> Result<(), SanitizerError> {
    if estimated > MAX_VERTICES_PER_LLM_GEN as u64 {
        Err(SanitizerError::TooManyVertices {
            estimated,
            limit: MAX_VERTICES_PER_LLM_GEN,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_tokens::{parse, StyleTokens};

    fn toks(shape: Shape) -> SemanticTokens {
        SemanticTokens {
            shape,
            style: StyleTokens::default(),
        }
    }

    #[test]
    fn benign_shapes_pass() {
        assert!(sanitize(&toks(Shape::Polygon {
            center: Vec2::ZERO,
            radius: 100.0,
            sides: 6,
            rotation: 0.0,
        }))
        .is_ok());
        assert!(sanitize(&toks(Shape::Spiral {
            center: Vec2::ZERO,
            inner_radius: 0.0,
            outer_radius: 100.0,
            turns: 8.0,
            samples_per_turn: 32,
            rotation: 0.0,
        }))
        .is_ok());
    }

    #[test]
    fn adversarial_turns_rejected() {
        let e = sanitize(&toks(Shape::Spiral {
            center: Vec2::ZERO,
            inner_radius: 0.0,
            outer_radius: 100.0,
            turns: 1_000_000.0,
            samples_per_turn: 32,
            rotation: 0.0,
        }))
        .unwrap_err();
        assert!(matches!(e, SanitizerError::ExceedsBound { param: "turns", .. }));
    }

    #[test]
    fn adversarial_huge_samples_rejected_by_vertex_cap() {
        // turns within cap but samples_per_turn explodes the vertex count.
        let e = sanitize(&toks(Shape::Spiral {
            center: Vec2::ZERO,
            inner_radius: 0.0,
            outer_radius: 100.0,
            turns: 64.0,
            samples_per_turn: u32::MAX,
            rotation: 0.0,
        }))
        .unwrap_err();
        assert!(matches!(e, SanitizerError::TooManyVertices { .. }));
    }

    #[test]
    fn nan_and_inf_coords_rejected() {
        assert!(matches!(
            sanitize(&toks(Shape::Ellipse {
                center: Vec2::new(f32::NAN, 0.0),
                radii: Vec2::splat(10.0),
            })),
            Err(SanitizerError::CoordOutOfBounds { .. })
        ));
        assert!(matches!(
            sanitize(&toks(Shape::Spiral {
                center: Vec2::ZERO,
                inner_radius: f32::INFINITY,
                outer_radius: 100.0,
                turns: 4.0,
                samples_per_turn: 16,
                rotation: 0.0,
            })),
            Err(SanitizerError::NonFinite { .. } | SanitizerError::CoordOutOfBounds { .. })
        ));
    }

    #[test]
    fn coord_blowup_rejected() {
        let e = sanitize(&toks(Shape::Rect {
            corner_a: Vec2::ZERO,
            corner_b: Vec2::splat(1.0e9),
        }))
        .unwrap_err();
        assert!(matches!(e, SanitizerError::CoordOutOfBounds { .. }));
    }

    #[test]
    fn too_many_path_vertices_rejected() {
        let verts = vec![Vec2::ZERO; MAX_VERTICES_PER_LLM_GEN + 1];
        let e = sanitize(&toks(Shape::Path {
            vertices: verts,
            closed: false,
        }))
        .unwrap_err();
        assert!(matches!(e, SanitizerError::TooManyVertices { .. }));
    }

    #[test]
    fn polygon_sides_over_cap_rejected() {
        let e = sanitize(&toks(Shape::Polygon {
            center: Vec2::ZERO,
            radius: 50.0,
            sides: MAX_POLYGON_SIDES + 1,
            rotation: 0.0,
        }))
        .unwrap_err();
        assert!(matches!(e, SanitizerError::ExceedsBound { param: "sides", .. }));
    }

    /// Fuzz-shaped property: parse+sanitize never panics, for any input.
    #[test]
    fn parse_then_sanitize_never_panics_on_adversarial_json() {
        let cases = [
            r#"{"shape_type":"spiral","params":{"turns":1e30}}"#,
            r#"{"shape_type":"polygon","params":{"sides":4294967295}}"#,
            r#"{"shape_type":"path","params":{"vertices":[[1e40,-1e40]]}}"#,
            r#"{"shape_type":"star","params":{"points":99999}}"#,
            r#"{"shape_type":"rect","params":{"corner_a":[null,0]}}"#,
            r#"{}"#,
            r#"garbage"#,
            r#"{"shape_type":"spiral","params":{"turns":"NaN"}}"#,
        ];
        for c in cases {
            // Either a parse error or (parsed → sanitize result); never a panic.
            if let Ok(t) = parse(c) {
                let _ = sanitize(&t);
            }
        }
    }
}
