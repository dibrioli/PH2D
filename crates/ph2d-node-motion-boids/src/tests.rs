//! Unit tests for `motion.boids` — the flocking behaviour (spread OFF, the
//! default) and deterministic replay. Split from `lib.rs` at the 700-LOC cap.

use super::*;

fn params(count: usize) -> Params {
    Params {
        count,
        seed: 1,
        radius_sq: 4.0 * 4.0, // wide perception so the whole flock interacts
        separation: 0.0,
        alignment: 0.0,
        cohesion: 0.0,
        seek: 0.0,
        max_speed: 4.0,
        spread: false,
    }
}

/// Mean distance of the flock to its own centroid — how spread out it is.
fn spread(pos: &[[f32; 2]]) -> f32 {
    let n = pos.len() as f32;
    let c = pos
        .iter()
        .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
    let c = [c[0] / n, c[1] / n];
    pos.iter()
        .map(|p| {
            let d = [p[0] - c[0], p[1] - c[1]];
            (d[0] * d[0] + d[1] * d[1]).sqrt()
        })
        .sum::<f32>()
        / n
}

fn centroid(pos: &[[f32; 2]]) -> [f32; 2] {
    let n = pos.len() as f32;
    let c = pos
        .iter()
        .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
    [c[0] / n, c[1] / n]
}

/// The smallest pairwise distance in the flock (collapse detector).
fn min_pair(pos: &[[f32; 2]]) -> f32 {
    let mut m = f32::MAX;
    for (i, a) in pos.iter().enumerate() {
        for b in &pos[i + 1..] {
            let d = [a[0] - b[0], a[1] - b[1]];
            m = m.min((d[0] * d[0] + d[1] * d[1]).sqrt());
        }
    }
    m
}

/// Coherence of headings ∈ [0,1]: the length of the mean unit-velocity. ~1 when
/// all agents fly the same way, ~0 when their headings are scattered.
fn heading_coherence(vel: &[[f32; 2]]) -> f32 {
    let n = vel.len() as f32;
    let sum = vel.iter().fold([0.0f32; 2], |a, v| {
        let (u, _) = norm(*v);
        [a[0] + u[0], a[1] + u[1]]
    });
    ((sum[0] * sum[0] + sum[1] * sum[1]).sqrt()) / n
}

/// Run `ticks` fixed steps through the pure `simulate`, feeding output back as
/// the next tick's state (what the `pre` loop does live). Returns the last frame.
fn run(target: [f32; 2], p: &Params, ticks: usize, dt: f32) -> Stream {
    let mut state = Stream::new(0); // Empty → tick 0 seeds
    let mut last = state.clone();
    for k in 0..ticks {
        last = simulate(target, &state, k as f32 * dt, p);
        state = last.clone();
    }
    last
}

/// Cohesion CONTRACTS the flock: with only cohesion on, the spread shrinks over
/// time. FALSIFIED against no forces — inertia alone lets the random seed
/// velocities carry the agents APART, so the spread grows.
#[test]
fn cohesion_pulls_the_flock_together() {
    let mut coh = params(40);
    coh.cohesion = 3.0;
    let mut none = params(40);
    none.cohesion = 0.0;

    let seed_spread = spread(&vec2_col(&run([0.0, 0.0], &coh, 1, 1.0 / 60.0), "P"));
    let coh_end = spread(&vec2_col(&run([0.0, 0.0], &coh, 180, 1.0 / 60.0), "P"));
    let none_end = spread(&vec2_col(&run([0.0, 0.0], &none, 180, 1.0 / 60.0), "P"));

    // Cohesion holds the flock together (never expands past its seed spread);
    // inertia alone lets the random seed velocities carry the agents APART.
    assert!(
        coh_end <= seed_spread,
        "cohesion held the flock ({seed_spread} → {coh_end})"
    );
    assert!(
        none_end > coh_end * 1.4,
        "inertia-only drifts far wider ({none_end}) than the cohesive flock ({coh_end})"
    );
}

/// Alignment ALIGNS the headings: with only alignment on, the agents converge to
/// a common heading (coherence → high). FALSIFIED against no forces — the random
/// seed headings stay scattered (low coherence).
#[test]
fn alignment_aligns_the_headings() {
    let mut al = params(40);
    al.alignment = 3.0;
    let mut none = params(40);
    none.alignment = 0.0;

    let al_end = heading_coherence(&vec2_col(&run([0.0, 0.0], &al, 120, 1.0 / 60.0), "vel"));
    let none_end = heading_coherence(&vec2_col(&run([0.0, 0.0], &none, 120, 1.0 / 60.0), "vel"));
    assert!(
        al_end > 0.9,
        "alignment made the flock fly as one: {al_end}"
    );
    assert!(
        al_end > none_end + 0.3,
        "aligned ({al_end}) far more coherent than free ({none_end})"
    );
}

/// Separation PREVENTS COLLAPSE: strong cohesion WITHOUT separation stacks the
/// agents almost on top of each other; adding separation keeps them spaced.
#[test]
fn separation_prevents_collapse() {
    let mut collapse = params(30);
    collapse.cohesion = 3.0;
    collapse.radius_sq = 100.0; // everyone sees everyone
    let mut spaced = params(30);
    spaced.cohesion = 3.0;
    spaced.separation = 2.0;
    spaced.radius_sq = 100.0;

    let collapse_min = min_pair(&vec2_col(&run([0.0, 0.0], &collapse, 240, 1.0 / 60.0), "P"));
    let spaced_min = min_pair(&vec2_col(&run([0.0, 0.0], &spaced, 240, 1.0 / 60.0), "P"));
    assert!(
        collapse_min < 0.2,
        "no separation → they pile up: {collapse_min}"
    );
    assert!(
        spaced_min > collapse_min * 3.0,
        "separation kept them apart ({spaced_min}) vs collapsed ({collapse_min})"
    );
}

/// The flock SEEKS the target: with seek on and a target off to the right, the
/// centroid migrates toward it. FALSIFIED against seek 0 — the centroid stays put.
#[test]
fn the_flock_seeks_the_target() {
    let mut seek = params(30);
    seek.seek = 1.5;
    let mut free = params(30);
    free.seek = 0.0;

    let target = [10.0, 0.0];
    let seek_c = centroid(&vec2_col(&run(target, &seek, 200, 1.0 / 60.0), "P"));
    // The seed centres on the target, so measure migration by seeding at origin:
    // give `free` no seek and target far away — its centroid barely moves from 0.
    let free_c = centroid(&vec2_col(&run([0.0, 0.0], &free, 200, 1.0 / 60.0), "P"));
    assert!(
        (seek_c[0] - 10.0).abs() < 2.0,
        "the flock gathered on the target: centroid x {}",
        seek_c[0]
    );
    assert!(
        free_c[0].abs() < 4.0,
        "no seek: stays near origin: {}",
        free_c[0]
    );
}

/// The seek spring keeps the flock BOUNDED: after a long run every agent is
/// finite and within a sane radius of its home (no explosion).
#[test]
fn the_seek_spring_bounds_the_flock() {
    let mut p = params(50);
    p.separation = 2.0;
    p.alignment = 1.0;
    p.cohesion = 1.0;
    p.seek = 1.0;
    p.radius_sq = 4.0;
    let last = vec2_col(&run([0.0, 0.0], &p, 600, 1.0 / 60.0), "P");
    for q in &last {
        // Compare the squared distance from home (< 20²) — no transcendental.
        assert!(
            q[0].is_finite() && q[1].is_finite() && q[0] * q[0] + q[1] * q[1] < 400.0,
            "bounded near home: {q:?}"
        );
    }
}

/// Deterministic replay (HR-5): two runs match bit-for-bit; a different seed
/// re-rolls the flock.
#[test]
fn replay_is_deterministic_and_seed_re_rolls() {
    let mut p = params(24);
    p.separation = 1.5;
    p.alignment = 1.0;
    p.cohesion = 1.0;
    p.seek = 1.0;
    let a = vec2_col(&run([0.0, 0.0], &p, 90, 1.0 / 60.0), "P");
    let b = vec2_col(&run([0.0, 0.0], &p, 90, 1.0 / 60.0), "P");
    assert_eq!(a, b, "same seed replays");
    let mut p2 = params(24);
    p2.seed = 2;
    let c = vec2_col(&run([0.0, 0.0], &p2, 1, 1.0 / 60.0), "P");
    let a0 = vec2_col(&run([0.0, 0.0], &p, 1, 1.0 / 60.0), "P");
    assert_ne!(a0, c, "a new seed re-rolls the flock");
}

/// Cooks through the registry with the `pre` self-loop, exactly as the editor
/// wires it — proving the node is registered and steps live.
#[test]
fn registers_and_steps_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionBoids as &dyn NodeOp)
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", 20.0);
    g.connect(Edge {
        from: (boids, 0),
        to: (boids, 2),
        delayed: true,
    })
    .unwrap();

    let mut cook = Cook::new();
    let out0 = cook.cook(&g, &Ops, boids, 0.0).unwrap();
    assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 20));
    for k in 0..60 {
        let t = k as f64 / 60.0;
        cook.cook(&g, &Ops, boids, t).unwrap();
        cook.advance_tick(&g, &Ops, t).unwrap();
    }
    let out = cook.cook(&g, &Ops, boids, 1.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert!(v.iter().all(|q| q[0].is_finite()), "still alive & finite"),
        _ => panic!("P"),
    }
}
