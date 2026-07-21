//! §11 Physics Body — the ECS write an edit turns into (ADR-0131 D8).
//!
//! Split from the sibling `inspector_physics` (the snapshot BUILDER) when the
//! material-combine arm (W-Material) pushed that file past the shell's 600-LOC
//! cap. `inspector_physics` re-exports [`apply_physics_edit`], so every caller
//! keeps its `inspector_physics::apply_physics_edit` path. Build reads; this
//! writes — the two halves of the section, one question each.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::PhysicsFieldEdit;

use super::inspector_ordering::{queue_remove, queue_set};

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
        BodyKind, Ccd, Collider, ColliderShape, CombineRule, DampMode, DampingOverride, Dominance,
        GravityScale, InitialVelocity, LockPositionX, LockPositionY, LockRotation, MassOverride,
        MaterialCombine, RigidBody,
    };
    const RIGID_BODY: &str = "ph2d::physics::RigidBody";
    const COLLIDER: &str = "ph2d::physics::Collider";
    const GRAVITY_SCALE: &str = "ph2d::physics::GravityScale";
    const INITIAL_VELOCITY: &str = "ph2d::physics::InitialVelocity";
    const CCD: &str = "ph2d::physics::Ccd";
    const LOCK_ROTATION: &str = "ph2d::physics::LockRotation";
    const LOCK_POSITION_X: &str = "ph2d::physics::LockPositionX";
    const LOCK_POSITION_Y: &str = "ph2d::physics::LockPositionY";
    const MASS_OVERRIDE: &str = "ph2d::physics::MassOverride";
    const DOMINANCE: &str = "ph2d::physics::Dominance";
    const MATERIAL_COMBINE: &str = "ph2d::physics::MaterialCombine";
    const DAMPING_OVERRIDE: &str = "ph2d::physics::DampingOverride";

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
    if let PhysicsFieldEdit::LockRotation(on) = edit {
        // Another RigidBody-level marker (Freeze Rotation), handled here beside CCD.
        // Gated on a live body: without one there is no rotation to freeze, and
        // this would attach an orphan marker to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // Attach on Locked, detach on Free — the presence-override idiom.
        if on {
            queue_set(queue, registry, entity_bits, LOCK_ROTATION, &LockRotation);
        } else {
            queue_remove(queue, registry, entity_bits, LOCK_ROTATION);
        }
        return;
    }
    if let PhysicsFieldEdit::LockPositionX(on) = edit {
        // Freeze Position X — another RigidBody-level marker, handled here beside the
        // other constraints. Gated on a live body: without one there is no position
        // to freeze, and this would attach an orphan marker to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        if on {
            queue_set(
                queue,
                registry,
                entity_bits,
                LOCK_POSITION_X,
                &LockPositionX,
            );
        } else {
            queue_remove(queue, registry, entity_bits, LOCK_POSITION_X);
        }
        return;
    }
    if let PhysicsFieldEdit::LockPositionY(on) = edit {
        // Freeze Position Y — the vertical sibling, same idiom and gate.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        if on {
            queue_set(
                queue,
                registry,
                entity_bits,
                LOCK_POSITION_Y,
                &LockPositionY,
            );
        } else {
            queue_remove(queue, registry, entity_bits, LOCK_POSITION_Y);
        }
        return;
    }
    if let PhysicsFieldEdit::MassMode(manual) = edit {
        // Mass source (W-Mass): another RigidBody-level, optional presence-override
        // component (`MassOverride`), handled here beside the constraints. Gated on a
        // live collider: without one there is no shape to seed the mass from.
        let Some(col) = world.get::<Collider>(entity).copied() else {
            return;
        };
        if manual {
            // Auto → Manual: seed the override with the current AUTO mass (density ×
            // authored-shape area) so the mass does not jump when the toggle flips —
            // Unity seeds the manual field from the auto value the same way. Exact for
            // an unscaled body (the common case); a scaled body's true mass also
            // scales by the area factor, but this is only a starting value the artist
            // then tunes, and reading the exact scaled mass would re-derive rapier's
            // own computation in a second place.
            queue_set(
                queue,
                registry,
                entity_bits,
                MASS_OVERRIDE,
                &MassOverride(col.auto_mass()),
            );
        } else {
            // Manual → Auto: drop the override so the body weighs density × area again
            // (the presence-override idiom — absent = the engine default).
            queue_remove(queue, registry, entity_bits, MASS_OVERRIDE);
        }
        return;
    }
    if let PhysicsFieldEdit::Mass(v) = edit {
        // The explicit mass value (Manual mode only). Clamp positive — a zero or
        // negative mass is degenerate. Gated on the override already existing (the
        // Mass row is painted only in Manual mode); a stale event must not attach an
        // override while the body is in Auto mode.
        if world.get::<MassOverride>(entity).is_none() {
            return;
        }
        queue_set(
            queue,
            registry,
            entity_bits,
            MASS_OVERRIDE,
            &MassOverride(v.max(1e-3)),
        );
        return;
    }
    if let PhysicsFieldEdit::Dominance(d) = edit {
        // Dominance (W-Dominance): a RigidBody-level valued override, handled here
        // beside gravity/mass. Gated on a live body: without one there is nothing to
        // prioritise, and this would attach an orphan component to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // Detach at the neutral 0 so a neutral body carries no component (the
        // presence-override idiom — a project file stays free of no-op zeros); any
        // other value attaches/updates.
        if d == 0 {
            queue_remove(queue, registry, entity_bits, DOMINANCE);
        } else {
            queue_set(queue, registry, entity_bits, DOMINANCE, &Dominance(d));
        }
        return;
    }
    if let PhysicsFieldEdit::RestitutionCombine(tag) | PhysicsFieldEdit::FrictionCombine(tag) = edit
    {
        // Collision-material combine (W-Material): read-modify-write the ONE rule the
        // edit names on the optional `MaterialCombine` component (a partial write
        // would drop the other). Gated on a live body: without one there is nothing
        // to give a material to, and this would attach an orphan to a plain sprite.
        // NOT Dynamic-only — a static floor's combine rule matters too.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        // A tag no rule claims is DROPPED, not folded onto Average — the chip that
        // sent it is the only thing that mints one, so an unknown value means the
        // chip list and the enum drifted (the `BodyKind::from_tag` discipline).
        let Some(rule) = CombineRule::from_tag(tag) else {
            debug_assert!(
                false,
                "§11 sent CombineRule tag {tag}, which no variant claims"
            );
            return;
        };
        let mut mat = world
            .get::<MaterialCombine>(entity)
            .copied()
            .unwrap_or_default();
        match edit {
            PhysicsFieldEdit::RestitutionCombine(_) => mat.restitution = rule,
            PhysicsFieldEdit::FrictionCombine(_) => mat.friction = rule,
            _ => unreachable!(),
        }
        // Detach at neutral (both Average) so a default material carries no component
        // — the presence-override idiom, keeping a project file free of the no-op.
        if mat.is_neutral() {
            queue_remove(queue, registry, entity_bits, MATERIAL_COMBINE);
        } else {
            queue_set(queue, registry, entity_bits, MATERIAL_COMBINE, &mat);
        }
        return;
    }
    if let PhysicsFieldEdit::LinearDamping(_)
    | PhysicsFieldEdit::AngularDamping(_)
    | PhysicsFieldEdit::DampMode(_) = edit
    {
        // Per-body damping (W-Damping): read-modify-write the ONE field the edit names
        // on the optional `DampingOverride` component (a partial write would drop the
        // others). Gated on a live body: without one there is nothing to damp, and this
        // would attach an orphan to a plain sprite.
        if world.get::<RigidBody>(entity).is_none() {
            return;
        }
        let mut d = world
            .get::<DampingOverride>(entity)
            .copied()
            .unwrap_or_default();
        match edit {
            // Drag coefficients are non-negative (a negative drag would ADD energy).
            PhysicsFieldEdit::LinearDamping(v) => d.linear = v.max(0.0),
            PhysicsFieldEdit::AngularDamping(v) => d.angular = v.max(0.0),
            PhysicsFieldEdit::DampMode(tag) => {
                // A tag no mode claims is DROPPED, not folded onto Combine (the
                // `BodyKind::from_tag` discipline).
                let Some(mode) = DampMode::from_tag(tag) else {
                    debug_assert!(
                        false,
                        "§11 sent DampMode tag {tag}, which no variant claims"
                    );
                    return;
                };
                d.mode = mode;
            }
            _ => unreachable!(),
        }
        // Detach at neutral (zero drag AND Combine) so a body on the world default
        // carries no component — the presence-override idiom. `Replace` with zero drag
        // is NOT neutral (it forces zero damping, ignoring a world drag), so it stays.
        if d.is_neutral() {
            queue_remove(queue, registry, entity_bits, DAMPING_OVERRIDE);
        } else {
            queue_set(queue, registry, entity_bits, DAMPING_OVERRIDE, &d);
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
        // Collider offset — a signed position, so no clamp (either direction is
        // legal). Writes the ONE axis the edit names; the other is left as it was.
        PhysicsFieldEdit::OffsetX(v) => next.offset[0] = v,
        PhysicsFieldEdit::OffsetY(v) => next.offset[1] = v,
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
        | PhysicsFieldEdit::Ccd(_)
        | PhysicsFieldEdit::LockRotation(_)
        | PhysicsFieldEdit::LockPositionX(_)
        | PhysicsFieldEdit::LockPositionY(_)
        | PhysicsFieldEdit::MassMode(_)
        | PhysicsFieldEdit::Mass(_)
        | PhysicsFieldEdit::Dominance(_)
        | PhysicsFieldEdit::RestitutionCombine(_)
        | PhysicsFieldEdit::FrictionCombine(_)
        | PhysicsFieldEdit::LinearDamping(_)
        | PhysicsFieldEdit::AngularDamping(_)
        | PhysicsFieldEdit::DampMode(_) => {
            unreachable!("handled above")
        }
    }
    if next != cur {
        queue_set(queue, registry, entity_bits, COLLIDER, &next);
    }
}
