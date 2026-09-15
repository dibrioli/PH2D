//! **Fase do quadro: AS EDIÇÕES DO PROJÉCTIL** (TOP-20 #14, W3) — fase-filha do [`super`], num
//! ficheiro irmão.
//!
//! ⚠️ **O nome TEM de começar por `fase_`:** o texto emendado do quadro colhe só esses, e com outro
//! nome esta fase desaparece do oráculo de **toda** lei de ordem desta shell, em silêncio. Foi a
//! wave da fábrica que o pagou.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::projectile_edits::ProjectileFieldEdit;

/// Aplica as edições que o painel emitiu neste quadro. `true` = o documento mudou.
pub(super) fn aplicar(sim: &mut SimWorld, edits: &[(u64, ProjectileFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if super::inspector_projectile::apply_projectile_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}
