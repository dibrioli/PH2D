//! Panel → shell. Every arm is derived from [`crate::rows`], so a row that
//! exists is a row that dispatches.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal, seam_reset_button};
use ph2d_physics_ecs::PhysicsSettings;

use crate::rows;
use crate::state::{self, PhysicsIntent};

pub(crate) fn apply_event(
    _state: &mut crate::state::PhysicsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        // A slider moved: read the track the drag left behind, turn it into
        // this row's value, and emit the WHOLE settings struct with that one
        // field replaced.
        WidgetEvent::ValueChanged(id) => {
            if let Some(row) = rows::row_for(id) {
                if id == row.chip {
                    // The chip edit was already mirrored onto its linked slider,
                    // which fired its own ValueChanged and was handled there.
                    // Swallow, or one edit notifies twice.
                    true
                } else {
                    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                    let mut settings = state::current().settings;
                    (row.set)(&mut settings, row.value_of(track));
                    state::push_intent(PhysicsIntent::SetSettings(settings));
                    true
                }
            } else {
                false
            }
        }
        // Section headers fold. Panel-local view state, so it never becomes an
        // intent — the shell has no opinion about which sections are open.
        WidgetEvent::Click(id)
            if id == ids::PHYSICS_SEC_DEBUG || rows::SECTIONS.iter().any(|s| s.id == id) =>
        {
            seam_reset_button(host, id);
            let collapsed = host.store().is_collapsed(id);
            host.store_mut().set_collapsed(id, !collapsed);
            true
        }
        WidgetEvent::Click(id) if id == ids::PHYSICS_SHOW_COLLIDERS => {
            seam_reset_button(host, id);
            state::push_intent(PhysicsIntent::ToggleColliders);
            true
        }
        WidgetEvent::Click(id) if id == ids::PHYSICS_RESET_DEFAULTS => {
            seam_reset_button(host, id);
            state::push_intent(PhysicsIntent::SetSettings(PhysicsSettings::default()));
            true
        }
        WidgetEvent::Click(id) if id == ids::PHYSICS_CLOSE => {
            seam_reset_button(host, id);
            host.set_panel_visible(crate::PhysicsPanel::ID, false);
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
