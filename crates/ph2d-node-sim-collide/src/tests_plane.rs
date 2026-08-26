//! Guards for the plane's TILT (doc 89, folha 13 P0). `super` is [`super::tests`] — the fixture
//! helpers — and `crate` is the node.
//!
//! The folha refuted both ways of getting a ramp out of the pieces that already existed
//! (`motion.rotate` never moves `P`; a chain of floors is a staircase), so the shape carries its
//! own orientation. These gates hold the four claims that makes: the untilted plane is the floor
//! that shipped, the tilted one catches where the Hesse form says it does, a ramp is a thing a
//! particle can SLIDE down, and the offset stays alive at every angle.

use super::*;
use crate::ui::PARAM_GATES;

/// Where the element ends up after one settle call — the whole product of a contact.
fn settle(p: [f32; 2], v: [f32; 2], angle: f32, offset: f32, fr: f32) -> ([f32; 2], [f32; 2]) {
    let out = collide(
        &one(p, v),
        SHAPE_PLANE,
        offset,
        [0.0, 0.0],
        0.0,
        0.0,
        fr,
        (RADIUS_POINT, 0.0, 0.0),
        plane_normal(angle),
        (0.0, 0),
    );
    read(&out)
}

/// **THE REDUCTION — an untilted plane is the floor that shipped, term for term.**
///
/// The tilt enters as a dot product against a normal, and at `angle = 0` that normal is `(0, 1)`
/// to the bit, so `dot(p, n)` IS `p.y` and the contact test is the literal the node carried
/// before this wave. This gate spells that literal out inline and demands byte equality, over
/// positions that are above, below and exactly on the surface.
///
/// FALSIFIED by a normal that is merely *close* to up: every number below drifts, and every one
/// of the sixteen older gates would drift with it — silently, because they were all written
/// against a floor.
#[test]
fn an_untilted_plane_is_the_floor_that_shipped_before_it() {
    for p in [[0.0, 3.0], [-7.5, -2.0], [4.25, -9.0], [0.0, -2.0]] {
        for v in [[0.0, -4.0], [3.0, -1.5], [0.0, 2.0]] {
            let (gp, gv) = settle(p, v, 0.0, -2.0, 0.35);
            // The floor, verbatim: what the node did before the plane could tilt.
            let (mut fp, mut fv) = (p, v);
            if fp[1] < -2.0 {
                let depth = -2.0 - fp[1];
                respond(&mut fp, &mut fv, [0.0, 1.0], depth, 0.0, 0.35);
            }
            assert_eq!((gp, gv), (fp, fv), "p={p:?} v={v:?}");
        }
    }
}

/// **The four cardinal normals are EXACT, and every normal is a UNIT.**
///
/// The polynomial that resolves the angle is only ~0.09% true trig in general, but it is exact at
/// the quarter turns — which is what makes the reduction above a byte-identity rather than a
/// tolerance. The `sqrt` then pins the length, and the length is what `depth` and the reflection
/// are both measured along: a normal 1% short makes a plane the particle sinks 1% into.
#[test]
fn the_cardinal_normals_are_exact_and_every_normal_is_a_unit() {
    assert_eq!(plane_normal(0.0), [0.0, 1.0], "0 deg is up");
    assert_eq!(plane_normal(90.0), [-1.0, 0.0], "90 deg looks left");
    assert_eq!(plane_normal(180.0), [0.0, -1.0], "180 deg looks down");
    assert_eq!(plane_normal(270.0), [1.0, 0.0], "270 deg looks right");
    // …and -90 is 270: the phase wraps, so a negative tilt is not a special case.
    assert_eq!(plane_normal(-90.0), plane_normal(270.0));

    for k in 0..72 {
        let n = plane_normal(k as f32 * 5.0);
        let len2 = n[0] * n[0] + n[1] * n[1];
        assert!((len2 - 1.0).abs() < 1e-6, "|n|^2 at {}deg = {len2}", k * 5);
    }
}

/// **A tilted plane catches exactly where the Hesse form says it does.**
///
/// The law is `world = { p : dot(p, n) >= offset }`, so an element pushed out of it comes to rest
/// with `dot(p, n) == offset` — one equation for every angle, which is the whole reason the shape
/// has an orientation instead of a special case per idiom.
///
/// FALSIFIED by a depth measured along anything but the normal: the element would land off the
/// surface by the cosine of the difference, and on a shallow ramp that error hides.
#[test]
fn a_tilted_plane_catches_where_the_hesse_form_says_it_does() {
    for angle in [-63.0, -20.0, 0.0, 12.5, 45.0, 118.0] {
        for offset in [-3.0, 0.0, 1.75] {
            let n = plane_normal(angle);
            // Start well inside the wall, moving into it.
            let deep = [n[0] * (offset - 6.0), n[1] * (offset - 6.0)];
            let (p, _) = settle(deep, [-n[0] * 4.0, -n[1] * 4.0], angle, offset, 0.0);
            let sd = p[0] * n[0] + p[1] * n[1];
            assert!(
                (sd - offset).abs() < 2e-5,
                "angle {angle} offset {offset}: rests at dot = {sd}"
            );
        }
    }
}

/// **A RAMP is a thing a particle slides DOWN — and a floor is not.**
///
/// This is the capability the folha said no chain of floors could build: a staircase catches, a
/// ramp *transports*. The fixture is the smallest honest one — gravity into the velocity, then
/// the contact — and the control is the same loop at `angle = 0`, where the particle must not
/// travel sideways at all.
///
/// FALSIFIED by a contact that keeps pushing straight up: the tangential component gravity feeds
/// in would be cancelled every step and the ramp would behave like a floor with a slope drawn on
/// it.
#[test]
fn a_particle_slides_down_a_ramp_and_stands_still_on_a_floor() {
    let run = |angle: f32| -> f32 {
        let (mut p, mut v) = ([0.0f32, 0.0f32], [0.0f32, 0.0f32]);
        for _ in 0..240 {
            v[1] -= 9.8 / 60.0;
            let out = collide(
                &one(p, v),
                SHAPE_PLANE,
                0.0,
                [0.0, 0.0],
                0.0,
                0.0,
                0.05,
                (RADIUS_POINT, 0.0, 0.0),
                plane_normal(angle),
                (0.0, 0),
            );
            let (np, nv) = read(&out);
            (p, v) = (np, nv);
            p[0] += v[0] / 60.0;
            p[1] += v[1] / 60.0;
        }
        p[0]
    };
    // A ramp descending to the RIGHT: the surface tangent is (cos, sin) of a negative angle.
    let downhill = run(-20.0);
    assert!(
        downhill > 2.0,
        "on a 20deg ramp the particle must travel downhill; it reached x = {downhill}"
    );
    // Same loop, same gravity, same friction — and a floor transports nothing.
    let flat_run = run(0.0);
    assert!(
        flat_run.abs() < 1e-4,
        "on a floor there is nowhere downhill to go; x = {flat_run}"
    );
    // And the mirrored ramp carries it the other way, by very nearly the same distance.
    //
    // ⚠️ NOT to the bit, and the reason is the FIXTURE, not the law: a particle sliding with no
    // restitution RIDES the surface, so `sd < offset` is decided at the level of float noise and
    // the two mirrored runs disagree about whether a given tick had a contact at all (measured by
    // `probe_ramp_symmetry`: at k=2 one run reads `sd = -2.6e-3` and its mirror reads `sd = 0`).
    // A missed tick costs one friction bleed, at most `friction * |v_t| / 60`, so over 240 ticks
    // the divergence is bounded near 5% — the bar below is that bound, not a number tuned until
    // it passed. Asserting tighter would be asserting the rounding of a boundary test.
    let uphill = run(20.0);
    assert!(
        uphill < -2.0,
        "the mirrored ramp carries it the OTHER way; x = {uphill}"
    );
    assert!(
        (uphill + downhill).abs() < 0.08 * downhill,
        "…and by nearly the same distance: {uphill} vs {downhill}"
    );
}

/// **At 90 degrees the plane is a WALL, and the offset places it — at every angle.**
///
/// This is the argument for the Hesse offset over the obvious alternative (*"the plane pivots
/// about `(0, height)`"*), which reads better for a small ramp tilt and pins every wall to
/// `x = 0` for ever: at 90 degrees that pivot lies ON the plane, so the knob slides the wall along
/// itself and does nothing. Here it moves the wall, which is the only thing a position control
/// may not fail to do.
#[test]
fn at_ninety_degrees_the_offset_places_a_wall_and_is_never_inert() {
    // n = (-1, 0), so the world is `p.x <= -offset`.
    for (offset, wall_x) in [(-3.0, 3.0), (-4.5, 4.5), (2.0, -2.0)] {
        let (p, v) = settle([wall_x + 4.0, 1.25], [6.0, 0.0], 90.0, offset, 0.0);
        assert!(
            (p[0] - wall_x).abs() < 1e-5,
            "offset {offset} must put the wall at x = {wall_x}; landed at {}",
            p[0]
        );
        assert_eq!(p[1], 1.25, "a wall does not lift what it stops");
        assert!(
            v[0] <= 0.0,
            "the wall took the speed that was going into it"
        );
    }
}

/// **At 180 degrees the plane is a CEILING** — the world is BELOW it, and a rising element is
/// pushed back down. One shape, four idioms, and that is why the enum label says "Plane".
#[test]
fn at_one_eighty_the_plane_is_a_ceiling() {
    // n = (0, -1), so the world is `p.y <= -offset`: offset -4 puts the ceiling at y = 4.
    let (p, v) = settle([0.5, 6.0], [0.0, 3.0], 180.0, -4.0, 0.0);
    assert!(
        (p[1] - 4.0).abs() < 1e-5,
        "held under the ceiling; y = {}",
        p[1]
    );
    assert_eq!(p[0], 0.5, "a ceiling does not shove sideways");
    assert!(
        v[1] <= 0.0,
        "and it took the speed that was going up into it"
    );
}

/// **The tilt and the particle radius COMPOSE** — the resting law becomes `dot(p, n) = offset + r`
/// for every angle, which is the same Minkowski inflation the floor already had, read along the
/// normal instead of along `y`.
#[test]
fn the_tilt_composes_with_the_particle_radius() {
    let angle = 34.0;
    let n = plane_normal(angle);
    let rest = |part: (i32, f32, f32)| -> f32 {
        let out = collide(
            &sized([-6.0, -6.0], [4.0, 4.0], [1.0, 1.0]),
            SHAPE_PLANE,
            0.0,
            [0.0, 0.0],
            0.0,
            0.0,
            0.0,
            part,
            n,
            (0.0, 0),
        );
        let p = read(&out).0;
        p[0] * n[0] + p[1] * n[1]
    };
    assert!(
        rest((RADIUS_POINT, 0.0, 0.0)).abs() < 2e-5,
        "a point rests ON the ramp"
    );
    assert!(
        (rest((RADIUS_SIZE, 0.0, 1.0)) - 0.5).abs() < 2e-5,
        "a unit quad rests its EDGE on the ramp, half a unit out along the normal"
    );
}

/// **A disc and a bowl are blind to the tilt, and that is geometry, not scope.**
///
/// Both are rotationally symmetric, so an angle on them could only ever be a knob that provably
/// changes nothing — which is why it carries a [`PARAM_GATES`] entry rather than a comment.
#[test]
fn a_disc_and_a_bowl_are_blind_to_the_tilt() {
    for shape in [SHAPE_DISC, SHAPE_BOWL] {
        let go = |angle: f32| {
            let out = collide(
                &one([0.6, 0.4], [1.0, -2.0]),
                shape,
                -2.0,
                [0.0, 0.0],
                1.0,
                0.3,
                0.2,
                (RADIUS_POINT, 0.0, 0.0),
                plane_normal(angle),
                (0.0, 0),
            );
            read(&out)
        };
        assert_eq!(go(0.0), go(37.0), "shape {shape} must not see the angle");
        assert_eq!(go(0.0), go(-91.5), "shape {shape} must not see the angle");
    }

    let gated: Vec<_> = PARAM_GATES
        .iter()
        .filter(|g| g.param == "angle")
        .map(|g| (g.when, g.values))
        .collect();
    assert_eq!(
        gated,
        vec![("shape", &[SHAPE_PLANE][..])],
        "the tilt is offered only where it is read"
    );
}

#[test]
#[ignore]
fn probe_ramp_symmetry() {
    for a in [-20.0f32, 20.0] {
        let n = plane_normal(a);
        eprintln!("angle {a}: n = {n:?}");
        let (mut p, mut v) = ([0.0f32, 0.0f32], [0.0f32, 0.0f32]);
        for k in 0..240 {
            v[1] -= 9.8 / 60.0;
            let out = collide(
                &one(p, v),
                SHAPE_PLANE,
                0.0,
                [0.0, 0.0],
                0.0,
                0.0,
                0.05,
                (RADIUS_POINT, 0.0, 0.0),
                n,
                (0.0, 0),
            );
            let (np, nv) = read(&out);
            (p, v) = (np, nv);
            p[0] += v[0] / 60.0;
            p[1] += v[1] / 60.0;
            if k < 4 || k % 60 == 0 {
                eprintln!("  k={k} p={p:?} v={v:?} sd={}", p[0] * n[0] + p[1] * n[1]);
            }
        }
    }
}
