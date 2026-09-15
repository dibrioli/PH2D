//! **A semente dos campos da secção STATE MACHINE** (TOP-20 #15, W3).
//!
//! ⚠️ **Irmão de [`crate::sync_sections`] por CAP de FICHEIRO** — o mesmo corte que o
//! [`crate::sync_sprite_value`] fez, e por responsabilidade: uma secção, um ficheiro.
//!
//! ⚠️ **DUAS sementes e não uma** — as listas de estados e de setas são independentes, e uma aresta
//! partilhada faria abrir um estado reescrever a seta que o artista estava a editar.
//!
//! ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
//! antes de o commit da shell chegar (a lei que as três irmãs do ficheiro-pai já pagaram).

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// **As DUAS arestas da secção** — chamada pelo [`crate::sync_sections`].
///
/// ⛔ **Sem isto o painel mostrava os valores de PARTIDA do `populate`**, nunca os do objecto — o
/// defeito que a auditoria de 2026-09-10 mediu nas irmãs, e que o `#13` e o `#14` pagaram por
/// escrito.
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    if let Some(sm) = crate::state_components::current_inspector_statemachine() {
        let est = if sm.states.is_empty() {
            0
        } else {
            inspector_state.sm_state_selected.min(sm.states.len() - 1)
        };
        let est_mudou = inspector_state.last_sm_state_row != Some(est);
        inspector_state.last_sm_state_row = Some(est);
        let tr = if sm.transitions.is_empty() {
            0
        } else {
            inspector_state
                .sm_trans_selected
                .min(sm.transitions.len() - 1)
        };
        let tr_mudou = inspector_state.last_sm_trans_row != Some(tr);
        inspector_state.last_sm_trans_row = Some(tr);
        sync_statemachine_fields(
            host,
            &sm,
            est,
            tr,
            entity_changed || est_mudou,
            entity_changed || tr_mudou,
        );
    }
}

/// Semeia os campos da secção STATE MACHINE a partir do snapshot.
///
/// ⚠️ **DUAS sementes e não uma** — as listas de estados e de setas são independentes, e uma aresta
/// partilhada faria abrir um estado reescrever a seta que o artista estava a editar.
///
/// ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
/// antes de o commit da shell chegar (a lei que as três irmãs acima já pagaram).
pub(crate) fn sync_statemachine_fields(
    host: &mut dyn PanelHostInternal,
    sm: &ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo,
    est: usize,
    tr: usize,
    semeia_estado: bool,
    semeia_seta: bool,
) {
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⚠️ **O INICIAL semeia-se na aresta da ENTIDADE**, e por isso viaja com a do estado: ele é do
    // objecto, não da linha aberta.
    if semeia_estado
        && focus != Some(crate::ids::INSP_SM_INITIAL)
        && drag != Some(crate::ids::INSP_SM_INITIAL)
    {
        host.store_mut()
            .set_number_value(crate::ids::INSP_SM_INITIAL, f64::from(sm.initial));
    }
    if semeia_estado && let Some(r) = sm.states.get(est) {
        for (id, value) in [
            (crate::ids::INSP_SM_STATE_NAME, &r.name),
            (crate::ids::INSP_SM_STATE_ON_ENTER, &r.on_enter),
            (crate::ids::INSP_SM_STATE_ON_EXIT, &r.on_exit),
        ] {
            escreve_texto(host, focus, id, value);
        }
    }
    if semeia_seta && let Some(t) = sm.transitions.get(tr) {
        for (id, v) in [
            (crate::ids::INSP_SM_TRANS_FROM, t.from),
            (crate::ids::INSP_SM_TRANS_TO, t.to),
        ] {
            if focus != Some(id) && drag != Some(id) {
                host.store_mut().set_number_value(id, f64::from(v));
            }
        }
        escreve_texto(host, focus, crate::ids::INSP_SM_TRANS_ON, &t.on);
    }
}

/// Escreve um campo de texto, **menos o que está em FOCO** — ele é do dedo, e reescrevê-lo
/// enquanto se digita apagaria a letra.
fn escreve_texto(
    host: &mut dyn PanelHostInternal,
    focus: Option<ph2d_a11y::NodeId>,
    id: ph2d_a11y::NodeId,
    value: &str,
) {
    if focus == Some(id) {
        return;
    }
    if let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = host.store_mut().get_mut(id)
    {
        text.clear();
        text.push_str(value);
        *caret = text.len();
        *selection_anchor = None;
    }
}
