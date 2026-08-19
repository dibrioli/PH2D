//! **UM CONTÊINER NÃO ROUBA O CLIQUE DOS PRÓPRIOS FILHOS.**
//!
//! ## O defeito, reportado pelo Enio (2026-08-19)
//!
//! > *"não consigo reorganizar as sprites na folha manualmente. Posso selecionar na hierarchy mas
//! > ao tentar mover no canvas toda a folha se move."*
//!
//! A lista do clique-cíclico é montada por camada de desenho: **vetor primeiro** (*"as formas
//! vetoriais desenham POR CIMA dos sprites"*), depois a arte Flip, depois os sprites. Faz sentido
//! para peças irmãs — e é exatamente a resposta errada para um **contêiner**, que por definição
//! cobre tudo o que tem dentro. A folha (e a moldura, pelo mesmo mecanismo) entrava sempre em
//! primeiro lugar e o índice `0` do ciclo nunca chegava ao filho.
//!
//! ⚠️ **O clique-cíclico não salvava:** ele existe para desempatar peças **sobrepostas**, e cada
//! segundo clique volta à seleção anterior. Chegar a um filho exigiria descobrir uma cadência de
//! cliques que ninguém documentou — e o artista concluiria, com razão, que a peça não se move.
//!
//! ## A lei
//!
//! Entre dois candidatos em que **um é ancestral do outro**, o **descendente vem primeiro**. É a
//! regra do Figma e do Illustrator: clicar dentro de uma moldura pega o conteúdo; a moldura pega-se
//! pela borda — ou por dentro, onde não há filho nenhum, porque aí ela é o único candidato.
//!
//! ⚠️ **E o ancestral não é DESCARTADO, é adiado.** Ele continua na lista, no fim: um segundo
//! clique no mesmo sítio ainda o alcança, que é para o que o ciclo serve. Removê-lo tornaria a
//! folha inalcançável por clique sempre que estivesse cheia.
//!
//! ## Por que é um módulo, e não três linhas no `input_dispatch`
//!
//! A regra é **pura** (uma lista e uma hierarquia entram, uma lista sai), e o `input_dispatch` é
//! o arquivo mais difícil de testar do repo. Aqui ela tem testes; lá teria um comentário.

use ph2d_ecs::{ChildOf, Entity};

/// `true` se `maybe_ancestor` está acima de `entity` na hierarquia.
///
/// ⚠️ O passeio é **limitado** por `MAX_DEPTH`: uma hierarquia com ciclo (que o ECS não deveria
/// permitir, mas que um load corrompido pode produzir) travaria o quadro em vez de pintar. Um
/// limite alto é indistinguível do infinito para uma cena real e é a diferença entre um bug e um
/// congelamento.
fn is_ancestor_of(world: &ph2d_ecs::World, maybe_ancestor: Entity, entity: Entity) -> bool {
    const MAX_DEPTH: usize = 64;
    let mut cur = entity;
    for _ in 0..MAX_DEPTH {
        let Some(parent) = world.get::<ChildOf>(cur).map(|c| c.0) else {
            return false;
        };
        if parent == maybe_ancestor {
            return true;
        }
        cur = parent;
    }
    false
}

/// Reordena a lista do clique para que um **descendente** venha antes do seu contêiner.
///
/// Estável: a ordem relativa dentro de cada grupo (descendentes e ancestrais) é preservada, então
/// a lei de camada que montou a lista continua a valer entre irmãos — esta regra só resolve o
/// empate entre **pai e filho**, que é o único que a camada resolve mal.
pub(crate) fn descendants_first(world: &ph2d_ecs::World, hits: &mut [u64]) {
    if hits.len() < 2 {
        return;
    }
    // Um hit é "contêiner" quando ALGUM outro hit da lista está por baixo dele. Note que a
    // pergunta é sobre a lista, não sobre a cena: uma moldura cujo filho não foi clicado não é
    // contêiner deste clique, e não tem por que ser adiada.
    let is_container: Vec<bool> = hits
        .iter()
        .map(|&a| {
            hits.iter().any(|&b| {
                b != a && is_ancestor_of(world, Entity::from_bits(a), Entity::from_bits(b))
            })
        })
        .collect();
    let mut order: Vec<(bool, u64)> = is_container.into_iter().zip(hits.iter().copied()).collect();
    // `sort_by_key` é estável — é o que preserva a ordem de camada dentro de cada grupo.
    order.sort_by_key(|&(container, _)| container);
    for (slot, (_, bits)) in hits.iter_mut().zip(order) {
        *slot = bits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::SimWorld;

    /// Monta `pai → filho → neto` e devolve os bits.
    fn family() -> (SimWorld, u64, u64, u64) {
        let mut sim = SimWorld::new();
        let parent = sim.world_mut().spawn(()).id();
        let child = sim.world_mut().spawn(ChildOf(parent)).id();
        let grand = sim.world_mut().spawn(ChildOf(child)).id();
        (sim, parent.to_bits(), child.to_bits(), grand.to_bits())
    }

    /// ⚠️ **O defeito que o Enio reportou.** O contêiner vinha primeiro (a camada de desenho
    /// põe-no lá) e o filho era inalcançável por clique.
    #[test]
    fn a_child_beats_its_container() {
        let (sim, parent, child, _) = family();
        let mut hits = vec![parent, child];
        descendants_first(sim.world(), &mut hits);
        assert_eq!(hits, vec![child, parent]);
    }

    /// ⚠️ **O ancestral é ADIADO, não descartado** — senão uma folha cheia ficaria inalcançável
    /// por clique, e ela também precisa de ser movida.
    #[test]
    fn the_container_stays_in_the_list() {
        let (sim, parent, child, _) = family();
        let mut hits = vec![parent, child];
        descendants_first(sim.world(), &mut hits);
        assert!(
            hits.contains(&parent),
            "o conteiner tem de continuar clicavel"
        );
        assert_eq!(hits.len(), 2, "nada se perde");
    }

    /// Vale para qualquer profundidade, não só para o pai direto.
    #[test]
    fn a_grandchild_beats_the_grandparent() {
        let (sim, parent, _, grand) = family();
        let mut hits = vec![parent, grand];
        descendants_first(sim.world(), &mut hits);
        assert_eq!(hits, vec![grand, parent]);
    }

    /// A lei de CAMADA continua a decidir entre irmãos — esta regra só desempata pai↔filho.
    #[test]
    fn siblings_keep_the_layer_order() {
        let mut sim = SimWorld::new();
        let a = sim.world_mut().spawn(()).id().to_bits();
        let b = sim.world_mut().spawn(()).id().to_bits();
        let c = sim.world_mut().spawn(()).id().to_bits();
        let mut hits = vec![a, b, c];
        descendants_first(sim.world(), &mut hits);
        assert_eq!(hits, vec![a, b, c], "sem parentesco, nada se move");
    }

    /// Um contêiner cujo filho NÃO foi clicado não é contêiner deste clique — adiá-lo seria punir
    /// um empate que não existe.
    #[test]
    fn a_container_whose_child_was_not_hit_is_not_demoted() {
        let (sim, parent, _, _) = family();
        let other = 0xDEAD_BEEF_u64;
        let mut hits = vec![parent, other];
        descendants_first(sim.world(), &mut hits);
        assert_eq!(hits, vec![parent, other]);
    }

    #[test]
    fn one_hit_or_none_is_left_alone() {
        let (sim, parent, _, _) = family();
        let mut one = vec![parent];
        descendants_first(sim.world(), &mut one);
        assert_eq!(one, vec![parent]);
        let mut none: Vec<u64> = Vec::new();
        descendants_first(sim.world(), &mut none);
        assert!(none.is_empty());
    }
}
