//! §11 Physics Body — the shell half of the Inspector section (ADR-0131 D8):
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

/// Build the §11 Physics Body snapshot (ADR-0131 D8).
///
/// **Returns `Some` for a Transform-bearing entity even when it has NO
/// body** — `has_body: false` is what lets the section offer the Add button.
/// Without that, physics would be authorable only on entities that already
/// have physics, which is nowhere.
pub(crate) fn build_physics_info(
    world: &World,
    entity_bits: u64,
    can_join: bool,
    bake_seconds: f32,
    bake_channels_tag: u8,
) -> Option<InspectorPhysicsInfo> {
    use ph2d_physics_ecs::{
        Ccd, Collider, ColliderShape, GravityScale, InitialVelocity, RigidBody,
    };
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let rb = world.get::<RigidBody>(entity);
    let col = world.get::<Collider>(entity);
    // Optional per-body gravity multiplier (W8); absent = the neutral 1.0.
    let gravity_scale = world
        .get::<GravityScale>(entity)
        .map_or(GravityScale::NEUTRAL, |g| g.0);
    // Optional authored initial velocity (W9); absent = at rest.
    let iv = world
        .get::<InitialVelocity>(entity)
        .copied()
        .unwrap_or(InitialVelocity::REST);
    // Optional CCD marker (W-CCD); its presence is the flag, absent = discrete.
    let ccd = world.get::<Ccd>(entity).is_some();
    let (Some(rb), Some(col)) = (rb, col) else {
        // The empty face. The dimensions are the values the Add button would
        // seed if the sprite had no bounds — the panel never shows them.
        return Some(InspectorPhysicsInfo {
            entity_bits,
            has_body: false,
            bake_seconds,
            kind_tag: 0,
            shape_tag: 1,
            radius: 0.5,
            half_x: 0.5,
            half_y: 0.5,
            density: 1.0,
            restitution: Collider::DEFAULT_RESTITUTION,
            friction: Collider::DEFAULT_FRICTION,
            layer: 0,
            // An entity with no body cannot be half of a joint, whatever the
            // selection looks like.
            can_join: false,
            is_sensor: false,
            bake_channels_tag,
            gravity_scale: GravityScale::NEUTRAL,
            cap_half_height: 0.25,
            linvel: InitialVelocity::REST.linvel,
            angvel: InitialVelocity::REST.angvel,
            ccd: false,
        });
    };
    // Each arm also carries what the OTHER shapes' rows would seed if the artist
    // switched — the rows for an inactive shape are not painted, so these are
    // display seeds, and `apply_physics_edit` owns the real conversion.
    let (shape_tag, radius, half_x, half_y, cap_half_height) = match col.shape {
        ColliderShape::Ball { radius } => (0u8, radius, radius, radius, 0.0),
        ColliderShape::Cuboid { half_x, half_y } => (
            1u8,
            half_x.max(half_y),
            half_x,
            half_y,
            (half_y - half_x).max(0.0),
        ),
        // A capsule's own radius IS the ball radius; its box equivalent is
        // `radius` wide and `half_height + radius` tall (the TOTAL half-extent,
        // which is what a box would need to cover the same silhouette).
        ColliderShape::Capsule {
            half_height,
            radius,
        } => (2u8, radius, radius, half_height + radius, half_height),
    };
    Some(InspectorPhysicsInfo {
        entity_bits,
        has_body: true,
        kind_tag: rb.kind.tag(),
        shape_tag,
        radius,
        half_x,
        half_y,
        density: col.density,
        restitution: col.restitution,
        friction: col.friction,
        layer: col.layer,
        can_join,
        bake_seconds,
        is_sensor: col.is_sensor,
        bake_channels_tag,
        gravity_scale,
        cap_half_height,
        linvel: iv.linvel,
        angvel: iv.angvel,
        ccd,
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
    use ph2d_physics_ecs::{
        BodyKind, Ccd, Collider, ColliderShape, GravityScale, InitialVelocity, RigidBody,
    };
    const RIGID_BODY: &str = "ph2d::physics::RigidBody";
    const COLLIDER: &str = "ph2d::physics::Collider";
    const GRAVITY_SCALE: &str = "ph2d::physics::GravityScale";
    const INITIAL_VELOCITY: &str = "ph2d::physics::InitialVelocity";
    const CCD: &str = "ph2d::physics::Ccd";

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
        // A tag no variant claims is DROPPED, not folded onto a plausible
        // neighbour: the chip that sent it is the only thing that mints one,
        // so an unknown value means the chip list and the enum have drifted,
        // and silently applying `Dynamic` would hide exactly that.
        let Some(kind) = BodyKind::from_tag(tag) else {
            debug_assert!(
                false,
                "§11 sent BodyKind tag {tag}, which no variant claims"
            );
            return;
        };
        // ⚠️ Only a body can change KIND. Without this, `Kind` was the one §11
        // arm that would ATTACH a `RigidBody` to an entity that had none — an
        // orphan with no `Collider`, which `build_physics_info` cannot show
        // (it needs both), so §11 offers Add rather than Remove and no gesture
        // anywhere takes it off again. It saves into the project file all the
        // same. The other arms are safe by accident rather than by check: they
        // read the live `Collider` and return early when it is missing.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        queue_set(
            queue,
            registry,
            entity_bits,
            RIGID_BODY,
            &RigidBody { kind },
        );
        return;
    }
    if let PhysicsFieldEdit::GravityScale(v) = edit {
        // A RigidBody-level property (the optional `GravityScale` component),
        // not a Collider field — so it is handled here, like `Kind`, before the
        // collider block below. Gated on a live body: without one there is
        // nothing to scale, and this would be a second arm that attaches an
        // orphan component to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // Detach at the neutral 1.0 so an unscaled body carries no component
        // (the presence-override idiom: absent = default, and a project file
        // stays free of no-op `1.0`s). Any other value attaches/updates.
        if v == GravityScale::NEUTRAL {
            queue_remove(queue, registry, entity_bits, GRAVITY_SCALE);
        } else {
            queue_set(
                queue,
                registry,
                entity_bits,
                GRAVITY_SCALE,
                &GravityScale(v),
            );
        }
        return;
    }
    if let PhysicsFieldEdit::LinvelX(_)
    | PhysicsFieldEdit::LinvelY(_)
    | PhysicsFieldEdit::Angvel(_) = edit
    {
        // Initial velocity (W9): another RigidBody-level, optional
        // presence-override component, so it is handled here beside gravity.
        // Read-modify-write the ONE axis the edit names — a partial write would
        // drop the other two — off the current component or REST if absent.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        let mut iv = world
            .get::<InitialVelocity>(entity)
            .copied()
            .unwrap_or(InitialVelocity::REST);
        match edit {
            PhysicsFieldEdit::LinvelX(v) => iv.linvel[0] = v,
            PhysicsFieldEdit::LinvelY(v) => iv.linvel[1] = v,
            PhysicsFieldEdit::Angvel(v) => iv.angvel = v,
            _ => unreachable!(),
        }
        // Detach at rest so a still body carries no component — the same
        // idiom as gravity, and it keeps the project file free of no-op zeros.
        if iv.is_rest() {
            queue_remove(queue, registry, entity_bits, INITIAL_VELOCITY);
        } else {
            queue_set(queue, registry, entity_bits, INITIAL_VELOCITY, &iv);
        }
        return;
    }
    if let PhysicsFieldEdit::Ccd(on) = edit {
        // A RigidBody-level flag carried by the optional `Ccd` MARKER — its
        // presence is the boolean — so it is handled here beside gravity/velocity,
        // not on the Collider. Gated on a live body: without one there is nothing
        // to sweep, and this would attach an orphan marker to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // Attach on Continuous, detach on Discrete — the presence-override idiom,
        // so a project file never carries an off-flag.
        if on {
            queue_set(queue, registry, entity_bits, CCD, &Ccd);
        } else {
            queue_remove(queue, registry, entity_bits, CCD);
        }
        return;
    }

    // Everything else edits the collider, so read the live one and write it
    // back changed — a partial write would drop the fields not being edited.
    let Some(cur) = world.get::<Collider>(entity).copied() else {
        return;
    };
    let mut next = cur;
    match edit {
        // Not a field edit at all: creating a joint is one gesture over a
        // PAIR, and it is intercepted before the per-entity fan-out that
        // brings us here. Reaching this arm would mean the interception was
        // removed — and the fan-out would then create one joint per selected
        // body instead of one joint.
        // Neither of these is a field edit. Both are one gesture over the
        // SELECTION, intercepted before the per-entity fan-out that brings us
        // here — `Join` because fanning out would make one joint per body
        // instead of one joint, `Bake` because it would re-run the whole
        // simulation once per body and file a separate undo step each time.
        // Reaching either arm means an interception was removed. `BakeChannels`
        // joins them: it is a GLOBAL bake option (app state), not a Collider
        // edit, and the render loop consumes it before this fan-out.
        PhysicsFieldEdit::Join | PhysicsFieldEdit::Bake | PhysicsFieldEdit::BakeChannels(_) => {}
        // Switching shape PRESERVES the footprint: a box becomes the ball
        // that fits inside it and back, so the object does not jump size.
        PhysicsFieldEdit::Shape(0) => {
            let r = match cur.shape {
                ColliderShape::Ball { radius } => radius,
                ColliderShape::Cuboid { half_x, half_y } => half_x.min(half_y),
                // The cap already IS a ball of this radius.
                ColliderShape::Capsule { radius, .. } => radius,
            };
            next.shape = ColliderShape::Ball {
                radius: r.max(1e-3),
            };
        }
        PhysicsFieldEdit::Shape(1) => {
            let (hx, hy) = match cur.shape {
                ColliderShape::Ball { radius } => (radius, radius),
                ColliderShape::Cuboid { half_x, half_y } => (half_x, half_y),
                // The box that covers the capsule's silhouette: as wide as the
                // caps, as tall as the TOTAL half-extent.
                ColliderShape::Capsule {
                    half_height,
                    radius,
                } => (radius, half_height + radius),
            };
            next.shape = ColliderShape::Cuboid {
                half_x: hx.max(1e-3),
                half_y: hy.max(1e-3),
            };
        }
        PhysicsFieldEdit::Shape(2) => {
            let (half_height, radius) = match cur.shape {
                // A zero-segment capsule is exactly that ball, so the silhouette
                // does not move at all on the switch.
                ColliderShape::Ball { radius } => (0.0, radius),
                // Radius from the SMALLER half-extent so the capsule never grows
                // wider than the box, and the segment takes up the rest — total
                // half-extent lands back on `half_y` exactly.
                ColliderShape::Cuboid { half_x, half_y } => {
                    let r = half_x.min(half_y);
                    ((half_y - r).max(0.0), r)
                }
                ColliderShape::Capsule {
                    half_height,
                    radius,
                } => (half_height, radius),
            };
            next.shape = ColliderShape::Capsule {
                half_height: half_height.max(0.0),
                radius: radius.max(1e-3),
            };
        }
        // A tag no shape claims is DROPPED rather than folded onto a plausible
        // neighbour — the same discipline `Kind` follows. With two shapes the
        // old catch-all was merely redundant; with three it would be a chip that
        // quietly selects a different shape.
        PhysicsFieldEdit::Shape(_) => {}
        // ⚠️ Radius must not FORCE a ball: a capsule's cap radius is the same
        // quantity under the same name, so on a capsule this edits the caps and
        // leaves it a capsule. Forcing `Ball` here would delete the artist's
        // capsule the first time they touched its radius.
        PhysicsFieldEdit::Radius(v) => {
            next.shape = match cur.shape {
                ColliderShape::Capsule { half_height, .. } => ColliderShape::Capsule {
                    half_height,
                    radius: v.max(1e-3),
                },
                _ => ColliderShape::Ball {
                    radius: v.max(1e-3),
                },
            };
        }
        // Only meaningful on a capsule, and the row is painted only there.
        PhysicsFieldEdit::CapHalfHeight(v) => {
            if let ColliderShape::Capsule { radius, .. } = cur.shape {
                next.shape = ColliderShape::Capsule {
                    half_height: v.max(0.0),
                    radius,
                };
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
        // Clamped to the layers that exist: a chip cannot produce an out-of-range
        // value today, but this is also the door a future build's project file
        // comes through, and `groups_for` would silently fold an overflow onto
        // the last layer.
        PhysicsFieldEdit::Layer(n) => next.layer = n.min(ph2d_physics_ecs::MAX_LAYERS as u8 - 1),
        PhysicsFieldEdit::Sensor(s) => next.is_sensor = s,
        PhysicsFieldEdit::Add
        | PhysicsFieldEdit::Remove
        | PhysicsFieldEdit::Kind(_)
        | PhysicsFieldEdit::GravityScale(_)
        | PhysicsFieldEdit::LinvelX(_)
        | PhysicsFieldEdit::LinvelY(_)
        | PhysicsFieldEdit::Angvel(_)
        | PhysicsFieldEdit::Ccd(_) => {
            unreachable!("handled above")
        }
    }
    if next != cur {
        queue_set(queue, registry, entity_bits, COLLIDER, &next);
    }
}
