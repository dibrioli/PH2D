//! Arch-gate: `ph2d-vector-traits` thread-safety invariant.
//!
//! Vector attribute evaluation, animation sampling and procedural-fill
//! shading run off the main thread (graph evaluation pool, GPU staging),
//! so the foundational value types and the mock evaluators that stand in
//! for them in tests must all be `Send + Sync`. A future field that
//! revokes that (an `Rc`, interior mutability without a lock, a raw
//! pointer) fails this build instead of surfacing as a runtime data race.
//!
//! Formalized from the W1 audit scratch (`_audit_send_sync.rs`) per the
//! gold-standard mandate: a real invariant becomes a named, documented
//! gate rather than an orphan file.

use ph2d_vector_traits::*;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn vector_value_and_mock_types_are_send_sync() {
    assert_send_sync::<AnimValue>();
    assert_send_sync::<WgslSource>();
    assert_send_sync::<MockAttributeEvaluator>();
    assert_send_sync::<MockAnimationCurveSampler>();
    assert_send_sync::<MockProceduralFillShader>();
}
