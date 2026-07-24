//! §11 Physics Body — the Inspector event arms (ADR-0131 D8).
//!
//! Its own module rather than another arm in `event_ordering`: physics is not
//! ordering, and that dispatcher is at its LOC cap. The split is the honest
//! one — this file is the whole answer to "what happens when the artist
//! clicks a physics control".

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit};

pub(crate) fn apply_physics_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // §11 Physics Body — Add / Remove, the two segmented groups, and the
    // dimension commits (ADR-0131 D8).
    //
    // Add and Remove are separated on `has_body` rather than trusted from
    // the click alone: the painter only ever offers one of them, but a
    // refusal that lives in the paint loop is not a refusal
    // ([[feedback_disabled_button_still_dispatches]]).
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_physics()
    {
        let edit = if id == ids::INSP_PHYS_ADD && !info.has_body {
            Some(PhysicsFieldEdit::Add)
        } else if id == ids::INSP_PHYS_REMOVE && info.has_body {
            Some(PhysicsFieldEdit::Remove)
        } else if let Some(i) = ids::INSP_PHYS_LAYER.iter().position(|&o| o == id) {
            // Gated on `has_body` like every other field edit: the chips are
            // only painted for a body, and dim is not a refusal
            // ([[feedback_disabled_button_still_dispatches]]).
            info.has_body.then_some(PhysicsFieldEdit::Layer(i as u8))
        } else if id == ids::INSP_PHYS_BAKE {
            // Gated on `has_body` AND on the body being the kind that actually
            // has simulated motion: a Static body never moves and a Kinematic
            // one is already driven by the scene, so a bake of either can only
            // report "nothing moved". The painter declines to offer the button
            // for those; this is the half that declines to honour it, because
            // the id lives in the store all session and dim is not a refusal.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::Bake)
        } else if id == ids::INSP_PHYS_JOIN && info.can_join {
            // Gated on `can_join`, which the SHELL computed — the painter only
            // offers the button when the selection is two bodies, and a
            // refusal that lives in the paint loop is not a refusal
            // ([[feedback_disabled_button_still_dispatches]]).
            Some(PhysicsFieldEdit::Join)
        } else if let Some(i) = ids::INSP_PHYS_KIND.iter().position(|&o| o == id) {
            // Gated like every sibling. Kind and Shape were the only two §11
            // controls with NO `has_body` check, which made `Kind` a second
            // door to attaching an orphan `RigidBody` to a plain sprite — the
            // chips are painted only inside the body block, and a refusal that
            // lives in the paint loop is not a refusal
            // ([[feedback_disabled_button_still_dispatches]]).
            info.has_body.then_some(PhysicsFieldEdit::Kind(i as u8))
        } else if let Some(i) = ids::INSP_PHYS_SENSOR.iter().position(|&o| o == id) {
            // Two segments: `0` Solid, `1` Sensor. Gated on `has_body` like its
            // siblings — the toggle is painted only inside the body block, and
            // dim is not a refusal.
            info.has_body.then_some(PhysicsFieldEdit::Sensor(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_CCD.iter().position(|&o| o == id) {
            // Two segments: `0` Discrete, `1` Continuous. Dynamic-only, the same
            // gate the painter offers it under (only a body the solver moves fast
            // can tunnel). Dim is not a refusal, so the check lives here too.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::Ccd(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_LOCKROT.iter().position(|&o| o == id) {
            // Two segments: `0` Free, `1` Locked. Dynamic-only, the same gate the
            // painter offers it under (only a body the solver rotates has a
            // rotation to freeze). Dim is not a refusal.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::LockRotation(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_LOCKX.iter().position(|&o| o == id) {
            // Freeze Position X — Free | Locked. Dynamic-only, the same gate the
            // painter offers it under (only a body the solver moves has a position
            // to freeze). Dim is not a refusal.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::LockPositionX(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_LOCKY.iter().position(|&o| o == id) {
            // Freeze Position Y — the vertical sibling, same gate.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::LockPositionY(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_MASSMODE.iter().position(|&o| o == id) {
            // Mass source: `0` Auto, `1` Manual. Dynamic-only, the same gate the
            // painter offers it under (a Static/Kinematic body has infinite mass).
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::MassMode(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_REST_COMBINE.iter().position(|&o| o == id) {
            // Restitution combine (W-Material): four segments Average/Min/Multiply/Max.
            // Gated on `has_body` like its siblings — but NOT Dynamic-only: it is a
            // collider material property, so a static floor's rule matters too. Dim is
            // not a refusal, so the check lives here.
            info.has_body
                .then_some(PhysicsFieldEdit::RestitutionCombine(i as u8))
        } else if let Some(i) = ids::INSP_PHYS_FRIC_COMBINE.iter().position(|&o| o == id) {
            // Friction combine — the sibling, same `has_body`-only gate.
            info.has_body
                .then_some(PhysicsFieldEdit::FrictionCombine(i as u8))
        } else if let Some(i) = ids::INSP_PHYS_ONEWAY.iter().position(|&o| o == id) {
            // Off | On (W-OneWay). NOT Dynamic-only — it is a collider property and a
            // platform is usually Static — but it IS solid-only: one-way works by
            // modifying solver CONTACTS, and a sensor generates none, so the painter
            // offers it for a solid collider alone. Dim is not a refusal, so the same
            // condition is asked here (W-Area made this row exclusive with Force).
            (info.has_body && !info.is_sensor).then_some(PhysicsFieldEdit::OneWay(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_FORCE_AXES.iter().position(|&o| o == id) {
            // Zone | World (W-AreaFrame) — the mirror image of the One-Way gate above:
            // SENSOR-only, because it qualifies the force rows, and those only exist for
            // a sensor. Dim is not a refusal, so the condition is asked here too.
            (info.has_body && info.is_sensor).then_some(PhysicsFieldEdit::ForceWorldAxes(i == 1))
        } else if let Some(i) = ids::INSP_PHYS_DAMPMODE.iter().position(|&o| o == id) {
            // Damp mode: `0` Combine, `1` Replace (W-Damping). Dynamic-only, the same
            // gate the painter offers it under (damping decays a velocity only a
            // Dynamic body has). Dim is not a refusal, so the check lives here.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::DampMode(i as u8))
        } else if let Some(i) = ids::INSP_PHYS_BAKE_CH.iter().position(|&o| o == id) {
            // The bake channel selector — a GLOBAL option, but painted only for a
            // Dynamic body (the only kind that bakes), so honoured under the same
            // condition the painter offers it.
            (info.has_body && info.kind_tag == 0).then_some(PhysicsFieldEdit::BakeChannels(i as u8))
        } else {
            ids::INSP_PHYS_SHAPE
                .iter()
                .position(|&o| o == id)
                .filter(|_| info.has_body)
                .map(|i| PhysicsFieldEdit::Shape(i as u8))
        };
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(info) = state::current_inspector_physics()
        && info.has_body
    {
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        let edit = match id {
            ids::INSP_PHYS_RADIUS => Some(PhysicsFieldEdit::Radius(v)),
            ids::INSP_PHYS_HALF_X => Some(PhysicsFieldEdit::HalfX(v)),
            ids::INSP_PHYS_HALF_Y => Some(PhysicsFieldEdit::HalfY(v)),
            ids::INSP_PHYS_CAP_HALF_H => Some(PhysicsFieldEdit::CapHalfHeight(v)),
            // Collider offset — a collider property, honoured for any body (not
            // Dynamic-gated like velocity/gravity).
            ids::INSP_PHYS_OFFSET_X => Some(PhysicsFieldEdit::OffsetX(v)),
            ids::INSP_PHYS_OFFSET_Y => Some(PhysicsFieldEdit::OffsetY(v)),
            ids::INSP_PHYS_DENSITY => Some(PhysicsFieldEdit::Density(v)),
            // The explicit Mass row (Manual mode) — Dynamic-only, the same gate the
            // painter offers it under. The row is only painted in Manual mode, so
            // this cannot fire otherwise, but a refusal in the paint loop is not one.
            ids::INSP_PHYS_MASS if info.kind_tag == 0 => Some(PhysicsFieldEdit::Mass(v)),
            ids::INSP_PHYS_RESTITUTION => Some(PhysicsFieldEdit::Restitution(v)),
            ids::INSP_PHYS_FRICTION => Some(PhysicsFieldEdit::Friction(v)),
            // Honoured only for a Dynamic body — the same gate the painter
            // offers it under (rapier applies gravity to Dynamic bodies only).
            // The row is not painted for other kinds, so this cannot fire for
            // them, but a refusal that lives in the paint loop is not a refusal.
            ids::INSP_PHYS_GRAVITY_SCALE if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::GravityScale(v))
            }
            // Initial velocity (W9), Dynamic-only like gravity. Angular is
            // displayed in deg/s and converted to the component's radians here.
            ids::INSP_PHYS_LINVEL_X if info.kind_tag == 0 => Some(PhysicsFieldEdit::LinvelX(v)),
            ids::INSP_PHYS_LINVEL_Y if info.kind_tag == 0 => Some(PhysicsFieldEdit::LinvelY(v)),
            ids::INSP_PHYS_ANGVEL if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::Angvel(v.to_radians()))
            }
            // Dominance (W-Dominance), Dynamic-only like gravity/velocity. The widget
            // is a float; dominance is an i8 priority, so round and clamp to i8 range
            // at the panel boundary (the shell's edit and the component stay i8).
            ids::INSP_PHYS_DOMINANCE if info.kind_tag == 0 => {
                // `safe_clamp` (NaN-aware) rather than `f32::clamp`: the bounds are
                // `i8::MIN/MAX` cast to f32 — dynamic values, not literal constants,
                // so `arch_safe_clamp_only` requires the safe variant.
                let clamped = ph2d_editor_core::math::safe_clamp(
                    v.round(),
                    f32::from(i8::MIN),
                    f32::from(i8::MAX),
                );
                Some(PhysicsFieldEdit::Dominance(clamped as i8))
            }
            // Per-body damping (W-Damping), Dynamic-only like gravity/velocity.
            ids::INSP_PHYS_LINEAR_DAMPING if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::LinearDamping(v))
            }
            ids::INSP_PHYS_ANGULAR_DAMPING if info.kind_tag == 0 => {
                Some(PhysicsFieldEdit::AngularDamping(v))
            }
            // As sete rows da ZONA — o que esta ÁREA faz a OUTROS corpos, extraídas para
            // um irmão pelo cap de 200 LOC desta fn (o mesmo corte que separou
            // `components/area.rs` e `inspector_physics_area.rs`).
            other => area_edit(other, v, info),
        };
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    false
}

/// **O roteamento das rows de ZONA** — a força, o frame dela, o torque, o falloff e os
/// três knobs de meio (W-Area .. W-AreaFalloff).
///
/// ⚠️ **Todas gateadas em `is_sensor`, nunca no `kind_tag`**, e o motivo é o mesmo para as
/// sete: a narrow phase registra sobreposição só quando um dos lados é sensor, então numa
/// zona sólida qualquer destes números seria autorado e nunca lido. O painel oferece as
/// rows sob exatamente essa condição — e uma recusa que mora no laço de pintura não é uma
/// recusa, por isso ela é repetida aqui.
///
/// Extraída de [`apply_physics_event`] pelo cap de 200 LOC das fns de painel, na linha de
/// corte que a família das zonas já vinha desenhando sozinha nos dois outros arquivos que
/// ela obrigou a separar.
fn area_edit(
    id: ph2d_editor_core::NodeId,
    v: f32,
    info: InspectorPhysicsInfo,
) -> Option<PhysicsFieldEdit> {
    if !info.is_sensor {
        return None;
    }
    match id {
        ids::INSP_PHYS_FORCE_X => Some(PhysicsFieldEdit::ForceX(v)),
        ids::INSP_PHYS_FORCE_Y => Some(PhysicsFieldEdit::ForceY(v)),
        ids::INSP_PHYS_AREA_TORQUE => Some(PhysicsFieldEdit::AreaTorque(v)),
        // Cru: a fração é clampada UMA vez, no apply da shell, onde ela vira componente.
        ids::INSP_PHYS_AREA_FALLOFF => Some(PhysicsFieldEdit::AreaFalloff(v)),
        ids::INSP_PHYS_AREA_DRAG => Some(PhysicsFieldEdit::AreaDrag(v)),
        ids::INSP_PHYS_AREA_DENSITY => Some(PhysicsFieldEdit::AreaDensity(v)),
        ids::INSP_PHYS_AREA_FORM_DRAG => Some(PhysicsFieldEdit::AreaFormDrag(v)),
        _ => None,
    }
}
