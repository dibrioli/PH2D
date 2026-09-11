//! Smoke tests for mock impls — exercises `AttributeEvaluator`,
//! `ProceduralFillShader`, and `AnimationCurveSampler` against
//! [`ph2d_vector_traits::mocks`].

use glam::Vec2;
use ph2d_vector_traits::{
    AnimValue, AnimationCurveSampler, AttributeEvaluator, MockAnimationCurveSampler,
    MockAttributeEvaluator, MockProceduralFillShader, ProceduralFillShader,
};

#[test]
fn mock_attribute_evaluator_returns_constant_for_any_t() {
    let m = MockAttributeEvaluator::new(AnimValue::Float(3.0));
    assert_eq!(m.sample(0.0), AnimValue::Float(3.0));
    assert_eq!(m.sample(1.0), AnimValue::Float(3.0));
    assert_eq!(m.sample(100.5), AnimValue::Float(3.0));
    assert_eq!(m.sample(-42.0), AnimValue::Float(3.0));
}

#[test]
fn mock_procedural_fill_shader_default_compiles_magenta_wgsl() {
    let m = MockProceduralFillShader::default();
    let src = m.compile();
    assert!(src.as_str().contains("@fragment"));
    assert!(src.as_str().contains("vec4<f32>"));
}

#[test]
fn mock_animation_curve_sampler_lerps_vec2_endpoints() {
    let m = MockAnimationCurveSampler::new(
        AnimValue::Vec2(Vec2::ZERO),
        AnimValue::Vec2(Vec2::new(10.0, 20.0)),
    );
    assert_eq!(m.at(0.0), AnimValue::Vec2(Vec2::ZERO));
    assert_eq!(m.at(1.0), AnimValue::Vec2(Vec2::new(10.0, 20.0)));
    assert_eq!(m.at(0.5), AnimValue::Vec2(Vec2::new(5.0, 10.0)));
}

#[test]
fn mock_animation_curve_sampler_clamps_t_out_of_range() {
    let m = MockAnimationCurveSampler::new(AnimValue::Float(0.0), AnimValue::Float(10.0));
    assert_eq!(m.at(-0.5), AnimValue::Float(0.0));
    assert_eq!(m.at(1.5), AnimValue::Float(10.0));
}

/// Demonstrates that traits are object-safe — callers can hold
/// `Box<dyn AttributeEvaluator>` for runtime polymorphism (UI dropdowns,
/// node graph dispatch).
#[test]
fn attribute_evaluator_is_object_safe() {
    let evals: Vec<Box<dyn AttributeEvaluator>> = vec![
        Box::new(MockAttributeEvaluator::new(AnimValue::Float(1.0))),
        Box::new(MockAttributeEvaluator::new(AnimValue::Bool(true))),
    ];
    assert_eq!(evals[0].sample(0.0), AnimValue::Float(1.0));
    assert_eq!(evals[1].sample(0.0), AnimValue::Bool(true));
}
