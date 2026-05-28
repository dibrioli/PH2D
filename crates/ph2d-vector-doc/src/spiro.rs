//! Spiro / Hyperbezier Assist Modes — **stub** for W2+.
//!
//! Per [ADR-0056 §2.4](../../../../docs/architecture/decisions/0056-vector-network-data-model.md):
//! Spiro = Raph Levien's clothoid splines; Hyperbezier = elastica-
//! under-tension. Both are **opt-in Assist Modes** toggled via HUD
//! `S` / `H` — Bézier cúbico stays the default visible representation
//! (decisão D Antigravity 1st iteration).
//!
//! W1 ships the empty module + a marker so the
//! [`crate::RepresentationMode::SpiroAssist`] /
//! [`crate::RepresentationMode::HyperbezierAssist`] variants have a
//! home to land their state into when W2 wires the Pencil tool.

/// Marker placeholder — populated W2+ with `SpiroNetwork` / `HyperbezNetwork`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AssistModeStub;
