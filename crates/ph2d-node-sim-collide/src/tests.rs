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

/// **The POINT collider** — what this node was before it could know how big anything is.
///
/// Named rather than defaulted, so every fixture below says which collider it means in one
/// token: a premise inherited in silence inverts its meaning the day the default moves, and then
/// stays green while testing the opposite. The gates that DO mean a radius call [`collide`]
/// directly.
#[allow(clippy::too_many_arguments)] // mirrors `collide`, minus the radius
fn collide_pt(
    s: &Stream,
    shape: i32,
    height: f32,
    c: [f32; 2],
    radius: f32,
    restitution: f32,
    friction: f32,
) -> Stream {
    collide(
        s,
        shape,
        height,
        c,
        radius,
        restitution,
        friction,
        (RADIUS_POINT, 0.0, 0.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    )
}

/// The normal of an untilted plane — what every fixture below means by "a floor".
///
/// It is [`plane_normal(0.0)`](plane_normal) and not a literal `[0, 1]`, so these fixtures cannot
/// disagree with the door: if the door ever stopped answering `(0, 1)` at angle 0, every gate
/// here would move with it instead of quietly testing a world the product no longer has.
fn flat() -> [f32; 2] {
    plane_normal(0.0)
}

/// The tilt's own guards live next door — `tests.rs` is the shape/radius file and would cross the
/// LOC cap with them. This is a CHILD, so the fixture helpers above are the same ones.
#[cfg(test)]
#[path = "tests_plane.rs"]
mod plane;

/// Same reason, same shape: the contact channel's guards are their own subject.
#[cfg(test)]
#[path = "tests_hit.rs"]
mod hit;

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
    let out = collide_pt(
        &one([0.0, -3.0], [1.0, -4.0]),
        SHAPE_PLANE,
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
    let dead = collide_pt(
        &one([0.0, -3.0], [0.0, -4.0]),
        SHAPE_PLANE,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.0,
        0.0,
    );
    assert_eq!(read(&dead).1[1], 0.0, "dead: it lands and stays");

    let elastic = collide_pt(
        &one([0.0, -3.0], [0.0, -4.0]),
        SHAPE_PLANE,
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
    let out = collide_pt(
        &one([0.0, -2.5], [10.0, -4.0]),
        SHAPE_PLANE,
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
    let on = collide_pt(
        &one([0.0, -2.0], [3.0, 0.0]),
        SHAPE_PLANE,
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
    let leaving = collide_pt(
        &one([0.0, -2.01], [0.0, 5.0]),
        SHAPE_PLANE,
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
            (SHAPE_PLANE, [0.0, -3.0]),
            (SHAPE_DISC, [0.3, 0.2]),
            (SHAPE_BOWL, [3.0, 1.0]),
        ] {
            let v0 = [2.0, -5.0];
            let out = collide_pt(&one(p, v0), shape, -2.0, [0.0, 0.0], 2.0, e, 0.0);
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
    let disc = collide_pt(
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
    let bowl = collide_pt(
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
    let out = collide_pt(
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
    g.set_param(floor, "shape", SHAPE_PLANE as f32);
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

// ── The particle has a SIZE (doc 89, folha 13 P0) ────────────────────────────

/// A stream of one, with a `size` column — the shape the renderer would draw there.
fn sized(p: [f32; 2], v: [f32; 2], size: [f32; 2]) -> Stream {
    one(p, v).with("size", Column::Vec2(vec![size]))
}

/// **THE DEFECT, with its number.** A collider that collides a point rests an element's CENTRE
/// on the floor, so the sprite the renderer draws there sinks by exactly half its height.
///
/// This gate is the *before* and the *after* in one place: a 1×1 quad on a floor at `y = −2`
/// settles with its bottom edge at **−2.5** as a point, and at **−2.0** — on the floor, which is
/// what a floor means — once the collider knows how big it is.
///
/// FALSIFIED by a radius that does not reach the contact test: the two numbers would be equal,
/// and the sprite would still be half-buried with a control that swears it is not.
#[test]
fn a_point_sinks_by_half_its_height_and_a_disc_rests_on_top() {
    let fall = |part: (i32, f32, f32)| -> f32 {
        // Deep below the floor and moving down: one call settles it exactly onto the surface.
        let out = collide(
            &sized([0.0, -9.0], [0.0, -4.0], [1.0, 1.0]),
            SHAPE_PLANE,
            -2.0,
            [0.0, 0.0],
            0.0,
            0.0,
            0.0,
            part,
            flat(),
            (0.0, 0),
            [0.0, 0.0],
        );
        read(&out).0[1]
    };
    let point = fall((RADIUS_POINT, 0.0, 0.0));
    let disc = fall((RADIUS_SIZE, 0.0, 1.0));
    assert_eq!(point, -2.0, "the point rests its CENTRE on the floor…");
    assert_eq!(
        point - 0.5,
        -2.5,
        "…so the 1x1 sprite drawn there has its bottom edge half a unit UNDER it"
    );
    assert_eq!(disc, -1.5, "with a radius the CENTRE sits half a unit up…");
    assert_eq!(disc - 0.5, -2.0, "…and the bottom edge is ON the floor");
}

/// **The radius is the circle INSIDE the sprite, and it is half the smaller side.**
///
/// The halving is not a convention: `sprite.wgsl` expands a unit quad in `[-0.5, 0.5]` by
/// `size`, so a sprite spans `±size/2`. A collider that used the whole side would hold every
/// sprite a full size-unit off the ground.
///
/// FALSIFIED by `max` instead of `min`: a wide flat sprite (2 × 0.5) would be held at 1.0 above
/// the floor — hovering by three quarters of a unit, with nothing on screen to explain it.
#[test]
fn the_radius_is_the_circle_inscribed_in_the_sprite() {
    assert_eq!(particle_radius(RADIUS_SIZE, 0.0, 1.0, [1.0, 1.0]), 0.5);
    assert_eq!(
        particle_radius(RADIUS_SIZE, 0.0, 1.0, [2.0, 0.5]),
        0.25,
        "a wide flat sprite is caught by its SHORT side, or it hovers"
    );
    assert_eq!(
        particle_radius(RADIUS_SIZE, 0.0, 1.0, [0.5, 2.0]),
        0.25,
        "…and so is a tall thin one: the inscribed circle does not care which way it is long"
    );
    // A mirrored sprite is the same size, not a negative one.
    assert_eq!(particle_radius(RADIUS_SIZE, 0.0, 1.0, [-3.0, 3.0]), 1.5);
    // `size_scale` reaches the circle AROUND a square sprite for whoever wants it.
    let circumscribed = particle_radius(RADIUS_SIZE, 0.0, std::f32::consts::SQRT_2, [2.0, 2.0]);
    assert!((circumscribed - std::f32::consts::SQRT_2).abs() < 1e-6);
    // And it can never come out negative, however the sliders are dragged.
    assert_eq!(particle_radius(RADIUS_SIZE, 0.0, -5.0, [1.0, 1.0]), 0.0);
    assert_eq!(particle_radius(RADIUS_FIXED, -5.0, 1.0, [1.0, 1.0]), 0.0);
}

/// **`Point` is the point collider, to the BIT** — the default, so a document that never touches
/// this control collides exactly what it collided before the control existed.
///
/// The sweep covers all three shapes and a stream that HAS a size column (the case where a
/// mistake would show), because "byte-identical" is only worth asserting where the new code path
/// had something to read.
#[test]
fn the_point_mode_is_the_collider_that_shipped_before_it() {
    for (shape, p) in [
        (SHAPE_PLANE, [0.4, -3.0]),
        (SHAPE_DISC, [0.3, 0.2]),
        (SHAPE_BOWL, [3.0, 1.0]),
    ] {
        let s = sized(p, [2.0, -5.0], [1.7, 0.9]);
        let want = collide_pt(&s, shape, -2.0, [0.0, 0.0], 2.0, 0.4, 0.2);
        // Non-zero fixed radius and scale: `Point` must ignore BOTH, not merely one.
        let got = collide(
            &s,
            shape,
            -2.0,
            [0.0, 0.0],
            2.0,
            0.4,
            0.2,
            (RADIUS_POINT, 1.3, 2.5),
            flat(),
            (0.0, 0),
            [0.0, 0.0],
        );
        assert_eq!(read(&want), read(&got), "shape {shape}");
    }
}

/// **The obstacle GROWS and the container SHRINKS** — the Minkowski inflation, and the sign is
/// the whole content of the sentence. Getting it backwards would let a disc swallow the thing it
/// is supposed to keep out.
#[test]
fn the_disc_grows_by_the_radius_and_the_bowl_shrinks_by_it() {
    // A solid disc of radius 2 at the origin, and a particle of radius 0.5 pressed into it.
    let disc = collide(
        &one([0.0, 1.0], [0.0, -3.0]),
        SHAPE_DISC,
        0.0,
        [0.0, 0.0],
        2.0,
        0.0,
        0.0,
        (RADIUS_FIXED, 0.5, 1.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    );
    assert!(
        (read(&disc).0[1] - 2.5).abs() < 1e-6,
        "its CENTRE ends 2.5 out — the rim plus its own radius: {:?}",
        read(&disc).0
    );

    // The same numbers as a bowl: the wall it may not cross is 0.5 INSIDE the rim.
    let bowl = collide(
        &one([0.0, 5.0], [0.0, 3.0]),
        SHAPE_BOWL,
        0.0,
        [0.0, 0.0],
        2.0,
        0.0,
        0.0,
        (RADIUS_FIXED, 0.5, 1.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    );
    assert!(
        (read(&bowl).0[1] - 1.5).abs() < 1e-6,
        "held at 1.5, not at the rim: {:?}",
        read(&bowl).0
    );
}

/// **A particle wider than its bowl lands in the CENTRE, not through the far wall.**
///
/// `radius − r` goes negative, and an unclamped depth would push the element PAST the centre by
/// the overshoot — a container that ejects what it cannot hold, outward, forever.
#[test]
fn a_particle_too_big_for_its_bowl_collapses_to_the_centre() {
    let out = collide(
        &one([1.0, 0.0], [3.0, 0.0]),
        SHAPE_BOWL,
        0.0,
        [0.0, 0.0],
        0.5,
        0.0,
        0.0,
        (RADIUS_FIXED, 4.0, 1.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    );
    let (p, _) = read(&out);
    assert!(
        p[0].abs() < 1e-6 && p[1].abs() < 1e-6,
        "it belongs at the centre, not shoved out the other side: {p:?}"
    );
}

/// **The radius is per ELEMENT, which is the whole reason a param could not do this job.**
///
/// Three sprites of different sizes fall onto one floor. Their CENTRES come to rest at three
/// different heights, and their BOTTOM EDGES at the same one — which is what "they are sitting
/// on the floor" means, and what no single `height` can produce.
///
/// FALSIFIED by reading the size of element 0 for everybody (the shape of a mistake that a
/// uniform fixture cannot see): the three bottoms would land at three different heights.
#[test]
fn three_sizes_rest_their_bottom_edges_on_the_same_line() {
    let sizes = [[0.4, 0.4], [1.0, 1.0], [2.2, 2.2]];
    let s = Stream::new(3)
        .with(
            "P",
            Column::Vec2(vec![[-1.0, -9.0], [0.0, -9.0], [1.0, -9.0]]),
        )
        .with("vel", Column::Vec2(vec![[0.0, -4.0]; 3]))
        .with("size", Column::Vec2(sizes.to_vec()));
    let out = collide(
        &s,
        SHAPE_PLANE,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.0,
        0.0,
        (RADIUS_SIZE, 0.0, 1.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    );
    let Some(Column::Vec2(p)) = out.get("P") else {
        panic!("no `P`")
    };
    let centres: Vec<f32> = p.iter().map(|q| q[1]).collect();
    // Three DIFFERENT heights — the thing a single `height` param cannot produce. (Approximate
    // because the settle is `-9 + depth` in f32; the claim is the geometry, not the last ulp.)
    for (got, want) in centres.iter().zip([-1.8, -1.5, -0.9]) {
        assert!((got - want).abs() < 1e-5, "centres {centres:?}");
    }
    for (i, (c, sz)) in centres.iter().zip(sizes).enumerate() {
        assert!(
            (c - sz[1] * 0.5 + 2.0).abs() < 1e-6,
            "element {i}: bottom edge at {}, not on the floor",
            c - sz[1] * 0.5
        );
    }
}

/// **An absent `size` column is a UNIT quad on both paths** — `SIZE_IDENTITY`, which is also the
/// shell's `default_size`, so it is literally the quad the renderer draws there.
///
/// FALSIFIED by falling back to zero: `Sprite Size` would silently be `Point` for every stream
/// that never passed through a sizing node — a mode that quietly does nothing.
#[test]
fn an_absent_size_column_is_the_unit_quad_the_renderer_draws() {
    let out = collide(
        &one([0.0, -9.0], [0.0, -4.0]),
        SHAPE_PLANE,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.0,
        0.0,
        (RADIUS_SIZE, 0.0, 1.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    );
    assert_eq!(
        read(&out).0[1],
        -1.5,
        "half of SIZE_IDENTITY = 0.5 above the floor"
    );
}

/// **A resting sprite still does not jitter.** The radius moves WHERE the contact is, never how
/// the response works — the `vn >= 0` guard is untouched, and this is the gate that says so on
/// the new geometry.
#[test]
fn a_resting_sized_particle_does_not_jitter_either() {
    // Exactly at rest on the floor for its size, sliding along it.
    let on = collide(
        &sized([0.0, -1.5], [3.0, 0.0], [1.0, 1.0]),
        SHAPE_PLANE,
        -2.0,
        [0.0, 0.0],
        0.0,
        0.8,
        0.0,
        (RADIUS_SIZE, 0.0, 1.0),
        flat(),
        (0.0, 0),
        [0.0, 0.0],
    );
    assert_eq!(
        read(&on),
        ([0.0, -1.5], [3.0, 0.0]),
        "touching the floor at its own radius must change nothing"
    );
}
