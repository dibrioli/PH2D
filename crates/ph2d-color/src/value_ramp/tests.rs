use super::*;

#[test]
fn default_is_identity_remap() {
    // black@0 → white@1, Linear ⇒ eval(t) == t.
    let r = ValueRamp::default();
    for &t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!((r.eval(t) - t).abs() < 1e-6, "eval({t}) = {}", r.eval(t));
    }
}

#[test]
fn bake_matches_eval() {
    let r = ValueRamp::default();
    let mut lut = [0.0f32; 256];
    r.bake_into(&mut lut);
    assert!((lut[0] - 0.0).abs() < 1e-6);
    assert!((lut[255] - 1.0).abs() < 1e-6);
    assert!((lut[128] - r.eval(128.0 / 255.0)).abs() < 1e-6);
}

#[test]
fn invert_flips_positions_and_eval() {
    // A 0→1 ramp inverted becomes 1→0, so eval(t) == 1 - t.
    let mut r = ValueRamp::default();
    r.invert();
    for &t in &[0.0, 0.3, 0.5, 1.0] {
        assert!(
            (r.eval(t) - (1.0 - t)).abs() < 1e-6,
            "eval({t}) = {}",
            r.eval(t)
        );
    }
    // Endpoints (their stable ids) survive the flip.
    assert_eq!(r.len(), 2);
}

#[test]
fn add_move_remove_keep_sorted_and_stable_ids() {
    let mut r = ValueRamp::new(
        vec![ValueStop::new(0.0, 0.0), ValueStop::new(1.0, 1.0)],
        RampInterp::Linear,
    );
    let idx = r.add_stop(ValueStop::new(0.5, 0.5));
    assert_eq!(r.len(), 3);
    let id = r.stops()[idx].id;
    // Drag the middle stop past the right end; it keeps its stable id.
    let new_idx = r.set_position(idx, 1.0);
    assert_eq!(r.stops()[r.index_of_id(id).unwrap()].id, id);
    r.remove_stop(new_idx);
    assert_eq!(r.len(), 2);
    // Never empties.
    r.remove_stop(0);
    r.remove_stop(0);
    assert_eq!(r.len(), 1, "a ramp always keeps ≥1 stop");
}

#[test]
fn constant_interp_holds_left_value() {
    let r = ValueRamp::new(
        vec![ValueStop::new(0.0, 0.2), ValueStop::new(1.0, 0.9)],
        RampInterp::Constant,
    );
    assert!(
        (r.eval(0.99) - 0.2).abs() < 1e-6,
        "holds left until the next stop"
    );
}
