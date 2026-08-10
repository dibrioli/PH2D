//! **Which way a particle leaves** — the `dir_mode` family, split from `lib_tests.rs` at
//! the HR-18 LOC cap along the seam the file already had: its neighbour answers *where is
//! a particle BORN* and *which ones exist*, this one *which way it goes*.
//!
//! A GRANDCHILD of `lib.rs`, so `use super::*` still reaches the fixtures
//! (`spec`/`shaped`/`vels_of`/`pos_of`) instead of copying them — a second `spec()` would
//! be a second answer to *what does an untouched emitter look like?*.

use super::*;
use crate::{DirMode, MANIFEST, MotionEmitter, Shape, Spawn, Spec, emit};

// ── `dir_mode` : Angle | Outwards | Inwards ─────────────────────────────────

/// A ring emitter, so every particle has a radius to leave along and the two radial modes are
/// answerable for ALL of them — a `Disc` would sometimes draw the centre, which is the fallback
/// case and belongs to its own gate.
fn ringed(dir: DirMode) -> Spec {
    let mut s = shaped(Shape::Ring, [2.0, 2.0]);
    s.dir = dir;
    s
}

/// The unit radius of particle `i` — where it was born, seen from the emitter's origin.
fn radius_of(s: &Spec, out: &Stream, i: usize) -> [f32; 2] {
    let p = pos_of(out)[i];
    let d = [p[0] - s.origin[0], p[1] - s.origin[1]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    [d[0] / l, d[1] / l]
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// **`Angle` is the expression that always shipped, to the BIT.** Not "the same picture" — the
/// same `f32`, because the arm keeps its single `cos_sin_cycles` call rather than composing two.
#[test]
fn the_angle_mode_is_the_launch_that_always_shipped() {
    let mut s = ringed(DirMode::Angle);
    s.spread = 40.0; // a cone, so the jitter term is live and the claim is not vacuous
    let out = emit(&s, 3.0);
    for (i, v) in vels_of(&out).iter().enumerate() {
        let id = match out.get("id").unwrap() {
            Column::Scalar(c) => c[i] as u32,
            _ => panic!("id"),
        };
        let want = vel_before_speed_random(&s, id);
        assert_eq!(v[0].to_bits(), want[0].to_bits(), "x of {id}");
        assert_eq!(v[1].to_bits(), want[1].to_bits(), "y of {id}");
    }
}

/// **Outwards leaves along the particle's own radius**, and the oracle is the DELIVERED pair:
/// the emitted velocity against the emitted position, never the draw that produced either.
#[test]
fn outwards_launches_along_the_radius_it_was_born_on() {
    let mut s = ringed(DirMode::Outwards);
    s.spread = 0.0; // a pencil beam, so the axis IS the direction
    let out = emit(&s, 3.0);
    for (i, v) in vels_of(&out).iter().enumerate() {
        let unit = [v[0] / speed_of(*v), v[1] / speed_of(*v)];
        assert!(
            dot(unit, radius_of(&s, &out, i)) > 0.999,
            "particle {i}: {unit:?} is not along its radius",
        );
    }
}

/// Inwards is the same axis with the sign flipped — the two are one law, not two.
#[test]
fn inwards_launches_back_towards_the_centre() {
    let mut s = ringed(DirMode::Inwards);
    s.spread = 0.0;
    let out = emit(&s, 3.0);
    for (i, v) in vels_of(&out).iter().enumerate() {
        let unit = [v[0] / speed_of(*v), v[1] / speed_of(*v)];
        assert!(
            dot(unit, radius_of(&s, &out, i)) < -0.999,
            "particle {i}: {unit:?} is not against its radius",
        );
    }
}

/// **`spread` keeps meaning ONE thing in all three modes** — the half-width of the cone, which
/// merely opens around a different axis. Without this the radial modes could have quietly
/// ignored it and every gate above would still pass.
#[test]
fn the_cone_still_opens_around_whatever_axis_was_chosen() {
    let mut s = ringed(DirMode::Outwards);
    s.spread = 60.0;
    let out = emit(&s, 3.0);
    let (mut spread_seen, mut off_axis) = (0.0_f32, 0.0_f32);
    for (i, v) in vels_of(&out).iter().enumerate() {
        let unit = [v[0] / speed_of(*v), v[1] / speed_of(*v)];
        // `cos` of the angle between the launch and the radius; the cone is ±30 deg, so this
        // never drops below `cos(30 deg)`.
        let c = dot(unit, radius_of(&s, &out, i));
        assert!(c > 0.86, "particle {i} left the cone: cos {c}");
        off_axis = off_axis.max(1.0 - c);
        spread_seen += 1.0 - c;
    }
    assert!(
        off_axis > 0.02 && spread_seen > 0.0,
        "the cone is INERT: every particle sat exactly on its radius",
    );
}

/// **A particle born at the centre has no radius**, so it falls back to the cone — bit for bit,
/// which is what makes a `Point` emitter under `Outwards` indistinguishable from `Angle` instead
/// of merely similar.
#[test]
fn a_particle_born_at_the_centre_falls_back_to_the_cone() {
    let cone = |dir: DirMode| -> Spec {
        let mut s = spec();
        s.spread = 40.0;
        s.spawn = Spawn::Continuous { rate: 100.0 };
        s.dir = dir;
        s
    };
    assert_eq!(
        cone(DirMode::Angle).shape,
        Shape::Point,
        "the fixture must contain the case",
    );
    let (a, b) = (cone(DirMode::Angle), cone(DirMode::Outwards));
    for (x, y) in vels_of(&emit(&a, 3.0)).iter().zip(vels_of(&emit(&b, 3.0))) {
        assert_eq!(x[0].to_bits(), y[0].to_bits());
        assert_eq!(x[1].to_bits(), y[1].to_bits());
    }
}

/// **The seam.** Every gate above builds the `Spec` by hand, so all of them stay green with
/// `ctx.param("dir_mode")` unread. This one authors it on a graph and cooks.
#[test]
fn the_authored_direction_reaches_the_launch() {
    let shape = |g: &mut ph2d_nodegraph::graph::Graph, n| {
        g.set_param(n, "shape_mode", 2.0); // Ring
        g.set_param(n, "shape_w", 2.0);
        g.set_param(n, "shape_h", 2.0);
        g.set_param(n, "spread", 0.0);
        g.set_param(n, "rate", 40.0);
        // ⚠️ A NON-zero origin, and the check below measures from it: the axis is the radius
        // inside the EMITTER, never a direction from the world origin. With `x = y = 0` the two
        // are the same vector and the gate could not tell them apart.
        g.set_param(n, "x", 1.0);
        g.set_param(n, "y", -1.0);
    };
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
        }
    }
    let cooked = |dir: Option<f32>| -> Stream {
        let mut g = Graph::new();
        let n = g.add_node("motion.emitter");
        shape(&mut g, n);
        if let Some(d) = dir {
            g.set_param(n, "dir_mode", d);
        }
        let mut cook = Cook::new();
        cook.cook(&g, &Ops, n, 3.0).unwrap()[0].as_stream().clone()
    };
    let cone = cooked(None);
    let out = cooked(Some(1.0));
    assert!(
        vels_of(&cone) != vels_of(&out),
        "the authored direction never reached the launch",
    );
    // And it reached it as OUTWARDS, not as some other change.
    for (i, v) in vels_of(&out).iter().enumerate() {
        let p = pos_of(&out)[i];
        let d = [p[0] - 1.0, p[1] + 1.0];
        let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let unit = [v[0] / speed_of(*v), v[1] / speed_of(*v)];
        assert!(dot(unit, [d[0] / l, d[1] / l]) > 0.999, "particle {i}");
    }
}

/// **The speed is the speed, whichever way the particle leaves.** The axis is a UNIT vector, so
/// a ring emitter does not launch its wide particles faster than its narrow ones — which is
/// exactly what an un-normalised axis would do, silently, with every direction gate still green.
#[test]
fn the_radial_launch_does_not_scale_the_speed_by_the_radius() {
    let s = ringed(DirMode::Outwards);
    // An ELLIPSE, so the radius genuinely varies per particle — on a circle an un-normalised
    // axis is a constant factor and this gate could not tell.
    let mut s = Spec {
        shape_wh: [2.0, 0.5],
        ..s
    };
    s.spread = 0.0;
    let out = emit(&s, 3.0);
    let radii: Vec<f32> = (0..out.count())
        .map(|i| {
            let p = pos_of(&out)[i];
            (p[0] - s.origin[0]).hypot(p[1] - s.origin[1])
        })
        .collect();
    let (lo, hi) = radii
        .iter()
        .fold((f32::MAX, 0.0f32), |(a, b), r| (a.min(*r), b.max(*r)));
    assert!(
        hi > lo * 2.0,
        "the fixture must vary the radius: {lo}..{hi}"
    );
    for v in vels_of(&out) {
        assert!(
            (speed_of(v) - s.speed).abs() < 1e-3,
            "{} is not the authored speed {}",
            speed_of(v),
            s.speed,
        );
    }
}

/// **A ZERO-SIZED shape is the other way to have no radius**, and it is a different mechanism
/// from a `Point`: there the offset is `None` and `radial_axis` short-circuits on the `?`; here
/// the offset EXISTS and is the zero vector, so only the length guard catches it.
///
/// ⚠️ Without that guard the axis is `0 / 0` — the velocity comes out `NaN` and every particle
/// vanishes from the screen. It was measured, not guessed: removing the guard leaves the
/// `Point` fallback gate GREEN, because that one never reaches this line
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn a_zero_sized_shape_falls_back_to_the_cone_too() {
    let base = |dir: DirMode| -> Spec {
        let mut s = shaped(Shape::Disc, [0.0, 0.0]);
        s.spread = 40.0;
        s.dir = dir;
        s
    };
    let (a, b) = (base(DirMode::Angle), base(DirMode::Outwards));
    let (va, vb) = (vels_of(&emit(&a, 3.0)), vels_of(&emit(&b, 3.0)));
    assert!(!va.is_empty(), "the fixture must emit something");
    for (x, y) in va.iter().zip(&vb) {
        assert!(x[0].is_finite() && x[1].is_finite(), "the cone is finite");
        assert_eq!(x[0].to_bits(), y[0].to_bits(), "x");
        assert_eq!(x[1].to_bits(), y[1].to_bits(), "y");
    }
}
