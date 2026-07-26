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
/// live next to each other where a mismatch is visible. `pub(crate)` so the
/// Join gesture can create a joint of the artist's chosen kind (the join-kind
/// selector stores a tag).
pub(crate) fn kind_of(tag: u8) -> JointKind {
    match tag {
        1 => JointKind::Spring,
        2 => JointKind::Rope,
        3 => JointKind::Weld,
        _ => JointKind::Pin,
    }
}

fn tag_of(kind: JointKind) -> u8 {
    match kind {
        JointKind::Pin => 0,
        JointKind::Spring => 1,
        JointKind::Rope => 2,
        JointKind::Weld => 3,
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
///
/// `pick_armed` (`0` none / `1` A / `2` B) is the shell's armed-pick state for
/// THIS joint, mirrored into the snapshot so the painter draws the waiting
/// eyedropper pressed.
pub(crate) fn build_joint_info(
    sim: &mut SimWorld,
    entity_bits: u64,
    pick_armed: u8,
) -> Option<InspectorJointInfo> {
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
        pick_armed,
    })
}

/// Re-bind slot A (or B, if `slot_b`) of the joint at `joint_bits` to `target`
/// — the resolve half of the §12 eyedropper pick, called from `input_dispatch`
/// when the armed pick's next canvas click lands on a body. Returns whether the
/// bind took (so the caller clears the armed pick only on success).
///
/// The target is NAMED if it lacks one (a joint refers to bodies by name hash,
/// the requirement `create_joint` also has), then the hash is written **in
/// place** — not through the editor queue like the field edits, because the pick
/// resolves mid-frame in the pointer handler, and the global diff-based undo
/// captures a direct write the same as a queued one. The value still goes
/// through `clamped()`, the one door before the solver.
///
/// ⚠️ **Refuses a self-joint.** Picking the body already in the OTHER slot would
/// name both ends the same body — a joint that can never bind — so it returns
/// `false` and the pick stays armed for another click, rather than writing a
/// silently-dormant joint.
#[must_use]
pub(crate) fn set_joint_body(
    sim: &mut SimWorld,
    joint_bits: u64,
    slot_b: bool,
    target: Entity,
) -> bool {
    let joint_e = Entity::from_bits(joint_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint_e) else {
        return false;
    };
    let Some(name) = ensure_named(sim, target, "Body") else {
        return false;
    };
    let hash = stable_name_id(&name);
    let mut next = current;
    if slot_b {
        next.body_b = hash;
    } else {
        next.body_a = hash;
    }
    if next.body_a == next.body_b {
        return false; // a self-joint is dormant — keep the pick armed instead
    }
    // Re-picking a body changes which frame the stored local anchor belongs to,
    // so it is meaningless for the new body: mark the joint un-anchored. The next
    // reconcile re-derives both body-local anchors from the joint's current
    // display pivot against the NEW bodies, re-gluing the pin where it visibly is.
    next.anchored = false;
    let next = next.clamped();
    if next != current
        && let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint_e)
    {
        *j = next;
    }
    true
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
    let Some(next) = joint_with_edit(current, edit) else {
        return;
    };
    if next != current {
        queue_set(queue, registry, entity_bits, JOINT, &next);
    }
}

/// **One edit applied to one joint** — the pure half of [`apply_joint_edit`],
/// and the single funnel every author of these fields goes through.
///
/// Extracted in W-J3 so that DRAGGING a limit wall or a length ring writes the
/// same way TYPING the number does: same degrees-to-radians boundary, same
/// per-field floors, same `clamped()`. Two conversions of "what does this edit
/// mean" is how a posed limit and a typed one would come to disagree — and the
/// disagreement would be invisible, because each looks right on its own.
///
/// `None` for the edits that are not a component write (the eyedroppers arm
/// shell state; `Remove` deletes the entity).
#[must_use]
pub(crate) fn joint_with_edit(current: PhysicsJoint, edit: JointFieldEdit) -> Option<PhysicsJoint> {
    let mut next = current;
    match edit {
        JointFieldEdit::Kind(tag) => {
            next.kind = kind_of(tag);
            // The anchor POLICY depends on the kind: a shared-point joint
            // (Pin/Weld) anchors both bodies at the pivot, a two-ended one
            // (Spring/Rope) anchors body B at its own centre. So a kind change is
            // a reposition of the B-end — mark it un-anchored so the next
            // reconcile re-derives the body-local anchors under the NEW policy
            // (the 4th authoring site, beside the dot drag, Position and re-pick).
            // Without this a Pin turned into a Rope keeps the Pin's shared-point
            // anchor and the rope hangs from the wrong spot on body B.
            next.anchored = false;
        }
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
        // The eyedropper ARMS a pick in the action loop (it sets shell state,
        // not a component), and the pick RESOLVES in `input_dispatch` via
        // `set_joint_body`. Neither reaches this per-joint apply; listed so the
        // match stays exhaustive, exactly like `Remove`.
        JointFieldEdit::PickBodyA | JointFieldEdit::PickBodyB => return None,
        JointFieldEdit::Remove => return None,
    }
    // Through the SAME clamp the bridge uses on the way to the solver, so the
    // Inspector cannot author a state the loader would have to repair.
    Some(next.clamped())
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
///
/// `kind` is the artist's choice from §11's join-kind selector — the gold
/// standard is to create the type you want, not to make a Pin and convert it.
/// `anchored` is left `false` (the default): the first reconcile seeds the
/// body-local anchors from the midpoint under THIS kind's policy (a Spring/Rope
/// anchors body B at its centre, a Pin/Weld at the shared point).
pub(crate) fn create_joint(
    sim: &mut SimWorld,
    a_bits: u64,
    b_bits: u64,
    kind: JointKind,
) -> Option<Entity> {
    create_joint_at(sim, a_bits, b_bits, kind, None)
}

/// The same creation, with the anchors the GESTURE named (W-J4).
///
/// `at` is `Some((anchor_a_world, anchor_b_world))` for the canvas gesture —
/// press point and release point — and `None` for the selection route, which has
/// no points to offer and lets the seed policy place them (`anchored: false`).
///
/// ⚠️ **With points, the joint is born ALREADY anchored**: the locals are
/// converted here, against each body's authored pose, and `anchored: true` tells
/// the reconcile to READ them rather than re-derive from the policy. Going
/// through the seed instead would throw the gesture away — the policy puts a
/// spring's B end at the body's CENTRE, which is exactly the *"anchors are born
/// in centres, not where I dragged"* failure this wave exists to remove.
///
/// One function for both routes, so the naming, the two `a == b` guards, the
/// unique label and the `RootOrder` stamp cannot drift between them.
pub(crate) fn create_joint_at(
    sim: &mut SimWorld,
    a_bits: u64,
    b_bits: u64,
    kind: JointKind,
    at: Option<([f32; 2], [f32; 2])>,
) -> Option<Entity> {
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
    // The authored poses of the two bodies — what a body-local anchor is
    // measured against (the seed uses the same `rest`, and using the LIVE pose
    // would bake a swing into the local; W-AnchorFollow).
    let pose = |e: Entity| {
        sim.world()
            .get::<Transform>(e)
            .map(|t| [t.translation.x, t.translation.y, t.rotation])
    };
    let anchored = at.and_then(|(wa, wb)| {
        // A shared-point kind is ONE place: the press point is the pivot, and
        // both bodies are anchored to it. A two-ended kind gets both points.
        let wb = if kind.shares_a_point() { wa } else { wb };
        Some((
            ph2d_physics_ecs::PhysicsWorld::local_anchor_at_pose(pose(a)?, wa),
            ph2d_physics_ecs::PhysicsWorld::local_anchor_at_pose(pose(b)?, wb),
            wa,
        ))
    });
    // The display pivot: the A anchor when the gesture named one (the value
    // `sync_joint_pivots` will keep deriving), the midpoint otherwise.
    let origin = anchored.map_or((pa + pb) * 0.5, |(_, _, wa)| {
        ph2d_core::Vec2::new(wa[0], wa[1])
    });
    let joint = sim
        .world_mut()
        .spawn((
            Name::new(label),
            PhysicsJoint {
                body_a: stable_name_id(&name_a),
                body_b: stable_name_id(&name_b),
                kind,
                local_a: anchored.map_or([0.0, 0.0], |(la, _, _)| la),
                local_b: anchored.map_or([0.0, 0.0], |(_, lb, _)| lb),
                anchored: anchored.is_some(),
                ..PhysicsJoint::default()
            },
            Transform::from_translation(origin),
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
