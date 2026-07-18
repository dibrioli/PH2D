//! §11 Physics Body — the shell half of the Inspector section (ADR-0130 D8):
//! the snapshot the panel reads, and the ECS write an edit turns into.
//!
//! Its own module rather than more of `inspector_ordering.rs`: physics is not
//! ordering, and that file was at the HR-18 cap. Same split the panel side
//! took (`event_physics.rs`).

use bevy_ecs::world::World;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::{InspectorPhysicsInfo, PhysicsFieldEdit};

use super::inspector_ordering::{queue_remove, queue_set};

/// Build the §11 Physics Body snapshot (ADR-0130 D8).
///
/// **Returns `Some` for a Transform-bearing entity even when it has NO
/// body** — `has_body: false` is what lets the section offer the Add button.
/// Without that, physics would be authorable only on entities that already
/// have physics, which is nowhere.
pub(crate) fn build_physics_info(world: &World, entity_bits: u64) -> Option<InspectorPhysicsInfo> {
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let rb = world.get::<RigidBody>(entity);
    let col = world.get::<Collider>(entity);
    let (Some(rb), Some(col)) = (rb, col) else {
        // The empty face. The dimensions are the values the Add button would
        // seed if the sprite had no bounds — the panel never shows them.
        return Some(InspectorPhysicsInfo {
            entity_bits,
            has_body: false,
            kind_tag: 0,
            shape_tag: 1,
            radius: 0.5,
            half_x: 0.5,
            half_y: 0.5,
            density: 1.0,
            restitution: Collider::DEFAULT_RESTITUTION,
            friction: Collider::DEFAULT_FRICTION,
        });
    };
    let (shape_tag, radius, half_x, half_y) = match col.shape {
        ColliderShape::Ball { radius } => (0u8, radius, radius, radius),
        ColliderShape::Cuboid { half_x, half_y } => (1u8, half_x.max(half_y), half_x, half_y),
    };
    Some(InspectorPhysicsInfo {
        entity_bits,
        has_body: true,
        kind_tag: match rb.kind {
            BodyKind::Dynamic => 0,
            BodyKind::Static => 1,
        },
        shape_tag,
        radius,
        half_x,
        half_y,
        density: col.density,
        restitution: col.restitution,
        friction: col.friction,
    })
}

/// Apply one [`PhysicsFieldEdit`] (§11).
///
/// **`Add` derives the collider from the sprite's own bounds** — the one
/// starting shape that can never disagree with what is drawn. (A default ball
/// under a square sprite is exactly the mismatch Enio caught on 2026-07-18;
/// Unity and Godot both fit the box to the renderer for the same reason.)
/// A sprite-less entity falls back to a half-metre box.
pub(crate) fn apply_physics_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
    const RIGID_BODY: &str = "ph2d::physics::RigidBody";
    const COLLIDER: &str = "ph2d::physics::Collider";

    let entity = Entity::from_bits(entity_bits);
    let world = sim.world();

    if matches!(edit, PhysicsFieldEdit::Add) {
        let half = world
            .get::<ph2d_render::Sprite>(entity)
            .map_or([0.5, 0.5], |s| {
                [(s.size[0] * 0.5).max(1e-3), (s.size[1] * 0.5).max(1e-3)]
            });
        queue_set(
            queue,
            registry,
            entity_bits,
            RIGID_BODY,
            &RigidBody {
                kind: BodyKind::Dynamic,
            },
        );
        queue_set(
            queue,
            registry,
            entity_bits,
            COLLIDER,
            &Collider {
                shape: ColliderShape::Cuboid {
                    half_x: half[0],
                    half_y: half[1],
                },
                ..Collider::default()
            },
        );
        return;
    }
    if matches!(edit, PhysicsFieldEdit::Remove) {
        queue_remove(queue, registry, entity_bits, RIGID_BODY);
        queue_remove(queue, registry, entity_bits, COLLIDER);
        return;
    }
    if let PhysicsFieldEdit::Kind(tag) = edit {
        queue_set(
            queue,
            registry,
            entity_bits,
            RIGID_BODY,
            &RigidBody {
                kind: if tag == 1 {
                    BodyKind::Static
                } else {
                    BodyKind::Dynamic
                },
            },
        );
        return;
    }

    // Everything else edits the collider, so read the live one and write it
    // back changed — a partial write would drop the fields not being edited.
    let Some(cur) = world.get::<Collider>(entity).copied() else {
        return;
    };
    let mut next = cur;
    match edit {
        // Switching shape PRESERVES the footprint: a box becomes the ball
        // that fits inside it and back, so the object does not jump size.
        PhysicsFieldEdit::Shape(0) => {
            let r = match cur.shape {
                ColliderShape::Ball { radius } => radius,
                ColliderShape::Cuboid { half_x, half_y } => half_x.min(half_y),
            };
            next.shape = ColliderShape::Ball {
                radius: r.max(1e-3),
            };
        }
        PhysicsFieldEdit::Shape(_) => {
            let (hx, hy) = match cur.shape {
                ColliderShape::Ball { radius } => (radius, radius),
                ColliderShape::Cuboid { half_x, half_y } => (half_x, half_y),
            };
            next.shape = ColliderShape::Cuboid {
                half_x: hx.max(1e-3),
                half_y: hy.max(1e-3),
            };
        }
        PhysicsFieldEdit::Radius(v) => {
            next.shape = ColliderShape::Ball {
                radius: v.max(1e-3),
            }
        }
        PhysicsFieldEdit::HalfX(v) => {
            if let ColliderShape::Cuboid { half_y, .. } = cur.shape {
                next.shape = ColliderShape::Cuboid {
                    half_x: v.max(1e-3),
                    half_y,
                };
            }
        }
        PhysicsFieldEdit::HalfY(v) => {
            if let ColliderShape::Cuboid { half_x, .. } = cur.shape {
                next.shape = ColliderShape::Cuboid {
                    half_x,
                    half_y: v.max(1e-3),
                };
            }
        }
        PhysicsFieldEdit::Density(v) => next.density = v.max(0.0),
        PhysicsFieldEdit::Restitution(v) => next.restitution = v.clamp(0.0, 1.0),
        PhysicsFieldEdit::Friction(v) => next.friction = v.max(0.0),
        PhysicsFieldEdit::Add | PhysicsFieldEdit::Remove | PhysicsFieldEdit::Kind(_) => {
            unreachable!("handled above")
        }
    }
    if next != cur {
        queue_set(queue, registry, entity_bits, COLLIDER, &next);
    }
}
