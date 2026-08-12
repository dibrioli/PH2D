//! The aiming law, exercised directly against the op's helpers — split from
//! `lib.rs` at the HR-18 LOC cap onto the seam this crate already uses
//! (`target_mode_tests.rs`). Still a CHILD module, so `use super::*` reaches
//! the private `atan2_approx` / `target_at` / `blend_aim` these gates measure.

use super::*;

/// Aim `input`'s `rot` at a target field `(tx, ty)` with `offset`, directly via
/// the op (no cook needed — a target field is just two scalar columns).
fn aim(input: Stream, tx: &[f32], ty: &[f32], offset: f32) -> Vec<f32> {
    let n = input.count();
    let p: Vec<[f32; 2]> = match input.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    };
    (0..n)
        .map(|i| {
            let dx = target_at(tx, i) - p[i][0];
            let dy = target_at(ty, i) - p[i][1];
            atan2_approx(dy, dx) * RAD_TO_DEG + offset
        })
        .collect()
}

/// The `atan2` approximation matches true `atan2` at the cardinals and a few
/// obliques, well within 0.003 rad — checked against KNOWN constants (no std
/// `atan2` call, so the test stays transcendental-free too).
#[test]
fn atan2_approx_matches_true_atan2() {
    let cases = [
        (0.0, 1.0, 0.0),                // +x → 0
        (1.0, 1.0, FRAC_PI_4),          // 45°
        (1.0, 0.0, FRAC_PI_2),          // +y → 90°
        (1.0, -1.0, 3.0 * FRAC_PI_4),   // 135°
        (0.0, -1.0, PI),                // -x → 180°
        (-1.0, -1.0, -3.0 * FRAC_PI_4), // -135°
        (-1.0, 0.0, -FRAC_PI_2),        // -90°
        (-1.0, 1.0, -FRAC_PI_4),        // -45°
        (1.0, 2.0, 0.4636476),          // atan(0.5)
        (2.0, 1.0, 1.1071488),          // atan(2)
    ];
    for (y, x, want) in cases {
        let got = atan2_approx(y, x);
        assert!(
            (got - want).abs() < 0.003,
            "atan2({y},{x}) = {got}, want {want}"
        );
    }
}

/// The origin (target on the element) is safe — no NaN, aim stays 0.
#[test]
fn a_coincident_target_is_zero_not_nan() {
    assert_eq!(atan2_approx(0.0, 0.0), 0.0);
}

/// Each element aims its `rot` (degrees) at the target. Two elements either
/// side of a target at the origin (unconnected) point in opposite ±x directions.
#[test]
fn each_element_aims_its_rotation_at_the_target() {
    let two = Stream::new(2).with("P", Column::Vec2(vec![[-1.0, 0.0], [1.0, 0.0]]));
    let rot = aim(two, &[], &[], 0.0); // empty target → origin
    assert!(rot[0].abs() < 0.2, "left (at -1) aims +x (0°): {}", rot[0]);
    assert!(
        (rot[1].abs() - 180.0).abs() < 0.2,
        "right (at +1) aims -x (180°): {}",
        rot[1]
    );
}

/// `offset` rotates the aim: +90 makes an element face across the target.
#[test]
fn offset_rotates_the_aim() {
    let one = Stream::new(1).with("P", Column::Vec2(vec![[-1.0, 0.0]])); // faces +x (0°)
    let rot = aim(one, &[], &[], 90.0);
    assert!(
        (rot[0] - 90.0).abs() < 0.2,
        "0° + offset 90 = 90°: {}",
        rot[0]
    );
}

/// An animated target (a value field) turns the aim: the same element faces
/// +x, then +y, as the target moves from the right to above it.
#[test]
fn a_moving_target_turns_the_aim() {
    let p = || Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
    let right = aim(p(), &[5.0], &[0.0], 0.0); // target to the right → 0°
    let up = aim(p(), &[0.0], &[5.0], 0.0); // target above → 90°
    assert!(right[0].abs() < 0.2, "target right → 0°: {}", right[0]);
    assert!((up[0] - 90.0).abs() < 0.2, "target up → 90°: {}", up[0]);
}

/// End to end through the cook: the op copies P through and writes `rot`.
#[test]
fn registers_and_writes_the_rotation_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.look_at.test.src"),
        name: "motion.look_at.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // One element at (-2, 0): aims +x (0°) at the unconnected origin.
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[-2.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionLookAt),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.look_at.test.src");
    let la = g.add_node("motion.look_at");
    g.connect(Edge {
        from: (src, 0),
        to: (la, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, la, 0.0).unwrap();
    let s = out[0].as_stream();
    match s.get("rot").unwrap() {
        Column::Scalar(v) => assert!(v[0].abs() < 0.2, "aims +x at the origin: {}", v[0]),
        _ => panic!("rot"),
    }
    assert!(s.get("P").is_some(), "P passes through");
}
