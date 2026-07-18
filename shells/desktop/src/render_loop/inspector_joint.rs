//! §12 Physics Joint — the shell half of the Inspector section (W3): the
//! snapshot the panel reads, the ECS write an edit turns into, and the gesture
//! that creates a joint in the first place.
//!
//! Its own module rather than more of `inspector_physics.rs`, for the same
//! reason that one is not more of `inspector_ordering.rs`: a joint is not a
//! body, and this is the whole answer to what the §12 controls do.

use bevy_ecs::world::World;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_editor::{InspectorJointInfo, JointFieldEdit};
use ph2d_physics_ecs::{JointKind, PhysicsJoint};

use super::inspector_ordering::queue_set;

const JOINT: &str = "ph2d::physics::PhysicsJoint";

/// Tag ↔ kind, in one place. The panel speaks tags (it never sees
/// `ph2d-physics-ecs`), so this is the only conversion and both directions
/// live next to each other where a mismatch is visible.
fn kind_of(tag: u8) -> JointKind {
    match tag {
        1 => JointKind::Spring,
        2 => JointKind::Rope,
        _ => JointKind::Pin,
    }
}

fn tag_of(kind: JointKind) -> u8 {
    match kind {
        JointKind::Pin => 0,
        JointKind::Spring => 1,
        JointKind::Rope => 2,
    }
}

/// The name of whichever entity hashes to `id`, or empty when none does.
///
/// A linear scan, on purpose: it runs for the ONE selected joint, twice, and
/// only while §12 is on screen. A cached index would be a second copy of a
/// fact the `Name`s already hold — and one that goes stale on every rename,
/// which is precisely the event this has to report correctly.
fn name_for(world: &World, id: u64, q: &mut bevy_ecs::query::QueryState<(&Name,)>) -> String {
    if id == 0 {
        return String::new();
    }
    q.iter(world)
        .map(|(n,)| n)
        .find(|n| stable_name_id(n.as_str()) == id)
        .map(|n| n.as_str().to_string())
        .unwrap_or_default()
}

/// Build the §12 snapshot. `None` for anything that is not a joint — unlike
/// §11, this section has no empty face: there is nothing useful to offer on an
/// object that is not a joint, and the button that CREATES one lives in §11.
pub(crate) fn build_joint_info(sim: &mut SimWorld, entity_bits: u64) -> Option<InspectorJointInfo> {
    let entity = Entity::from_bits(entity_bits);
    let joint = *sim.world().get::<PhysicsJoint>(entity)?;
    let mut q = sim.world_mut().query::<(&Name,)>();
    let world = sim.world();
    let a = name_for(world, joint.body_a, &mut q);
    let b = name_for(world, joint.body_b, &mut q);
    Some(InspectorJointInfo {
        entity_bits,
        kind_tag: tag_of(joint.kind),
        // `bound` is about the NAMES resolving, which is the thing the artist
        // can act on. Whether the solver also built it depends on those bodies
        // having colliders, and saying "not connected" for a body that is
        // merely not physical yet would point at the wrong problem.
        bound: joint.names_two_bodies() && !a.is_empty() && !b.is_empty(),
        body_a_name: a,
        body_b_name: b,
        limits_enabled: joint.limits_enabled,
        limit_min_deg: joint.limit_min.to_degrees(),
        limit_max_deg: joint.limit_max.to_degrees(),
        motor_enabled: joint.motor_enabled,
        motor_speed_deg: joint.motor_speed.to_degrees(),
        motor_max_force: joint.motor_max_force,
        rest_length: joint.rest_length,
        stiffness: joint.stiffness,
        damping: joint.damping,
        max_length: joint.max_length,
    })
}

/// Apply one [`JointFieldEdit`].
///
/// Every arm reads the live joint and writes it back changed — a partial write
/// would drop the fields not being edited, and this component has eleven of
/// them. `Remove` is not here: deleting a joint is deleting an OBJECT, and the
/// shell already knows how to do that.
pub(crate) fn apply_joint_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: JointFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(entity) else {
        return;
    };
    let mut next = current;
    match edit {
        JointFieldEdit::Kind(tag) => next.kind = kind_of(tag),
        JointFieldEdit::LimitsEnabled(on) => next.limits_enabled = on,
        // Degrees on the way in, radians in the component — the same boundary
        // `Transform::rotation_rad` keeps.
        JointFieldEdit::LimitMinDeg(v) => next.limit_min = v.to_radians(),
        JointFieldEdit::LimitMaxDeg(v) => next.limit_max = v.to_radians(),
        JointFieldEdit::MotorEnabled(on) => next.motor_enabled = on,
        JointFieldEdit::MotorSpeedDeg(v) => next.motor_speed = v.to_radians(),
        JointFieldEdit::MotorMaxForce(v) => next.motor_max_force = v.max(0.0),
        JointFieldEdit::RestLength(v) => next.rest_length = v.max(0.0),
        JointFieldEdit::Stiffness(v) => next.stiffness = v.max(0.0),
        JointFieldEdit::Damping(v) => next.damping = v.max(0.0),
        // A rope of zero length is a weld nobody asked for, and rapier's own
        // docs require the distance to be strictly positive.
        JointFieldEdit::MaxLength(v) => next.max_length = v.max(1e-3),
        JointFieldEdit::Remove => return,
    }
    // Through the SAME clamp the bridge uses on the way to the solver, so the
    // Inspector cannot author a state the loader would have to repair.
    let next = next.clamped();
    if next != current {
        queue_set(queue, registry, entity_bits, JOINT, &next);
    }
}

/// Create a joint between two bodies — the gesture behind §11's *Join Selected
/// Bodies*.
///
/// **Both bodies are given a `Name` if they lack one.** The joint stores name
/// hashes, so an unnamed body is one a joint cannot refer to; naming it here is
/// not a side effect to be apologised for, it is how identity works in this
/// editor (the timeline's bindings have the same requirement).
///
/// The new joint lands at the **midpoint** of the two bodies. One rule for
/// every kind: for a Pin between two touching bodies — a chain link, the
/// common case — the midpoint IS the correct pivot, and for the others it is a
/// sensible place to start dragging from.
pub(crate) fn create_joint(sim: &mut SimWorld, a_bits: u64, b_bits: u64) -> Option<Entity> {
    let (a, b) = (Entity::from_bits(a_bits), Entity::from_bits(b_bits));
    if a == b {
        return None;
    }
    let pa = sim.world().get::<Transform>(a)?.translation;
    let pb = sim.world().get::<Transform>(b)?.translation;

    let name_a = ensure_named(sim, a, "Body")?;
    let name_b = ensure_named(sim, b, "Body")?;

    // ⚠️ The `a == b` guard above compares ENTITIES; this compares the thing a
    // joint actually stores. Two bodies that happen to share a name resolve to
    // one id, so the joint could never bind — and it would report success.
    if ph2d_ecs::stable_name_id(&name_a) == ph2d_ecs::stable_name_id(&name_b) {
        return None;
    }
    let label = crate::name_unique::unique_name(sim, "Joint");
    let mid = (pa + pb) * 0.5;
    let joint = sim
        .world_mut()
        .spawn((
            Name::new(label),
            PhysicsJoint {
                body_a: stable_name_id(&name_a),
                body_b: stable_name_id(&name_b),
                ..PhysicsJoint::default()
            },
            Transform::from_translation(mid),
        ))
        .id();
    // Every root object gets an explicit z, or the tree falls back to sorting
    // by entity bits — which the undo's respawn changes (the W3-era lesson
    // that `assign_missing_root_order` exists for).
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    Some(joint)
}

/// The entity's name, assigning a unique one first if it has none.
fn ensure_named(sim: &mut SimWorld, entity: Entity, base: &str) -> Option<String> {
    if let Some(n) = sim.world().get::<Name>(entity)
        && !n.as_str().is_empty()
    {
        return Some(n.as_str().to_string());
    }
    let fresh = crate::name_unique::unique_name(sim, base);
    sim.world_mut()
        .get_entity_mut(entity)
        .ok()?
        .insert(Name::new(fresh.clone()));
    Some(fresh)
}
