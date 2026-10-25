//! **O despacho do cartão CONSTRAINTS.** A linha é achada pela POSIÇÃO do id na tabela dele; o que
//! vai ao barramento é o `Item::id` dessa linha no instantâneo — nunca a posição.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::constraint_edits::{ConstraintFieldEdit as E, InspectorConstraintsInfo};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

use crate::ids;
use crate::state::InspectorState;

/// O `Item::id` da linha cujo widget é `id` na `tabela`.
fn item(
    info: &InspectorConstraintsInfo,
    tabela: &[ph2d_a11y::NodeId],
    id: ph2d_a11y::NodeId,
) -> Option<u32> {
    let k = tabela.iter().position(|&o| o == id)?;
    info.rows.get(k).map(|r| r.id)
}

/// Despacha um evento do cartão **CONSTRAINTS**. `true` = consumido.
pub(crate) fn apply_constraint_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_constraints() else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_CONSTRAINT_ADD => {
            panel.constraint_add_open = !panel.constraint_add_open;
            demote(host, id);
            return true;
        }
        WidgetEvent::Click(id) => {
            if let Some(i) = ids::INSP_CONSTRAINT_TIPO.iter().position(|&o| o == id) {
                panel.constraint_add_open = false;
                E::Adicionar(u8::try_from(i).unwrap_or(0))
            } else if let Some(r) = item(&info, &ids::INSP_CONSTRAINT_SUBIR, id) {
                E::Mover(r, -1)
            } else if let Some(r) = item(&info, &ids::INSP_CONSTRAINT_DESCER, id) {
                E::Mover(r, 1)
            } else if let Some(r) = item(&info, &ids::INSP_CONSTRAINT_APAGAR, id) {
                E::Apagar(r)
            } else if let Some(r) = item(&info, &ids::INSP_CONSTRAINT_ALVO, id) {
                E::EscolherAlvo(r)
            } else {
                return false;
            }
        }
        // O clique afirma o CONTRÁRIO do que o instantâneo diz — nunca o que o store guarda.
        WidgetEvent::Toggled(id) => {
            let Some(k) = ids::INSP_CONSTRAINT_LIGADA.iter().position(|&o| o == id) else {
                return false;
            };
            let Some(row) = info.rows.get(k) else {
                return false;
            };
            E::Ligada(row.id, !row.ligada)
        }
        WidgetEvent::ValueChanged(id) => {
            let Some(r) = item(&info, &ids::INSP_CONSTRAINT_FORCA, id) else {
                return false;
            };
            let v = host.store().slider(id).map_or(0.0, |(_, v)| v);
            E::Forca(r, v.clamp(0.0, 1.0))
        }
        _ => return false,
    };
    if let WidgetEvent::Click(id) = ev {
        demote(host, id);
    }
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits: info.entity_bits,
        edit: ComponentEdit::Constraint(edit),
    });
    true
}

fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
