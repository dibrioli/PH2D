//! **A ordem de z do vetor** — a projeção da árvore, e quem a reescreve.
//!
//! Módulo irmão do [`crate::vec_entities`] (teto de 600 LOC da shell). Os dois vivem juntos por
//! assunto: a ponte doc↔árvore é quem sabe **onde cada forma está na pilha**, porque a pilha é uma
//! **projeção da árvore** (ADR-0110) e não uma propriedade do documento.
//!
//! É a regra que este arquivo inteiro serve: **quem quiser mandar no z escreve no `RootOrder`.**
//! A ordem do vetor da cena é reescrita a cada frame pela projeção — mexer nela é falar com a
//! porta errada, e o frame seguinte desfaz.

use super::VecEntityMap;
use ph2d_ecs::scene::HierarchySnapshot;
use ph2d_ecs::{ChildOf, Entity, RootOrder, SimWorld, Without};
use ph2d_vec_scene::VecPathId;

/// Reempilha um GRUPO de paths numa sequência contígua de z (`run` vem **fundo → topo**),
/// mantendo a ordem relativa de todo o resto.
///
/// # Por que isto existe
///
/// A ordem de z é a projeção da ÁRVORE (ADR-0110): quem quiser mandar no z tem de escrever no
/// `RootOrder`, não na ordem do vetor da cena — essa é reescrita a cada frame pela projeção.
/// (É a mesma armadilha do "duas portas para a mesma pergunta": `VecScene::reorder_path`, que os
/// botões Arrange usam, mexe na porta ERRADA e o frame seguinte desfaz.)
///
/// O Blend precisa disto: os passos que ele cria só ganham entidade no `sync` do frame seguinte,
/// e ele quer a sequência inteira (fontes inclusas) empilhada na ordem certa.
pub(crate) fn restack(sim: &mut SimWorld, map: &VecEntityMap, run: &[VecPathId]) {
    let members: Vec<Entity> = run
        .iter()
        .filter_map(|id| map.get(id).copied())
        .map(Entity::from_bits)
        .filter(|e| sim.world().get_entity(*e).is_ok())
        .filter(|e| sim.world().get::<ChildOf>(*e).is_none()) // só raízes: um filho vive no pai
        .collect();
    if members.len() < 2 {
        return;
    }
    // A pilha de z de TODAS as raízes, fundo → topo (RootOrder decrescente).
    let mut stack: Vec<Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &RootOrder), Without<ChildOf>>();
        let mut roots: Vec<(Entity, u32)> = q.iter(sim.world()).map(|(e, r)| (e, r.0)).collect();
        roots.sort_by_key(|(_, o)| std::cmp::Reverse(*o));
        roots.into_iter().map(|(e, _)| e).collect()
    };
    // O grupo entra na fatia de z da mais de TRÁS dele — o resultado não salta para o topo do
    // documento, que é o que o Illustrator faz com um blend.
    let anchor = stack
        .iter()
        .take_while(|e| !members.contains(e))
        .filter(|e| !members.contains(e))
        .count();
    stack.retain(|e| !members.contains(e));
    let at = anchor.min(stack.len());
    for (k, e) in members.iter().enumerate() {
        stack.insert(at + k, *e);
    }
    let n = u32::try_from(stack.len()).unwrap_or(u32::MAX);
    for (i, e) in stack.iter().enumerate() {
        // fundo (i=0) = maior RootOrder.
        let order = n
            .saturating_sub(u32::try_from(i).unwrap_or(0))
            .saturating_sub(1);
        if let Ok(mut em) = sim.world_mut().get_entity_mut(*e) {
            em.insert(RootOrder(order));
        }
    }
}

/// Os gates do ponto fixo (o conserto do "undo só faz uma etapa") — módulo irmão,
/// pelo teto de 600 LOC por arquivo da shell (HR-18).
#[cfg(test)]
#[path = "vec_zorder_fixpoint_tests.rs"]
mod zorder_fixpoint_tests;

/// A ordem de z que a árvore dita: **fundo → topo**, pronta para
/// `VecScene::reorder_to`.
///
/// A Hierarquia lista em DFS com a primeira linha à frente (convenção
/// Illustrator/Figma), então a pilha de z é o inverso.
///
/// **A fonte é o snapshot da ÁRVORE, não a lista do painel** — e isso não é
/// arrumação, é o conserto de BUGS #15. O painel publica a lista dele no prólogo do
/// frame, **antes** de o [`sync`] dar entidade à forma recém-criada; projetar por
/// ela deixa a forma nova de fora, e quem o `reorder_to` não conhece recebe chave 0 e
/// vai pro **FUNDO**. A cena só convergia um frame depois — e como o snapshot do undo
/// é tirado no fim do frame da AÇÃO, ele capturava um estado que **não é ponto fixo
/// dos sistemas**: restaurá-lo e deixar o frame rodar produzia outra coisa, o diff
/// por-frame lia a diferença como ação do usuário, e nascia um passo espúrio que
/// limpava o redo. Era o "o undo só faz uma etapa e não funciona mais".
///
/// O snapshot vem de `build_hierarchy_snapshot` — a **mesma** função que alimenta o
/// painel, chamada num momento diferente (depois do `sync`). Um DFS próprio aqui
/// seria uma segunda porta para a mesma pergunta, e duas portas divergem.
#[must_use]
pub(crate) fn z_order(snap: &HierarchySnapshot) -> Vec<VecPathId> {
    snap.entries
        .iter()
        .filter_map(|e| e.vec_path)
        .rev()
        .collect()
}
