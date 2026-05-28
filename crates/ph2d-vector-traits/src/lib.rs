#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! `ph2d-vector-traits` — foundational trait crate for the Vector Module
//! animation, attribute evaluation, procedural fill, and animation curve
//! contracts.
//!
//! Per [ADR-0056 §2.5](../../../docs/architecture/decisions/0056-vector-network-data-model.md)
//! and [ADR-0067](../../../docs/architecture/decisions/0067-brush-traits-decoupling.md)
//! parallel decoupling pattern.
//!
//! ## Mandate
//!
//! Minimal surface: 4 traits + 1 typed enum + Mock impls. Exists so W1-W5
//! fan-out crates can depend on stable contracts before real implementations
//! land (Shader Graph W6+, Animation System W10+).
//!
//! ## Why `AnimValue` typed enum
//!
//! Antigravity 2nd iteration L1F4 (CRITICAL): `sample(t: f32) -> f32` would
//! break W10+ retroactively when animating `Vec2` / `Color` / `Bool`.
//! `AnimValue` typed enum is canonical since W1 — small cost upfront, zero
//! retroactive rework when keyframe / state-machine / motion-driver systems
//! arrive.
//!
//! ## Why `t: f64`
//!
//! Antigravity 3rd iteration L1F1: preserves precision in sessions > 4 hours
//! at 120 Hz. `f32` loses sub-frame resolution past ~3 h. A `TimeContext`
//! typed wrapper is documented as future V2.0 path if sub-microsecond
//! precision becomes necessary.
//!
//! ## HR-3 (zero-alloc render hot path)
//!
//! All Mock impls are zero-allocation in their `sample` / `at` paths.
//! `compile()` may allocate (called off the render tick).

pub mod anim_value;
pub mod animation_curve;
pub mod attribute_evaluator;
pub mod mocks;
pub mod procedural_fill_shader;

pub use anim_value::{AnimValue, LinearInterp};
pub use animation_curve::AnimationCurveSampler;
pub use attribute_evaluator::AttributeEvaluator;
pub use mocks::{MockAnimationCurveSampler, MockAttributeEvaluator, MockProceduralFillShader};
pub use procedural_fill_shader::{ProceduralFillShader, WgslSource};
