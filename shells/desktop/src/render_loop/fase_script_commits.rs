//! **Fase do quadro: AS EDIÇÕES DO SCRIPT** (TOP-20 #16, W3) — fase-filha do [`super`], num
//! ficheiro irmão.
//!
//! ⚠️ **Ela é um ficheiro irmão e não um bloco na mãe**, e o tecto de LOC é que o disse: o bloco
//! levava a `fase_inspector_commits` a `203` contra `200`. A fronteira é a de sempre — o corpo mora
//! na família (`ph2d_app_components::script_inspector`); daqui sai só o DIÁLOGO, que é a janela.
//!
//! ⚠️ **O nome TEM de começar por `fase_`:** o texto emendado do quadro colhe só esses.

use ph2d_app_components::script_inspector::{SCRIPT_EXTENSIONS, apply_all};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::script_edits::ScriptFieldEdit;

/// Aplica as edições que o painel emitiu neste quadro. `true` = o documento mudou.
pub(super) fn aplicar(sim: &mut SimWorld, edits: &[(u64, ScriptFieldEdit)]) -> bool {
    apply_all(sim, edits, || {
        rfd::FileDialog::new()
            .add_filter("Luau", SCRIPT_EXTENSIONS)
            .pick_file()
            .map(|p| p.to_string_lossy().into_owned())
    })
}
