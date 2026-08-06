//! **The three places a look-at can get its target** (Wave B, the Enio's report:
//! the node shipped *"sem alvo por nome/mouse"*).
//!
//! The load-bearing gate is not "Object mode works" — it is that `Point` is
//! BYTE-IDENTICAL to the node before this wave. Every document already in a file
//! carries `mode` absent ⇒ the manifest default ⇒ `Point`, so a single moved
//! degree here is a silent edit of art nobody touched.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};

/// Three elements on a row, so an aim is a direction and not a coin flip.
fn field() -> Stream {
    Stream::new(3).with("P", Column::Vec2(vec![[-1.0, 0.0], [0.0, 0.0], [1.0, 0.0]]))
}

/// Cook `in -> look_at` with the given mode, and return the `rot` column.
fn aim(reg: &NodeRegistry, mode: f32, target: Option<&str>, ext: &[(&str, [f32; 2])]) -> Vec<f32> {
    let mut g = Graph::new();
    let src = g.add_node("debug.const_field");
    let la = g.add_node("motion.look_at");
    g.connect(Edge {
        from: (src, 0),
        to: (la, 0),
        delayed: false,
    })
    .expect("edge");
    g.set_param(la, "mode", mode);
    if let Some(t) = target {
        g.set_text_param(la, "target", t.to_string());
    }
    let mut cook = Cook::new();
    for (name, p) in ext {
        cook.set_external(
            (*name).to_string(),
            Stream::new(1).with("P", Column::Vec2(vec![*p])),
        );
    }
    let set = cook.cook(&g, reg, la, 0.0).expect("cooks");
    match set
        .iter()
        .next()
        .expect("one stream")
        .as_stream()
        .get("rot")
    {
        Some(Column::Scalar(v)) => v.clone(),
        other => panic!("no rot column: {other:?}"),
    }
}

/// A registry with the node under test plus a constant three-element source.
fn reg() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("debug.const_field"),
        name: "debug.const_field",
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
            ctx.emit(field());
        }
    }
    reg.register(Box::new(Src)).expect("src");
    register(&mut reg).expect("look_at");
    reg
}

/// **The control, and the one that matters most.** `Point` is the default, so it is
/// what every saved document reads. It must be the pre-wave node to the last bit:
/// the target inputs are unconnected ⇒ the origin ⇒ the outer two aim at each other
/// and the middle one, sitting ON the target, holds 0 rather than producing a NaN.
#[test]
fn point_mode_is_the_node_before_this_wave() {
    let r = reg();
    let rot = aim(&r, 0.0, None, &[]);
    assert_eq!(rot.len(), 3);
    assert!(
        (rot[0] - 0.0).abs() < 0.1,
        "left aims +x at the origin: {rot:?}"
    );
    assert!((rot[1] - 0.0).abs() < 0.1, "coincident holds 0: {rot:?}");
    assert!(
        (rot[2].abs() - 180.0).abs() < 0.1,
        "right aims back at the origin: {rot:?}"
    );
}

/// **A named object is a target.** The node reads the centroid of the external the
/// app published under the name in its `target` text param — the same channel and
/// the same picker `motion.path` walks, so nothing new had to be invented.
#[test]
fn object_mode_aims_at_the_named_external() {
    let r = reg();
    // A target ABOVE the row: all three must aim upward (+y ⇒ near +90°), which the
    // origin target of `Point` mode could never produce for the outer two.
    let rot = aim(&r, 1.0, Some("Sun"), &[("Sun", [0.0, 10.0])]);
    for (i, a) in rot.iter().enumerate() {
        assert!(
            *a > 45.0 && *a < 135.0,
            "element {i} must aim up at the Sun, got {a} ({rot:?})"
        );
    }
}

/// **The cursor is a target**, published by the editor under the reserved name. The
/// node never learns what a window or a camera is — it reads a point out of the same
/// table.
#[test]
fn cursor_mode_aims_at_the_reserved_external() {
    let r = reg();
    let rot = aim(
        &r,
        2.0,
        None,
        &[(ph2d_nodegraph::external::CURSOR, [0.0, -10.0])],
    );
    for (i, a) in rot.iter().enumerate() {
        assert!(
            *a < -45.0 && *a > -135.0,
            "element {i} must aim down at the cursor, got {a} ({rot:?})"
        );
    }
}

/// **An unresolvable name falls back to the value inputs, it does not aim at the
/// origin.** Aiming at `(0,0)` would be a deliberate-looking choice the artist never
/// made — and it is indistinguishable from a target that resolved to the centre, so
/// the artist could not tell a typo from a working graph.
#[test]
fn a_target_nobody_published_falls_back_instead_of_aiming_at_the_origin() {
    let r = reg();
    let missing = aim(&r, 1.0, Some("NotHere"), &[]);
    let point = aim(&r, 0.0, None, &[]);
    assert_eq!(
        missing, point,
        "an unpublished name must read exactly as Point mode"
    );
    // An EMPTY name is the freshly-switched state (the picker has not been used yet)
    // and must behave the same rather than resolving to some external named "".
    assert_eq!(aim(&r, 1.0, Some("  "), &[]), point);
}

/// The mode index is read defensively: a value outside the three falls back to
/// `Point`, because a visible no-op beats aiming somewhere nobody asked for.
#[test]
fn an_out_of_range_mode_is_point() {
    assert_eq!(TargetMode::of(-1.0), TargetMode::Point);
    assert_eq!(TargetMode::of(7.0), TargetMode::Point);
    assert_eq!(TargetMode::of(0.4), TargetMode::Point);
    assert_eq!(TargetMode::of(1.0), TargetMode::Object);
    assert_eq!(TargetMode::of(2.0), TargetMode::Cursor);
}

/// **The GPU kernel covers only the mode it can express.** The device does not see
/// the external table, so a kernel that ran in Object/Cursor mode would aim at the
/// unconnected value ports — the origin — while the CPU aimed at the object: two
/// producers disagreeing, and the one nobody reads a number from wins on screen.
#[test]
fn the_kernel_recuses_from_the_modes_it_cannot_see() {
    let applicable = GPU_KERNEL.applicable.expect("declared");
    let p = |m: f32| move |name: &str| if name == "mode" { m } else { 0.0 };
    assert!(
        applicable(&p(0.0)),
        "Point is the mode the kernel expresses"
    );
    assert!(!applicable(&p(1.0)), "Object resolves outside the device");
    assert!(!applicable(&p(2.0)), "Cursor resolves outside the device");
}
