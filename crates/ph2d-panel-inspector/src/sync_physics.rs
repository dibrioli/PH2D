//! **O `sync` da FAMÍLIA de física** — §11 Physics Body, §12 Physics Joint e
//! §13 Pulley Wheel.
//!
//! Irmão do [`super::sync`] pelo cap de 600 LOC do arquivo, e o corte é o mesmo
//! que esta linha já desenhou em `populate_physics.rs`, `inspector_model_physics.rs`
//! e `inspector_physics_area.rs`: a churn de física passa a morar num arquivo
//! que ela possui, em vez de empurrar o orquestrador compartilhado do Inspector
//! contra o teto a cada wave.
//!
//! ⚠️ **Os três rodam sob o guarda `entity_changed`** do chamador: um re-seed por
//! frame brigaria com a digitação do artista, sobrescrevendo o buffer no meio da
//! edição.

use crate::state;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// §12 Physics Joint — mirror the snapshot into the number boxes, exactly as
/// the body's dimensions are mirrored below. Only the numbers: the three
/// segmented groups read their selection straight off the snapshot at paint
/// time, so there is nothing in the store for them to fall out of date with.
pub(crate) fn sync_joint_fields(host: &mut dyn PanelHostInternal) {
    let Some(info) = state::current_inspector_joint() else {
        return;
    };
    for (id, v) in [
        (ids::INSP_JOINT_LIMIT_MIN, info.limit_min_ui),
        (ids::INSP_JOINT_LIMIT_MAX, info.limit_max_ui),
        (ids::INSP_JOINT_MOTOR_SPEED, info.motor_speed_ui),
        (ids::INSP_JOINT_MOTOR_TARGET, info.motor_target_ui),
        (ids::INSP_JOINT_MOTOR_FORCE, info.motor_max_force),
        (ids::INSP_JOINT_REST_LENGTH, info.rest_length),
        (ids::INSP_JOINT_STIFFNESS, info.stiffness),
        (ids::INSP_JOINT_DAMPING, info.damping),
        (ids::INSP_JOINT_MAX_LENGTH, info.max_length),
        // W-J7. Without these two the rows would be WRITE-ONLY — the failure
        // the area rows shipped with (W-AreaTorque): typing works, and then
        // re-selecting the joint shows the seed instead of what was authored.
        (ids::INSP_JOINT_BREAK_FORCE, info.break_force),
        (ids::INSP_JOINT_BREAK_TORQUE, info.break_torque),
    ] {
        host.store_mut().set_number_value(id, f64::from(v));
    }
}

/// §11 Physics Body — mirror the snapshot's dimensions into the number boxes.
/// Runs under the same `entity_changed` guard as its siblings: a re-seed on
/// every frame would fight the user's own typing, overwriting the buffer
/// mid-edit.
///
/// Only the dimensions need mirroring — the two segmented groups read their
/// selection straight off the snapshot at paint time, so there is nothing in
/// the store for them to fall out of date with.
pub(crate) fn sync_physics_fields(host: &mut dyn PanelHostInternal) {
    let Some(info) = state::current_inspector_physics() else {
        return;
    };
    if !info.has_body {
        return;
    }
    for (id, v) in [
        (ids::INSP_PHYS_RADIUS, info.radius),
        (ids::INSP_PHYS_HALF_X, info.half_x),
        (ids::INSP_PHYS_HALF_Y, info.half_y),
        (ids::INSP_PHYS_CAP_HALF_H, info.cap_half_height),
        (ids::INSP_PHYS_OFFSET_X, info.offset[0]),
        (ids::INSP_PHYS_OFFSET_Y, info.offset[1]),
        (ids::INSP_PHYS_DENSITY, info.density),
        (ids::INSP_PHYS_MASS, info.mass),
        (ids::INSP_PHYS_RESTITUTION, info.restitution),
        (ids::INSP_PHYS_FRICTION, info.friction),
        (ids::INSP_PHYS_GRAVITY_SCALE, info.gravity_scale),
        (ids::INSP_PHYS_LINVEL_X, info.linvel[0]),
        (ids::INSP_PHYS_LINVEL_Y, info.linvel[1]),
        (ids::INSP_PHYS_ANGVEL, info.angvel.to_degrees()),
        (ids::INSP_PHYS_DOMINANCE, f32::from(info.dominance)),
        (ids::INSP_PHYS_LINEAR_DAMPING, info.linear_damping),
        (ids::INSP_PHYS_ANGULAR_DAMPING, info.angular_damping),
        // The area-zone rows (W-Area..W-AreaTorque). Without these the widgets were
        // WRITE-ONLY: authoring a Force/Torque/Drag worked, but re-selecting the zone
        // showed 0 (or the previous selection's stale value) instead of the number that
        // is actually on the collider — so the artist could not read back what they set.
        // Synced here like every other field; this runs once on selection change (not
        // per frame), so it never fights the value being typed.
        (ids::INSP_PHYS_FORCE_X, info.force[0]),
        (ids::INSP_PHYS_FORCE_Y, info.force[1]),
        (ids::INSP_PHYS_AREA_TORQUE, info.area_torque),
        (ids::INSP_PHYS_AREA_FALLOFF, info.area_falloff),
        (ids::INSP_PHYS_AREA_DRAG, info.area_drag),
        (ids::INSP_PHYS_AREA_DENSITY, info.area_density),
        (ids::INSP_PHYS_AREA_FORM_DRAG, info.area_form_drag),
    ] {
        host.store_mut().set_number_value(id, f64::from(v));
    }
}

/// §13 Pulley Wheel — o mesmo espelho, e ele é o que impede as rows de nascerem
/// WRITE-ONLY (a falha com que as rows de área shiparam, W-AreaTorque: digitar
/// funciona, e re-selecionar mostra o seed em vez do que foi autorado).
///
/// Só os números: os chips de `Wrap` leem a seleção direto do snapshot na hora
/// de pintar, então não há nada no store para eles desatualizarem.
pub(crate) fn sync_wheel_fields(host: &mut dyn PanelHostInternal) {
    let Some(info) = state::current_inspector_wheel() else {
        return;
    };
    host.store_mut()
        .set_number_value(ids::INSP_WHEEL_RADIUS, f64::from(info.radius));
    host.store_mut()
        .set_number_value(ids::INSP_WHEEL_ORDER, f64::from(info.order_ui));
}
