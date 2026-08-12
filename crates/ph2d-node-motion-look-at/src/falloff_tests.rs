//! **THE FAMILY'S WEIGHT REACHES THE AIM** (doc 89 folha 08 — the P0 and the P1
//! that ship together).
//!
//! Every other modifier in this module scales its effect by the multiplicative
//! `falloff` column times a `strength` param; `motion.look_at` did not, and the
//! sheet measured it by grep: **zero** occurrences of `falloff` in the crate while
//! `move`/`rotate`/`scale`/`noise`/`oscillator`/`wiggle`/`stagger`/`drive`/`tint`/
//! `spring`/`step`/`orbit`/`twist`/`bend` all honour it.
//!
//! These gates drive the op through the REAL cook (a local resolver + a source op),
//! never through a re-implementation of the aiming loop — the sibling `aim_tests`
//! already owns that mirror, and a mirror cannot see a law that lives in `eval`.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A source that emits exactly the columns a weight law reads: where each element
/// is, where it already points, and how much of the effect it should receive.
struct Src;

static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.look_at.falloff.src"),
    name: "motion.look_at.falloff.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // `rot` and `falloff` are seeded from params so one fixture covers every case.
    params: &[
        ParamSpec {
            name: "rot",
            default: 0.0,
        },
        ParamSpec {
            name: "falloff",
            default: 1.0,
        },
        ParamSpec {
            name: "has_falloff",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // One element at the origin: the aim is then purely the target's angle,
        // so every number below is readable by hand.
        let mut s = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("rot", Column::Scalar(vec![ctx.param("rot")]));
        if ctx.param("has_falloff") != 0.0 {
            s.set("falloff", Column::Scalar(vec![ctx.param("falloff")]));
        }
        ctx.emit(s);
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

/// Aim one element (at the origin, already pointing at `rot`) at `(tx, ty)` with
/// the given weight, **through the cook**. `falloff = None` means the column is
/// absent, which is the shape almost every stream in the wild has.
fn aimed(rot: f32, falloff: Option<f32>, strength: f32, tx: f32, ty: f32) -> f32 {
    aimed_off(rot, falloff, strength, tx, ty, 0.0)
}

/// The same, with the `offset` the artist typed — the knob that can push the aim
/// PAST +-180, which is the only place the two verbatim arms are observable.
fn aimed_off(rot: f32, falloff: Option<f32>, strength: f32, tx: f32, ty: f32, offset: f32) -> f32 {
    let mut g = Graph::new();
    let src = g.add_node("motion.look_at.falloff.src");
    g.set_param(src, "rot", rot);
    g.set_param(src, "has_falloff", f32::from(u8::from(falloff.is_some())));
    if let Some(f) = falloff {
        g.set_param(src, "falloff", f);
    }
    let la = g.add_node("motion.look_at");
    g.set_param(la, "strength", strength);
    g.set_param(la, "target_x", tx);
    g.set_param(la, "target_y", ty);
    g.set_param(la, "offset", offset);
    g.connect(Edge {
        from: (src, 0),
        to: (la, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, la, 0.0).expect("cook");
    match out[0].as_stream().get("rot") {
        Some(Column::Scalar(v)) => v[0],
        _ => panic!("the node always writes `rot`"),
    }
}

/// **The default is the node before this wave, to the BIT.**
///
/// No `falloff` column and `strength = 1` must reproduce the aim verbatim — not
/// "close to", VERBATIM: `orig + (aimed - orig)` is not `aimed` for every pair in
/// `f32`, so a lerp that merely lands there would move documents nobody edited.
/// The element starts at a rotation far from its aim precisely so a blend that
/// leaked even slightly would show.
#[test]
fn a_stream_without_the_column_gets_the_whole_aim_verbatim() {
    for (tx, ty, want) in [
        (5.0f32, 0.0f32, 0.0f32),
        (0.0, 5.0, 90.0),
        (-5.0, 0.0, 180.0),
        (3.0, 4.0, 53.13),
    ] {
        let got = aimed(137.0, None, 1.0, tx, ty);
        assert!(
            (got - want).abs() < 0.2,
            "aim at ({tx},{ty}) from rot=137 is {want}, got {got}"
        );
    }
    // ⚠️ And the bit-exactness, which needs a case where the LERP would give a
    // different NUMBER for the same angle. `offset` is that case: it can push the
    // aim past +-180, and then `aimed - orig` leaves the fold range, so a blend
    // that routed full weight through `orig + wrap180(aimed - orig)` would write
    // `aimed - 360`. Same heading, different value — and anything reading `rot`
    // as a number (a `value.attribute`, a downstream `motion.drive`) sees it.
    //
    // The first fixture here did NOT contain this: every delta it built happened
    // to sit inside +-180, so the mutation that removes the verbatim arm survived
    // the whole file. A fixture only proves what it contains.
    let far = aimed_off(0.0, None, 1.0, -5.0, 0.0873, 180.0);
    assert!(
        far > 350.0,
        "offset 180 on a 179-degree aim is ~359 degrees, kept as written: {far}"
    );
    // Same heading whichever way it is expressed — so the assert above is about
    // the NUMBER, which is exactly what "verbatim" has to mean here.
    assert!(
        ((far - 360.0) + 1.0).abs() < 0.2,
        "and it IS the -1 heading: {far}"
    );
}

/// **Zero weight is a pass-through, VERBATIM** — the promise a falloff makes is
/// that outside it nothing happens, and `orig ± 360` is not nothing.
#[test]
fn zero_weight_leaves_the_rotation_exactly_where_it_was() {
    for orig in [137.0f32, -179.0, 0.0, 1234.5] {
        assert_eq!(
            aimed(orig, Some(0.0), 1.0, 3.0, 4.0).to_bits(),
            orig.to_bits(),
            "falloff 0 must return the original rotation bit for bit"
        );
        assert_eq!(
            aimed(orig, None, 0.0, 3.0, 4.0).to_bits(),
            orig.to_bits(),
            "strength 0 must do the same — they are one number"
        );
        // ⚠️ And a NEGATIVE weight, which is what makes this arm load-bearing
        // rather than arithmetic: at exactly zero `orig + delta * 0.0` is already
        // `orig`, so the guard looks redundant — until the chip accepts a minus
        // sign and the element starts turning AWAY from the target. The arm is
        // the clamp, and this is the half of it a zero can never show.
        for w in [-0.5f32, -3.0] {
            assert_eq!(
                aimed(orig, None, w, 3.0, 4.0).to_bits(),
                orig.to_bits(),
                "a negative weight is no weight, never a turn the other way"
            );
        }
    }
}

/// **Half weight turns HALF WAY, and the family scales it the way `rotate` does.**
#[test]
fn the_falloff_column_scales_how_far_the_element_turns() {
    // Starts at 0, aims at 90: the quarter-turn is easy to read.
    let full = aimed(0.0, Some(1.0), 1.0, 0.0, 5.0);
    let half = aimed(0.0, Some(0.5), 1.0, 0.0, 5.0);
    let quarter = aimed(0.0, Some(0.25), 1.0, 0.0, 5.0);
    assert!(
        (full - 90.0).abs() < 0.2,
        "full weight is the whole aim: {full}"
    );
    assert!(
        (half - 45.0).abs() < 0.2,
        "half weight is half the turn: {half}"
    );
    assert!(
        (quarter - 22.5).abs() < 0.2,
        "a quarter is a quarter of the turn: {quarter}"
    );
}

/// **`strength` and `falloff` are ONE number by the time the blend sees them** —
/// the P1 of the sheet, which is why it ships with the P0 rather than after it.
#[test]
fn strength_and_the_column_multiply_into_a_single_weight() {
    let by_column = aimed(0.0, Some(0.25), 1.0, 0.0, 5.0);
    let by_param = aimed(0.0, None, 0.25, 0.0, 5.0);
    let by_both = aimed(0.0, Some(0.5), 0.5, 0.0, 5.0);
    assert_eq!(
        by_column.to_bits(),
        by_param.to_bits(),
        "a 0.25 column and a 0.25 strength are the same weight"
    );
    assert_eq!(
        by_both.to_bits(),
        by_column.to_bits(),
        "0.5 x 0.5 is the same weight as 0.25 from either side alone"
    );
}

/// **The turn takes the SHORT arc, and this is the gate the whole wave rests on.**
///
/// An element pointing at `-179` that should aim at `179` is **two degrees away**.
/// A plain lerp at half weight sends it to `0` — the long way, through pointing at
/// nothing at all — and every other gate here would stay green while it did.
#[test]
fn a_partial_turn_takes_the_short_way_round() {
    // Target at 179 degrees: just left of straight up-left, in the upper half.
    let (tx, ty) = (-5.0f32, 0.0873); // atan2(0.0873, -5) ~ 179.0 deg
    let half = aimed(-179.0, Some(0.5), 1.0, tx, ty);
    // The short arc from -179 to +179 is +2 deg through 180, so half is +-180.
    assert!(
        half.abs() > 175.0,
        "half of a two-degree turn stays near the half-turn, not at zero: {half}"
    );
    // And the long way round is what a naive lerp would give.
    assert!(
        half.abs() > 90.0,
        "a lerp would have landed near 0 degrees: {half}"
    );
}

/// **A weight above 1 is the whole aim, not an overshoot past it.**
///
/// Extrapolating a heading is unbounded and unreadable — unlike a position, where
/// overshoot has a picture, turning *past the thing you are looking at* is not a
/// thing anyone asks for, so the two ends of the blend ARE the clamp.
#[test]
fn a_weight_past_one_is_still_the_whole_aim() {
    let one = aimed(0.0, None, 1.0, 0.0, 5.0);
    for over in [1.5f32, 4.0, 1e6] {
        assert_eq!(
            aimed(0.0, None, over, 0.0, 5.0).to_bits(),
            one.to_bits(),
            "strength {over} is the aim, bit for bit — never past it"
        );
    }
}

/// **The half-turn resolves the same way on both languages, and `floor` is why.**
///
/// The closed form wants a nearest-integer; Rust's `round` breaks ties away from
/// zero and WGSL's breaks them to even, so at exactly `d = +-180` the two would
/// disagree by a whole revolution and the element would turn to opposite sides on
/// CPU and GPU. `floor` has no ties.
#[test]
fn the_exact_half_turn_folds_one_way_and_it_is_clockwise() {
    assert_eq!(
        wrap180(180.0),
        -180.0,
        "a positive half-turn folds negative"
    );
    assert_eq!(wrap180(-180.0), -180.0, "and so does the negative one");
    // The neighbourhood is continuous around it — no jump of a revolution.
    assert!((wrap180(179.9) - 179.9).abs() < 1e-3);
    assert!((wrap180(-179.9) + 179.9).abs() < 1e-3);
    // And a plain difference in range is left alone, so the common case is exact.
    for d in [0.0f32, 1.0, -1.0, 90.0, -90.0, 179.0] {
        assert_eq!(
            wrap180(d).to_bits(),
            d.to_bits(),
            "in range, wrap180 is identity"
        );
    }
}
