//! **Fase do quadro: AS EDIÇÕES DA SECÇÃO TAGS** (TOP-20 #9) — fase-filha do [`super`], num
//! ficheiro irmão.
//!
//! ⚠️ **O nome TEM de começar por `fase_`:** o texto emendado do quadro colhe só esses, e com outro
//! nome esta fase desaparece do oráculo de **toda** lei de ordem desta shell, em silêncio.
//!
//! ⚠️ **Ela é a única secção do Inspector que escreve em DOIS documentos** — a cena e a ÁRVORE de
//! tags, e a segunda nem sequer está no mundo. É essa a fronteira que a torna uma fase própria.
//!
//! ⚠️ **Nada invalida a cache do documento das tags aqui, e isso é uma propriedade:** ela compara a
//! REVISÃO, que toda mutação incrementa. O `invalidate()` é de quem SUBSTITUI a árvore (o load),
//! não de quem lhe mexe — ver o cabeçalho do `inspector_tags`.

use ph2d_ecs::SimWorld;
use ph2d_tags::TagTree;

/// Aplica as edições que o painel emitiu neste quadro. `true` = o documento mudou.
pub(super) fn aplicar(
    sim: &mut SimWorld,
    tags: &mut TagTree,
    edits: &[(u64, ph2d_editor_core::TagsFieldEdit)],
    editor_queue: &mut ph2d_ecs::scene::EditorCommandQueue,
    component_registry: &ph2d_ecs::scene::ComponentRegistry,
) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        super::inspector_tags::apply_tags_edit(
            sim.world(),
            tags,
            *bits,
            edit,
            editor_queue,
            component_registry,
        );
        mexeu = true;
    }
    mexeu
}
