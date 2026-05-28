//! Mock implementations of every trait.
//!
//! Used by W1-W5 fan-out crates BEFORE real implementations exist
//! (Shader Graph W6+ per ADR-0060; Animation System W10+ per the
//! Vector Module plan §10 Animation).
//!
//! ## HR-3 (zero-alloc render path)
//!
//! Mock `sample` / `at` paths are allocation-free (return owned `AnimValue`
//! by value). `compile` clones a pre-built `WgslSource` (allowed — called
//! only on shader DAG topology change, not per-frame).

use crate::{
    AnimValue, AnimationCurveSampler, AttributeEvaluator, LinearInterp, ProceduralFillShader,
    WgslSource,
};

/// Constant-value [`AttributeEvaluator`] — returns the same `AnimValue`
/// regardless of `t`.
#[derive(Debug, Clone, Copy)]
pub struct MockAttributeEvaluator {
    /// Constant value returned by every call to `sample`.
    pub value: AnimValue,
}

impl MockAttributeEvaluator {
    /// Construct a constant-value evaluator.
    #[must_use]
    pub const fn new(value: AnimValue) -> Self {
        Self { value }
    }
}

impl AttributeEvaluator for MockAttributeEvaluator {
    fn sample(&self, _t: f64) -> AnimValue {
        self.value
    }
}

/// Stub [`ProceduralFillShader`] returning a fixed magenta-fill WGSL string.
///
/// Magenta (1, 0, 1, 1) is a common "missing texture" sentinel — visible
/// breakage if a real shader fails to swap in.
#[derive(Debug, Clone)]
pub struct MockProceduralFillShader {
    src: WgslSource,
}

impl Default for MockProceduralFillShader {
    fn default() -> Self {
        Self {
            src: WgslSource::new(
                "@fragment fn fs() -> @location(0) vec4<f32> { \
                 return vec4<f32>(1.0, 0.0, 1.0, 1.0); }",
            ),
        }
    }
}

impl MockProceduralFillShader {
    /// Construct with a custom WGSL source (useful in tests).
    #[must_use]
    pub fn with_source(src: WgslSource) -> Self {
        Self { src }
    }
}

impl ProceduralFillShader for MockProceduralFillShader {
    fn compile(&self) -> WgslSource {
        self.src.clone()
    }
}

/// Linear-blend [`AnimationCurveSampler`].
///
/// Holds two `AnimValue` endpoints and interpolates linearly via
/// [`LinearInterp::lerp`]. `t` is clamped to `[0.0, 1.0]` to make
/// out-of-range samples behaviorally safe.
#[derive(Debug, Clone, Copy)]
pub struct MockAnimationCurveSampler {
    /// Endpoint at `t = 0.0`.
    pub from: AnimValue,
    /// Endpoint at `t = 1.0`.
    pub to: AnimValue,
}

impl MockAnimationCurveSampler {
    /// Construct a linear-blend curve between two endpoints.
    #[must_use]
    pub const fn new(from: AnimValue, to: AnimValue) -> Self {
        Self { from, to }
    }
}

impl AnimationCurveSampler for MockAnimationCurveSampler {
    fn at(&self, t: f64) -> AnimValue {
        AnimValue::lerp(self.from, self.to, t.clamp(0.0, 1.0))
    }
}
