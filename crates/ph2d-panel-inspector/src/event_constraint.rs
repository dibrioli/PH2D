//! **O despacho do cartão CONSTRAINTS.** A linha é achada pela POSIÇÃO do id na tabela dele; o que
//! vai ao barramento é o `Item::id` dessa linha no instantâneo — nunca a posição.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::constraint_edits::{
    AjusteNumero, ConstraintFieldEdit as E, InspectorConstraintsInfo,
};
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

/// O `(Item::id, ajuste)` do número `id`: a linha pela coluna da tabela, o ajuste pelo `j` dela.
fn numero(info: &InspectorConstraintsInfo, id: ph2d_a11y::NodeId) -> Option<(u32, AjusteNumero)> {
    ids::INSP_CONSTRAINT_NUMERO
        .iter()
        .enumerate()
        .find_map(|(j, tabela)| {
            let row = info.rows.get(tabela.iter().position(|&o| o == id)?)?;
            Some((row.id, *row.numeros.get(j)?))
        })
}

/// A edição da opção `id`: a escolha `j`, a opção `o`, a linha `k`.
fn opcao(info: &InspectorConstraintsInfo, id: ph2d_a11y::NodeId) -> Option<E> {
    ids::INSP_CONSTRAINT_OPCAO
        .iter()
        .enumerate()
        .find_map(|(j, opcoes)| {
            opcoes.iter().enumerate().find_map(|(o, tabela)| {
                let row = info.rows.get(tabela.iter().position(|&n| n == id)?)?;
                let e = row.escolhas.get(j).filter(|e| o < e.opcoes.len())?;
                #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
                Some(E::Campo(row.id, e.campo, o as f32))
            })
        })
}

/// A edição da opção `id` do ▾ de propriedade aberto: o `(linha, ▾)` é o de quem REGISTOU as opções
/// (o `open` do chip já é `false` quando o clique chega), a opção pela posição na lista dele.
fn propriedade(info: &InspectorConstraintsInfo, id: ph2d_a11y::NodeId) -> Option<E> {
    let i = ids::INSP_CONSTRAINT_PROP_OPCAO
        .iter()
        .position(|&o| o == id)?;
    let (k, j) = crate::state_popovers::constraint_prop_pintado()?;
    let row = info.rows.get(k)?;
    let p = row.propriedades.get(j)?;
    Some(E::Propriedade(row.id, p.campo, p.opcoes.get(i)?.prop))
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
            } else if let Some(r) = item(&info, &ids::INSP_CONSTRAINT_ZERAR, id) {
                E::ZerarDesvio(r)
            } else if let Some(c) = ids::INSP_COLA_DESCOLAR
                .iter()
                .position(|&o| o == id)
                .and_then(|k| info.colas.get(k))
            {
                E::Descolar(c.dono_bits, c.id)
            } else if id == ids::INSP_CONSTRAINT_BAKE && !info.rows.is_empty() {
                E::Cozer
            } else if let Some(e) = opcao(&info, id) {
                e
            } else if let Some(e) = propriedade(&info, id) {
                if let Some((k, j)) = crate::state_popovers::constraint_prop_pintado()
                    && let Some(&chip) = ids::INSP_CONSTRAINT_PROP.get(j).and_then(|t| t.get(k))
                    && let Some(InteractiveState::Dropdown { open, .. }) =
                        host.store_mut().get_mut(chip)
                {
                    *open = false;
                }
                e
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
            if let Some(r) = item(&info, &ids::INSP_CONSTRAINT_FORCA, id) {
                let v = host.store().slider(id).map_or(0.0, |(_, v)| v);
                E::Forca(r, v.clamp(0.0, 1.0))
            } else if let Some((r, n)) = numero(&info, id) {
                let v = host.store().number_value(id).unwrap_or(0.0);
                E::Campo(
                    r,
                    n.campo,
                    crate::sync_constraint::do_ecra(n.unidade, v, host.project()),
                )
            } else {
                return false;
            }
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
