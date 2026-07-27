//! Panel → shell. Every arm is derived from [`crate::rows`], so a row that
//! exists is a row that dispatches.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal, seam_reset_button};
use ph2d_physics_ecs::{InteractionSettings, PhysicsSettings};

use crate::state::{self, PhysicsIntent};
use crate::{interact, rows};

/// Inverse of `ids::physics_layer_cell_index` — the `(row, col)` a flat cell
/// index names. Derived by walking the triangle rather than stored, so the two
/// directions cannot drift apart.
fn cell_pair(index: usize) -> (usize, usize) {
    let mut i = 0usize;
    let mut remaining = index;
    while remaining > i {
        remaining -= i + 1;
        i += 1;
    }
    (i, remaining)
}

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
            } else if let Some(row) = interact::irow_for(id) {
                // The SECOND table (W-Hand). Same shape, other destination: a
                // runtime tool, not the document.
                if id == row.chip {
                    true
                } else {
                    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                    let mut it = state::current().interaction;
                    (row.set)(&mut it, row.value_of(track));
                    state::push_intent(PhysicsIntent::SetInteraction(it));
                    true
                }
            } else {
                false
            }
        }
        // The two radios of the Interaction section. Each option id names exactly
        // one variant, resolved through `interact::tool_for`/`hold_for` — the
        // inverse of the ALL-order the painter uses, so the chip that lights up and
        // the value that lands cannot disagree.
        WidgetEvent::Click(id) if interact::tool_for(id).is_some() => {
            seam_reset_button(host, id);
            let tool = interact::tool_for(id).expect("guard matched");
            let it = state::current().interaction;
            state::push_intent(PhysicsIntent::SetInteraction(InteractionSettings {
                tool,
                ..it
            }));
            true
        }
        WidgetEvent::Click(id) if interact::hold_for(id).is_some() => {
            seam_reset_button(host, id);
            let hold = interact::hold_for(id).expect("guard matched");
            let it = state::current().interaction;
            state::push_intent(PhysicsIntent::SetInteraction(InteractionSettings {
                hold,
                ..it
            }));
            true
        }
        // Section headers fold. Panel-local view state, so it never becomes an
        // intent — the shell has no opinion about which sections are open.
        WidgetEvent::Click(id)
            if id == ids::PHYSICS_SEC_DEBUG
                || id == ids::PHYSICS_SEC_INTERACT
                || rows::SECTIONS.iter().any(|s| s.id == id) =>
        {
            seam_reset_button(host, id);
            let collapsed = host.store().is_collapsed(id);
            host.store_mut().set_collapsed(id, !collapsed);
            true
        }
        // A matrix cell toggles ONE pair. `LayerMatrix::set` writes both halves,
        // so the asymmetric state the panel could otherwise author does not exist.
        WidgetEvent::Click(id) if ids::PHYSICS_LAYER_CELL.contains(&id) => {
            seam_reset_button(host, id);
            let idx = ids::PHYSICS_LAYER_CELL
                .iter()
                .position(|&c| c == id)
                .expect("guard matched");
            let (a, b) = cell_pair(idx);
            let settings = state::current().settings;
            let mut m = ph2d_physics_ecs::LayerMatrix::from_rows(settings.layer_matrix);
            m.set(a, b, !m.collides(a, b));
            state::push_intent(PhysicsIntent::SetSettings(PhysicsSettings {
                layer_matrix: m.rows(),
                ..settings
            }));
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
