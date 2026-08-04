//! **A ordem de z do vetor** — a projeção da árvore, e quem a reescreve.
//!
//! Módulo irmão do [`crate::vec_entities`] (teto de 600 LOC da shell). Os dois vivem juntos por
//! assunto: a ponte doc↔árvore é quem sabe **onde cada forma está na pilha**, porque a pilha é uma
//! **projeção da árvore** (ADR-0110) e não uma propriedade do documento.
//!
//! É a regra que este arquivo inteiro serve: **quem quiser mandar no z escreve na ÁRVORE.** A
//! ordem do vetor da cena é reescrita a cada frame pela projeção — mexer nela é falar com a porta
//! errada, e o frame seguinte desfaz.
//!
//! ⚠️ *"A árvore"* são **dois** lugares, e um deles não é o `RootOrder`: a ordem das RAÍZES mora
//! nele, a dos FILHOS mora na sequência do `Children`. Uma nota anterior deste cabeçalho dizia só
//! *"escreva no `RootOrder`"* e estava certa pela metade — um filho não tem `RootOrder`, e
//! escrever um nele não move nada.

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
/// (É a mesma armadilha do "duas portas para a mesma pergunta": `VecScene::reorder_path` mexe na
/// porta ERRADA e o frame seguinte desfaz. Os botões Arrange chamavam-na — e por isso estavam
/// MORTOS até 2026-08-04; hoje passam pelo [`reorder`].)
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

/// **Os IRMÃOS de `e` na ordem em que a Hierarquia os lista** — o primeiro é o da FRENTE.
///
/// Uma raiz tem por irmãos as outras raízes (ordenadas por `RootOrder`, com os bits a desempatar —
/// o mesmo critério do `build_hierarchy_snapshot`, e é **por ser o mesmo** que o número que o
/// painel mostra é o lugar em que a forma de facto está). Um filho tem os irmãos do `Children` do
/// pai, na ordem de inserção.
fn siblings(sim: &mut SimWorld, e: Entity) -> Vec<Entity> {
    if let Some(c) = sim.world().get::<ChildOf>(e) {
        return sim
            .world()
            .get::<bevy_ecs::hierarchy::Children>(c.parent())
            .map(|k| k.iter().copied().collect())
            .unwrap_or_default();
    }
    let mut q = sim
        .world_mut()
        .query_filtered::<(Entity, Option<&RootOrder>), Without<ChildOf>>();
    let mut roots: Vec<(Entity, u32)> = q
        .iter(sim.world())
        .map(|(e, r)| (e, r.map_or(u32::MAX, |r| r.0)))
        .collect();
    roots.sort_by_key(|(e, o)| (*o, e.to_bits()));
    roots.into_iter().map(|(e, _)| e).collect()
}

/// **O Z-INDEX de um caminho**: `(z, quantos)`, com **maior = mais à FRENTE** (a convenção do
/// Godot e da Unity).
///
/// ⚠️ **Ele é DERIVADO da árvore, e não um número guardado ao lado dela.** A ordem de z é a
/// projeção da hierarquia (ADR-0110); um `z_index` próprio seria uma **segunda resposta** a *"quem
/// está na frente?"*, e as duas divergiriam no primeiro reparent — com o sintoma a ser uma forma
/// que se recusa a subir porque o outro número manda. O que o painel mostra é o lugar que ela
/// ocupa, e é o mesmo lugar que os botões movem.
///
/// ⚠️ **Entre IRMÃOS, e não no documento inteiro.** Um filho nunca passa à frente do vizinho do pai: ele vive
/// dentro do pai, e a pilha do documento é a árvore achatada. Um número global mentiria sobre o que
/// os botões conseguem fazer.
#[must_use]
pub(crate) fn z_index(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<(u32, u32)> {
    let e = Entity::from_bits(*map.get(&id)?);
    let sibs = siblings(sim, e);
    let i = sibs.iter().position(|x| *x == e)?;
    let n = u32::try_from(sibs.len()).unwrap_or(u32::MAX);
    // A lista vem FRENTE → fundo; o z conta ao contrário.
    Some((
        n.saturating_sub(1).saturating_sub(u32::try_from(i).ok()?),
        n,
    ))
}

/// **Move `id` na pilha dos IRMÃOS dele.** `true` = a árvore mudou.
///
/// ⚠️ **Esta é a porta dos botões Arrange, e a anterior escrevia na porta ERRADA.** Eles chamavam
/// `VecScene::reorder_path`, que mexe na ordem do vetor da cena — e essa é **reescrita a cada
/// frame** pela projeção da árvore. Os quatro botões estavam MORTOS: acendiam, mexiam, e o frame
/// seguinte desfazia. (O cabeçalho deste módulo já dizia qual era a porta certa; o que faltava era
/// alguém a usar.)
pub(crate) fn reorder(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    order: ph2d_vec_scene::ZOrder,
) -> bool {
    use ph2d_vec_scene::ZOrder;
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let mut sibs = siblings(sim, e);
    let Some(i) = sibs.iter().position(|x| *x == e) else {
        return false;
    };
    if sibs.len() < 2 {
        return false; // filho único: não há pilha em que andar
    }
    // A lista é FRENTE → fundo, então "à frente" é para o ÍNDICE MENOR.
    let to = match order {
        ZOrder::ToFront => 0,
        ZOrder::Raise => i.saturating_sub(1),
        ZOrder::Lower => (i + 1).min(sibs.len() - 1),
        ZOrder::ToBack => sibs.len() - 1,
    };
    if to == i {
        return false; // já está lá: um passo de undo vazio é ruído
    }
    let moved = sibs.remove(i);
    sibs.insert(to, moved);
    write_sibling_order(sim, e, &sibs);
    true
}

/// Grava a ordem `sibs` (FRENTE → fundo) de volta na árvore.
///
/// ⚠️ **Duas escritas diferentes para dois lugares diferentes**, e não é acidente: a ordem das
/// RAÍZES mora no `RootOrder` (é o que o snapshot ordena) e a dos FILHOS mora na sequência do
/// `Children`, que só se reescreve **re-inserindo o `ChildOf`** — o bevy apenda no fim, então
/// re-inserir todos na ordem desejada é a única porta que existe. É o mesmo mecanismo que o
/// arrastar da Hierarquia usa, e reusá-lo é o que impede o painel e a árvore de discordarem.
fn write_sibling_order(sim: &mut SimWorld, e: Entity, sibs: &[Entity]) {
    if let Some(parent) = sim.world().get::<ChildOf>(e).map(ChildOf::parent) {
        for &child in sibs {
            if let Ok(mut em) = sim.world_mut().get_entity_mut(child) {
                em.remove::<ChildOf>();
                em.insert(ChildOf(parent));
            }
        }
        return;
    }
    for (i, &root) in sibs.iter().enumerate() {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(root) {
            em.insert(RootOrder(u32::try_from(i).unwrap_or(u32::MAX)));
        }
    }
}

/// Os gates do ponto fixo (o conserto do "undo só faz uma etapa") — módulo irmão,
/// pelo teto de 600 LOC por arquivo da shell (HR-18).
#[cfg(test)]
#[path = "vec_zorder_fixpoint_tests.rs"]
mod zorder_fixpoint_tests;

/// Os gates do Z-index e dos botões Arrange — irmão pelo mesmo teto.
#[cfg(test)]
#[path = "vec_zorder_arrange_tests.rs"]
mod arrange_tests;

/// A ordem de z que a árvore dita: **fundo → topo**, pronta para
/// `VecScene::reorder_to`.
///
/// A Hierarquia lista em DFS com a primeira linha à frente (convenção
/// Illustrator/Figma), então a pilha de z é o inverso.
///
/// ⚠️ **Corolário load-bearing: um contêiner é o ÚLTIMO membro da própria sub-árvore.** O
/// recorte de moldura (`ph2d_vec_render::frame_clip`) empurra a camada na ABERTURA do intervalo
/// e a fecha quando a vez da moldura chega — se ela deixasse de ser a última, o `pop_layer`
/// fecharia a camada de outra pessoa e sumiria com arte alheia. E é por isso que *"o pai pinta
/// atrás dos filhos"* **não** se resolve aqui: quem antecipa o desenho dele é o renderer.
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
