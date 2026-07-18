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

use super::button;
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Os widgets da seção Envelope: criar, e as duas saídas (materializar / desfazer).
pub(super) fn populate_envelope(store: &mut WidgetStore) {
    button(store, ids::VECTOR_ENVELOPE_RUN);
    button(store, ids::VECTOR_ENVELOPE_EXPAND);
    button(store, ids::VECTOR_ENVELOPE_RELEASE);
}
