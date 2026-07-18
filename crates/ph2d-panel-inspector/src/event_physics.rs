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
            // Gated on `has_body` like the field edits: there is no motion to
            // bake off an entity that is not simulated, and the painter only
            // offers the button inside the body block.
            info.has_body.then_some(PhysicsFieldEdit::Bake)
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
            ids::INSP_PHYS_DENSITY => Some(PhysicsFieldEdit::Density(v)),
            ids::INSP_PHYS_RESTITUTION => Some(PhysicsFieldEdit::Restitution(v)),
            ids::INSP_PHYS_FRICTION => Some(PhysicsFieldEdit::Friction(v)),
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
