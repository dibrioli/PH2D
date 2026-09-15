//! **Fase do quadro: AS EDIÇÕES DO CÉREBRO** (TOP-20 #15, W3) — fase-filha do [`super`], num
//! ficheiro irmão.
//!
//! ⚠️ **O nome TEM de começar por `fase_`:** o texto emendado do quadro colhe só esses, e com outro
//! nome esta fase desaparece do oráculo de **toda** lei de ordem desta shell, em silêncio. Foi a
//! wave da fábrica que o pagou.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::statemachine_edits::StateMachineFieldEdit;

/// Aplica as edições que o painel emitiu neste quadro. `true` = o documento mudou.
pub(super) fn aplicar(sim: &mut SimWorld, edits: &[(u64, StateMachineFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if super::inspector_statemachine::apply_statemachine_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}
