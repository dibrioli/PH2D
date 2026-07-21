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

pub mod brush;
pub mod colorops;
pub mod drying;
pub mod grid;
pub mod jsmath;
pub mod opacity;
pub mod painter;
pub mod paper;
pub mod render;
pub mod rng;
pub mod sim;
pub mod solver;
pub mod stroke;
pub mod tools;
pub mod trail;
pub mod tuning;
