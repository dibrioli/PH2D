//! `AttributeEvaluator` trait — sample an animated attribute at time `t`.
//!
//! Distinct from [`crate::AnimationCurveSampler`]: an `AttributeEvaluator`
//! is the higher-level abstraction "thing that resolves a parameter at
//! time `t`". Real implementations may internally hold an
//! `AnimationCurveSampler` plus pre/post-processing (e.g. a Luau script
//! that calls into a Bézier curve).

use crate::AnimValue;

/// Animated attribute sampler.
///
/// Typical implementations:
/// - **Constant**: `sample(_) -> fixed`
/// - **Keyframe**: piecewise interpolation over Bézier handles
/// - **Procedural**: noise / wave / spring driving the value
/// - **Expression**: Luau script evaluating user formula
///
/// `t` is `f64` to preserve precision over multi-hour sessions
/// (per [ADR-0056 §6](../../../../docs/architecture/decisions/0056-vector-network-data-model.md)
/// L1F1 Antigravity 3rd iteration).
///
/// ## HR-3 (zero-alloc render path)
///
/// Implementations called on the render hot path MUST NOT allocate.
/// Mock impls in [`crate::mocks`] satisfy this.
///
/// ## Thread-safety policy
///
/// `AttributeEvaluator` deliberately does **not** require `Send + Sync`
/// as a supertrait. Single-thread tools (Pen / Pencil / Direct Select)
/// can hold non-thread-safe state (e.g. RefCell caches) in their impls.
/// Callers that need to cross thread boundaries pin the marker bounds
/// at use site:
///
/// ```ignore
/// let eval: Box<dyn AttributeEvaluator + Send + Sync> = ...;
/// ```
///
/// All mock impls in [`crate::mocks`] satisfy `Send + Sync` (verified
/// in `ph2d-vector-doc/tests/_audit_send_sync.rs` — R4 audit Lens-L).
pub trait AttributeEvaluator {
    /// Sample the attribute at time `t`.
    fn sample(&self, t: f64) -> AnimValue;
}
