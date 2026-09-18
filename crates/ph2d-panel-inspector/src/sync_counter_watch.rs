//! **A semente dos campos da secção COUNTER WATCH** (TOP-20, a vigia do contador).
//!
//! ⛔⛔⛔ **Ela NÃO existia, e o defeito foi achado por uma FOTO** (2026-09-17): com as três regras
//! na lista e a primeira escolhida (`vidas ≤ 2 → luz3`), o editor por baixo mostrava
//! `Counter name` VAZIO, `Value 0` e `Signal name` vazio — os valores de PARTIDA do
//! [`crate::populate_counter_watch`], nunca os da regra aberta.
//!
//! ⚠️⚠️ **Nenhum dos 27 gates da wave podia vê-lo, e a razão é a PARTIÇÃO do que lê o quê:** o
//! chip, a caixa *Only once*, os avisos e o resumo da linha fechada leem todos o **SNAPSHOT** e
//! estavam certos — só os **três campos editáveis** vivem no `WidgetStore`, e o store só é
//! escrito por uma semente. *Uma secção meio-semeada lê-se como semeada em qualquer gate que
//! pergunte pelo snapshot.* O `last_watch_row` do [`crate::state::InspectorState`] estava
//! declarado desde a wave e **nunca lido**: a aresta existia e a semente é que faltava.
//!
//! ⚠️ **Irmão de [`crate::sync_sections`] por CAP de FICHEIRO** — o mesmo corte que o cérebro, o
//! script e as partículas fizeram, e por responsabilidade: uma secção, um ficheiro.

use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// **A aresta da secção** — chamada pelo [`crate::sync_sections`].
///
/// ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
/// antes de o commit da shell chegar (a lei que as cinco irmãs já pagaram).
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(cw) = crate::state_components::current_inspector_counter_watch() else {
        return;
    };
    // ⚠️ **O `min` é a mesma cerca das irmãs:** apagar a última regra deixa o índice fora da lista,
    // e sem ele a semente cairia no `else` e o editor ficava a mostrar a regra APAGADA.
    let linha = if cw.rows.is_empty() {
        0
    } else {
        inspector_state.watch_selected.min(cw.rows.len() - 1)
    };
    let mudou = inspector_state.last_watch_row != Some(linha);
    inspector_state.last_watch_row = Some(linha);
    if !(entity_changed || mudou) {
        return;
    }
    let Some(row) = cw.rows.get(linha) else {
        return;
    };
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⛔ **A caixa `Only once` NÃO entra aqui** — ela é pintada a partir do snapshot
    //    (`paint_check_rows(.., row.once)`) e o `event` decide a partir do snapshot também.
    //    Semeá-la seria a SEGUNDA resposta à mesma pergunta, que é o defeito oposto a este.
    if focus != Some(crate::ids::INSP_WATCH_VALUE) && drag != Some(crate::ids::INSP_WATCH_VALUE) {
        #[expect(
            clippy::cast_precision_loss,
            reason = "o limiar é um i64 e o campo é um f64: acima de 2^53 o painel não é o \
                      instrumento certo, e a lei continua a comparar em i64"
        )]
        host.store_mut()
            .set_number_value(crate::ids::INSP_WATCH_VALUE, row.value as f64);
    }
    // ⚠️ **Os DOIS textos pelo mesmo laço, e pela porta partilhada** — um deles escrito à mão ao
    // lado do outro é como o segundo nasce sem a cerca do FOCO e apaga a letra que se está a
    // digitar (a lição que o `sync_text_field` carrega no cabeçalho).
    for (id, valor) in [
        (crate::ids::INSP_WATCH_COUNTER, &row.counter),
        (crate::ids::INSP_WATCH_SIGNAL, &row.signal),
    ] {
        crate::sync_text_field::escreve_texto(host, focus, id, valor);
    }
}
