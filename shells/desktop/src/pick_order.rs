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

/// ⭐⭐⭐ **ONDE O CICLO COMEÇA: o primeiro clique é de quem JÁ ESTÁ SELECIONADO** (Enio,
/// 2026-08-26).
///
/// > *«Como os objetos filhos ficam com um z-index relativamente maior que o pai, quando tentamos
/// > arrastar o pai (objeto previamente vazio) selecionamos um filho. Precisamos que a preferência
/// > do primeiro clique seja do objeto que está selecionado na hierarquia e depois a cada clique a
/// > seleção passa a ciclar.»*
///
/// # ⚠️ Ela NÃO revoga a lei do contêiner — escolhe outra coisa
///
/// [`descendants_first`] responde *«em que ORDEM os candidatos ficam»*, e continua a valer: um
/// clique dentro de um grupo que ainda não está selecionado pega o filho, como no Figma. Esta
/// função responde *«por onde o ciclo COMEÇA»*, e a resposta é o objeto que o artista já escolheu
/// — porque o gesto seguinte a escolher um objeto é **mexer nele**, e pedir a um artista que
/// descubra uma cadência de cliques para voltar ao que ele acabou de selecionar é o mesmo defeito
/// que a lei do contêiner curou, do outro lado.
///
/// ⚠️ **É só o PRIMEIRO clique.** A lista devolvida é a mesma, e o ciclo dos cliques seguintes
/// anda a partir daqui — quem quer o filho por baixo continua a chegar lá.
///
/// # ⚠️ `inside_its_gizmo` — o caso em que o selecionado não está nos hits
///
/// A caixa de um objeto vazio é um quadrado e o anel dele é o disco **inscrito**: premir numa quina
/// da caixa é premir o gizmo dele e **não** o corpo dele, e sem esta metade o clique caía no filho
/// por baixo. ⇒ quando o press é um *Translate* no gizmo primário, o selecionado entra na lista
/// (no FIM, para a ordem de camada dos outros ficar intacta) e o ciclo começa nele. *É a mesma lei
/// que já dizia «sem nada sob o cursor, cai na seleção atual» (Enio, 2026-07-09), com algo sob o
/// cursor.*
///
/// ⛔ **NUNCA num clique com modificador**: `Shift`/`Cmd` estão a curar a seleção, e preferir o
/// primário faria o `Shift`+clique num filho alternar o PAI. Quem chama passa `false` aí.
pub(crate) fn start_on_selection(
    hits: &mut Vec<u64>,
    selected: Option<u64>,
    inside_its_gizmo: bool,
) -> usize {
    let Some(sel) = selected else {
        return 0;
    };
    if let Some(i) = hits.iter().position(|&b| b == sel) {
        return i;
    }
    if inside_its_gizmo && !hits.is_empty() {
        hits.push(sel);
        return hits.len() - 1;
    }
    0
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

    /// ⭐⭐⭐ **O primeiro clique é de quem já está selecionado** (report do Enio, 2026-08-26).
    ///
    /// ⚠️ E a lista **não** é reordenada: o ciclo dos cliques seguintes continua a alcançar o
    /// filho, que é a lei do contêiner intacta.
    ///
    /// (Mutação: devolver sempre `0` ⇒ RED.)
    #[test]
    fn the_first_click_starts_on_what_is_already_selected() {
        let (_sim, parent, child, _) = family();
        let mut hits = vec![child, parent];
        assert_eq!(
            start_on_selection(&mut hits, Some(parent), false),
            1,
            "o ciclo comecou no filho — arrastar o pai selecionado seleciona um filho"
        );
        assert_eq!(hits, vec![child, parent], "a lista foi reordenada");
    }

    /// **Sem seleção, ou com uma seleção que não está sob o cursor, o ciclo começa no topo.**
    ///
    /// (Mutação: devolver `hits.len() - 1` no caso de fora ⇒ RED.)
    #[test]
    fn a_selection_that_is_not_under_the_cursor_changes_nothing() {
        let (_sim, parent, child, grand) = family();
        let mut hits = vec![child, parent];
        assert_eq!(start_on_selection(&mut hits, None, false), 0);
        assert_eq!(start_on_selection(&mut hits, Some(grand), false), 0);
        assert_eq!(
            hits,
            vec![child, parent],
            "a lista mudou sem o gizmo primario"
        );
    }

    /// ⚠️ **Premir a QUINA da caixa de um objeto vazio** — o gizmo é dele, o corpo (o disco
    /// inscrito) não está sob o cursor, e sem esta metade o clique caía no filho.
    ///
    /// (Mutação: ignorar `inside_its_gizmo` ⇒ RED.)
    #[test]
    fn pressing_inside_its_own_gizmo_brings_the_selection_into_the_cycle() {
        let (_sim, parent, child, _) = family();
        let mut hits = vec![child];
        assert_eq!(start_on_selection(&mut hits, Some(parent), true), 1);
        assert_eq!(
            hits,
            vec![child, parent],
            "o selecionado tem de entrar no FIM — a ordem de camada dos outros fica intacta"
        );
    }

    /// ⛔ **Uma lista VAZIA continua vazia** — o caminho do clique no nada, que limpa a seleção.
    /// Empurrar o selecionado aqui faria um clique fora de tudo re-selecionar o que já estava.
    #[test]
    fn an_empty_list_is_left_alone() {
        let (_sim, parent, _, _) = family();
        let mut hits: Vec<u64> = Vec::new();
        assert_eq!(start_on_selection(&mut hits, Some(parent), true), 0);
        assert!(hits.is_empty(), "o clique no nada passou a selecionar algo");
    }
}
