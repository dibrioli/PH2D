//! ⭐⭐⭐ **EDITAR A RECEITA é um MODO, e o gesto que entra nele é selecioná-la** (F4.6, o §14).
//!
//! # As duas verdades que se contradiziam
//!
//! 1. *«Uma receita não está na cena»* — o gesto *Criar componente* esconde-a, senão o artista vê
//!    **dois objetos empilhados**, um que cai e outro que não (F4.5, e o smoke do Enio confirmou).
//! 2. *«O artista tem de conseguir mudar a forma da receita»* — foi assim que a cena 2 do smoke
//!    passou: com a receita **desenhada**, longe das cópias, mover um nó dela chega às três.
//!
//! Esconder sempre mata a (2); desenhar sempre mata a (1). ⇒ **a receita sai da cena, e volta
//! enquanto está selecionada.** É o *Prefab Mode* do Unity reduzido ao que esta casa já tem: a
//! Hierarquia é onde a biblioteca vive, e escolher uma linha lá é dizer *«é nisto que estou a
//! mexer»*.
//!
//! # ⚠️ Porque é uma MARCA DERIVADA, e não um argumento
//!
//! A pergunta *«esta entidade está na cena?»* é feita em dois sítios muito distantes — o extract
//! de sprites ([`super::off_canvas`]) e a cadeia de visibilidade do vetor
//! ([`crate::vec_entities`]) — e nenhum dos dois tem a selecção à mão. Enfiar um `Option<Entity>`
//! nos dois caminhos seria a mesma resposta a viajar por duas estradas, e elas divergiriam.
//!
//! ⇒ um passe carimba [`ph2d_ecs::MasterEditing`] na sub-árvore da receita seleccionada e
//! **desmarca todo o resto**; as duas perguntas leem o mundo. ⚠️ **As duas metades são
//! obrigatórias** — marcar sem desmarcar deixa uma receita visível para sempre depois de o artista
//! mudar de selecção, que é o defeito (1) de volta pela porta do lado.
//!
//! ⚠️ **Não é registada**, como o `MasterPiece` e pela mesma razão: é derivada da selecção, que é
//! vista e não documento. Um valor derivado no arquivo envenena o undo.

use ph2d_ecs::{Entity, MasterEditing, SimWorld};

/// ⭐ **Marca as receitas que estão a ser editadas.** Devolve `true` quando mexeu em alguma coisa.
///
/// `selection` são os bits de **toda** a selecção — a primária e os extras. Uma selecção que não
/// toque receita nenhuma desmarca tudo, que é o caso comum e o mais barato (e não aloca: um
/// `collect` de um iterador vazio não pede memória).
///
/// ⚠️⚠️ **O parâmetro era um `Option<u64>` — só o primário — e isso era o defeito §1.6 da
/// auditoria de 2026-08-27.** Duas rotas correntes deixam uma receita seleccionada sem ser
/// primária: o Shift/Ctrl-clique (`add_to_selection`/`toggle_in_selection`) e o atalho
/// `preserves_multi` do ramo `Replace`. ⇒ a linha ficava realçada na Hierarquia e o canvas
/// continuava vazio, que é o report *«cliquei nela e não aconteceu nada»*. ⛔ **E o gate não podia
/// vê-lo**: um `Option<u64>` torna o defeito inexprimível na assinatura, então
/// `the_recipe_comes_back_while_it_is_being_edited` ficava verde por construção. *Uma assinatura
/// que não consegue exprimir o caso é um gate que nunca o mede.*
///
/// ⚠️ **N receitas ao mesmo tempo é o comportamento certo, não uma tolerância:** seleccionar duas
/// linhas de biblioteca e ver as duas é o que a multi-selecção promete em todo o resto do app.
pub(crate) fn mark(sim: &mut SimWorld, selection: impl IntoIterator<Item = u64>) -> bool {
    let editing: Vec<Entity> = selection
        .into_iter()
        .map(Entity::from_bits)
        .filter(|&e| sim.world().get_entity(e).is_ok())
        .filter_map(|e| ph2d_ecs::master_root_of(sim.world(), e))
        .collect();
    let mut want: std::collections::BTreeSet<Entity> = std::collections::BTreeSet::new();
    for root in editing {
        want.extend(subtree(sim, root));
    }
    let have: std::collections::BTreeSet<Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<Entity, bevy_ecs::query::With<MasterEditing>>();
        q.iter(sim.world()).collect()
    };
    let mut touched = false;
    for &e in want.difference(&have) {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(MasterEditing);
            touched = true;
        }
    }
    // ⚠️ A metade que se esquece: sem ela a receita fica visível para sempre.
    for &e in have.difference(&want) {
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.remove::<MasterEditing>();
            touched = true;
        }
    }
    touched
}

/// A sub-árvore de `root`, ela incluída.
fn subtree(sim: &SimWorld, root: Entity) -> std::collections::BTreeSet<Entity> {
    let mut out = std::collections::BTreeSet::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if !out.insert(e) {
            continue;
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out
}

#[cfg(test)]
#[path = "master_editing_tests.rs"]
mod tests;
