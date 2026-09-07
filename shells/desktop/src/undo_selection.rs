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
        .filter(|e| keeps_its_selection(world, *e))
        .filter_map(|e| ph2d_ecs::stable_id_of(world, e))
        .filter(|id| !id.is_none())
        .collect()
}

/// ⭐⭐⭐ **QUEM CONTINUA ESCOLHIDO DEPOIS DO RESPAWN** — as famílias cuja selecção não é um caminho
/// vectorial (essas seguem a [`surviving_selection`]).
///
/// ⛔⛔ **Era uma família SÓ, e a segunda pagou o mesmo report** (Enio, 2026-09-07: *«undo … não
/// funcionam plenamente»*). O filtro dizia *«só quem é nó do MODELADOR»* e um **osso** não é: todo
/// `Ctrl+Z` desescolhia o osso, a secção SKELETON perdia o sujeito e os controlos dela
/// **desapareciam** — o undo fazia o trabalho certo e o artista via o app partir-se. É, à letra, o
/// report que criou esta função em 03/09 (*«o undo/redo do módulo não obedece cada etapa»*), noutro
/// módulo.
///
/// ⚠️ **A nota que ficou aqui previu-o e não o impediu:** *«o resto da selecção segue as leis de
/// quem a possui»* — e a lei do esqueleto nunca foi escrita, porque nada obriga uma família nova a
/// vir aqui. É a terceira lista escrita à mão desta linha a morder pelo mesmo mecanismo (as outras
/// duas: o `publishes_its_own_handles` e a allowlist de cliques do painel).
///
/// ⛔ **A generalização — *«tudo o que tem identidade durável sobrevive»* — NÃO foi feita aqui, e é
/// uma cerca de Chesterton:** a nota original diz que alargar mudaria o comportamento de módulos que
/// não o pediram, e essa decisão é de quem os possui. O que se pode fazer sem os acordar é
/// **nomear** as famílias, uma linha cada, com o motivo ao lado.
fn keeps_its_selection(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> bool {
    // O modelador 3D (W113, report de 03/09) — a pose de um nó nem sequer é o `Transform` da casa.
    world.get::<ph2d_field_ecs::FieldNode>(e).is_some()
        // ⭐ O ESQUELETO (report de 07/09): o osso é o **sujeito** da secção SKELETON, e sem ele o
        // painel deixa de mostrar `Length`, `Strength` e os verbos da âncora.
        || world.get::<ph2d_skeleton_ecs::Bone>(e).is_some()
        // ⭐ E o ALVO de uma âncora, que é o que o artista tem na mão enquanto anima a corrente.
        || world.get::<ph2d_skeleton_ecs::IkTarget>(e).is_some()
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
