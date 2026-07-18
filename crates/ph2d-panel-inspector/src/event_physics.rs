//! §11 Physics Body — the Inspector event arms (ADR-0130 D8).
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
    // dimension commits (ADR-0130 D8).
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
        } else if let Some(i) = ids::INSP_PHYS_KIND.iter().position(|&o| o == id) {
            Some(PhysicsFieldEdit::Kind(i as u8))
        } else {
            ids::INSP_PHYS_SHAPE
                .iter()
                .position(|&o| o == id)
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
