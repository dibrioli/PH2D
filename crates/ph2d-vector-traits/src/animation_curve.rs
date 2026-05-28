//! `AnimationCurveSampler` trait — sample a curve at time `t`.
//!
//! Distinct from [`crate::AttributeEvaluator`]: a curve is a *first-class*
//! geometric object (Bézier control points, easing presets, etc.) whose
//! shape can be edited in the Animation panel. An `AttributeEvaluator` may
//! internally hold one or more `AnimationCurveSampler` instances plus
//! pre/post-processing.

use crate::AnimValue;

/// Animation curve sampler.
///
/// `t` is `f64` to match [`crate::AttributeEvaluator::sample`] and preserve
/// precision over long sessions (per ADR-0056 §6 L1F1 Antigravity 3rd iter).
///
/// ## HR-3 (zero-alloc render path)
///
/// Implementations on the render hot path MUST NOT allocate. Mock impls in
/// [`crate::mocks`] satisfy this.
///
/// ## Thread-safety policy
///
/// `AnimationCurveSampler` does **not** require `Send + Sync` as a
/// supertrait — single-thread tool impls may carry non-thread-safe
/// state. Callers crossing thread boundaries pin the marker at use
/// site: `Box<dyn AnimationCurveSampler + Send + Sync>`. All mock
/// impls in [`crate::mocks`] satisfy `Send + Sync` (R4 audit Lens-L).
pub trait AnimationCurveSampler {
    /// Sample the curve at parameter `t`.
    fn at(&self, t: f64) -> AnimValue;
}
