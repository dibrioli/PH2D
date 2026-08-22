//! Gates da `force.buoyancy` — o empuxo, a densidade e o arrasto do fluido.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a casa já usa no irmão `density_tests.rs`: a LEI no `lib.rs`, as
//! PROVAS aqui. `#[path]` mantém o módulo a chamar-se `tests`.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A one-instance source at `(x, y)` with velocity `vel` — and, when asked, a half
/// `falloff` so the field's gating is visible too.
struct Src {
    p: [f32; 2],
    vel: [f32; 2],
    falloff: Option<f32>,
}
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.buoyancy.test.src"),
    name: "force.buoyancy.test.src",
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
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mut s = Stream::new(1)
            .with("P", Column::Vec2(vec![self.p]))
            .with("vel", Column::Vec2(vec![self.vel]));
        if let Some(f) = self.falloff {
            s = s.with("falloff", Column::Scalar(vec![f]));
        }
        ctx.emit(s);
    }
}
struct Ops(Src);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == MANIFEST.id {
            Some(&ForceBuoyancy as &dyn NodeOp)
        } else if ty == SRC_MAN.id {
            Some(&self.0 as &dyn NodeOp)
        } else {
            None
        }
    }
}

/// The acceleration this force contributes to one instance.
fn accel(src: Src, params: &[(&str, f32)], t: f64) -> [f32; 2] {
    let mut g = Graph::new();
    let s = g.add_node("force.buoyancy.test.src");
    let b = g.add_node("force.buoyancy");
    g.connect(Edge {
        from: (s, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
    for (k, v) in params {
        g.set_param(b, *k, *v);
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops(src), b, t).unwrap();
    match out[0].as_stream().get("accel") {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("a force writes accel"),
    }
}

/// A still sea: no wave, so the only thing acting is the vertical push.
const FLAT: &[(&str, f32)] = &[("wave_amplitude", 0.0), ("level", 0.0), ("drag", 0.0)];

fn at(y: f32) -> Src {
    Src {
        p: [0.0, y],
        vel: [0.0, 0.0],
        falloff: None,
    }
}

/// **Above the water the node does nothing.** The gate that keeps a force honest: it
/// acts where its field is, and nowhere else. (A missing `clamp(0,1)` makes the
/// submersion negative up here and the sea would *suck things down* from the sky.)
#[test]
fn a_thing_in_the_air_is_untouched() {
    assert_eq!(accel(at(1.0), FLAT, 0.0), [0.0, 0.0]);
    // …including a fast-moving one: the drag is gated by submersion, not applied to
    // everything that passes overhead.
    let flying = Src {
        p: [0.0, 1.0],
        vel: [5.0, -9.0],
        falloff: None,
    };
    assert_eq!(accel(flying, &[("drag", 8.0)], 0.0), [0.0, 0.0]);
}

/// Fully under, the push is the density, straight up.
#[test]
fn a_submerged_thing_is_pushed_up_by_its_density() {
    let a = accel(at(-1.0), &[("wave_amplitude", 0.0), ("density", 12.0)], 0.0);
    assert!((a[1] - 12.0).abs() < 1e-4, "expected +12 up, got {a:?}");
    assert!(a[0].abs() < 1e-6, "a flat sea pushes straight up");
}

/// **The submersion RAMPS** — this is the difference between floating and standing on
/// a floor. Half a draft under the surface, half the force; a node that binarised the
/// test (`under ? density : 0`) passes the two gates above and fails this one.
#[test]
fn the_push_grows_with_how_deep_it_sits() {
    let params = &[("wave_amplitude", 0.0), ("density", 12.0), ("depth", 0.4)];
    let quarter = accel(at(-0.1), params, 0.0)[1];
    let half = accel(at(-0.2), params, 0.0)[1];
    let full = accel(at(-0.4), params, 0.0)[1];
    let deeper = accel(at(-4.0), params, 0.0)[1];
    assert!((quarter - 3.0).abs() < 1e-4, "a quarter under: {quarter}");
    assert!((half - 6.0).abs() < 1e-4, "half under: {half}");
    assert!((full - 12.0).abs() < 1e-4, "fully under: {full}");
    assert!(
        (deeper - 12.0).abs() < 1e-4,
        "and it does not keep growing below that: {deeper}"
    );
}

/// **It FLOATS** — the product claim, not a component one. With gravity `g` and
/// density `d` the thing settles where the two cancel: `submersion = g/d`, i.e. a
/// draft-fraction `g/d` under the surface. Assert the net acceleration there is zero.
#[test]
fn it_settles_where_buoyancy_cancels_gravity() {
    const G: f32 = 4.0;
    const D: f32 = 12.0;
    const DEPTH: f32 = 0.3;
    // g/d = 1/3 of the draft below the surface.
    let y = -DEPTH * (G / D);
    let a = accel(
        at(y),
        &[
            ("wave_amplitude", 0.0),
            ("density", D),
            ("depth", DEPTH),
            ("drag", 0.0),
        ],
        0.0,
    );
    assert!(
        (a[1] - G).abs() < 1e-3,
        "at the waterline the lift should exactly answer gravity ({G}), got {}",
        a[1]
    );
    // Push it under and it comes back up harder; lift it and it falls back.
    assert!(accel(at(y - 0.05), FLAT_D, 0.0)[1] > accel(at(y), FLAT_D, 0.0)[1]);
    assert!(accel(at(y + 0.05), FLAT_D, 0.0)[1] < accel(at(y), FLAT_D, 0.0)[1]);
}
const FLAT_D: &[(&str, f32)] = &[
    ("wave_amplitude", 0.0),
    ("density", 12.0),
    ("depth", 0.3),
    ("drag", 0.0),
];

/// Water is thick: the drag opposes the velocity, and only under water.
#[test]
fn drag_brakes_the_submerged() {
    let moving = Src {
        p: [0.0, -1.0],
        vel: [2.0, -3.0],
        falloff: None,
    };
    let a = accel(
        moving,
        &[("wave_amplitude", 0.0), ("density", 0.0), ("drag", 2.0)],
        0.0,
    );
    assert!((a[0] - -4.0).abs() < 1e-4, "−k·v in x: {a:?}");
    assert!((a[1] - 6.0).abs() < 1e-4, "−k·v in y: {a:?}");
}

/// **The wave travels.** A crest at `x` now is at `x + speed·Δt` later — so the force
/// on a float at `x` now equals the force on a float at `x + speed·Δt` then. This is
/// the identity that pins BOTH the sign of `wave_speed` and the sign of the phase,
/// which no static snapshot of the surface can.
#[test]
fn the_swell_moves_downstream() {
    let p = &[
        ("wave_amplitude", 0.3),
        ("wave_length", 2.0),
        ("wave_speed", 0.5),
    ];
    let here_now = accel(
        Src {
            p: [0.6, -0.2],
            vel: [0.0, 0.0],
            falloff: None,
        },
        p,
        0.0,
    );
    // 2 s later the same water is 1.0 world unit downstream.
    let there_later = accel(
        Src {
            p: [1.6, -0.2],
            vel: [0.0, 0.0],
            falloff: None,
        },
        p,
        2.0,
    );
    assert!(
        (here_now[0] - there_later[0]).abs() < 1e-3 && (here_now[1] - there_later[1]).abs() < 1e-3,
        "the wave should have carried this exact water downstream: {here_now:?} vs \
         {there_later:?}"
    );
}

/// **The push tilts downhill.** On the flank of a wave the buoyant force is normal to
/// the surface, so it has a horizontal component pointing toward the trough — which is
/// what makes a float ride a swell instead of pumping on the spot. (Straight-up
/// buoyancy passes every gate above and fails this one.)
///
/// A frozen wave (speed 0) of wavelength 4, so the geometry is nameable: the surface
/// **climbs** from the zero-crossing at `x=0` to the crest at `x=1`, is **flat** on the
/// crest, and **falls** from there to the trough at `x=3`. The float leans away from
/// the climb on the way up and away from the fall on the way down — always into the
/// trough. (The mirror flank is at `x=2`, NOT at `x=−2`: a sine is odd, so its slope
/// is *symmetric* about the origin, and both sides of `x=0` climb the same way. My
/// first version of this gate asserted the opposite and this test caught me, not the
/// code.)
#[test]
fn on_a_slope_the_float_is_pushed_toward_the_trough() {
    let p = &[
        ("wave_amplitude", 0.5),
        ("wave_length", 4.0),
        ("wave_speed", 0.0),
        ("drag", 0.0),
    ];
    let under = |x: f32| Src {
        p: [x, -1.0],
        vel: [0.0, 0.0],
        falloff: None,
    };
    let climbing = accel(under(0.5), p, 0.0);
    assert!(
        climbing[0] < -0.1,
        "the surface climbs to the right here, so the push leans left: {climbing:?}"
    );
    let falling = accel(under(2.0), p, 0.0);
    assert!(
        falling[0] > 0.1,
        "and on the far flank, where it falls, the push leans right: {falling:?}"
    );
    // On the crest and in the trough the surface is flat: no lean either way.
    assert!(
        accel(under(1.0), p, 0.0)[0].abs() < 0.05,
        "the crest is flat"
    );
    assert!(
        accel(under(3.0), p, 0.0)[0].abs() < 0.05,
        "the trough is flat"
    );
}

/// The multiplicative `falloff` field gates it like every other force (plan §1.6): a
/// half falloff is half a sea.
#[test]
fn the_falloff_field_scales_the_force() {
    let half = Src {
        p: [0.0, -1.0],
        vel: [0.0, 0.0],
        falloff: Some(0.5),
    };
    let a = accel(half, &[("wave_amplitude", 0.0), ("density", 12.0)], 0.0);
    assert!(
        (a[1] - 6.0).abs() < 1e-4,
        "half the field, half the lift: {a:?}"
    );
}
