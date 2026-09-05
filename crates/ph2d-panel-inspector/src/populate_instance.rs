//! ⭐⭐⭐ **O CARTÃO DE INSTÂNCIA** — o registo de focalização das três superfícies dele
//! (ADR-0164 / F5).
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::populate`]**, pelo tecto de 600 LOC dos painéis — o mesmo
//! corte que o `populate_anchor`, o `populate_anim` e o `populate_physics` já fizeram. ⛔ Ele foi
//! imposto pelo `architecture_panel_loc_cap` ao acrescentar a escada do *Aplicar*, e a lei da casa
//! é pagar um tecto com um **CORTE**, nunca com uma linha de isenção.
//!
//! # ⛔⛔ Focalizar não é pintar, e as três já pagaram isso
//!
//! Um widget pintado e no hit-index continua **morto sob o ponteiro** se não estiver aqui: sem o
//! registo ele nunca é focável, logo o `Down`/`Up` nunca dispara e o `Click` **nunca nasce**. O
//! botão dos órfãos nasceu assim (apanhado pelo `hit_indexed_ids_are_registered`) e o botão
//! *Salvar Variação…* também (apanhado pelo `seam_properties`, na primeira corrida).
//! *Pintado e hit-indexado não é vivo.*

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

/// Um botão do cartão, no estado neutro.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// As três superfícies do cartão: limpar órfãos · a escada do *Aplicar* · os chips das versões.
pub(crate) fn populate_instance_card(store: &mut WidgetStore) {
    // ⭐ **Limpar as excepções SEM ALVO** (F5.3) — o gesto existe porque elas **nunca** se apagam
    // sozinhas (a lei do *«unused overrides»* do Unity).
    button(store, ids::INSP_INSTANCE_CLEAR_ORPHANS);
    // ⭐⭐⭐ **Os botões da ESCADA do *Aplicar*** (F5 critério 4).
    //
    // ⚠️ **Registados os OITO, e não só os que a cena de hoje pinta:** o `WidgetStore` é o estado
    // de focalização e não uma vista, e um degrau que aparecesse depois de uma receita ser
    // aninhada nasceria morto sob o dedo. *A tabela é a população; o cartão é a vista.*
    for &id in &ids::INSP_INSTANCE_APPLY_LEVEL {
        button(store, id);
    }
    // ⭐⭐⭐ **O `✕` de cada excepção sem alvo** (F5.3-ter) — os DEZASSEIS, pela mesma razão da
    // escada: o `WidgetStore` é a população, o cartão é a vista, e um `✕` que aparecesse depois de
    // uma sexta peça morrer nasceria morto sob o dedo.
    for &id in &ids::INSP_INSTANCE_DROP_ORPHAN {
        button(store, id);
    }
    // ⭐ **Os chips da fileira de VERSÕES** — a outra superfície do cartão desde que o mecanismo de
    // propriedades foi adiado (2026-09-01).
    for &id in ids::INSP_INSTANCE_AXIS_OPTION.iter().flatten() {
        button(store, id);
    }
}
