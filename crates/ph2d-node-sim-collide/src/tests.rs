//! Guards for `sim.collide` (O4 world collision, doc 52). `super` is the crate root.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn one(p: [f32; 2], v: [f32; 2]) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![p]))
        .with("vel", Column::Vec2(vec![v]))
}

fn read(s: &Stream) -> ([f32; 2], [f32; 2]) {
    let g = |name| match s.get(name) {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("no `{name}`"),
    };
    (g("P"), g("vel"))
}

/// **A collision is a BOUNCE, not a shove** — and the bounce is the thing only a zone makes
/// possible, because only a zone has a velocity to reflect.
///
/// The element is placed BELOW the floor, moving down. It comes out exactly ON the floor, moving
/// UP, at `restitution` of the speed it arrived with.
///
/// FALSIFIED by a collider that only clamps the position (the classic "push out of the wall"):
/// the velocity would still point down, so the next tick would drive it straight back in and it
/// would ooze through the floor, or grind along it, forever.
#[test]
fn a_particle_hitting_the_floor_comes_back_up() {
    let out = collide(
        &one([0.0, -3.0], [1.0, -4.0]),
        SHAPE_FLOOR,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.5,
        0.0,
    );
    let (p, v) = read(&out);
    assert_eq!(p, [0.0, -2.0], "out of the wall, exactly onto its surface");
    assert_eq!(v[1], 2.0, "…and coming UP at half the speed it arrived");
    assert_eq!(v[0], 1.0, "frictionless: the sliding speed is untouched");
}

/// **Restitution 0 sticks; restitution 1 gives it all back.** The two ends of the one parameter
/// an artist actually feels.
#[test]
fn restitution_spans_dead_to_perfectly_elastic() {
    let dead = collide(
        &one([0.0, -3.0], [0.0, -4.0]),
        SHAPE_FLOOR,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.0,
        0.0,
    );
    assert_eq!(read(&dead).1[1], 0.0, "dead: it lands and stays");

    let elastic = collide(
        &one([0.0, -3.0], [0.0, -4.0]),
        SHAPE_FLOOR,
        -2.0,
        [0.0, 0.0],
        0.0,
        1.0,
        0.0,
    );
    assert_eq!(read(&elastic).1[1], 4.0, "elastic: it comes back as fast");
}

/// **Friction eats the SLIDING speed, not the bounce.** A surface that ate both would be a
/// surface that stops a particle skimming along it — which is a wall, not a floor.
#[test]
fn friction_bleeds_the_tangential_speed_only() {
    let out = collide(
        &one([0.0, -2.5], [10.0, -4.0]),
        SHAPE_FLOOR,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.5,
        0.25,
    );
    let (_, v) = read(&out);
    assert_eq!(v[0], 7.5, "a quarter of the slide is eaten");
    assert_eq!(v[1], 2.0, "…and the bounce is untouched by it");
}

/// **A particle already resting on the floor is left alone.** It is not penetrating and it is not
/// moving into the surface, so touching it must change nothing.
///
/// FALSIFIED by reflecting on every contact regardless of direction: the classic collider
/// JITTER — the element buzzes on the ground forever, fed by its own contact test, and the pile
/// of settled particles boils.
#[test]
fn a_resting_particle_does_not_jitter() {
    // Exactly on the surface, sliding along it.
    let on = collide(
        &one([0.0, -2.0], [3.0, 0.0]),
        SHAPE_FLOOR,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.8,
        0.0,
    );
    assert_eq!(
        read(&on),
        ([0.0, -2.0], [3.0, 0.0]),
        "no contact, no change"
    );

    // Barely inside, but moving AWAY (it has already bounced): pushed out, never re-reflected.
    let leaving = collide(
        &one([0.0, -2.01], [0.0, 5.0]),
        SHAPE_FLOOR,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.8,
        0.0,
    );
    let (p, v) = read(&leaving);
    assert!((p[1] + 2.0).abs() < 1e-6, "lifted onto the surface");
    assert_eq!(v[1], 5.0, "…and its escape was NOT reversed");
}

/// **A collision never returns more energy than it took** — at any restitution, on any shape. A
/// bounce that gained speed is a machine for making energy, and the scene ends up in orbit.
#[test]
fn a_bounce_never_gains_speed() {
    for e in [0.0, 0.25, 0.5, 1.0] {
        for (shape, p) in [
            (SHAPE_FLOOR, [0.0, -3.0]),
            (SHAPE_DISC, [0.3, 0.2]),
            (SHAPE_BOWL, [3.0, 1.0]),
        ] {
            let v0 = [2.0, -5.0];
            let out = collide(&one(p, v0), shape, -2.0, [0.0, 0.0], 2.0, e, 0.0);
            let (_, v) = read(&out);
            let speed = |q: [f32; 2]| (q[0] * q[0] + q[1] * q[1]).sqrt();
            assert!(
                speed(v) <= speed(v0) + 1e-4,
                "shape {shape}, e {e}: {v0:?} -> {v:?} GAINED speed"
            );
        }
    }
}

/// The disc is an OBSTACLE (the world is outside it) and the bowl is a CONTAINER (the world is
/// inside it) — the same contact, the normal flipped. One response, three shapes.
#[test]
fn the_disc_pushes_out_and_the_bowl_holds_in() {
    // Falling into a solid disc of radius 2 at the origin: pushed back out to its rim.
    let disc = collide(
        &one([0.0, 1.0], [0.0, -3.0]),
        SHAPE_DISC,
        0.0,
        [0.0, 0.0],
        2.0,
        0.5,
        0.0,
    );
    let (p, v) = read(&disc);
    assert!((p[1] - 2.0).abs() < 1e-5, "back out to the rim: {p:?}");
    assert_eq!(v[1], 1.5, "…and thrown back up (half of 3)");

    // Escaping a bowl of radius 2: caught at the rim, thrown back inwards.
    let bowl = collide(
        &one([0.0, 3.0], [0.0, 4.0]),
        SHAPE_BOWL,
        0.0,
        [0.0, 0.0],
        2.0,
        0.5,
        0.0,
    );
    let (p, v) = read(&bowl);
    assert!((p[1] - 2.0).abs() < 1e-5, "caught at the rim: {p:?}");
    assert_eq!(v[1], -2.0, "…and turned back inwards");
}

/// Dead centre of a solid disc there is no "way out" — every direction is as good. It must pick
/// one, not divide by zero and turn the element into a NaN that poisons the whole state.
#[test]
fn the_centre_of_a_disc_is_not_a_nan() {
    let out = collide(
        &one([0.0, 0.0], [0.0, 0.0]),
        SHAPE_DISC,
        0.0,
        [0.0, 0.0],
        2.0,
        0.5,
        0.0,
    );
    let (p, v) = read(&out);
    assert!(p.iter().chain(&v).all(|x| x.is_finite()), "{p:?} {v:?}");
}

/// **In a zone, through the real registry**: falling elements PILE UP on the floor instead of
/// falling through it, and they stay there.
#[test]
fn a_zone_with_a_floor_stops_its_particles_falling_through() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");

    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 2.0);
    g.set_param(seed, "cols", 3.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 270.0); // gravity
    g.set_param(wind, "strength", 6.0);
    g.set_param(wind, "gust", 0.0);
    let step = g.add_node("sim.step");
    let floor = g.add_node("sim.collide");
    g.set_param(floor, "shape", SHAPE_FLOOR as f32);
    g.set_param(floor, "height", -2.0);
    g.set_param(floor, "restitution", 0.2);
    for (from, fp, to, tp, delayed) in [
        (seed, 0u16, zone, 0u16, false),
        (zone, 0, wind, 0, true), // the state entry
        (wind, 0, step, 0, false),
        (step, 0, floor, 0, false),
        (floor, 0, zone, 1, false),
    ] {
        g.connect(Edge {
            from: (NodeId(from.0), fp),
            to: (NodeId(to.0), tp),
            delayed,
        })
        .expect("wire");
    }
    assert!(g.validate(&reg).is_ok());

    let mut cook = Cook::new();
    let mut lowest = f32::MAX;
    for k in 0..300u64 {
        let t = k as f64 / 60.0;
        let out = cook.cook(&g, &reg, zone, t).expect("cooks");
        if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
            lowest = lowest.min(p.iter().map(|q| q[1]).fold(f32::MAX, f32::min));
        }
        cook.advance_tick(&g, &reg, t).expect("tick");
    }
    assert!(
        lowest >= -2.001,
        "five seconds of gravity and nothing fell through the floor: lowest {lowest}"
    );
    assert!(
        lowest <= -1.99,
        "…and they DID land on it (they are resting at -2, not floating): {lowest}"
    );
}
