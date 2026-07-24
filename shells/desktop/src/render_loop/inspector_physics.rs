//! §11 Physics Body — the shell half of the Inspector section (ADR-0131 D8):
//! the snapshot the panel reads. The ECS WRITE half (`apply_physics_edit`) lives
//! in the sibling `inspector_physics_apply` and is re-exported below, so every
//! caller keeps its `inspector_physics::apply_physics_edit` path (the file hit
//! the HR-18 600-LOC cap when the material-combine arm landed).

use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_editor::InspectorPhysicsInfo;

/// The WRITE half, in the sibling module. Re-exported so callers are unchanged.
pub(crate) use super::inspector_physics_apply::apply_physics_edit;

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
        AreaBuoyancy, AreaDrag, AreaEffector, AreaFormDrag, AreaTorque, Ccd, Collider,
        ColliderShape, DampingOverride, Dominance, GravityScale, InitialVelocity, LockPositionX,
        LockPositionY, LockRotation, MassOverride, MaterialCombine, OneWayPlatform, RigidBody,
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
    // Optional LockRotation marker (Freeze Rotation); presence is the flag.
    let lock_rotation = world.get::<LockRotation>(entity).is_some();
    // Optional Freeze Position markers (W-LockPos); each presence is a flag.
    let lock_x = world.get::<LockPositionX>(entity).is_some();
    let lock_y = world.get::<LockPositionY>(entity).is_some();
    // Optional mass override (W-Mass); presence = Manual mode, value = the kg. Absent
    // = Auto (density-derived), and the Mass row is not shown.
    let mass_ov = world.get::<MassOverride>(entity).map(|m| m.0);
    // Optional dominance (W-Dominance); absent = neutral 0.
    let dominance = world.get::<Dominance>(entity).map_or(0, |d| d.0);
    // Optional material combine (W-Material); absent = both Average (tags 0, 0). A
    // collider material property, so it is read for any body kind (not Dynamic-only).
    let material = world
        .get::<MaterialCombine>(entity)
        .copied()
        .unwrap_or_default();
    // Optional damping override (W-Damping); absent = the world default drag, shown as
    // 0/0/Combine (tags). Read for any kind here; the rows are Dynamic-only in paint.
    let damping = world
        .get::<DampingOverride>(entity)
        .copied()
        .unwrap_or_default();
    // Optional OneWayPlatform marker (W-OneWay); its presence is the flag. A collider
    // property, so it is read for any body kind (a platform is usually Static).
    let one_way = world.get::<OneWayPlatform>(entity).is_some();
    // Optional AreaForceWorldAxes marker (W-AreaFrame); its presence pins the zone's
    // force to world axes, its absence authors it in the zone's own frame. Read for any
    // kind for the same reason as the marker above: it is a collider question.
    let force_world_axes = world
        .get::<ph2d_physics_ecs::AreaForceWorldAxes>(entity)
        .is_some();
    // Optional AreaEffector (W-Area); absent = a body that pushes nothing. Read for any
    // kind here; the rows are SENSOR-only in paint, which is a collider question, not a
    // body-kind one.
    let force = world
        .get::<AreaEffector>(entity)
        .map_or([0.0, 0.0], |a| a.force);
    // Optional AreaDrag (W-AreaDrag) — the medium half, its own component so that
    // adding it costs no `PROJECT_SCHEMA` bump. Same Sensor-only condition in paint.
    let area_drag = world.get::<AreaDrag>(entity).map_or(0.0, |d| d.0);
    // Optional AreaBuoyancy (W-Buoyancy) — a densidade do fluido, terceiro componente
    // da mesma área e pela mesma razão: um campo novo seria bump de schema.
    let area_density = world.get::<AreaBuoyancy>(entity).map_or(0.0, |b| b.0);
    let area_form_drag = world.get::<AreaFormDrag>(entity).map_or(0.0, |f| f.0);
    // Optional AreaTorque (W-AreaTorque) — o giro que a área imprime, quinto componente
    // da mesma zona e pela mesma razão (campo novo seria bump). Mesma condição de Sensor.
    let area_torque = world.get::<AreaTorque>(entity).map_or(0.0, |t| t.0);
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
            lock_rotation: false,
            offset: [0.0, 0.0],
            lock_x: false,
            lock_y: false,
            mass_manual: false,
            mass: 1.0,
            dominance: 0,
            restitution_combine_tag: 0,
            friction_combine_tag: 0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            damp_mode_tag: 0,
            one_way: false,
            force: [0.0, 0.0],
            force_world_axes: false,
            area_drag: 0.0,
            area_density: 0.0,
            area_form_drag: 0.0,
            area_torque: 0.0,
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
        lock_rotation,
        offset: col.offset,
        lock_x,
        lock_y,
        // Manual mode = the override is present; its value is shown in the Mass row.
        // In Auto mode the Mass row is not shown, so its value is unused (0.0).
        mass_manual: mass_ov.is_some(),
        mass: mass_ov.unwrap_or(0.0),
        dominance,
        restitution_combine_tag: material.restitution.tag(),
        friction_combine_tag: material.friction.tag(),
        linear_damping: damping.linear,
        angular_damping: damping.angular,
        damp_mode_tag: damping.mode.tag(),
        one_way,
        force,
        force_world_axes,
        area_drag,
        area_density,
        area_form_drag,
        area_torque,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{
        AreaEffector, AreaForceWorldAxes, BodyKind, Collider, ColliderShape, RigidBody,
    };

    /// **Selecting a zone shows the frame it was AUTHORED with** (W-AreaFrame).
    ///
    /// The read half of the marker. Its failure mode is the one Enio found in the five
    /// area rows on 2026-07-22: authoring works, the solver honours it, and re-selecting
    /// the zone shows the OTHER value — a control that lies about the state of the thing
    /// it edits. The panel's chip highlight is drawn straight from this field, and it is
    /// the only observable the seam cannot reach (a `seg_row`'s selection is a highlight
    /// in the scene, not a widget value), so the gate belongs here, at the source.
    #[test]
    fn selecting_a_zone_shows_the_force_frame_it_was_authored_with() {
        let zone = |marked: bool| {
            let mut world = World::new();
            let e = world
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 1.0,
                            half_y: 1.0,
                        },
                        is_sensor: true,
                        ..Collider::default()
                    },
                    AreaEffector { force: [3.0, 0.0] },
                ))
                .id();
            if marked {
                world.entity_mut(e).insert(AreaForceWorldAxes);
            }
            build_physics_info(&world, e.to_bits(), false, 5.0, 0)
                .expect("a zone has a Transform, so §11 describes it")
                .force_world_axes
        };
        assert!(
            !zone(false),
            "an unmarked zone is authored in its OWN frame — the default"
        );
        assert!(
            zone(true),
            "a zone carrying `AreaForceWorldAxes` must read back as world-axes; the \
             builder is not reading the marker, so the chip would show the wrong side"
        );
    }
}
