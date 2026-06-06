//! LLM4SVG **semantic tokens** (ADR-0061 §2.3): the LLM answers with structured
//! JSON describing an *editable* shape (a `spiral` with `turns`, a `polygon`
//! with `sides`…), **not** an opaque SVG dump. That is the differentiator — the
//! output is a standard `VectorNetwork` the user keeps editing downstream.
//!
//! This module is the **first validation layer**: typed parsing rejects unknown
//! shapes and ill-typed params before the [`crate::sanitizer`] enforces numeric
//! bounds. Parsing is *lenient on missing* params (sensible defaults) but
//! *strict on malformed* ones.

use glam::Vec2;
use serde::Deserialize;
use serde_json::{Map, Value};

/// A parsed (not yet sanitized) shape description.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Spiral {
        center: Vec2,
        inner_radius: f32,
        outer_radius: f32,
        turns: f32,
        samples_per_turn: u32,
        rotation: f32,
    },
    Polygon {
        center: Vec2,
        radius: f32,
        sides: u32,
        rotation: f32,
    },
    Star {
        center: Vec2,
        outer_radius: f32,
        inner_radius: f32,
        points: u32,
        rotation: f32,
    },
    Ellipse {
        center: Vec2,
        radii: Vec2,
    },
    Rect {
        corner_a: Vec2,
        corner_b: Vec2,
    },
    /// A free polyline of explicit vertices — the case the vertex cap guards.
    Path {
        vertices: Vec<Vec2>,
        closed: bool,
    },
}

/// How a shape is stroked / filled (applied downstream by the StyleTable;
/// parsed here so the LLM's intent round-trips).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct StyleTokens {
    #[serde(default = "default_stroke_width")]
    pub stroke_width: f32,
    /// `[L, C, H]` OKLCH.
    #[serde(default)]
    pub stroke_color_oklch: [f32; 3],
    #[serde(default)]
    pub fill: FillMode,
}

impl Default for StyleTokens {
    fn default() -> Self {
        Self {
            stroke_width: default_stroke_width(),
            stroke_color_oklch: [0.0, 0.0, 0.0],
            fill: FillMode::None,
        }
    }
}

fn default_stroke_width() -> f32 {
    1.0
}

/// Fill intent.
#[derive(Copy, Clone, Debug, PartialEq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FillMode {
    #[default]
    None,
    Solid,
}

/// A fully-parsed semantic token set: one shape + its style.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticTokens {
    pub shape: Shape,
    pub style: StyleTokens,
}

/// Why a token blob failed to parse (structurally) — distinct from a
/// [`crate::sanitizer::SanitizerError`] (numeric bounds).
#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    /// `serde_json` could not read the bytes as JSON.
    Json(String),
    /// `shape_type` is missing or not a recognized shape.
    UnknownShape(String),
    /// A parameter was present but of the wrong JSON type.
    BadParam {
        key: &'static str,
        reason: &'static str,
    },
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::Json(e) => write!(f, "invalid JSON: {e}"),
            ParseError::UnknownShape(s) => write!(f, "unknown shape_type: {s:?}"),
            ParseError::BadParam { key, reason } => write!(f, "param {key:?}: {reason}"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Deserialize)]
struct RawTokens {
    shape_type: String,
    #[serde(default)]
    params: Map<String, Value>,
    #[serde(default)]
    style: StyleTokens,
}

/// Parse an LLM4SVG JSON blob into typed [`SemanticTokens`]. Lenient on missing
/// params (defaults), strict on malformed ones and unknown shapes.
pub fn parse(json: &str) -> Result<SemanticTokens, ParseError> {
    let raw: RawTokens = serde_json::from_str(json).map_err(|e| ParseError::Json(e.to_string()))?;
    let p = &raw.params;
    let shape = match raw.shape_type.as_str() {
        "spiral" => Shape::Spiral {
            center: vec2(p, "center", Vec2::ZERO)?,
            inner_radius: num(p, "inner_radius", 0.0)?,
            outer_radius: num(p, "outer_radius", 100.0)?,
            turns: num(p, "turns", 8.0)?,
            samples_per_turn: count(p, "samples_per_turn", 32)?,
            rotation: num(p, "rotation", 0.0)?,
        },
        "polygon" => Shape::Polygon {
            center: vec2(p, "center", Vec2::ZERO)?,
            radius: num(p, "radius", 100.0)?,
            sides: count(p, "sides", 6)?,
            rotation: num(p, "rotation", 0.0)?,
        },
        "star" => Shape::Star {
            center: vec2(p, "center", Vec2::ZERO)?,
            outer_radius: num(p, "outer_radius", 100.0)?,
            inner_radius: num(p, "inner_radius", 50.0)?,
            points: count(p, "points", 5)?,
            rotation: num(p, "rotation", 0.0)?,
        },
        "ellipse" => Shape::Ellipse {
            center: vec2(p, "center", Vec2::ZERO)?,
            radii: vec2(p, "radii", Vec2::splat(100.0))?,
        },
        "rect" => Shape::Rect {
            corner_a: vec2(p, "corner_a", Vec2::splat(-50.0))?,
            corner_b: vec2(p, "corner_b", Vec2::splat(50.0))?,
        },
        "path" => Shape::Path {
            vertices: vec2_list(p, "vertices")?,
            closed: boolean(p, "closed", false)?,
        },
        other => return Err(ParseError::UnknownShape(other.to_owned())),
    };
    Ok(SemanticTokens {
        shape,
        style: raw.style,
    })
}

// ── typed param readers (strict-on-malformed) ──────────────────────────────

fn num(p: &Map<String, Value>, key: &'static str, default: f32) -> Result<f32, ParseError> {
    match p.get(key) {
        None => Ok(default),
        Some(Value::Number(n)) => n.as_f64().map(|v| v as f32).ok_or(ParseError::BadParam {
            key,
            reason: "not a finite number",
        }),
        Some(_) => Err(ParseError::BadParam {
            key,
            reason: "expected a number",
        }),
    }
}

fn count(p: &Map<String, Value>, key: &'static str, default: u32) -> Result<u32, ParseError> {
    match p.get(key) {
        None => Ok(default),
        Some(Value::Number(n)) => {
            // Accept ints; reject negatives/fractions/huge via u64 then clamp at
            // cast (the sanitizer enforces the real cap).
            let v = n.as_u64().ok_or(ParseError::BadParam {
                key,
                reason: "expected a non-negative integer",
            })?;
            Ok(v.min(u32::MAX as u64) as u32)
        }
        Some(_) => Err(ParseError::BadParam {
            key,
            reason: "expected an integer",
        }),
    }
}

fn boolean(p: &Map<String, Value>, key: &'static str, default: bool) -> Result<bool, ParseError> {
    match p.get(key) {
        None => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(ParseError::BadParam {
            key,
            reason: "expected a boolean",
        }),
    }
}

fn vec2(p: &Map<String, Value>, key: &'static str, default: Vec2) -> Result<Vec2, ParseError> {
    match p.get(key) {
        None => Ok(default),
        Some(Value::Array(a)) if a.len() == 2 => {
            let x = a[0].as_f64().ok_or(ParseError::BadParam {
                key,
                reason: "x is not a number",
            })?;
            let y = a[1].as_f64().ok_or(ParseError::BadParam {
                key,
                reason: "y is not a number",
            })?;
            Ok(Vec2::new(x as f32, y as f32))
        }
        Some(_) => Err(ParseError::BadParam {
            key,
            reason: "expected a [x, y] array",
        }),
    }
}

fn vec2_list(p: &Map<String, Value>, key: &'static str) -> Result<Vec<Vec2>, ParseError> {
    match p.get(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => a
            .iter()
            .map(|pt| match pt {
                Value::Array(c) if c.len() == 2 => {
                    let x = c[0].as_f64().ok_or(ParseError::BadParam {
                        key,
                        reason: "vertex x is not a number",
                    })?;
                    let y = c[1].as_f64().ok_or(ParseError::BadParam {
                        key,
                        reason: "vertex y is not a number",
                    })?;
                    Ok(Vec2::new(x as f32, y as f32))
                }
                _ => Err(ParseError::BadParam {
                    key,
                    reason: "each vertex must be [x, y]",
                }),
            })
            .collect(),
        Some(_) => Err(ParseError::BadParam {
            key,
            reason: "expected an array of [x, y] vertices",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spiral_from_adr_example() {
        let json = r#"{
            "shape_type": "spiral",
            "params": { "turns": 8, "scale_ratio": 1.618, "center": [0, 0], "initial_radius": 100 },
            "style": { "stroke_width": 2, "stroke_color_oklch": [0.5, 0.2, 30], "fill": "none" }
        }"#;
        let t = parse(json).unwrap();
        assert!(matches!(t.shape, Shape::Spiral { turns, .. } if turns == 8.0));
        assert_eq!(t.style.stroke_width, 2.0);
        assert_eq!(t.style.fill, FillMode::None);
    }

    #[test]
    fn unknown_shape_rejected() {
        let json = r#"{ "shape_type": "fractal_dragon", "params": {} }"#;
        assert!(matches!(parse(json), Err(ParseError::UnknownShape(_))));
    }

    #[test]
    fn malformed_param_rejected() {
        let json = r#"{ "shape_type": "polygon", "params": { "sides": "lots" } }"#;
        assert!(matches!(
            parse(json),
            Err(ParseError::BadParam { key: "sides", .. })
        ));
    }

    #[test]
    fn missing_params_use_defaults() {
        let t = parse(r#"{ "shape_type": "polygon", "params": {} }"#).unwrap();
        assert!(matches!(t.shape, Shape::Polygon { sides: 6, radius, .. } if radius == 100.0));
    }

    #[test]
    fn invalid_json_rejected() {
        assert!(matches!(parse("{ not json"), Err(ParseError::Json(_))));
    }

    #[test]
    fn path_vertices_parse() {
        let json = r#"{ "shape_type": "path", "params": { "vertices": [[0,0],[10,0],[5,8]], "closed": true } }"#;
        let t = parse(json).unwrap();
        assert!(
            matches!(t.shape, Shape::Path { ref vertices, closed: true } if vertices.len() == 3)
        );
    }
}
