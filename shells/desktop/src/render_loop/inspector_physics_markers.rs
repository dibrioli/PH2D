//! §11 Physics Body — **the flags whose PRESENCE is their value**.
//!
//! CCD, Freeze Rotation, Freeze Position X, Freeze Position Y, One-Way platform:
//! five marker components, five §11 toggles, and — until this module — five arms in
//! `inspector_physics_apply` that differed only in which name and which unit struct
//! they named. Split out when the force-zone arm (W-Area) took that file to 597 of
//! its 600 lines: a cap is a prompt to find the responsibility that wants its own
//! home, never to widen the cap.
//!
//! Putting them together made the duplication visible, so it is gone: one gate, one
//! branch, five one-line rows. A sixth marker is a row, not another eighteen lines.

use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_editor::PhysicsFieldEdit;
use serde::Serialize;

use super::inspector_ordering::{queue_remove, queue_set};

/// Attach the marker or detach it — the presence-override idiom, so a project file
/// never carries an off-flag (and a body that never touched the control carries no
/// component at all, which is what makes an unset body byte-identical to one authored
/// before the marker existed).
fn set_or_clear<T: Serialize>(
    on: bool,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
    entity_bits: u64,
    name: &str,
    value: &T,
) {
    if on {
        queue_set(queue, registry, entity_bits, name, value);
    } else {
        queue_remove(queue, registry, entity_bits, name);
    }
}

/// Handle the edit if it is one of the marker toggles; `false` means "not mine",
/// and the caller falls through to its remaining arms.
///
/// **Gated on a live body**, once for all five: without a `RigidBody` there is
/// nothing to sweep / freeze / make one-way, and honouring the edit would attach an
/// orphan marker to a plain sprite — a component the §11 section cannot show and the
/// project file would carry forever.
pub(super) fn apply_marker_edit(
    world: &World,
    entity: Entity,
    entity_bits: u64,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    use ph2d_physics_ecs::{
        AreaForceWorldAxes, Ccd, LockPositionX, LockPositionY, LockRotation, OneWayPlatform,
    };

    if world.get::<ph2d_physics_ecs::RigidBody>(entity).is_none() {
        // Not "not mine": these edits ARE ours, they are simply refused here. Saying
        // `false` would send them down the collider path, which is a different write.
        return matches!(
            edit,
            PhysicsFieldEdit::Ccd(_)
                | PhysicsFieldEdit::LockRotation(_)
                | PhysicsFieldEdit::LockPositionX(_)
                | PhysicsFieldEdit::LockPositionY(_)
                | PhysicsFieldEdit::OneWay(_)
                | PhysicsFieldEdit::ForceWorldAxes(_)
        );
    }
    match edit {
        // A RigidBody-level sweep flag: without it a fast body is only tested at each
        // tick's end pose and can pass clean through thin geometry.
        PhysicsFieldEdit::Ccd(on) => {
            set_or_clear(on, queue, registry, entity_bits, "ph2d::physics::Ccd", &Ccd)
        }
        // Freeze Rotation — the angular DOF.
        PhysicsFieldEdit::LockRotation(on) => set_or_clear(
            on,
            queue,
            registry,
            entity_bits,
            "ph2d::physics::LockRotation",
            &LockRotation,
        ),
        // Freeze Position X / Y — one translation DOF each, independent.
        PhysicsFieldEdit::LockPositionX(on) => set_or_clear(
            on,
            queue,
            registry,
            entity_bits,
            "ph2d::physics::LockPositionX",
            &LockPositionX,
        ),
        PhysicsFieldEdit::LockPositionY(on) => set_or_clear(
            on,
            queue,
            registry,
            entity_bits,
            "ph2d::physics::LockPositionY",
            &LockPositionY,
        ),
        // One-way (jump-through) platform — a COLLIDER property, and the only one of
        // the five that is not Dynamic-only (a platform is usually Static).
        PhysicsFieldEdit::OneWay(on) => set_or_clear(
            on,
            queue,
            registry,
            entity_bits,
            "ph2d::physics::OneWayPlatform",
            &OneWayPlatform,
        ),
        // The frame of a force zone — also a COLLIDER property, and the second marker
        // here that is not Dynamic-only (a wind column is Static scenery).
        PhysicsFieldEdit::ForceWorldAxes(on) => set_or_clear(
            on,
            queue,
            registry,
            entity_bits,
            "ph2d::physics::AreaForceWorldAxes",
            &AreaForceWorldAxes,
        ),
        _ => return false,
    }
    true
}
