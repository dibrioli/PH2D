#![forbid(unsafe_code)]
//! `ph2d-expr` — the shared per-element compute IR (ADR-0033), PH2D's analogue
//! of Houdini VEX / a Blender field.
//!
//! A scalar-`f32` expression over the *current element* of an attribute stream:
//! attribute reads, node parameters, arithmetic, comparisons, a small set of
//! built-in functions, and a `select` (ternary). It is the inline compute that
//! lives **inside** the main graph (Blender-style field), and the body of a
//! textual "code node" (the escape hatch, ADR-0033).
//!
//! One IR, multiple lowerings ("compile, not interpret" — ADR-0030/0033):
//! - [`eval`] — the native CPU reference evaluator (deterministic except for
//!   `f32` transcendentals, which differ across libm; fine for the presentation
//!   side, which is HR-5-exempt — gameplay must avoid them or use a fixed-point
//!   lowering).
//! - [`wgsl`] — emits a WGSL expression string (the GPU lowering).
//! - Luau lowering: lands with the gameplay domain (reuses `ph2d-script`).
//!
//! This crate is dependency-free and value-typed; a consuming node implements
//! [`eval::Bindings`] to feed attribute/param values from its `EvalCtx`.
//!
//! 🔒 **FROZEN at W2.T4 (ADR-0039, 2026-05-22)** alongside `ph2d-nodegraph`:
//! the per-element compute IR is part of the node-authoring contract, so its
//! surface (the `Expr` shape, `eval`/`eval_column`, the WGSL lowering + prelude,
//! the `Bindings` traits) is stable. Adding an op or `Func` variant is a
//! deliberate Coordenador-only event — CPU↔WGSL parity must be re-proven for
//! any new transcendental, and a node may already depend on the existing shape.

pub mod eval;
pub mod expr;
pub mod stream;
pub mod wgsl;

pub use eval::{Bindings, eval};
pub use expr::{BinOp, Expr, Func, UnaryOp};
pub use stream::{StreamBindings, eval_column};
pub use wgsl::{to_wgsl, wgsl_prelude};
