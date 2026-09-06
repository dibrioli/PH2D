//! ⭐⭐⭐ **A PEÇA QUE O ARTISTA ACRESCENTOU a uma cópia** (ADR-0164 / plano F5.11) — o *Added
//! GameObject* do Unity, e o espelho exacto da [`crate::instance_structure::refuse_pieces`].
//!
//! # ⚠️ O modelo não guarda NADA, e a ausência é a decisão
//!
//! A F5.10 teve de inventar um campo (`ObjectInstance.removed`) porque *«esta cópia recusou a peça
//! X»* não é dedutível de nada: a peça continua viva na receita e ausente na cópia, e uma ausência
//! não distingue *«recusei»* de *«ainda não materializei»*.
//!
//! ⭐ **Aqui a verdade já está escrita, e é load-bearing desde a F4.2:** uma entidade dentro de uma
//! cópia **sem** [`ph2d_ecs::InstanceOf`] é autoria do artista — é literalmente a linha por que o
//! passe estrutural não lhe toca (*«só o que a receita deu é que a receita tira»*) e a linha por
//! que o apagar a deixa morrer (`is_a_recipe_given_piece`). ⇒ a lista deste ficheiro é **derivada**,
//! o `PROJECT_SCHEMA` não se mexe, e um projecto gravado antes desta wave já a tem certa.
//!
//! *Guardar um valor cria duas fontes para o mesmo facto, e isso só é aceitável quando não há
//! primeira* — o critério da refutação da F4.4, aplicado ao contrário.
//!
//! # ⭐⭐ O que APLICAR faz, e por que o elo nasce no ORIGINAL
//!
//! Promover copia a sub-árvore para dentro da receita e **liga a original à cópia nova**. As duas
//! metades são obrigatórias e o defeito de cada uma é mudo:
//!
//! - sem a cópia para a receita, as irmãs nunca recebem a peça (é o gesto inteiro);
//! - sem o elo no original, o passe seguinte vê uma peça do mestre que esta cópia *«não tem»* e
//!   **materializa uma segunda** — o artista fica com duas onde pediu uma, na cópia onde ele
//!   trabalhou.
//!
//! ⛔ **E é por isso que ela não passa pelo `apply_to_master`:** aquele verbo escreve BYTES de
//! componentes que os dois lados já têm. Aqui não há par — o que falta na receita é a peça.

use ph2d_ecs::{ChildOf, Children, Entity, InstanceOf, MasterRoot, Name, SimWorld, StableId};
use std::collections::BTreeMap;

/// **Uma peça que a receita não deu**, pronta para o cartão e para o verbo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AddedPiece {
    /// A entidade dela na cena.
    pub(crate) entity: Entity,
    /// ⚠️ **O `StableId` dela — o que viaja pelo barramento.** O cartão é reconstruído a cada
    /// quadro e os bits de entidade morrem no primeiro Ctrl+Z; a identidade sobrevive ao respawn.
    pub(crate) piece_id: u64,
    /// O nome que a Hierarquia mostra.
    pub(crate) name: String,
    /// O `StableId` da RECEITA que a recebe.
    pub(crate) master: u64,
    /// O nome dessa receita — o cartão nomeia o destino, e com aninhamento ele não é sempre o mesmo.
    pub(crate) master_name: String,
}

/// O que a promoção fez.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Promoted {
    /// Quantas entidades entraram na receita — a sub-árvore inteira, não só a raiz dela.
    pub(crate) pieces: usize,
    /// O `StableId` da receita que as recebeu.
    pub(crate) master: u64,
}

/// Por que a promoção não aconteceu. ⚠️ **Cada arma tem voz própria** — *duas recusas que devolvem
/// o mesmo `None` produzem o mesmo aviso inútil* (a lei do `Refusal` dos verbos).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum AddRefusal {
    /// Não está dentro de cópia nenhuma.
    NotInACopy,
    /// A receita **deu** esta peça (ela tem elo), ou ela é a raiz da própria cópia.
    NotAdded,
    /// A cópia profunda falhou — a entidade sumiu entre o clique e o gesto.
    CopyFailed,
}

/// **Esta entidade foi acrescentada pelo artista dentro de uma cópia?**
///
/// ⚠️ **As três metades são precisas:** dentro de uma cópia · **não** a raiz dela (a raiz é um
/// objecto da cena, e o elo dela aponta para o `MasterRoot`) · **sem** elo. É a negação exacta do
/// [`crate::instance_verbs::is_a_recipe_given_piece`], e por isso as duas leem o mesmo campo.
pub(crate) fn is_added(sim: &mut SimWorld, entity: Entity) -> bool {
    if sim.world().get::<InstanceOf>(entity).is_some() {
        return false;
    }
    matches!(crate::instance_verbs::instance_root_of(sim, entity), Some(root) if root != entity)
}

/// ⭐⭐ **O TOPO da cadeia acrescentada a que `entity` pertence.**
///
/// ⚠️ **Sobe enquanto o pai também não tiver elo.** Um *Add Child* dentro de um *Add Child* não é
/// uma segunda peça a promover: promover a de dentro sozinha poria na receita uma peça cujo pai lá
/// não existe. ⇒ o sujeito é sempre o topo, e a recusa *«aplique o pai primeiro»* deixa de existir
/// — *uma pergunta que a normalização responde não precisa de uma voz.*
fn top_of_added_chain(sim: &mut SimWorld, entity: Entity) -> Entity {
    let mut e = entity;
    loop {
        let Some(parent) = sim.world().get::<ChildOf>(e).map(|c| c.0) else {
            return e;
        };
        if sim.world().get::<InstanceOf>(parent).is_some() {
            return e;
        }
        e = parent;
    }
}

/// ⭐⭐⭐ **As peças que esta cópia GANHOU** — uma linha por cadeia acrescentada, do topo dela.
///
/// ⚠️ **A travessia PÁRA numa cópia aninhada.** Uma peça pendurada dentro da roda de um carro
/// pertence à cópia da *Roda*, e é o cartão dela que a mostra — o
/// [`crate::instance_verbs::instance_root_of`] devolve a raiz **mais interna**, então listá-la aqui
/// também daria a mesma linha em dois cartões, com dois destinos diferentes.
///
/// ⚠️ **Ordenada por `StableId`** — a ordem de autoria, e a única igual em toda máquina. Uma lista
/// que reordena entre quadros é uma lista cujo botão aponta para outra linha.
pub(crate) fn added_pieces(sim: &mut SimWorld, root: Entity) -> Vec<AddedPiece> {
    let by_id = crate::instance_verbs::stable_index(sim);
    let mut out: Vec<AddedPiece> = Vec::new();
    // Só entram no percurso entidades COM elo — é o que garante que todo achado tem destino.
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        let Some(kids) = sim.world().get::<Children>(e).map(|k| k.to_vec()) else {
            continue;
        };
        // O destino das peças acrescentadas debaixo de `e`: a peça do mestre de que `e` nasceu.
        let host = sim
            .world()
            .get::<InstanceOf>(e)
            .and_then(|l| by_id.get(&l.master))
            .copied();
        for kid in kids {
            if sim.world().get::<InstanceOf>(kid).is_some() {
                // ⛔ Uma cópia ANINHADA é o fim deste percurso — ver o doc.
                if !is_nested_root(sim, &by_id, kid) {
                    stack.push(kid);
                }
                continue;
            }
            let (Some(host), Some(piece_id)) =
                (host, sim.world().get::<StableId>(kid).map(|s| s.0))
            else {
                continue;
            };
            let Some(master) = ph2d_ecs::master_root_of(sim.world(), host)
                .and_then(|m| sim.world().get::<StableId>(m).map(|s| s.0))
            else {
                continue;
            };
            out.push(AddedPiece {
                entity: kid,
                piece_id,
                name: named(sim, kid),
                master,
                master_name: by_id
                    .get(&master)
                    .map_or_else(|| "component".to_string(), |&m| named(sim, m)),
            });
            // ⛔ **Não desce**: a sub-árvore inteira vai junto quando esta linha for aplicada.
        }
    }
    out.sort_by_key(|a| a.piece_id);
    out
}

/// Esta peça é a raiz de uma cópia aninhada — o elo dela aponta para um [`MasterRoot`].
fn is_nested_root(sim: &SimWorld, by_id: &BTreeMap<u64, Entity>, e: Entity) -> bool {
    sim.world()
        .get::<InstanceOf>(e)
        .and_then(|l| by_id.get(&l.master))
        .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some())
}

fn named(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map_or_else(String::new, |n| n.0.clone())
}

/// ⭐⭐⭐ **APLICAR: a peça entra na receita, e todas as cópias a recebem.**
///
/// ⚠️ **O sujeito é o TOPO da cadeia** ([`top_of_added_chain`]), e não necessariamente o que se
/// clicou.
///
/// ⚠️ **A ORDEM é load-bearing e o erro é mudo:** primeiro a peça entra na receita, e só então o
/// original se liga a ela. Ao contrário, o `StableId` que o elo precisa ainda não existe.
pub(crate) fn promote(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    clicked: Entity,
) -> Result<Promoted, AddRefusal> {
    let Some(root) = crate::instance_verbs::instance_root_of(sim, clicked) else {
        return Err(AddRefusal::NotInACopy);
    };
    if root == clicked || sim.world().get::<InstanceOf>(clicked).is_some() {
        return Err(AddRefusal::NotAdded);
    }
    let added = top_of_added_chain(sim, clicked);
    let by_id = crate::instance_verbs::stable_index(sim);
    let Some(host) = sim
        .world()
        .get::<ChildOf>(added)
        .and_then(|c| sim.world().get::<InstanceOf>(c.0))
        .and_then(|l| by_id.get(&l.master))
        .copied()
    else {
        return Err(AddRefusal::NotAdded);
    };
    let Some(pieces) = crate::instantiate::promote_piece(sim, registry, docs, added, host) else {
        return Err(AddRefusal::CopyFailed);
    };
    let master = ph2d_ecs::master_root_of(sim.world(), host)
        .and_then(|m| sim.world().get::<StableId>(m).map(|s| s.0))
        .unwrap_or_default();
    Ok(Promoted { pieces, master })
}

#[cfg(test)]
#[path = "instance_added_tests.rs"]
mod tests;
