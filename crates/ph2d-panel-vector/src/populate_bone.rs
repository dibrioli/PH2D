//! O registro dos widgets da seção **SKELETON** — irmão do [`super`] pelo teto de 600 LOC do
//! painel, e par natural do `paint_bone` (que os PINTA).
//!
//! Registar é o que os torna clicáveis: pintar + hit-rect não basta — é a classe de bug que já
//! matou botões do vetor duas vezes (o mais recente é o [bug #29], em que TRÊS rotas morreram ao
//! mesmo tempo com o gate de registo **verde**). O gate `architecture_panel_wiring_parity` cobra a
//! correspondência entre os dois ficheiros.
//!
//! Os cinco são registados **incondicionalmente**, mesmo os que só são PINTADOS com uma forma presa
//! ou um osso em foco: o store é agnóstico de estado, e quem decide se o clique é possível é a
//! PINTURA (sem hit-rect não há `Click`).
//!
//! [bug #29]: ../../../docs/Vector%20Module/BUGS_vector.md

use super::{button, world_number_field};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Os widgets do esqueleto: prender, as duas saídas, e os dois números do osso.
pub(super) fn populate_bone(store: &mut WidgetStore) {
    button(store, ids::VECTOR_BONE_BIND);
    button(store, ids::VECTOR_BONE_EXPAND);
    button(store, ids::VECTOR_BONE_RELEASE);
    // ⚠️ **Pela porta do MUNDO** (`world_number_field`, sem `set_number_range`): o comprimento de um
    // osso vive nas unidades do documento, e emprestar-lhe a faixa de outro recurso é exactamente
    // o defeito que o `CLAUDE.md` §0.0 nomeia — a v21 já o pagou com a largura de traço a limitar
    // um deslocamento.
    world_number_field(store, ids::VECTOR_BONE_LENGTH, 0.0);
    world_number_field(store, ids::VECTOR_BONE_STRENGTH, 1.0);
}
