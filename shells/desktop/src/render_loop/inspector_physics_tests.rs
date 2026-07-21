//! **The other half of the §11 seam: does the click produce a body that
//! FALLS?**
//!
//! `ph2d-panel-inspector/tests/seam_physics.rs` proves panel → bus. These
//! prove bus → ECS → simulation, which is the half the repo has been burned
//! by before: a tool can pass every gate in its own crate and be completely
//! dead in the product ([[feedback_tool_unit_green_integration_dead]]).
//!
//! So the oracle here is not "the components exist". It is: the sprite the
//! artist clicked **Add Physics Body** on is, a second later, lying on the
//! floor.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor::PhysicsFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};
use ph2d_render::Sprite;

use super::inspector_physics::{apply_physics_edit, build_physics_info};

/// The registry the shell boots with, minus everything §11 does not touch.
fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}

/// A sprite 2 m wide and 1 m tall, sitting in the air — deliberately NOT
/// square, so a collider that ignores the art's bounds is visible in the
/// numbers rather than hidden behind a lucky default.
fn sprite_scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 3.0)),
            Sprite::atlas(0, [2.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ))
        .id();
    (sim, e)
}

fn apply(sim: &mut SimWorld, e: Entity, edit: PhysicsFieldEdit) {
    let reg = registry();
    let queue = EditorCommandQueue::default();
    apply_physics_edit(sim, e.to_bits(), edit, &queue, &reg);
    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commit");
}

/// **The whole feature, end to end.** Add on a plain sprite, then run the
/// clock: it has to fall and land on the floor.
#[test]
fn adding_a_body_from_the_inspector_makes_the_sprite_fall() {
    let (mut sim, e) = sprite_scene();

    // A floor to land on.
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
    ));

    let before = sim.world().get::<Transform>(e).unwrap().translation.y;
    apply(&mut sim, e, PhysicsFieldEdit::Add);

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let after = sim.world().get::<Transform>(e).unwrap().translation.y;
    assert!(
        after < before - 1.0,
        "the sprite never fell (y {before} -> {after}) — Add Physics Body reached the ECS but \
         the entity is not being simulated"
    );
    // Half-height 0.5 (the sprite is 1 m tall) resting on a floor whose top
    // is at y = 0.1.
    assert!(
        (after - 0.6).abs() < 0.15,
        "the sprite settled at y={after}, expected ~0.6 (floor top 0.1 + half height 0.5)"
    );
}

/// **The collider is the sprite's own box.** This is the rule that prevents
/// the mismatch Enio reported on 2026-07-18 from ever being *authored*: a
/// default ball under a 2×1 sprite would draw as a rectangle and behave as a
/// circle from the very first click.
#[test]
fn the_added_collider_is_boxed_to_the_sprite() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);

    let col = *sim.world().get::<Collider>(e).expect("collider attached");
    assert_eq!(
        col.shape,
        ColliderShape::Cuboid {
            half_x: 1.0,
            half_y: 0.5,
        },
        "the collider does not match the 2×1 sprite — it was defaulted instead of derived"
    );
    let rb = *sim.world().get::<RigidBody>(e).expect("body attached");
    assert_eq!(rb.kind, BodyKind::Dynamic, "a new body should be dynamic");
}

/// Remove takes BOTH components. Leaving one behind is a half-body: the
/// bridge needs the pair, so the entity would look physics-free in the panel
/// while carrying a dead component into the save file.
#[test]
fn removing_a_body_detaches_both_components() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Remove);
    assert!(
        sim.world().get::<RigidBody>(e).is_none(),
        "RigidBody stayed"
    );
    assert!(sim.world().get::<Collider>(e).is_none(), "Collider stayed");

    let info =
        build_physics_info(sim.world(), e.to_bits(), false, 5.0, 0).expect("still inspectable");
    assert!(
        !info.has_body,
        "the panel would still show the body rows for an entity with no body"
    );
}

/// Editing one collider field must not silently reset the others. The write
/// path reads the live component and writes it back changed, precisely so a
/// partial write cannot drop the fields the artist is not touching.
#[test]
fn editing_one_field_preserves_the_rest() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Density(4.0));
    apply(&mut sim, e, PhysicsFieldEdit::Restitution(0.9));
    apply(&mut sim, e, PhysicsFieldEdit::Friction(1.25));

    let col = *sim.world().get::<Collider>(e).unwrap();
    assert_eq!(col.density, 4.0);
    assert_eq!(col.restitution, 0.9);
    assert_eq!(col.friction, 1.25);
    assert_eq!(
        col.shape,
        ColliderShape::Cuboid {
            half_x: 1.0,
            half_y: 0.5,
        },
        "editing the material fields moved the shape"
    );
}

/// Switching shape preserves the FOOTPRINT rather than resetting to a
/// default size — the object must not jump size when the artist is only
/// choosing between a box and a ball.
#[test]
fn switching_shape_keeps_the_footprint() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add); // box 1.0 × 0.5
    apply(&mut sim, e, PhysicsFieldEdit::Shape(0)); // → ball
    let col = *sim.world().get::<Collider>(e).unwrap();
    assert_eq!(
        col.shape,
        ColliderShape::Ball { radius: 0.5 },
        "a 1.0×0.5 box should become the ball that FITS it (r = 0.5), not r = 1.0 (which would \
         poke out of the sprite) and not a default"
    );

    apply(&mut sim, e, PhysicsFieldEdit::Shape(1)); // → box again
    assert_eq!(
        sim.world().get::<Collider>(e).unwrap().shape,
        ColliderShape::Cuboid {
            half_x: 0.5,
            half_y: 0.5,
        },
        "coming back from a ball should give the box that circumscribes it"
    );
}

/// A body kind flip is a real change in the simulation, not just a tag: a
/// Static body must stop falling.
#[test]
fn making_a_body_static_stops_it_falling() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Kind(1)); // Static

    let before = sim.world().get::<Transform>(e).unwrap().translation.y;
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let after = sim.world().get::<Transform>(e).unwrap().translation.y;
    assert_eq!(
        after, before,
        "a Static body moved — the kind edit reached the component but not the solver"
    );
}

/// The snapshot the panel reads back is the component that was written.
/// Without this the rows could show stale numbers forever and every other
/// gate here would still pass.
#[test]
fn the_snapshot_reflects_what_was_written() {
    let (mut sim, e) = sprite_scene();
    let empty = build_physics_info(sim.world(), e.to_bits(), false, 5.0, 0)
        .expect("plain sprite is inspectable");
    assert!(
        !empty.has_body,
        "a plain sprite must report has_body = false, or the Add button is never offered"
    );

    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Friction(0.25));
    let info = build_physics_info(sim.world(), e.to_bits(), false, 5.0, 0).unwrap();
    assert!(info.has_body);
    assert_eq!(info.friction, 0.25);
    assert_eq!(info.half_x, 1.0);
    assert_eq!(info.half_y, 0.5);
    assert_eq!(info.shape_tag, 1, "a Cuboid should report shape tag 1");
}

/// **A combine-rule edit read-modify-writes the ONE rule it names, and the
/// component detaches when BOTH rules return to Average** (W-Material).
///
/// The two rules live on one `MaterialCombine` component, so editing Bounce
/// Combine must preserve Friction Combine (a partial write would silently reset
/// it), and a body that carries neither override must carry no component (the
/// presence-override idiom — a project file free of the no-op). Two mutations
/// this catches: dropping the read-modify-write resets the untouched rule; skipping
/// the neutral-detach leaves a `{Average, Average}` component clinging to every body
/// that ever touched the control.
#[test]
fn combine_rule_read_modify_writes_and_detaches_at_neutral() {
    use ph2d_physics_ecs::{CombineRule, MaterialCombine};

    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    // No override yet: a fresh body carries no component.
    assert!(
        sim.world().get::<MaterialCombine>(e).is_none(),
        "a freshly-added body should carry no MaterialCombine (Average is the default)"
    );

    // Bounce Combine → Max: the component attaches with friction still Average.
    apply(&mut sim, e, PhysicsFieldEdit::RestitutionCombine(3));
    assert_eq!(
        sim.world().get::<MaterialCombine>(e).copied(),
        Some(MaterialCombine {
            restitution: CombineRule::Max,
            friction: CombineRule::Average,
        }),
        "setting Bounce Combine = Max should attach the component with friction at Average"
    );

    // Friction Combine → Min: RESTITUTION MUST BE PRESERVED (the read-modify-write).
    apply(&mut sim, e, PhysicsFieldEdit::FrictionCombine(1));
    assert_eq!(
        sim.world().get::<MaterialCombine>(e).copied(),
        Some(MaterialCombine {
            restitution: CombineRule::Max,
            friction: CombineRule::Min,
        }),
        "setting Friction Combine must not reset the restitution rule — the two share \
         one component and a partial write would drop the other"
    );

    // Bounce Combine back to Average: still attached (friction is not neutral).
    apply(&mut sim, e, PhysicsFieldEdit::RestitutionCombine(0));
    assert_eq!(
        sim.world().get::<MaterialCombine>(e).copied(),
        Some(MaterialCombine {
            restitution: CombineRule::Average,
            friction: CombineRule::Min,
        }),
        "with friction still Min the component must remain — only BOTH neutral detaches"
    );

    // Friction Combine back to Average: now BOTH are neutral → the component detaches.
    apply(&mut sim, e, PhysicsFieldEdit::FrictionCombine(0));
    assert!(
        sim.world().get::<MaterialCombine>(e).is_none(),
        "with both rules back at Average the component must detach — a project file \
         should not carry a no-op {{Average, Average}}"
    );
}

/// **A damping edit read-modify-writes the ONE field it names, and the component
/// detaches only at the MODE-AWARE neutral** (W-Damping).
///
/// The three fields share one `DampingOverride`, so editing one must preserve the
/// others. Detach is mode-aware: zero drag + `Combine` IS neutral (the body is on the
/// world default), but zero drag + `Replace` is NOT (it forces zero damping, ignoring
/// a world drag — a real choice). Mutations this catches: dropping the read-modify-write
/// resets the untouched fields; treating `Replace(0,0)` as neutral throws away a
/// deliberate "no drag, ignore the world" body.
#[test]
fn damping_read_modify_writes_and_detaches_only_at_the_mode_aware_neutral() {
    use ph2d_physics_ecs::{DampMode, DampingOverride};

    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    assert!(
        sim.world().get::<DampingOverride>(e).is_none(),
        "a freshly-added body should carry no DampingOverride (the world default drag)"
    );

    // Linear → 2.0: attaches, angular still 0, mode still Combine.
    apply(&mut sim, e, PhysicsFieldEdit::LinearDamping(2.0));
    assert_eq!(
        sim.world().get::<DampingOverride>(e).copied(),
        Some(DampingOverride {
            linear: 2.0,
            angular: 0.0,
            mode: DampMode::Combine,
        }),
        "setting Linear Damping should attach the component with the others at default"
    );

    // Angular → 1.5: LINEAR MUST BE PRESERVED (the read-modify-write).
    apply(&mut sim, e, PhysicsFieldEdit::AngularDamping(1.5));
    assert_eq!(
        sim.world()
            .get::<DampingOverride>(e)
            .map(|d| (d.linear, d.angular)),
        Some((2.0, 1.5)),
        "setting Angular Damping must not reset the linear one — they share one component"
    );

    // Mode → Replace: the values must be preserved.
    apply(&mut sim, e, PhysicsFieldEdit::DampMode(1));
    assert_eq!(
        sim.world().get::<DampingOverride>(e).copied(),
        Some(DampingOverride {
            linear: 2.0,
            angular: 1.5,
            mode: DampMode::Replace,
        }),
        "setting the mode must not reset the drag values"
    );

    // Zero both drags, mode still Replace: NOT neutral (Replace(0,0) forces zero
    // damping, ignoring a world drag), so the component STAYS.
    apply(&mut sim, e, PhysicsFieldEdit::LinearDamping(0.0));
    apply(&mut sim, e, PhysicsFieldEdit::AngularDamping(0.0));
    assert_eq!(
        sim.world().get::<DampingOverride>(e).copied(),
        Some(DampingOverride {
            linear: 0.0,
            angular: 0.0,
            mode: DampMode::Replace,
        }),
        "Replace(0,0) is a deliberate 'no drag, ignore the world' — it must NOT detach"
    );

    // Back to Combine: now zero drag + Combine = the world default = neutral → detach.
    apply(&mut sim, e, PhysicsFieldEdit::DampMode(0));
    assert!(
        sim.world().get::<DampingOverride>(e).is_none(),
        "zero drag + Combine is the world default — the component must detach"
    );
}
