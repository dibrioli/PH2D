//! `motion.emitter`'s unit suite — split from `lib.rs` at the HR-18 LOC cap.
//! The `#[path]` include keeps it a child module, so it still sees the
//! crate-private count law (`window`, `MAX_ALIVE`, `ID_WRAP`) it exists to
//! pin.

use super::*;
use ph2d_nodegraph::gpu::CountLawCtx;

fn spec() -> Spec {
    Spec {
        rate: 10.0,
        life: 1.0,
        speed: 2.0,
        // The single speed the node had before `speed_random` existed — declared, never
        // inherited: a fixture that reaches its state through a default flips meaning the day
        // the default moves, and stays green testing the opposite.
        speed_random: 0.0,
        angle: 90.0,
        spread: 0.0, // a pencil beam: the launch is exactly `angle`
        origin: [1.0, -1.0],
        // The point emitter the node had before `shape_mode` existed — declared, never
        // inherited, for the same reason `speed_random` is.
        shape: Shape::Point,
        shape_wh: [1.0, 1.0],
        seed: 0,
        max: 1024,
        size: 0.2,
    }
}

/// The muzzle velocity **as it stood before `speed_random` existed**, frozen verbatim. It is
/// here only to be disagreed with — a copy reachable from the product with no caller would be
/// a second answer waiting for one.
fn vel_before_speed_random(spec: &Spec, id: u32) -> [f32; 2] {
    let jitter = rand01(spec.seed, id, LANE_ANGLE) - 0.5;
    let deg = spec.angle + jitter * spec.spread;
    let (cx, sy) = cos_sin_cycles(deg / 360.0);
    [cx * spec.speed, sy * spec.speed]
}

fn vels_of(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("vel").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("vel is Vec2"),
    }
}

fn speed_of(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// **`speed_random = 0` is the single speed that always shipped** — every particle, bit for
/// bit, against the frozen law. This is what makes the default cost nothing rather than
/// merely look the same.
#[test]
fn speed_random_zero_is_byte_identical_to_the_law_that_shipped() {
    let mut s = spec();
    s.spread = 40.0; // a real cone, so the angle lane is exercised too
    s.seed = 11;
    let out = emit(&s, 4.0);
    let ids = ids_of(&out);
    assert!(ids.len() > 5, "a populated window: {}", ids.len());
    for (v, id) in vels_of(&out).into_iter().zip(&ids) {
        assert_eq!(v, vel_before_speed_random(&s, *id as u32), "id {id}");
    }
}

/// **The speeds spread around the authored one, not away from it.** The draw is centred, so
/// the mean stays on `speed` while the extremes reach out toward `speed × (1 ± r)`.
///
/// FALSIFIED by a dead knob (no spread at all) and by a one-sided draw (a mean that walks).
#[test]
fn the_speeds_spread_around_the_authored_one() {
    let mut s = spec();
    s.speed_random = 0.5;
    s.rate = 400.0;
    s.life = 2.0;
    s.seed = 3;
    let speeds: Vec<f32> = vels_of(&emit(&s, 4.0)).into_iter().map(speed_of).collect();
    assert!(speeds.len() > 500, "a big sample: {}", speeds.len());

    let (lo, hi) = speeds
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), &x| (l.min(x), h.max(x)));
    let mean = speeds.iter().sum::<f32>() / speeds.len() as f32;
    // `speed = 2.0`, `r = 0.5` ⇒ the reachable band is 1.0 .. 3.0.
    assert!(lo < 1.15 && hi > 2.85, "the band is reached: {lo} .. {hi}");
    assert!((mean - 2.0).abs() < 0.06, "centred on `speed`: mean {mean}");
}

/// **A particle keeps its speed while the window slides beneath it** — the property the whole
/// hash-by-identity design exists for, and the one `value.instance_field(Random)` cannot have:
/// it hashes the *index*, and an emitter's index shifts the moment an older particle dies, so
/// anything built on it FLICKERS.
///
/// The oracle is the `id` column: match the two frames by identity, not by row.
#[test]
fn a_particle_keeps_its_speed_while_the_window_slides() {
    let mut s = spec();
    s.speed_random = 0.7;
    s.seed = 5;

    let sample = |t: f32| -> Vec<(u32, f32)> {
        let out = emit(&s, t);
        ids_of(&out)
            .into_iter()
            .map(|i| i as u32)
            .zip(vels_of(&out).into_iter().map(speed_of))
            .collect()
    };
    let early = sample(1.0);
    let later = sample(1.55);
    // The window really did slide — some ids died, some were born.
    assert!(
        early[0].0 != later[0].0,
        "the oldest changed: {} vs {}",
        early[0].0,
        later[0].0
    );

    let mut shared = 0usize;
    for (id, sp) in &early {
        if let Some((_, sp2)) = later.iter().find(|(i, _)| i == id) {
            assert_eq!(sp, sp2, "id {id} kept its speed");
            shared += 1;
        }
    }
    assert!(shared >= 3, "the two frames overlap: {shared} shared ids");
}

/// **The authored `speed_random` REACHES the launch.** Every other gate above builds a `Spec`
/// by hand, so all of them stay green with `ctx.param("speed_random")` unread — this is the one
/// that walks the seam from `set_param` to a muzzle velocity.
#[test]
fn the_authored_speed_random_reaches_the_launch() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
        }
    }
    let cook_speeds = |random: Option<f32>| -> Vec<f32> {
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        g.set_param(em, "rate", 60.0);
        g.set_param(em, "life", 2.0);
        g.set_param(em, "speed", 2.0);
        g.set_param(em, "spread", 0.0);
        if let Some(r) = random {
            g.set_param(em, "speed_random", r);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, em, 3.0).unwrap();
        vels_of(out[0].as_stream())
            .into_iter()
            .map(speed_of)
            .collect()
    };

    // ⚠️ The half that names the DEFAULT does not mention it: a graph that never sets the
    // param launches every particle at exactly the authored speed.
    let untouched = cook_speeds(None);
    assert!(untouched.len() > 10, "a populated window");
    for s in &untouched {
        assert!((s - 2.0).abs() < 1e-4, "one speed for all: {s}");
    }

    let varied = cook_speeds(Some(0.6));
    let (lo, hi) = varied
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), &x| (l.min(x), h.max(x)));
    assert!(hi - lo > 1.0, "the authored spread arrived: {lo} .. {hi}");
}

fn pos_of(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P is Vec2"),
    }
}

/// A shaped emitter with a big enough window to say something about a distribution.
fn shaped(shape: Shape, wh: [f32; 2]) -> Spec {
    let mut s = spec();
    s.shape = shape;
    s.shape_wh = wh;
    s.rate = 400.0;
    s.life = 2.0;
    s.seed = 17;
    s
}

/// **A point emitter is byte-identical to the one that shipped** — and the fixture is the case
/// that a lazier implementation gets wrong: an origin the artist typed as `-0`.
///
/// ⚠️ Compared through `to_bits`, because `-0.0 == 0.0` is TRUE in IEEE-754: an `assert_eq!` on
/// the floats would pass over exactly the divergence this gate exists for. Summing a `[0.0,0.0]`
/// offset instead of returning `None` flips that sign, and only the bits can see it.
#[test]
fn a_point_emitter_is_byte_identical_to_the_one_that_shipped() {
    let mut s = shaped(Shape::Point, [3.0, 3.0]);
    s.origin = [-0.0, 5.0];
    let pts = pos_of(&emit(&s, 4.0));
    assert!(pts.len() > 100, "a populated window: {}", pts.len());
    for p in &pts {
        assert_eq!(p[0].to_bits(), (-0.0f32).to_bits(), "the origin, verbatim");
        assert_eq!(p[1].to_bits(), 5.0f32.to_bits());
    }
}

/// **A disc is filled, and filled EVENLY OVER ITS AREA** — the property the `sqrt` exists for.
///
/// A raw-radius draw is the obvious wrong answer and it does not look wrong at a glance: the
/// points still fill the circle, they just pile into the middle, which reads as a bright core the
/// artist never authored. The oracle separates the two exactly — for an area-uniform disc **half**
/// the points fall inside `r/√2`; for a radius-uniform one it is `1/√2 ≈ 71%`.
#[test]
fn a_disc_is_filled_evenly_over_its_area() {
    let s = shaped(Shape::Disc, [2.0, 3.0]);
    let pts = pos_of(&emit(&s, 4.0));
    assert!(pts.len() > 700, "a big sample: {}", pts.len());

    let mut inside_half_area = 0usize;
    let mut reached = 0.0f32;
    for p in &pts {
        let (dx, dy) = (p[0] - s.origin[0], p[1] - s.origin[1]);
        // Normalised radius in the ellipse's own frame.
        let rn = ((dx / 2.0).powi(2) + (dy / 3.0).powi(2)).sqrt();
        assert!(rn <= 1.02, "inside the ellipse: {rn}");
        reached = reached.max(rn);
        if rn <= std::f32::consts::FRAC_1_SQRT_2 {
            inside_half_area += 1;
        }
    }
    assert!(reached > 0.95, "the rim is reached: {reached}");
    let half = inside_half_area as f32 / pts.len() as f32;
    assert!(
        (half - 0.5).abs() < 0.06,
        "area-uniform: {half} of the points inside r/sqrt(2) (0.5 even, 0.71 radius-uniform)"
    );
}

/// **A ring is only the outline.** Every particle sits ON the ellipse, none inside it.
#[test]
fn a_ring_is_only_its_outline() {
    let s = shaped(Shape::Ring, [2.0, 1.0]);
    let pts = pos_of(&emit(&s, 4.0));
    assert!(pts.len() > 700, "a big sample");
    for p in &pts {
        let (dx, dy) = (p[0] - s.origin[0], p[1] - s.origin[1]);
        let rn = ((dx / 2.0).powi(2) + (dy / 1.0).powi(2)).sqrt();
        // The parabolic wave is ~0.09% off the unit circle — the band is its error, not slack.
        assert!((rn - 1.0).abs() < 0.01, "on the outline: {rn}");
    }
}

/// **A rect fills its extents, and its two axes are INDEPENDENT.** Sharing one hash lane between
/// them would lay every particle on the diagonal — a rect emitter that emits a line.
#[test]
fn a_rect_fills_its_extents_on_two_independent_axes() {
    let s = shaped(Shape::Rect, [2.0, 0.5]);
    let pts = pos_of(&emit(&s, 4.0));
    assert!(pts.len() > 700, "a big sample");

    let d: Vec<[f32; 2]> = pts
        .iter()
        .map(|p| [p[0] - s.origin[0], p[1] - s.origin[1]])
        .collect();
    let (mut mx, mut my) = (0.0f32, 0.0f32);
    for q in &d {
        assert!(q[0].abs() <= 2.001 && q[1].abs() <= 0.501, "inside: {q:?}");
        mx = mx.max(q[0].abs());
        my = my.max(q[1].abs());
    }
    assert!(mx > 1.9 && my > 0.47, "both extents reached: {mx}, {my}");

    // Independence, read off the emitted positions: exactly ±1 with the lanes shared.
    let n = d.len() as f32;
    let (ax, ay) = d
        .iter()
        .fold((0.0f32, 0.0f32), |(a, b), q| (a + q[0] / n, b + q[1] / n));
    let (mut cov, mut vx, mut vy) = (0.0f32, 0.0f32, 0.0f32);
    for q in &d {
        let (ex, ey) = (q[0] - ax, q[1] - ay);
        cov += ex * ey;
        vx += ex * ex;
        vy += ey * ey;
    }
    let corr = cov / (vx.sqrt() * vy.sqrt());
    assert!(corr.abs() < 0.2, "the two axes are separate draws: {corr}");
}

/// **A particle keeps its birthplace while the window slides** — the same identity property the
/// speed has, on the lanes the shape uses. An index-hashed birthplace would make the whole cloud
/// jump every time an older particle died.
#[test]
fn a_particle_keeps_its_birthplace_while_the_window_slides() {
    let mut s = shaped(Shape::Disc, [2.0, 2.0]);
    // ⚠️ **TWO premises, and the first version of this gate broke each in turn** — both caught by
    // its own controls rather than by a false green. `shaped` sets a 2 s life for a big sample and
    // nothing dies inside it, so the window had never SLID; shortening the life to 0.5 s fixed
    // that and broke the other half, because the two samples were 0.55 s apart and every particle
    // of the first frame was dead by the second, so there was nothing left to compare. The gap has
    // to be shorter than the life **and** long enough to kill somebody.
    s.life = 0.5;
    let sample = |t: f32| -> Vec<(u32, [f32; 2])> {
        let out = emit(&s, t);
        ids_of(&out)
            .into_iter()
            .map(|i| i as u32)
            .zip(pos_of(&out))
            .collect()
    };
    let (early, later) = (sample(1.0), sample(1.3));
    assert!(early[0].0 != later[0].0, "the window really slid");
    let mut shared = 0usize;
    for (id, p) in &early {
        if let Some((_, q)) = later.iter().find(|(i, _)| i == id) {
            assert_eq!(p, q, "id {id} kept its birthplace");
            shared += 1;
        }
    }
    assert!(shared >= 3, "the frames overlap: {shared}");
}

/// **The authored shape REACHES the birth.** Every gate above builds a `Spec` by hand, so all of
/// them stay green with the three params unread — this walks the seam from `set_param` to a
/// placed particle, and its first half never names the default it is testing.
#[test]
fn the_authored_shape_reaches_the_birth() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
        }
    }
    let cook_spread = |shape: Option<f32>| -> f32 {
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        g.set_param(em, "rate", 200.0);
        g.set_param(em, "life", 2.0);
        g.set_param(em, "x", 1.0);
        g.set_param(em, "y", 2.0);
        if let Some(m) = shape {
            g.set_param(em, "shape_mode", m);
            g.set_param(em, "shape_w", 3.0);
            g.set_param(em, "shape_h", 3.0);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, em, 3.0).unwrap();
        pos_of(out[0].as_stream())
            .into_iter()
            .fold(0.0f32, |m, p| m.max((p[0] - 1.0).hypot(p[1] - 2.0)))
    };
    assert_eq!(cook_spread(None), 0.0, "untouched: one spot, at the origin");
    assert!(cook_spread(Some(1.0)) > 2.8, "a disc arrived");
    assert!(cook_spread(Some(3.0)) > 2.8, "a rect arrived");
}

/// **How fast a particle leaves is not a function of WHERE in the cone it leaves.** Sharing one
/// hash lane between the angle and the speed ties the two together exactly: the fastest particle
/// always sits at one edge of the cone, and a spray reads as a fan.
///
/// ⚠️ **The oracle is the OUTPUT, and the first version of this gate was not.** It asserted
/// `rand01(seed, id, LANE_ANGLE) != rand01(seed, id, LANE_SPEED)` — a fact about the *hash*,
/// which stays true no matter which lane the emitter passes it. The mutation that matters
/// (`LANE_SPEED` → `LANE_ANGLE` at the call site) sailed straight through it. What is measured
/// now is the correlation between each particle's launch direction and its speed, both read off
/// the emitted `vel` column: **exactly ±1 with the lanes shared, ~0 with them apart.**
#[test]
fn how_fast_a_particle_leaves_is_not_where_in_the_cone_it_leaves() {
    let mut s = spec();
    s.spread = 90.0;
    s.speed_random = 0.8;
    s.rate = 400.0;
    s.life = 2.0;
    s.seed = 13;

    let vels = vels_of(&emit(&s, 4.0));
    assert!(vels.len() > 500, "a big sample: {}", vels.len());
    // The launch direction, straight off the wire (std `atan2` — no shared line with the
    // parabolic wave the node uses), against the speed.
    let pairs: Vec<(f32, f32)> = vels
        .iter()
        .map(|v| (v[1].atan2(v[0]).to_degrees() - s.angle, speed_of(*v)))
        .collect();

    let n = pairs.len() as f32;
    let (ma, ms) = pairs
        .iter()
        .fold((0.0f32, 0.0f32), |(a, b), (x, y)| (a + x / n, b + y / n));
    let (mut cov, mut va, mut vs) = (0.0f32, 0.0f32, 0.0f32);
    for (a, sp) in &pairs {
        let (da, ds) = (a - ma, sp - ms);
        cov += da * ds;
        va += da * da;
        vs += ds * ds;
    }
    let corr = cov / (va.sqrt() * vs.sqrt());
    assert!(
        corr.abs() < 0.2,
        "direction and speed are separate draws, correlation {corr}"
    );
}

fn ids_of(s: &Stream) -> Vec<f32> {
    match s.get("id").unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("id"),
    }
}

/// **Identity survives any rate, at any playhead** — the invariant that lets
/// this engine aim at millions of particles instead of thousands.
///
/// This gate was written RED, asserting the opposite. Ids are stored as `f32`
/// (the stream's only scalar column), so a spawn index past 2²⁴ used to
/// collapse onto its neighbour: measured, rate 4.000.000 at t = 5 s gave
/// 4096 particles and **2049 distinct ids**, and both pairings — the CPU's
/// `BTreeMap<id,row>` and the GPU's `id − prev_first` — then handed two
/// particles the same prior state, in silence.
///
/// The emitter's own doc filed this under "≈ 4,8 days at rate 40, out of
/// scope". That was TRUE while the rate slider stopped at 200 (23 hours). It
/// stopped being true the moment the ceiling moved — 23 minutes at 12.000/s,
/// **4 seconds** at the millions-per-second a real particle sim wants. A note
/// that says "unreachable" is a claim about a number someone else can change.
///
/// Now the index WRAPS at [`ID_WRAP`] and every consumer reads identity as a
/// difference inside one window, which is orders of magnitude smaller than
/// the wrap period. So there is no rate at which this breaks.
#[test]
fn identity_is_exact_at_any_rate_because_it_wraps() {
    let distinct_ids = |rate: f32, t: f32| {
        let mut sp = spec();
        sp.rate = rate;
        sp.life = 1.0;
        sp.max = 4096;
        let mut ids = ids_of(&emit(&sp, t));
        let n = ids.len();
        ids.sort_by(f32::total_cmp);
        ids.dedup();
        (n, ids.len())
    };
    // Well under the old cliff, far past it, and absurdly past it.
    for (rate, t) in [
        (200.0, 5.0),
        (12_000.0, 5.0),
        (4_000_000.0, 5.0),     // used to collapse to 2049 of 4096
        (4_000_000.0, 3_600.0), // an hour in: 1,4e10 spawns, ~858 wraps
    ] {
        let (n, distinct) = distinct_ids(rate, t);
        assert!(
            n > 1,
            "rate {rate} at t={t}: the fixture must have a window"
        );
        assert_eq!(
            distinct, n,
            "rate {rate} at t={t}: every id in a window must be its own"
        );
    }
    // …and the wrap is REACHED, not merely tolerated — otherwise the loop
    // above would be four restatements of "we stayed under 2²⁴".
    let w = window(4_000_000.0, 1.0, 4096, 3_600.0);
    assert!(
        (w.first as u64) < u64::from(ID_WRAP),
        "the stored index lives inside the exact range"
    );
    assert!(
        4_000_000.0f64 * 3_600.0 > f64::from(ID_WRAP),
        "the fixture must actually pass the wrap point"
    );
    // The age stays a small, honest number instead of the difference of two
    // large ones: at rate 4e6 the old `t − id/rate` was 3600.0 − 3600.0.
    assert!(
        w.age_first > 0.0 && w.age_first <= 1.0,
        "the oldest particle's age must be within `life`, got {}",
        w.age_first
    );
}

#[test]
fn the_alive_set_is_the_id_window_born_within_life() {
    // rate 10, life 1 → at t = 1.55 the alive ids are those born in
    // (0.55, 1.55]: k/10 ∈ (0.55, 1.55] → k ∈ 6..=15.
    let s = emit(&spec(), 1.55);
    assert_eq!(ids_of(&s).first().copied(), Some(6.0));
    assert_eq!(ids_of(&s).last().copied(), Some(15.0));
    assert_eq!(s.count(), 10);
    // Ids are strictly ascending (oldest first) so Index reads as age order.
    assert!(ids_of(&s).windows(2).all(|w| w[1] > w[0]));
}

#[test]
fn particles_are_born_and_die_on_schedule() {
    // At t=0 only particle 0 exists; by t just under 1 life later it still
    // does; a hair past, it is gone (its successors remain).
    assert_eq!(ids_of(&emit(&spec(), 0.0)), vec![0.0]);
    assert!(ids_of(&emit(&spec(), 0.99)).contains(&0.0), "still alive");
    assert!(!ids_of(&emit(&spec(), 1.01)).contains(&0.0), "died at life");
}

#[test]
fn scrubbing_backwards_reproduces_the_scene_exactly() {
    // The reference's stateful emitter could not do this. Ask for a time,
    // get the same particles — no matter what was asked before.
    let forward = emit(&spec(), 2.5);
    for t in [0.3, 7.1, 0.0, 2.5] {
        let _ = emit(&spec(), t);
    }
    let revisited = emit(&spec(), 2.5);
    assert_eq!(ids_of(&forward), ids_of(&revisited));
    assert_eq!(forward.get("vel"), revisited.get("vel"));
}

#[test]
fn launch_velocity_follows_angle_and_speed() {
    // Zero spread, angle 90 (up in this Y-up world) → vel = (0, speed).
    let s = emit(&spec(), 0.5);
    match s.get("vel").unwrap() {
        Column::Vec2(v) => {
            for q in v {
                assert!(q[0].abs() < 1e-4, "no lateral component");
                assert!((q[1] - 2.0).abs() < 1e-4, "speed straight up");
            }
        }
        _ => panic!("vel"),
    }
    // And they all start at the origin — displacement is the integrator's.
    match s.get("P").unwrap() {
        Column::Vec2(v) => assert!(v.iter().all(|p| *p == [1.0, -1.0])),
        _ => panic!("P"),
    }
}

#[test]
fn spread_fans_the_cone_but_never_past_its_half_angle() {
    let mut sp = spec();
    sp.spread = 60.0;
    sp.rate = 200.0; // plenty of samples
    let s = emit(&sp, 1.0);
    let Column::Vec2(v) = s.get("vel").unwrap() else {
        panic!("vel")
    };
    // angle 90 ± 30 → the x-component is bounded by speed·sin(30°) = 1.0.
    let max_lateral = v.iter().map(|q| q[0].abs()).fold(0.0f32, f32::max);
    assert!(
        max_lateral <= 1.01,
        "cone half-angle respected: {max_lateral}"
    );
    assert!(max_lateral > 0.5, "the cone actually fans out");
    // All still flying upward.
    assert!(v.iter().all(|q| q[1] > 0.0));
}

#[test]
fn the_cap_keeps_the_newest_particles() {
    let mut sp = spec();
    sp.max = 3;
    let s = emit(&sp, 1.55); // would be 10 alive
    assert_eq!(s.count(), 3);
    assert_eq!(ids_of(&s), vec![13.0, 14.0, 15.0], "the newest three");
}

#[test]
fn a_dead_or_absurd_emitter_yields_an_empty_stream_not_a_panic() {
    let mut sp = spec();
    sp.rate = 0.0;
    assert_eq!(emit(&sp, 5.0).count(), 0);
    let mut sp = spec();
    sp.life = 0.0;
    assert_eq!(emit(&sp, 5.0).count(), 0);
    // A negative playhead (a scrub past zero) emits nothing rather than
    // hashing negative ids.
    assert_eq!(emit(&spec(), -1.0).count(), 0);
    // `emit` honours whatever cap it is handed…
    let mut sp = spec();
    sp.rate = 1e6;
    sp.max = 4096;
    assert_eq!(emit(&sp, 10.0).count(), 4096);
    // …and the HARD ceiling is asserted on the count law itself rather than
    // by building the stream: at 4M elements that would allocate ~176 MB to
    // learn a number `window` already knows. It is also the only place worth
    // asserting it, now that `emit` and the GPU `count_law` are the same
    // function — the two paths cannot disagree about `n` any more, which is
    // the point of unifying them.
    assert_eq!(
        window(1e9, 1e9, MAX_ALIVE, 10.0).count,
        MAX_ALIVE,
        "an absurd ask clamps to the ceiling"
    );
    let gpu_window = GPU_KERNEL.count_law.expect("the emitter is a generator");
    assert_eq!(
        gpu_window(&CountLawCtx {
            inputs: &[],
            param: &|_| 1e9,
            playhead: 10.0,
            dt: 0.0,
        })
        .count,
        MAX_ALIVE,
        "the GPU path reaches the same ceiling through the same law"
    );
}

#[test]
fn age_and_index_track_the_stream_order() {
    let s = emit(&spec(), 1.55);
    let Column::Scalar(age) = s.get("age").unwrap() else {
        panic!("age")
    };
    // Oldest first: age descends, and every age is within [0, life).
    assert!(age.windows(2).all(|w| w[1] < w[0]));
    assert!(age.iter().all(|a| *a >= 0.0 && *a < 1.0));
    let Column::Scalar(idx) = s.get("Index").unwrap() else {
        panic!("Index")
    };
    assert_eq!(idx.first().copied(), Some(0.0));
    assert_eq!(idx.last().copied(), Some(9.0));
}

#[test]
fn particles_carry_their_own_size() {
    // Without this column the lowering would fall back to the caller's
    // default (a grid dot), and a dense jet would read as a poured ribbon.
    let s = emit(&spec(), 0.5);
    match s.get("size").unwrap() {
        Column::Vec2(v) => assert!(v.iter().all(|q| *q == [0.2, 0.2])),
        _ => panic!("size"),
    }
}

/// The `eval` seam itself (audit 2026-07-10: every behavioural test above
/// drives `emit()` directly, so the ctx.param/ctx.playhead → `Spec` wiring
/// was untested): cook the REAL node with overridden params and assert the
/// alive set reflects them — rate·window rows, all at the (x, y) origin,
/// sized by the `size` param as a Vec2 column.
#[test]
fn eval_maps_params_and_playhead_into_the_spec() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    for (name, v) in [
        ("rate", 2.0),
        ("life", 1.0),
        ("speed", 0.0),
        ("x", 3.0),
        ("y", 4.0),
        ("size", 0.25),
    ] {
        g.set_param(em, name, v);
    }
    let mut cook = Cook::new();
    // t = 0.9, rate 2 → births at 0 and 0.5, both younger than life 1.
    let out = cook.cook(&g, &Ops, em, 0.9).unwrap();
    let s = out[0].as_stream();
    assert_eq!(s.count(), 2, "rate × window alive");
    match s.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[3.0, 4.0]; 2], "the (x,y) origin"),
        _ => panic!("P"),
    }
    match s.get("size").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[0.25, 0.25]; 2], "size param → Vec2 column"),
        _ => panic!("size"),
    }
}

#[test]
fn registers_and_resolves() {
    use ph2d_nodegraph::cook::OpResolver;
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
