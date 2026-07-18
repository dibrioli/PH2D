//! O registro dos widgets da seção **ENVELOPE** — irmão do [`super`] pelo teto de 600 LOC do
//! painel, e par natural do `paint_envelope` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta — é a classe de bug que já
//! matou botões do vetor uma vez. O gate `architecture_panel_wiring_parity` cobra a correspondência
//! entre os dois arquivos.
//!
//! Os três são registrados **incondicionalmente**, mesmo o Expand/Release que só são PINTADOS com um
//! envelope selecionado: o store é agnóstico de modo (mesma regra dos botões de tipo de vértice). O
//! que decide se o clique é possível é a PINTURA (sem hit-rect não há Click).

use super::{button, slider_chip};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Os widgets da seção Envelope: criar, e as duas saídas (materializar / desfazer).
pub(super) fn populate_envelope(store: &mut WidgetStore) {
    button(store, ids::VECTOR_ENVELOPE_RUN);
    button(store, ids::VECTOR_ENVELOPE_EXPAND);
    button(store, ids::VECTOR_ENVELOPE_RELEASE);
    button(store, ids::VECTOR_ENVELOPE_PERSPECTIVE);
    button(store, ids::VECTOR_ENVELOPE_MESH);
    button(store, ids::VECTOR_ENVELOPE_PINS);
    button(store, ids::VECTOR_ENVELOPE_CLEAR_PINS);
    // Os presets: o TETO de botões, sempre. O `paint` desenha só os que o shell publicou, então
    // registrar de menos aqui deixaria um preset novo clicável-e-morto e registrar de mais é
    // inerte. É o padrão do catálogo de formas e das pontas de traço.
    for i in 0..ids::MAX_ENVELOPE_PRESETS {
        button(store, ids::vector_envelope_preset_id(i));
    }
    // O Bend é bipolar: track `0..1` → `-1..1`. O campo mostra o VALOR (não o track), com o mesmo
    // passo que o slider anda.
    slider_chip(
        store,
        ids::VECTOR_ENVELOPE_BEND,
        ids::VECTOR_ENVELOPE_BEND_NUM,
        BEND_DEFAULT_TRACK,
        f64::from(ph2d_ecs_default_bend()),
        2.0,
        -1.0,
    );
    store.set_number_range(ids::VECTOR_ENVELOPE_BEND_NUM, -1.0, 1.0, BEND_STEP);
}

/// O `bend` com que um envelope nasce, no domínio do documento (`-1..1`).
///
/// ⚠️ **Tem de concordar com `ph2d_ecs::EnvelopeWarp::DEFAULT_BEND`**, e não pode simplesmente
/// LÊ-LO: este painel não depende do `ph2d-ecs` (a UI vive de snapshots publicados, não do mundo).
/// A concordância importa só até o primeiro frame — dali em diante o `paint` desenha o bend que o
/// shell publicou —, mas sem ela o slider nasceria num lugar e saltaria para outro no 1º frame.
fn ph2d_ecs_default_bend() -> f32 {
    0.5 // LITERAL-PX-OK: valor do DOCUMENTO (bend), não medida de design
}

/// O track correspondente a [`ph2d_ecs_default_bend`] no mapa bipolar `track = (bend + 1) / 2`.
const BEND_DEFAULT_TRACK: f32 = 0.75; // LITERAL-PX-OK: track de slider, não medida de design

/// Passo do campo numérico do Bend, no domínio do documento.
const BEND_STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento, não medida de design
