//! Unit gates for `motion.color_ramp`.
//!
//! Split out of `lib.rs` when the falloff mask took the file past the 700-LOC cap.
//! ⚠️ Still a CHILD module (`#[path]` from `lib.rs`), not a sibling: these gates call
//! `colorize`, `mixed_tint` and `ramp_of`, which are private, and `use super::*` is
//! what reaches them. A `tests/` integration file would only see the public surface.
use super::*;
use ph2d_color::GradientPreset;

/// **A length-1 `t` field is a BROADCAST** — the value convention's `0/1/n`
/// ladder (`motion.look_at::target_at` is the canon). This node used to
/// take the `_` arm for it: element 0 got the value and every other element
/// got `t = 0`, so a `value.lfo` driving the ramp coloured exactly one
/// spark (found porting the `t` path to the GPU, ADR-0136).
#[test]
fn a_length_one_t_field_broadcasts_to_every_element() {
    let ramp = GradientPreset::Grayscale.ramp();
    let tinted = colorize(5, &ramp, &[0.75], &Stream::new(5));
    for (i, c) in tinted.iter().enumerate() {
        assert_eq!(
            c, &tinted[0],
            "element {i} must wear the SAME broadcast colour"
        );
    }
    // …and the broadcast value is the field's, not the positional key: the
    // grayscale ramp at t = 0.75 is 0.75 grey, not black.
    assert!(
        (tinted[0][0] - 0.75).abs() < 1e-6,
        "broadcast t = 0.75 on grayscale: got {:?}",
        tinted[0]
    );
}

/// **The gradient paints only where the field reaches** (doc 89 fam. 9, P0).
///
/// A `field.*` node writes the `falloff` column; before this the ramp REPLACED
/// `tint` unconditionally and a masked gradient was inexpressible — the only
/// blend in the library (`motion.mixer`) takes a global scalar, not a field.
///
/// The fixture carries three weights on purpose: `1` must land the ramp colour,
/// `0` must leave the existing colour untouched, and the middle one must land
/// strictly between them — a gate that only checked the endpoints would pass on
/// an implementation that treated the mask as a BOOLEAN.
#[test]
fn the_field_masks_the_gradient_instead_of_replacing_the_colour() {
    let ramp = GradientPreset::Grayscale.ramp();
    // Existing: opaque RED everywhere. Mask: full, half, none.
    let existing = vec![[1.0, 0.0, 0.0, 1.0]; 3];
    let input = Stream::new(3)
        .with("tint", Column::Vec4(existing.clone()))
        .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0]));
    // `t` unconnected → the positional key, so the grayscale ramp gives
    // black / mid / white across the three.
    let got = colorize(3, &ramp, &[], &input);
    let unmasked = colorize(3, &ramp, &[], &Stream::new(3));

    // f = 1: exactly the ramp colour, bit for bit (the endpoint-exact form).
    assert_eq!(
        got[0], unmasked[0],
        "at full falloff the ramp must land EXACTLY, not approximately"
    );
    // f = 0: exactly the colour that was already there.
    assert_eq!(
        got[2], existing[2],
        "at zero falloff the existing colour must survive EXACTLY"
    );
    // f = 0.5: strictly between — this is the half a boolean mask would miss.
    for k in 0..3 {
        let (lo, hi) = (
            existing[1][k].min(unmasked[1][k]),
            existing[1][k].max(unmasked[1][k]),
        );
        if hi - lo > 1e-6 {
            assert!(
                got[1][k] > lo + 1e-6 && got[1][k] < hi - 1e-6,
                "channel {k} at half falloff must be BETWEEN {lo} and {hi}, got {}",
                got[1][k]
            );
        }
    }
}

/// **A stream with no field is byte-identical to the day before the mask existed.**
///
/// The neutral is not a promise about intent, it is arithmetic: `falloff` absent
/// reads `1.0`, and `existing·(1−1) + target·1` is `existing·0 + target`, which
/// IEEE-754 makes exactly `target` for any finite channel. Asserted with `assert_eq!`
/// on the raw bits rather than an epsilon, because "approximately the same colour"
/// is precisely the claim that would let a regression through.
#[test]
fn a_stream_with_no_field_takes_the_ramp_colour_exactly() {
    let ramp = ramp_of(None);
    // A stream that carries an existing tint but NO falloff: the mask is absent,
    // so the ramp must overwrite it exactly — the substitution this node has
    // always performed.
    let with_tint = Stream::new(6).with("tint", Column::Vec4(vec![[0.3, 0.7, 0.2, 0.5]; 6]));
    for (i, (a, b)) in colorize(6, &ramp, &[], &with_tint)
        .iter()
        .zip(colorize(6, &ramp, &[], &Stream::new(6)))
        .enumerate()
    {
        assert_eq!(*a, b, "element {i} must be the bare ramp colour");
    }
}

/// Grayscale by normalised index: the first element is black, the last is white, and
/// the middle is mid-grey. FALSIFIED if the ramp were a single solid colour.
#[test]
fn grayscale_spreads_black_to_white_by_index() {
    let c = colorize(5, &GradientPreset::Grayscale.ramp(), &[], &Stream::new(5));
    assert!(c[0][0] < 0.05, "first is black: {:?}", c[0]);
    assert!(c[4][0] > 0.95, "last is white: {:?}", c[4]);
    assert!((c[2][0] - 0.5).abs() < 0.1, "middle is grey: {:?}", c[2]);
}

/// The `t` value field overrides the index: two elements both fed `t=1` are both the
/// ramp's end colour (white), regardless of their index.
#[test]
fn the_t_field_overrides_the_index() {
    let c = colorize(
        2,
        &GradientPreset::Grayscale.ramp(),
        &[1.0, 1.0],
        &Stream::new(2),
    );
    assert!(c[0][0] > 0.95 && c[1][0] > 0.95, "both white: {c:?}");
}

/// **The gradient string colours the set** (doc 85). A red→green→blue gradient laid
/// across the set colours the first element red, the middle green, the last blue.
/// FALSIFIED if the node ignored the string.
#[test]
fn a_gradient_string_colours_the_set() {
    let ramp = ramp_of(Some("g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1"));
    let c = colorize(3, &ramp, &[], &Stream::new(3));
    assert!(c[0][0] > 0.95 && c[0][1] < 0.05, "first red: {:?}", c[0]);
    assert!(c[1][1] > 0.95 && c[1][0] < 0.05, "middle green: {:?}", c[1]);
    assert!(c[2][2] > 0.95 && c[2][0] < 0.05, "last blue: {:?}", c[2]);
}

/// An unset / malformed string falls back to the default gradient (Rainbow) — never a
/// half-built gradient, never a crash. A fresh node is colourful.
#[test]
fn unset_falls_back_to_the_rainbow_default() {
    let none = colorize(7, &ramp_of(None), &[], &Stream::new(7));
    let bad = colorize(7, &ramp_of(Some("nonsense")), &[], &Stream::new(7));
    assert_eq!(none, bad, "None and malformed both use the default");
    // Rainbow: first stop is red.
    assert!(
        none[0][0] > 0.95 && none[0][2] < 0.05,
        "first red: {:?}",
        none[0]
    );
    // …and it spans hues (not a flat colour).
    let (lo, hi) = none.iter().fold((f32::MAX, f32::MIN), |(lo, hi), c| {
        (lo.min(c[2]), hi.max(c[2]))
    });
    assert!(hi - lo > 0.8, "the default rainbow spans (blue {lo}..{hi})");
}

/// **The GPU LUT fill mirrors the CPU `eval`** (doc 85, the device half). Baking the red
/// channel of a red→green→blue gradient gives red at t=0 and zero red at t=1 — the same
/// colour the CPU `colorize` paints. The malformed string bakes the default (Rainbow),
/// matching the CPU fallback, so the two paths agree on "nothing authored".
#[test]
fn the_lut_fill_samples_each_channel_and_falls_back() {
    let grad = "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1";
    let mut r = [0.0f32; 256];
    fill_grad_r(grad, &mut r);
    assert!(r[0] > 0.95, "red LUT starts at 1.0: {}", r[0]);
    assert!(r[255] < 0.05, "red LUT ends at 0.0: {}", r[255]);
    let ramp = parse_gradient(grad).unwrap();
    assert!((r[0] - ramp.eval(0.0)[0]).abs() < 1e-6, "LUT[0] == eval(0)");
    assert!(
        (r[255] - ramp.eval(1.0)[0]).abs() < 1e-6,
        "LUT[255] == eval(1)"
    );
    // Malformed → the default gradient (Rainbow): red at t=0 (the first stop is red).
    let mut bad = [9.0f32; 256];
    fill_grad_r("nonsense", &mut bad);
    assert!(
        bad[0] > 0.95,
        "fallback rainbow baked (red at 0): {}",
        bad[0]
    );
}

/// Deterministic + cooks through the registry: writes the `tint` column at the full
/// count and passes the geometry columns through. The ramp comes from the text param.
#[test]
fn registers_and_colours_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.color_ramp.test.src"),
        name: "motion.color_ramp.test.src",
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
            ctx.emit(
                Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionColorRamp),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.color_ramp.test.src");
    let cr = g.add_node("motion.color_ramp");
    // A grayscale gradient (black→white) so the index sweep is black to white.
    g.set_text_param(cr, RAMP_KEY, "g1 2 0:0,0,0 1:1,1,1".to_string());
    g.connect(Edge {
        from: (src, 0),
        to: (cr, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, cr, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(s.get("P").is_some(), "geometry passes through");
    match s.get("tint").unwrap() {
        Column::Vec4(v) => {
            assert_eq!(v.len(), 3, "tint at full count");
            assert!(v[0][0] < 0.05 && v[2][0] > 0.95, "black to white by index");
        }
        _ => panic!("tint"),
    }
}

/// The gradient cooks through the registry from a `set_text_param` — the end-to-end path
/// the panel drives. FALSIFIED if the cook ignored the text param.
#[test]
fn gradient_cooks_through_the_text_param() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.color_ramp.test.src2"),
        name: "motion.color_ramp.test.src2",
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
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionColorRamp),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();

    let mut g = Graph::new();
    let src = g.add_node("motion.color_ramp.test.src2");
    let cr = g.add_node("motion.color_ramp");
    g.set_text_param(cr, RAMP_KEY, "g1 2 0:1,0,0 1:0,0,1".to_string());
    g.connect(Edge {
        from: (src, 0),
        to: (cr, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, cr, 0.0).unwrap();
    match out[0].as_stream().get("tint").unwrap() {
        Column::Vec4(v) => {
            assert!(v[0][0] > 0.95 && v[0][2] < 0.05, "first red: {:?}", v[0]);
            assert!(v[1][2] > 0.95 && v[1][0] < 0.05, "last blue: {:?}", v[1]);
        }
        _ => panic!("tint"),
    }
}
