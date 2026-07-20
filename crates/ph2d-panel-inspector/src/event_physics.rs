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
use ph2d_editor_core::screens::hero::PhysicsFieldEdit;

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
            ids::INSP_PHYS_DENSITY => Some(PhysicsFieldEdit::Density(v)),
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
            _ => None,
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
