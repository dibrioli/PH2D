#![forbid(unsafe_code)]
//! `ph2d-vector-llm` — LLM-as-graph-node authoring (Inovação #4,
//! [ADR-0061](../../../docs/architecture/decisions/0061-vector-llm-authoring.md)).
//!
//! The first vector tool where an LLM emits **editable strokes**, not an opaque
//! SVG dump: the model answers with LLM4SVG **semantic tokens** (a `spiral` with
//! `turns`, a `polygon` with `sides`…) that lower to a standard, editable
//! [`VectorNetwork`](ph2d_vector_doc::VectorNetwork).
//!
//! ## The deterministic pipeline lives here; the LLM call does not
//!
//! [`build_network_from_json`] is the pure, synchronous core —
//! **parse → sanitize → lower** — exactly what a graph node needs (`Effect::Pure`
//! over a cached response). The LLM *call* (async, network, the 15 s timeout of
//! ADR-0061 §2.6) is host/`ph2d-mcp` I/O and is **not** in this crate.
//!
//! ## Security is non-negotiable (ADR-0061 §2.4)
//!
//! LLM output is untrusted. Every input passes a **byte-size guard**
//! ([`MAX_INPUT_BYTES`], pre-parse) then the [`sanitizer`] (numeric bounds,
//! pre-allocation) before any geometry exists — `turns: 1e9` / a billion-vertex
//! path is rejected, never materialized. Destructive MCP tools are HR-11
//! [`governance`]-gated.
//!
//! ## Module map
//!
//! | Module             | Role                                                      |
//! |--------------------|-----------------------------------------------------------|
//! | [`semantic_tokens`]| LLM4SVG JSON → typed [`SemanticTokens`] (1st validation)  |
//! | [`sanitizer`]      | numeric bounds, pre-allocation (the L4F1 security core)    |
//! | [`to_network`]     | sanitized tokens → `VectorNetwork` (editable)             |
//! | [`governance`]     | HR-11 destructive-tool confirmation tokens                |
//! | [`cache`]          | result cache by `(prompt_hash, seed)` (+ timeout fallback) |

pub mod cache;
pub mod governance;
pub mod sanitizer;
pub mod semantic_tokens;
pub mod to_network;

use ph2d_vector_doc::VectorNetwork;

pub use cache::{CacheKey, ResultCache};
pub use governance::{Governance, GovernanceError, ToolKind, tool_kind};
pub use sanitizer::{MAX_COORD, SanitizerError, sanitize};
pub use semantic_tokens::{FillMode, ParseError, SemanticTokens, Shape, StyleTokens, parse};
pub use to_network::tokens_to_network;

/// Pre-parse byte cap on an LLM response (ADR-0061 §2.4 defense-in-depth). A
/// semantic-token blob is tiny; rejecting oversized input stops a giant payload
/// from OOM-ing `serde_json` *before* the per-field sanitizer can run.
pub const MAX_INPUT_BYTES: usize = 64 * 1024;

/// Anything that can go wrong turning an LLM response into geometry.
#[derive(Clone, Debug, PartialEq)]
pub enum LlmError {
    /// The response exceeds [`MAX_INPUT_BYTES`].
    InputTooLarge { bytes: usize, limit: usize },
    /// Structural parse failure (bad JSON / unknown shape / ill-typed param).
    Parse(ParseError),
    /// Numeric bounds violation (adversarial / out-of-range).
    Sanitize(SanitizerError),
}

impl core::fmt::Display for LlmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LlmError::InputTooLarge { bytes, limit } => {
                write!(f, "LLM response {bytes} bytes exceeds cap {limit}")
            }
            LlmError::Parse(e) => write!(f, "{e}"),
            LlmError::Sanitize(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<ParseError> for LlmError {
    fn from(e: ParseError) -> Self {
        LlmError::Parse(e)
    }
}

impl From<SanitizerError> for LlmError {
    fn from(e: SanitizerError) -> Self {
        LlmError::Sanitize(e)
    }
}

/// The deterministic core pipeline: an LLM4SVG JSON response → an **editable**
/// [`VectorNetwork`]. Size-guard → parse → sanitize → lower. Pure and total
/// (never panics); a graph node wraps this over a cached response.
pub fn build_network_from_json(json: &str) -> Result<VectorNetwork, LlmError> {
    let (_, network) = build_from_json(json)?;
    Ok(network)
}

/// As [`build_network_from_json`] but also returns the validated tokens (for the
/// [`cache`] + the audit log).
pub fn build_from_json(json: &str) -> Result<(SemanticTokens, VectorNetwork), LlmError> {
    if json.len() > MAX_INPUT_BYTES {
        return Err(LlmError::InputTooLarge {
            bytes: json.len(),
            limit: MAX_INPUT_BYTES,
        });
    }
    let tokens = semantic_tokens::parse(json)?;
    sanitizer::sanitize(&tokens)?;
    let network = to_network::tokens_to_network(&tokens);
    Ok((tokens, network))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_spiral_is_editable_network() {
        let json = r#"{
            "shape_type": "spiral",
            "params": { "turns": 6, "inner_radius": 0, "outer_radius": 120, "samples_per_turn": 24 },
            "style": { "stroke_width": 2, "fill": "none" }
        }"#;
        let net = build_network_from_json(json).unwrap();
        assert!(net.validate().is_ok());
        assert!(net.vertices.len() > 4, "a real spiral, editable downstream");
    }

    #[test]
    fn adversarial_response_is_rejected_not_materialized() {
        let json = r#"{ "shape_type": "spiral", "params": { "turns": 1000000000 } }"#;
        assert!(matches!(
            build_network_from_json(json),
            Err(LlmError::Sanitize(SanitizerError::ExceedsBound { .. }))
        ));
    }

    #[test]
    fn oversized_input_rejected_before_parse() {
        let big = format!(
            "{{ \"shape_type\": \"path\", \"params\": {{ \"note\": \"{}\" }} }}",
            "x".repeat(MAX_INPUT_BYTES)
        );
        assert!(matches!(
            build_network_from_json(&big),
            Err(LlmError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn garbage_never_panics() {
        for s in ["", "null", "[]", "{\"shape_type\":42}", "\u{0}"] {
            let _ = build_network_from_json(s); // total function — no panic
        }
    }
}
