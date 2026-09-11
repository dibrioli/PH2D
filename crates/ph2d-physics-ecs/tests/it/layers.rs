//! **W2c — collision layers at the bridge seam.**
//!
//! The claim the whole feature exists to make: *two bodies on layers the matrix
//! separates pass through each other.* The oracle is that, not "the component
//! has the value I wrote" — a layer that reaches the component and stops there
//! is exactly the shape of bug this line keeps finding.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LayerMatrix, MAX_LAYERS, PhysicsBridge, PhysicsSettings,
    RigidBody,
};

/// A floor on `layer`, and one ball dropped from `y = 3` on `ball_layer`.
fn scene(floor_layer: u8, ball_layer: u8) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            density: 1.0,
            layer: floor_layer,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                layer: ball_layer,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 3.0)),
        ))
        .id();
    (sim, ball)
}

/// Where the ball ends up after two seconds.
fn drop_for(settings: PhysicsSettings, floor_layer: u8, ball_layer: u8) -> f32 {
    let (mut sim, ball) = scene(floor_layer, ball_layer);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(settings);
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    sim.world().get::<Transform>(ball).unwrap().translation.y
}

/// The matrix with layers `a` and `b` separated.
fn matrix_without(a: usize, b: usize) -> PhysicsSettings {
    let mut m = LayerMatrix::all();
    m.set(a, b, false);
    PhysicsSettings {
        layer_matrix: m.rows(),
        ..PhysicsSettings::default()
    }
}

/// **The feature, stated as the artist would see it.**
///
/// Same scene twice. With the default (permissive) matrix the ball lands on the
/// floor; with layers 0 and 3 separated it falls straight through. The oracle
/// is the ball's position — an APPEARANCE, not a component read.
///
/// Mutation that must bleed: `body_desc` not forwarding `col.layer`, or
/// `stamp_layer` not being called on spawn.
#[test]
fn bodies_on_separated_layers_pass_through_each_other() {
    let landed = drop_for(PhysicsSettings::default(), 0, 3);
    let fell_through = drop_for(matrix_without(0, 3), 0, 3);

    assert!(
        landed > 0.0,
        "with the default permissive matrix the ball must land ON the floor, \
         but it ended at y={landed} — the fixture is not testing what it claims"
    );
    assert!(
        fell_through < -2.0,
        "layers 0 and 3 were separated, so the ball must fall THROUGH the floor; \
         it stopped at y={fell_through}"
    );
}

/// **Separating a pair that is not in the scene changes nothing.**
///
/// The sibling of the gate above, and it is what keeps that one honest: a bug
/// that made *everything* fall through would satisfy the first assertion just as
/// well. Here layers 5 and 6 are separated while the scene lives on 0 and 3.
#[test]
fn separating_unrelated_layers_leaves_the_scene_alone() {
    let baseline = drop_for(PhysicsSettings::default(), 0, 3);
    let unrelated = drop_for(matrix_without(5, 6), 0, 3);
    assert_eq!(
        baseline.to_bits(),
        unrelated.to_bits(),
        "separating layers 5 and 6 moved a scene that lives on layers 0 and 3 \
         ({baseline} -> {unrelated})"
    );
}

/// **A matrix edit reaches colliders that ALREADY exist.**
///
/// The same failure mode the body defaults had, one level up: applying the rule
/// only at spawn makes the checkbox look dead, and the only way to see it work
/// would be to delete and re-add every object.
///
/// Mutation that must bleed: `set_layer_matrix` storing the matrix without
/// walking the live colliders.
#[test]
fn a_matrix_edit_reaches_colliders_that_already_exist() {
    let (mut sim, ball) = scene(0, 3);
    let mut bridge = PhysicsBridge::new();
    // Bodies spawn FIRST, under the permissive default...
    for tick in 1..=20 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // ...and only then are the layers separated.
    bridge.set_settings(matrix_without(0, 3));
    for tick in 21..=140 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let y = sim.world().get::<Transform>(ball).unwrap().translation.y;
    assert!(
        y < -2.0,
        "the matrix was edited after the bodies existed, so the ball must now \
         fall through; it is at y={y}"
    );
}

/// **The default matrix is byte-identical to having no layers at all.**
///
/// Every project authored before W2c is on layer 0 with a permissive matrix, and
/// must simulate exactly as it did — including both cross-OS C9 hashes.
///
/// The oracle is the trajectory, not the endpoint (the W1.5 lesson).
#[test]
fn the_default_matrix_does_not_move_the_simulation() {
    let (mut a_sim, a_ball) = scene(0, 0);
    let (mut b_sim, b_ball) = scene(0, 0);
    let mut untouched = PhysicsBridge::new();
    let mut explicit = PhysicsBridge::new();
    explicit.set_settings(PhysicsSettings::default());

    for tick in 1..=180 {
        untouched.dispatch(&mut a_sim, true, tick);
        explicit.dispatch(&mut b_sim, true, tick);
        let ya = a_sim
            .world()
            .get::<Transform>(a_ball)
            .unwrap()
            .translation
            .y;
        let yb = b_sim
            .world()
            .get::<Transform>(b_ball)
            .unwrap()
            .translation
            .y;
        assert_eq!(
            ya.to_bits(),
            yb.to_bits(),
            "tick {tick}: installing the DEFAULT matrix moved the simulation \
             ({ya} vs {yb}) — every existing project and both C9 hashes changed"
        );
    }
}

/// **A lopsided matrix from a file resolves the way rapier would ACT.**
///
/// rapier ANDs both directions, so a half-set pair means "no collision". A file
/// that says `[0][3]` collides while `[3][0]` does not must therefore load as
/// *separated* — not as the half that happens to be set, and not as a state the
/// type claims cannot exist.
///
/// Mutation that must bleed: `clamped` copying `layer_matrix` verbatim instead
/// of routing it through `LayerMatrix::from_rows`.
#[test]
fn a_lopsided_matrix_from_a_file_is_read_the_way_rapier_acts() {
    let mut rows = LayerMatrix::all().rows();
    rows[3] &= !1; // layer 3 drops layer 0; layer 0 still claims layer 3.
    let settings = PhysicsSettings {
        layer_matrix: rows,
        ..PhysicsSettings::default()
    };

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(settings);

    // ⚠️ Read the RAW rows. Wrapping them in `LayerMatrix::from_rows` here —
    // which the first version of this gate did — normalizes with the very
    // function under test, so `clamped` skipping the symmetrize stayed green.
    //
    // And the stored value is what matters independently of the solver:
    // `apply_to` symmetrizes on the way into rapier, so the SIMULATION is safe
    // either way. What is not safe is the panel, which paints checkboxes from
    // these rows, and the project file, which saves them. Stored asymmetric
    // means a checked cell whose pair is unchecked and whose bodies do not
    // collide — the UI would be lying about a rule the artist can see.
    let stored = bridge.settings().layer_matrix;
    for a in 0..MAX_LAYERS {
        for b in 0..MAX_LAYERS {
            let ab = stored[a] & (1 << b) != 0;
            let ba = stored[b] & (1 << a) != 0;
            assert_eq!(
                ab, ba,
                "the STORED matrix is asymmetric at ({a},{b}): {ab} vs {ba}. \
                 The panel paints these rows, so one cell would show checked \
                 while its pair shows unchecked and the bodies pass through"
            );
        }
    }
    assert!(
        stored[0] & (1 << 3) == 0,
        "a half-set pair must resolve to SEPARATED — the reading rapier's AND \
         acts on — not to the half that happened to be set"
    );

    let (mut sim, ball) = scene(0, 3);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(settings);
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let y = sim.world().get::<Transform>(ball).unwrap().translation.y;
    assert!(
        y < -2.0,
        "…and the scene must agree with that reading: the ball should fall \
         through, but it is at y={y}"
    );
}
