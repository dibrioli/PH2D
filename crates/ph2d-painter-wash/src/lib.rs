//! # ph2d-painter-wash — minimal watercolor core (ADR-0086)
//!
//! The "ultra-brutal simplification" of the v2 watercolor stack. The Curtis-1997
//! *minimal* core and nothing else:
//!
//! - **State:** `water: f32` + `pigment: vec4` (`(absorb.rgb, mass)`, RGB-linear
//!   Beer–Lambert) + static `paper: f32`. ~20 B/cell.
//! - **3 kernels:** `cs_splat` (brush input) → `cs_step` (the whole physics in one
//!   gather kernel: gated diffusion = bloom · FlowOutward `−λ∇w` = edge-darkening ·
//!   evaporation) → `cs_composite` (`backdrop · exp(−absorb)` subtractive glaze).
//! - **Stable + bounded by construction:** no autonomous water spread (no capillary
//!   wick) ⇒ nothing to bound; explicit diffusion `D ≤ 0.25` + upwind CFL ⇒
//!   unconditionally stable; conservative gather ⇒ pigment mass conserved;
//!   ping-pong ⇒ deterministic.
//!
//! Deposition is **implicit**: a drying cell closes its wet-gate, freezing pigment
//! where it sits; FlowOutward concentrates it at the receding boundary → the dark rim.
//!
//! `--features gpu` compiles the real solver; without it the crate is empty.

/// Optional spectral Kubelka–Munk pigment-mixing colour model (ADR-0086 §8.1). Pure Rust, always
/// compiled (the GPU composite mirrors it); the RGB Beer–Lambert path stays the default — K–M is a
/// per-brush CHOICE, not a replacement.
pub mod km;

#[cfg(feature = "gpu")]
mod composite;
#[cfg(feature = "gpu")]
mod solver;
#[cfg(feature = "gpu")]
pub use composite::WashCompositor;
#[cfg(feature = "gpu")]
pub use solver::{Dab, WashParams, WashSolver};

#[cfg(not(feature = "gpu"))]
/// No-op stub — the crate pulls zero dependencies without `--features gpu`.
pub mod stub {}
