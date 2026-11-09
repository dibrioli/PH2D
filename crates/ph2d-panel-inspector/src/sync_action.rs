//! **A semente dos campos da secção SIGNAL ACTIONS** — irmã de [`crate::sync_action_trigger`].
//!
//! Faltava desde o painel (9b0714dfa6): a lista lia o snapshot e mostrava `coin_taken → Destroy`, e
//! os campos de texto por baixo ficavam com o vazio do `populate_action` (smoke do dono 09/11).

use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// De ARESTA (entidade ou linha aberta), nunca por quadro: o campo em foco é do dedo.
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(g) = crate::state_components::current_inspector_action() else {
        return;
    };
    let linha = inspector_state
        .action_selected
        .min(g.rows.len().saturating_sub(1));
    let mudou = inspector_state.last_action_row != Some(linha);
    inspector_state.last_action_row = Some(linha);
    if !(entity_changed || mudou) {
        return;
    }
    let Some(row) = g.rows.get(linha) else {
        return;
    };
    let focus = host.store().focus_id();
    for (id, valor) in [
        (crate::ids::INSP_ACTION_ON, &row.on),
        (crate::ids::INSP_ACTION_TARGET, &row.target),
        (crate::ids::INSP_ACTION_ARG, &row.arg),
    ] {
        crate::sync_text_field::escreve_texto(host, focus, id, valor);
    }
}
