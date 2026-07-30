//! Wet Paint — the fluid-simulation paint engine (ADR-0134).
//!
//! Port of the reference engine in `docs/Painter/ph2d_wet_paint/js/engine/`,
//! module for module. The reference's `SPEC.md` is the behavioral source of
//! truth; `tests/acceptance.rs` is its §18 acceptance suite running in Rust.
//!
//! Port law (do not bend it in one module "for speed"):
//! - arithmetic in `f64`, storage in `f32` — what JS does with `Float32Array`
//!   (every store rounds to nearest, ties to even; Rust `as f32` matches);
//! - JS integer/rounding semantics only through [`jsmath`];
//! - transcendentals only through `libm` (cross-OS bit-identical);
//! - determinism: seeded splitmix32 + stateless integer hashes, nothing else.

#![forbid(unsafe_code)]
// The port law above is also why these lints are off CRATE-WIDE: the JS is the
// spec, and each of these rewrites changes the code's correspondence to it —
// counter loops carry the reference's exact indices; `min/max` chains keep JS
// `Math.min/max` NaN semantics (`clamp` panics on inverted bounds and orders
// NaN differently); signatures mirror the reference's; numeric literals are
// the reference's bytes (a "more precise" constant would CHANGE the output).
// The session fingerprint (`tests/fingerprint.rs`) is the gate that keeps all
// of this honest — silence here is not license to drift there.
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::approx_constant)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::should_implement_trait)]

pub mod brush;
pub mod colorops;
pub mod drying;
pub mod flow;
pub mod grid;
pub mod jsmath;
pub mod opacity;
pub mod painter;
pub mod paper;
pub mod par;
pub mod render;
pub mod rng;
pub mod sim;
pub mod solver;
pub mod stroke;
pub mod tools;
pub mod trail;
pub mod tuning;
