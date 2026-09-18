//! **A semente dos campos da secção GATILHO** (suplente #24).
//!
//! ⚠️⚠️ **Irmã de [`crate::sync_counter_watch`], e ela existe por um defeito MEDIDO na irmã**
//! (2026-09-17, achado por uma FOTO): com três regras na lista e a primeira escolhida, o editor por
//! baixo mostrava os valores de PARTIDA do `populate`, nunca os da regra aberta. *Uma secção
//! meio-semeada lê-se como semeada em qualquer gate que pergunte pelo snapshot* — o chip, a lista e
//! os avisos leem todos o snapshot e estavam certos; só os campos editáveis vivem no `WidgetStore`.
//!
//! ⇒ esta secção nasce com a semente, e não com a dívida.

use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// **A aresta da secção** — chamada pelo [`crate::sync_sections`].
///
/// ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
/// antes de o commit da shell chegar (a lei que as seis irmãs já pagaram).
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(g) = crate::state_components::current_inspector_action_trigger() else {
        return;
    };
    // ⚠️ **O `min` é a mesma cerca das irmãs:** apagar a última linha deixa o índice fora da lista,
    // e sem ele a semente cairia no `else` e o editor ficava a mostrar a linha APAGADA.
    let linha = if g.rows.is_empty() {
        0
    } else {
        inspector_state.trigger_selected.min(g.rows.len() - 1)
    };
    let mudou = inspector_state.last_trigger_row != Some(linha);
    inspector_state.last_trigger_row = Some(linha);
    if !(entity_changed || mudou) {
        return;
    }
    let Some(row) = g.rows.get(linha) else {
        return;
    };
    let focus = host.store().focus_id();
    // ⛔ **O CHIP da aresta NÃO entra aqui** — ele é pintado a partir do snapshot
    //    (`dd.select(row.edge)`) e o `event` decide a partir do snapshot também. Semeá-lo seria a
    //    SEGUNDA resposta à mesma pergunta, que é o defeito oposto a este.
    for (id, valor) in [
        (crate::ids::INSP_TRIGGER_ACTION, &row.action),
        (crate::ids::INSP_TRIGGER_SIGNAL, &row.signal),
    ] {
        crate::sync_text_field::escreve_texto(host, focus, id, valor);
    }
}
