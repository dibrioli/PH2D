//! **O despacho da secção DRAWING RULES.** A linha é achada pela POSIÇÃO do id na tabela dele; o que
//! vai ao barramento é o `RegraId` dessa linha no instantâneo — nunca a posição.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::cad_edits::{CadFieldEdit as E, InspectorCadInfo, InspectorRegraRow};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

use crate::ids;

/// A linha cujo widget é `id` na `tabela`.
fn linha<'a>(
    info: &'a InspectorCadInfo,
    tabela: &[ph2d_a11y::NodeId],
    id: ph2d_a11y::NodeId,
) -> Option<&'a InspectorRegraRow> {
    info.rows.get(tabela.iter().position(|&o| o == id)?)
}

/// Despacha um evento da secção **DRAWING RULES**. `true` = consumido.
pub(crate) fn apply_cad_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_cad() else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) => {
            let Some(r) = linha(&info, &ids::INSP_CAD_APAGAR, id) else {
                return false;
            };
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            E::Apagar(r.id)
        }
        // Uma REF só mostra a medida: o campo nem é pintado, e um evento perdido não a escreve.
        WidgetEvent::ValueChanged(id) => {
            let Some(r) = linha(&info, &ids::INSP_CAD_MEDIDA, id) else {
                return false;
            };
            if r.referencia || r.medida.is_none() {
                return false;
            }
            #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
            let v = host.store().number_value(id).unwrap_or(0.0) as f32;
            E::Medida(r.id, v)
        }
        _ => return false,
    };
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits: info.entity_bits,
        edit: ComponentEdit::Cad(edit),
    });
    true
}
