//! **Fase do quadro: AS EDIÇÕES DO MOVER DE VISTA DE CIMA** (TOP-20 #13, W3) — fase-filha do
//! [`super`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC e está certo por outra razão:** ao contrário da irmã
//! [`super::fase_factory_commits`], esta fase **não pede a árvore de tags** — todos os campos do
//! mover são números e modos, e nenhum guarda uma identidade. *Partir por responsabilidade, nunca
//! subir o número.*

use ph2d_ecs::SimWorld;
use ph2d_editor_core::topdown_edits::TopDownFieldEdit;

/// Aplica as edições que o painel emitiu neste quadro. `true` = o documento mudou.
pub(super) fn aplicar(sim: &mut SimWorld, edits: &[(u64, TopDownFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if super::inspector_topdown::apply_topdown_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}
