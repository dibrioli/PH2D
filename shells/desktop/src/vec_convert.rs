//! **Convert to Curves** — o converter UNIFICADO: explode texto, descarta formas paramétricas,
//! e ASSA a pilha de efeitos.
//!
//! Extraído do `render_loop` para ter um **seam testável**. A fiação *"o botão também bakeia os
//! efeitos"* ficaria pintada e morta sem um teste que a percorra headless — e foi exatamente
//! o efeito que faltava (Enio, 2026-07-19): o "Convert to Curves" só olhava o `VecShape`, então
//! um caminho só com efeitos vivos não era sequer convertível.
//!
//! Os três são não-destrutivos-vivos que "Convert to Curves" congela em geometria crua; a porta
//! única garante que o botão da barra e o botão "Apply" da seção Effects usam o MESMO bake
//! ([`ph2d_vec_scene::VecScene::bake_effects`]) — duas respostas divergiriam.

use crate::vec_entities::VecEntityMap;
use ph2d_ecs::SimWorld;
use ph2d_vec_scene::{VecPathId, VecScene};

/// Converte a seleção em paths crus e devolve a nova seleção:
/// - o **TEXTO** explode num grupo por-letra (glyph-paths individuais);
/// - as **PARAMÉTRICAS** descartam o `VecShape` (a geometria já é a forma);
/// - **QUALQUER** caminho com pilha de efeitos tem a pilha assada no cozido.
///
/// A ordem importa: o texto explode ANTES (cria os glyph-paths que os passos seguintes veem), e
/// o bake é por ÚLTIMO (sobre a seleção final, incluindo os glyphs recém-criados — que não têm
/// efeitos, então o bake ali é um no-op barato).
pub(crate) fn to_curves(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    selection: &[VecPathId],
) -> Vec<VecPathId> {
    let new_sel = crate::vec_text::convert_text_selection_to_curves(sim, scene, map, selection);
    crate::vec_shape_live::drop_shape_params(sim, map, &new_sel);
    for id in &new_sel {
        scene.bake_effects(*id);
    }
    new_sel
}

#[cfg(test)]
#[path = "vec_convert_tests.rs"]
mod tests;
