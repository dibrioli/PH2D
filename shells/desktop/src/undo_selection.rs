//! ⭐⭐⭐ **A SELEÇÃO QUE SOBREVIVE AO UNDO** — as três leis, e nada mais.
//!
//! # Por que um arquivo irmão
//!
//! O [`super`] passou dos `600` do teto de LOC do shell (HR-18) quando a W113 acrescentou a metade
//! 3D. ⛔ *Split, nunca allowlist* — e o corte é por assunto: aqui está *«quem continua escolhido
//! depois de o mundo ser reconstruído»*, e no pai fica *«quando nasce um passo»*.
//!
//! ⚠️ **As três são PURAS de propósito:** o `apply_project` exige `gfx` (janela + GPU) e não é
//! alcançável headless, então a política mora aqui e os gates são sobre ela. O irmão de arquitectura
//! (`tests/the_undo_preserves_the_vector_selection.rs`) prova que o `apply_project` as CHAMA — sem
//! ele, alguém volta a zerar a seleção lá dentro e estes ficam todos verdes.

use ph2d_vec_scene::{VecPathId, VecScene};

/// Os ids de forma que **sobreviveram** a um restore — a seleção que o artista tinha, menos o que o
/// estado restaurado não contém.
#[must_use]
pub(crate) fn surviving_selection(was: &[VecPathId], scene: &VecScene) -> Vec<VecPathId> {
    was.iter()
        .copied()
        .filter(|id| scene.paths().iter().any(|p| p.id == *id))
        .collect()
}

/// ⭐⭐⭐ **A SELEÇÃO DE UMA PEÇA 3D, EM IDENTIDADE DURÁVEL** — a irmã da
/// [`surviving_selection`], para quem não é um caminho vetorial.
///
/// # ⛔⛔ O report que ela fecha (Enio, 2026-09-03)
///
/// *«O undo/redo do módulo não obedece cada etapa, principalmente se transformação.»* O
/// [`App::apply_project`] **limpa a seleção inteira** e devolve só a do vetorial (pelo `vec_pen`);
/// um nó do modelador 3D ficava de fora, então **todo `Ctrl+Z` apagava a seleção e o gizmo
/// desaparecia**. Mover · desfazer · mover outra vez obrigava a re-escolher a peça no meio — que é
/// exactamente *«não obedece cada etapa»* visto de fora.
///
/// ⚠️ **Pelo `StableId` e nunca pelos bits**, que é a lei da casa: o undo **respawna** o mundo e os
/// `Entity::to_bits()` mudam todos. Guardar os bits traria de volta uma seleção que aponta para
/// outro nó — pior do que nenhuma.
pub(crate) fn field_selection_ids(
    world: &bevy_ecs::world::World,
    bits: &[u64],
) -> Vec<ph2d_ecs::StableId> {
    bits.iter()
        .map(|b| bevy_ecs::entity::Entity::from_bits(*b))
        // ⚠️ **Só quem é nó do MODELADOR** — o resto da seleção segue as leis de quem a possui, e
        // alargar isto seria mudar o comportamento de módulos que não o pediram.
        .filter(|e| world.get::<ph2d_field_ecs::FieldNode>(*e).is_some())
        .filter_map(|e| ph2d_ecs::stable_id_of(world, e))
        .filter(|id| !id.is_none())
        .collect()
}

/// E de volta: os bits **novos** de quem sobreviveu ao respawn. Quem morreu simplesmente não volta.
pub(crate) fn field_selection_back(
    world: &mut bevy_ecs::world::World,
    ids: &[ph2d_ecs::StableId],
) -> Vec<u64> {
    ids.iter()
        .filter_map(|id| ph2d_ecs::entity_of_stable_id(world, *id))
        .map(|e| e.to_bits())
        .collect()
}
