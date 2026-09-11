//! Force zones — an area that pushes what is inside it (ADR-0131 W-Area).
//!
//! ⚠️ **Every fixture sweeps the spawn order.** The zone and the body it pushes
//! reach the narrow phase as an unordered pair, and the module picks "the other
//! collider" by comparing against the zone's own handle. A fixture that always
//! spawned the zone first would leave the other branch unproven — which is
//! precisely how the one-way platform's sign flip survived three gates one wave
//! ago. Ordering is a parameter here from the start.

use ph2d_physics::{
    AreaEffect, BodyDesc, LayerMatrix, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc,
};

/// A body with everything neutral — the fixtures override the few fields they
/// are about, so a new `BodyDesc` field lands in ONE place.
fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A static sensor box carrying a force — the zone itself. Drag gets its own helper
/// below, so the force gates read without a `0.0` nobody is testing.
fn zone(x: f32, y: f32, half_x: f32, half_y: f32, force: [f32; 2]) -> BodyDesc {
    zone_with(x, y, half_x, half_y, force, 0.0)
}

/// The general zone: a force, a drag, or both.
fn zone_with(x: f32, y: f32, half_x: f32, half_y: f32, force: [f32; 2], drag: f32) -> BodyDesc {
    zone_full(x, y, half_x, half_y, force, drag, 0.0)
}

/// The fullest zone: force, drag and fluid density.
fn zone_full(
    x: f32,
    y: f32,
    half_x: f32,
    half_y: f32,
    force: [f32; 2],
    drag: f32,
    density: f32,
) -> BodyDesc {
    BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force,
            drag,
            density,
            form_drag: 0.0,
            torque: 0.0,
            world_axes: false,
            falloff: 0.0,
            mirror: [1.0, 1.0],
        }),
        ..desc(
            RigidBodyType::Fixed,
            x,
            y,
            ShapeDesc::Cuboid { half_x, half_y },
        )
    }
}

fn ball(x: f32, y: f32, density: f32) -> BodyDesc {
    BodyDesc {
        density,
        ..desc(
            RigidBodyType::Dynamic,
            x,
            y,
            ShapeDesc::Ball { radius: 0.25 },
        )
    }
}

fn x_of(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.body_pose(h).expect("body alive").translation.x
}

fn vx_of(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).expect("body alive").linvel().x
}

/// A static sensor box carrying a TORQUE and nothing else — the spin zone
/// (W-AreaTorque). Force/drag/density are all zero so the torque gates read without a
/// stray field.
fn torque_zone(x: f32, y: f32, half_x: f32, half_y: f32, torque: f32) -> BodyDesc {
    BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 0.0,
            density: 0.0,
            form_drag: 0.0,
            torque,
            world_axes: false,
            falloff: 0.0,
            mirror: [1.0, 1.0],
        }),
        ..desc(
            RigidBodyType::Fixed,
            x,
            y,
            ShapeDesc::Cuboid { half_x, half_y },
        )
    }
}

/// A dynamic box of the given half-extents at `(x, y)`. Its MOMENT OF INERTIA is a
/// function of those extents, which is the whole point of the inertia gate.
fn box_body(x: f32, y: f32, half_x: f32, half_y: f32) -> BodyDesc {
    desc(
        RigidBodyType::Dynamic,
        x,
        y,
        ShapeDesc::Cuboid { half_x, half_y },
    )
}

fn w_of(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).expect("body alive").angvel()
}

/// Spawn a zone and one ball in the requested order, so both branches of "which
/// collider of the pair is the zone" are exercised.
fn zone_and_ball(
    w: &mut PhysicsWorld,
    z: BodyDesc,
    b: BodyDesc,
    zone_first: bool,
) -> (RigidBodyHandle, RigidBodyHandle) {
    if zone_first {
        let zh = w.spawn_body(z);
        (zh, w.spawn_body(b))
    } else {
        let bh = w.spawn_body(b);
        (w.spawn_body(z), bh)
    }
}

#[test]
fn a_body_inside_the_zone_is_pushed_and_one_outside_is_not() {
    for zone_first in [true, false] {
        let mut w = PhysicsWorld::new();
        // Zero gravity so the only thing acting on either ball is the zone.
        w.set_gravity(0.0, 0.0);
        let (_z, inside) = zone_and_ball(
            &mut w,
            zone(0.0, 0.0, 2.0, 2.0, [2.0, 0.0]),
            ball(0.0, 0.0, 1.0),
            zone_first,
        );
        // ⚠️ The control sits ABOVE the zone, not beside it. Placed downrange it
        // was rammed by the very ball the zone had just launched (12.9 m from a
        // body that "must not move") — the fixture, not the product: a control
        // has to be out of the experiment's way.
        let outside = w.spawn_body(ball(0.0, 6.0, 1.0));
        for _ in 0..60 {
            w.step();
        }
        assert!(
            x_of(&w, inside) > 0.5,
            "zone_first={zone_first}: a body inside the zone should be carried, x={}",
            x_of(&w, inside)
        );
        assert!(
            x_of(&w, outside).abs() < 1e-6,
            "zone_first={zone_first}: a body outside must not move, x={}",
            x_of(&w, outside)
        );
    }
}

#[test]
fn the_zone_pushes_with_a_force_so_mass_resists_it() {
    // ⚠️ The gate that says this is a FORCE and not an acceleration. `a = F/m`,
    // so over the same time the light body travels in proportion to `1/m`: 4×
    // the density is 4× the mass is a quarter of the distance. Applying the
    // force as an acceleration (dividing out the mass) makes both travel the
    // SAME distance, and this is the only gate that can see the difference.
    for zone_first in [true, false] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (_z, light) = zone_and_ball(
            &mut w,
            zone(0.0, 0.0, 8.0, 4.0, [2.0, 0.0]),
            ball(0.0, -1.5, 1.0),
            zone_first,
        );
        // Same radius, 4× the density — so 4× the mass, in the same zone.
        let heavy = w.spawn_body(ball(0.0, 1.5, 4.0));
        for _ in 0..60 {
            w.step();
        }
        let (dl, dh) = (x_of(&w, light), x_of(&w, heavy));
        assert!(dl > 0.1 && dh > 0.0, "both must move: {dl} / {dh}");
        let ratio = dl / dh;
        assert!(
            (ratio - 4.0).abs() < 0.2,
            "zone_first={zone_first}: 4x the mass should travel a quarter as far \
             (ratio ~4), got {ratio} ({dl} / {dh})"
        );
    }
}

#[test]
fn a_torque_zone_spins_a_body_inside_it_and_one_outside_is_still() {
    // The rotational sibling of the push gate. A body inside a torque zone spins up;
    // an identical body outside it does not. Mutation: dropping `apply_torque_impulse`
    // leaves the inside body at zero angular velocity, which this asserts against.
    for zone_first in [true, false] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (_z, inside) = zone_and_ball(
            &mut w,
            torque_zone(0.0, 0.0, 2.0, 2.0, 4.0),
            box_body(0.0, 0.0, 0.3, 0.3),
            zone_first,
        );
        // Control ABOVE the zone (out of the way), never beside — the spinner does not
        // travel, but keeping the convention makes the pair of gates read the same.
        let outside = w.spawn_body(box_body(0.0, 6.0, 0.3, 0.3));
        for _ in 0..60 {
            w.step();
        }
        assert!(
            w_of(&w, inside) > 0.5,
            "zone_first={zone_first}: a body inside a +torque zone should spin up (w>0), \
             but w={} -- the zone is not applying its torque",
            w_of(&w, inside)
        );
        assert!(
            w_of(&w, outside).abs() < 1e-6,
            "zone_first={zone_first}: a body outside must not spin, w={}",
            w_of(&w, outside)
        );
    }
}

#[test]
fn the_torque_sign_sets_the_spin_direction() {
    // The sign IS the direction, which is why the neutral is `== 0` and not `<= 0`: a
    // negative torque is clockwise, a real thing, not an invalid value. Mutation:
    // clamping the torque to `>= 0` (as the drag knobs are clamped) makes the CW case
    // stand still, and this is the gate that sees it.
    let spin = |torque: f32| {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        w.spawn_body(torque_zone(0.0, 0.0, 2.0, 2.0, torque));
        let b = w.spawn_body(box_body(0.0, 0.0, 0.3, 0.3));
        for _ in 0..60 {
            w.step();
        }
        w_of(&w, b)
    };
    let (ccw, cw) = (spin(4.0), spin(-4.0));
    assert!(
        ccw > 0.5,
        "a positive torque spins counter-clockwise (w>0), got {ccw}"
    );
    assert!(
        cw < -0.5,
        "a negative torque spins clockwise (w<0), got {cw}"
    );
    assert!(
        (ccw + cw).abs() < 1e-4,
        "the two directions must be mirror images (|+w| == |-w|), got {ccw} and {cw}"
    );
}

#[test]
fn the_zone_spins_with_a_torque_so_the_moment_of_inertia_resists_it() {
    // ⚠️ The gate that says this is a TORQUE and not an angular acceleration — the exact
    // mirror of `the_zone_pushes_with_a_force_so_mass_resists_it`. A long bar has a far
    // larger moment of inertia than a compact box of the SAME mass, so the same torque
    // spins it up much less. Applying the torque as an acceleration (dividing out the
    // inertia) makes both reach the same angular velocity, and only this gate can see it.
    //
    // Same area (=> same mass at the same density): the compact box is 1x1 and the bar is
    // 4x0.25. Box I = m(1+1)/12; bar I = m(16+0.0625)/12 ~= 8x the box's — so the bar
    // spins up roughly 8x slower.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    w.spawn_body(torque_zone(0.0, 0.0, 6.0, 6.0, 4.0));
    let compact = w.spawn_body(box_body(-2.0, 0.0, 0.5, 0.5));
    let bar = w.spawn_body(box_body(2.0, 0.0, 2.0, 0.125));
    for _ in 0..60 {
        w.step();
    }
    let (wc, wb) = (w_of(&w, compact), w_of(&w, bar));
    assert!(
        wc > 0.1 && wb > 0.0,
        "both must spin: compact {wc} / bar {wb}"
    );
    assert!(
        wc > wb * 4.0,
        "the compact box (small inertia) must spin up far faster than the long bar \
         (large inertia) under the same torque -- got {wc} vs {wb}. If they are close, \
         the torque is being applied as an ACCELERATION, not a torque"
    );
}

#[test]
fn a_solid_torque_zone_spins_nothing() {
    // The sensor coupling, at the torque layer: a solid collider records no intersection,
    // so it has nobody to spin (and it blocks instead). Mirrors `a_solid_zone_pushes_
    // nothing`; the §11 row offers Torque only for a sensor for the same reason.
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let mut z = torque_zone(0.0, 0.0, 2.0, 2.0, 8.0);
    z.is_sensor = false;
    w.spawn_body(z);
    // Beside the solid box, not on it, so this measures the ABSENCE of a spin, not a
    // collision.
    let b = w.spawn_body(box_body(3.0, 0.0, 0.3, 0.3));
    for _ in 0..60 {
        w.step();
    }
    assert!(
        w_of(&w, b).abs() < 1e-6,
        "a SOLID torque zone must spin nothing, w={}",
        w_of(&w, b)
    );
}

/// A ball dropped on a static floor and left alone until it **falls asleep** —
/// the fixture the two sleep-facing gates share.
///
/// ⚠️ It has to sleep BEFORE the zone exists. The first version of this fixture
/// spawned the zone up front, so the ball was pushed from tick one and never slept
/// — and both mutations these gates exist to kill (a zero force still registering,
/// an impulse that does not wake) passed cleanly. A fixture that does not contain
/// the phenomenon proves nothing about it.
fn settled_ball() -> (PhysicsWorld, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    w.spawn_body(desc(
        RigidBodyType::Fixed,
        0.0,
        -1.0,
        ShapeDesc::Cuboid {
            half_x: 20.0,
            half_y: 0.5,
        },
    ));
    let b = w.spawn_body(ball(0.0, -0.2, 1.0));
    for _ in 0..200 {
        w.step();
    }
    assert!(
        w.bodies().get(b).expect("ball").is_sleeping(),
        "the fixture's premise: the ball must be ASLEEP before the zone arrives"
    );
    (w, b)
}

#[test]
fn the_zone_starts_a_body_that_had_already_fallen_asleep() {
    // The reason the impulse passes `wake_up: true`. A body that has settled and
    // fallen asleep is the MAIN case for an updraft or a conveyor: if the zone
    // cannot start it, the feature is broken exactly where an artist would use it.
    // ⚠️ rapier does not integrate a sleeping body, so an impulse that changes its
    // velocity without waking it moves nothing, forever.
    let (mut w, b) = settled_ball();
    w.spawn_body(zone(0.0, 0.0, 6.0, 3.0, [30.0, 0.0]));
    for _ in 0..60 {
        w.step();
    }
    assert!(
        !w.bodies().get(b).expect("ball").is_sleeping(),
        "a body being pushed must not be asleep"
    );
    assert!(
        x_of(&w, b) > 0.5,
        "the zone should have started the sleeping body, x={}",
        x_of(&w, b)
    );
}

#[test]
fn a_zero_force_zone_is_inert_and_lets_the_body_stay_asleep() {
    // ⚠️ The gate on the ZERO guard in `zone_force`. Registering a zero-force zone
    // would apply a zero impulse — arithmetically nothing — but `apply_impulse`
    // also WAKES the body, so the sleeping ball would be roused every substep and
    // never sleep again. Byte identity is asserted against the same world with no
    // effector at all, which is the world that existed before this module.
    let (mut zero, b) = settled_ball();
    let (mut none, _) = settled_ball();
    zero.spawn_body(zone(0.0, 0.0, 6.0, 3.0, [0.0, 0.0]));
    let mut inert = zone(0.0, 0.0, 6.0, 3.0, [0.0, 0.0]);
    inert.effector = None;
    none.spawn_body(inert);
    for _ in 0..120 {
        zero.step();
        none.step();
    }
    assert!(
        zero.bodies().get(b).expect("ball").is_sleeping(),
        "a zero-force zone must not keep the body awake"
    );
    assert_eq!(
        zero.deterministic_hash(),
        none.deterministic_hash(),
        "a zero-force zone must be byte-identical to no zone at all"
    );
}

#[test]
fn the_zone_stops_pushing_once_the_body_leaves_it() {
    // The overlap is asked EVERY substep, not once at spawn: a body that has left
    // the area coasts at the speed it carried out. With no gravity and no drag,
    // "no longer pushed" is exactly "velocity stopped changing".
    for zone_first in [true, false] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (_z, b) = zone_and_ball(
            &mut w,
            zone(0.0, 0.0, 1.0, 1.0, [20.0, 0.0]),
            ball(-0.9, 0.0, 1.0),
            zone_first,
        );
        for _ in 0..120 {
            w.step();
        }
        let (x1, v1) = (x_of(&w, b), vx_of(&w, b));
        assert!(
            x1 > 1.5,
            "zone_first={zone_first}: the ball should have left the zone, x={x1}"
        );
        for _ in 0..60 {
            w.step();
        }
        assert!(
            (vx_of(&w, b) - v1).abs() < 1e-4,
            "zone_first={zone_first}: outside the zone the speed must not change, \
             {v1} -> {}",
            vx_of(&w, b)
        );
    }
}

#[test]
fn the_zone_pushes_what_overlaps_its_shape_not_its_bounding_box() {
    // ⚠️ The narrow phase reports a pair as soon as the **bounding volumes** touch and
    // says separately whether the SHAPES do. For a box zone the two coincide, which is
    // why every fixture above stays green with that distinction deleted — the fixtures
    // did not contain the phenomenon. A ROUND zone does: a body parked at the corner of
    // the circle's AABB is a full 0.34 r outside the circle itself, and a wind that
    // blew on it would be blowing outside its own column.
    for zone_first in [true, false] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let round = BodyDesc {
            is_sensor: true,
            effector: Some(AreaEffect {
                force: [5.0, 0.0],
                drag: 0.0,
                density: 0.0,
                form_drag: 0.0,
                torque: 0.0,
                world_axes: false,
                falloff: 0.0,
                mirror: [1.0, 1.0],
            }),
            ..desc(
                RigidBodyType::Fixed,
                0.0,
                0.0,
                ShapeDesc::Ball { radius: 2.0 },
            )
        };
        // Dead centre: inside both the circle and its box.
        let (_z, inside) = zone_and_ball(&mut w, round, ball(0.0, 0.0, 1.0), zone_first);
        // The AABB corner: |(1.9, 1.9)| = 2.69 > 2.0 + 0.25, so it is outside the
        // circle by more than its own radius while sitting well inside the box.
        let corner = w.spawn_body(ball(1.9, 1.9, 1.0));
        for _ in 0..60 {
            w.step();
        }
        assert!(
            x_of(&w, inside) > 0.5,
            "zone_first={zone_first}: the body at the centre must be pushed"
        );
        assert!(
            (x_of(&w, corner) - 1.9).abs() < 1e-6,
            "zone_first={zone_first}: a body outside the ROUND zone but inside its \
             bounding box must not be pushed, x={}",
            x_of(&w, corner)
        );
    }
}

#[test]
fn the_collision_layer_matrix_decides_who_the_zone_can_touch() {
    // **The per-object answer to "does THIS body feel THIS area?"** — and it is not a
    // new knob: the world's layer matrix already filters the narrow phase, and a force
    // zone is read from the narrow phase, so a body on a layer the zone's layer does not
    // collide with is invisible to it. (Unity spells the same rule as the effector's own
    // `colliderMask`; here it is the matrix the artist already authored in the Physics
    // panel, so there is no second place to say who interacts with whom.)
    //
    // Measured, not assumed — and gated, because it is a promise the product now makes:
    // blocked, the ball does not move by a single float; the same-layer control in the
    // SAME run is carried 8.5 m, which is what proves the zone was alive at all.
    for blocked in [false, true] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let mut matrix = LayerMatrix::all();
        if blocked {
            matrix.set(0, 1, false);
        }
        w.set_layer_matrix(matrix);

        // The zone is on layer 0 (the default).
        w.spawn_body(zone(0.0, 0.0, 2.0, 2.0, [5.0, 0.0]));
        let other_layer = w.spawn_body(BodyDesc {
            layer: 1,
            ..ball(0.0, 0.0, 1.0)
        });
        let same_layer = w.spawn_body(ball(0.0, 1.0, 1.0));
        for _ in 0..60 {
            w.step();
        }

        assert!(
            x_of(&w, same_layer) > 0.5,
            "blocked={blocked}: the same-layer control must be carried — without it a \
             zone that pushed NOBODY would pass the half below"
        );
        if blocked {
            assert_eq!(
                x_of(&w, other_layer),
                0.0,
                "a body whose layer does not collide with the zone's must be untouched"
            );
        } else {
            assert!(
                x_of(&w, other_layer) > 0.5,
                "with the matrix open, layer 1 is carried like layer 0"
            );
        }
    }
}

#[test]
fn a_drag_zone_slows_what_falls_through_it() {
    // The other half of a medium: a force zone with no drag is a vacuum that blows.
    // Falling the SAME distance through the same air, the body that crossed the pool
    // arrives slower — and the control proves the fall itself is not what slowed it.
    let speeds: Vec<f32> = [0.0f32, 4.0]
        .iter()
        .map(|&drag| {
            let mut w = PhysicsWorld::new();
            w.spawn_body(zone_with(0.0, 0.0, 2.0, 1.5, [0.0, 0.0], drag));
            let b = w.spawn_body(ball(0.0, 3.0, 1.0));
            for _ in 0..60 {
                w.step();
            }
            -w.bodies().get(b).expect("ball").linvel().y
        })
        .collect();
    assert!(
        speeds[0] > 5.0,
        "the control must be falling fast by now ({} m/s)",
        speeds[0]
    );
    assert!(
        speeds[1] < speeds[0] * 0.6,
        "a body that fell through a drag zone should be markedly slower ({} vs {})",
        speeds[1],
        speeds[0]
    );
}

#[test]
fn the_zone_drag_is_the_world_drag_law_off_by_exactly_one_substep() {
    // ⚠️ "Drag" must mean ONE thing in this app: the world default
    // (`BodyDefaults::linear_damping`), the per-body override and this all use
    // rapier's `v /= 1 + d·dt` (its own `apply_damping`, verified in the source).
    //
    // They do NOT come out bit-identical, and the reason is worth pinning rather than
    // hiding under a tolerance. rapier applies damping deep inside the velocity
    // solver, just before integrating positions; a zone applies it at the top of the
    // substep, and it can only apply it to bodies the PREVIOUS substep's narrow phase
    // reported as overlapping — the one-substep lag this module documents. So the zone
    // performs exactly **one fewer** decay over the same run.
    //
    // Measured at d = 3, launch 6 m/s, 30 ticks: world 1.3512776, zone 1.3681686 —
    // a ratio of 1.0125, which is exactly `1 + d·dt_substep`. This gate asserts that
    // number, so a change in WHERE the drag is applied shows up as a named quantity
    // instead of a mysterious 1%.
    let launch = 6.0f32;
    let coeff = 3.0f32;

    let mut by_world = PhysicsWorld::new();
    by_world.set_gravity(0.0, 0.0);
    by_world.set_body_defaults(ph2d_physics::BodyDefaults {
        linear_damping: coeff,
        ..by_world.body_defaults()
    });
    let a = by_world.spawn_body(BodyDesc {
        linvel: [launch, 0.0],
        ..ball(0.0, 0.0, 1.0)
    });

    let mut by_zone = PhysicsWorld::new();
    by_zone.set_gravity(0.0, 0.0);
    // Big enough that the body never leaves it over the run.
    by_zone.spawn_body(zone_with(0.0, 0.0, 60.0, 5.0, [0.0, 0.0], coeff));
    let b = by_zone.spawn_body(BodyDesc {
        linvel: [launch, 0.0],
        ..ball(0.0, 0.0, 1.0)
    });

    for _ in 0..30 {
        by_world.step();
        by_zone.step();
    }
    let (vw, vz) = (vx_of(&by_world, a), vx_of(&by_zone, b));
    let one_substep = 1.0 + coeff * by_zone.substep_dt();
    assert!(
        (vz / vw - one_substep).abs() < 0.002,
        "the zone drag should be the world drag law short exactly one substep \
         (ratio {one_substep}), got {} ({vz} vs {vw})",
        vz / vw
    );
}

#[test]
fn the_drag_resists_a_spin_too() {
    // A medium resists rotation, and that is why one knob damps both: syrup that let
    // a coin spin freely would not read as syrup. (Godot exposes the two separately
    // on an Area2D; the per-BODY override is where the asymmetric case lives here.)
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    w.spawn_body(zone_with(0.0, 0.0, 4.0, 4.0, [0.0, 0.0], 4.0));
    let spinner = w.spawn_body(BodyDesc {
        angvel: 12.0,
        ..ball(0.0, 0.0, 1.0)
    });
    // The control sits outside the pool with the identical spin.
    let free = w.spawn_body(BodyDesc {
        angvel: 12.0,
        ..ball(10.0, 0.0, 1.0)
    });
    for _ in 0..60 {
        w.step();
    }
    let inside = w.bodies().get(spinner).expect("spinner").angvel();
    let outside = w.bodies().get(free).expect("free").angvel();
    assert!(
        (outside - 12.0).abs() < 1e-3,
        "the control outside the pool keeps its spin ({outside})"
    );
    assert!(
        inside < outside * 0.3,
        "a spin inside the pool must be resisted ({inside} vs {outside})"
    );
}

#[test]
fn an_inert_zone_is_byte_identical_whether_it_is_zero_force_or_zero_drag() {
    // The registration guard now has TWO ways to be inert, and both must leave the
    // world untouched — a zone that neither pushes nor resists is a plain sensor.
    let hashes: Vec<[u8; 32]> = [
        Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 0.0,
            density: 0.0,
            form_drag: 0.0,
            torque: 0.0,
            world_axes: false,
            falloff: 0.0,
            mirror: [1.0, 1.0],
        }),
        None,
    ]
    .into_iter()
    .map(|effector| {
        let (mut w, _) = settled_ball();
        let mut z = zone(0.0, 0.0, 6.0, 3.0, [0.0, 0.0]);
        z.effector = effector;
        w.spawn_body(z);
        for _ in 0..120 {
            w.step();
        }
        w.deterministic_hash()
    })
    .collect();
    assert_eq!(
        hashes[0], hashes[1],
        "a zone with neither force nor drag must be byte-identical to no zone at all"
    );
}
