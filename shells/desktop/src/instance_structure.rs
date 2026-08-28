//! ⭐⭐⭐ **A FORMA de uma instância segue a do mestre** (ADR-0164 / F5.1) — irmão de
//! [`crate::instance_sync`] por ASSUNTO, e porque aquele ficheiro está perto do tecto de 600 LOC.
//!
//! Lá mora *«que VALOR cada peça tem»*; aqui *«que PEÇAS existem»*. São perguntas diferentes e o
//! passe de valores não podia responder à segunda: ele percorre **pares** (peça do mestre ↔ peça da
//! instância), e uma peça que só existe de um dos lados não forma par nenhum — ela é invisível para
//! aquele laço, por construção.
//!
//! # ⛔ O que estava em falta, medido por sonda em 2026-08-27
//!
//! ```text
//! A = receita vazia · a_inst = instância de A na cena
//! (o artista acrescenta uma peça a A)
//! a_inst tem 0 filho(s) depois do passe   ← esperado: 1
//! ```
//!
//! A tabela do doc 04 §2.6 promete *«adicionar peça → **materializa em todas**»*, e nada o fazia.
//! Para o artista: *«acrescentei uma peça ao componente e as cópias não mudaram»* — a mesma frase
//! que já custou três reports a esta linha, por três mecanismos diferentes.
//!
//! # As DUAS metades, e porque nenhuma pode ir sozinha
//!
//! Acrescentar sem remover deixa uma cópia com peças que a receita já não tem — *um objeto que o
//! artista apagou da biblioteca e que continua na cena*, mudo. É a mesma forma do
//! `assign_master_pieces`, que também tem de marcar **e** desmarcar.
//!
//! ⚠️ **E a fronteira que fica:** um elo cujo MESTRE inteiro desapareceu não é uma peça a mais —
//! é uma instância órfã, e a lei dela já existe (`a_dangling_link_is_left_alone`). Este passe só
//! olha para dentro de instâncias **vivas**: mestre presente, peça ausente.

use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, SimWorld, StableId};
use std::collections::{BTreeMap, BTreeSet};

/// Quantas peças o passe criou e quantas apagou — o `0/0` é o estado normal, e é o que faz disto
/// um ponto fixo.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct StructureReport {
    pub(crate) added: usize,
    pub(crate) removed: usize,
}

/// ⭐⭐ **Põe a forma de cada instância a par com a do mestre dela.**
///
/// ⚠️ **Corre ANTES do passe de valores**, e é isso que dá a promessa de UM quadro: uma peça
/// materializada aqui forma par no laço seguinte e recebe os bytes do mestre no mesmo passe. Ao
/// contrário, ela ficaria com os valores do momento da cópia até alguém tocar no mestre outra vez.
pub(crate) fn reconcile(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> StructureReport {
    let mut out = StructureReport::default();
    for root in instance_roots(sim) {
        reconcile_one(sim, registry, docs, root, &mut out);
    }
    out
}

/// As raízes de instância viva, em ordem determinística — o mesmo filtro do passe de valores
/// (*a peça cujo mestre é um [`MasterRoot`]*), e pela mesma razão: os bits mudam a cada respawn.
fn instance_roots(sim: &mut SimWorld) -> Vec<Entity> {
    let by_id = stable_index(sim);
    let mut roots: Vec<(u64, Entity)> = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf, &StableId)>();
        q.iter(sim.world())
            .filter(|(_, link, _)| {
                by_id
                    .get(&link.master)
                    .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some())
            })
            .map(|(e, _, s)| (s.0, e))
            .collect()
    };
    roots.sort_unstable();
    roots.into_iter().map(|(_, e)| e).collect()
}

fn stable_index(sim: &mut SimWorld) -> BTreeMap<u64, Entity> {
    let mut q = sim.world_mut().query::<(Entity, &StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

/// A sub-árvore de `root`, ela incluída, em ordem de travessia determinística.
fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        out.push(e);
        if let Some(kids) = sim.world().get::<Children>(e) {
            let mut k: Vec<Entity> = kids.iter().copied().collect();
            k.sort_by_key(|&c| ph2d_ecs::sibling_key(sim.world(), c));
            stack.extend(k.into_iter().rev());
        }
    }
    out
}

fn reconcile_one(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    root: Entity,
    out: &mut StructureReport,
) {
    let by_id = stable_index(sim);
    let Some(master_root) = sim
        .world()
        .get::<InstanceOf>(root)
        .and_then(|l| by_id.get(&l.master))
        .copied()
    else {
        return;
    };
    // `StableId` de cada peça do MESTRE → a entidade dela.
    let master_pieces: BTreeMap<u64, Entity> = subtree(sim, master_root)
        .into_iter()
        .filter_map(|e| sim.world().get::<StableId>(e).map(|s| (s.0, e)))
        .collect();
    // `StableId` da peça do mestre → a entidade correspondente NA INSTÂNCIA.
    let mut have: BTreeMap<u64, Entity> = BTreeMap::new();
    let mut extra: Vec<Entity> = Vec::new();
    for e in subtree(sim, root) {
        let Some(link) = sim.world().get::<InstanceOf>(e).copied() else {
            // ⚠️ Uma entidade SEM elo dentro de uma instância é autoria do artista (um *Add Child*
            // sobre uma peça), e **não** uma peça a mais: ela nunca veio do mestre, logo apagá-la
            // seria apagar trabalho que ninguém pediu. *Só o que a receita deu é que a receita tira.*
            continue;
        };
        if master_pieces.contains_key(&link.master) {
            have.insert(link.master, e);
        } else {
            extra.push(e);
        }
    }

    // ── as que FALTAM ────────────────────────────────────────────────────────────────────────
    //
    // ⚠️ **Só as de TOPO**: a cópia profunda leva a sub-árvore inteira, então materializar uma peça
    // cujo pai também falta duplicaria os descendentes. Uma peça está no topo quando o pai dela já
    // tem contrapartida.
    let linked = sim.world().get::<ph2d_ecs::LinkedArt>(root).is_some();
    let missing: Vec<(u64, Entity, Entity)> = master_pieces
        .iter()
        .filter(|(sid, _)| !have.contains_key(sid))
        .filter_map(|(&sid, &m)| {
            let parent_sid = sim
                .world()
                .get::<ph2d_ecs::ChildOf>(m)
                .and_then(|c| sim.world().get::<StableId>(c.0))
                .map(|s| s.0)?;
            let host = if parent_sid == master_root_sid(sim, master_root)? {
                root
            } else {
                *have.get(&parent_sid)?
            };
            Some((sid, m, host))
        })
        .collect();
    for (_, piece, host) in missing {
        // ⚠️ **Pela PORTA da cópia profunda**, e não com uma segunda montagem: o gate
        // `only_the_instantiate_door_calls_the_deep_copy` existe porque uma 2.ª montagem esquece
        // sempre um dos passos (o remap, os documentos, o `MasterRoot`), e o defeito é mudo. Ele
        // apanhou-me a escrever exactamente isso.
        if crate::instantiate::materialise_piece(sim, registry, docs, piece, host, linked).is_some()
        {
            out.added += 1;
        }
    }

    // ── as que SOBRAM ────────────────────────────────────────────────────────────────────────
    //
    // ⚠️ Recolhidas como CONJUNTO da sub-árvore de cada uma: apagar só a entidade deixaria os
    // descendentes dela pendurados num pai que já não existe.
    let doomed: BTreeSet<Entity> = extra
        .into_iter()
        .flat_map(|e| subtree(sim, e).into_iter())
        .collect();
    for e in doomed {
        if let Ok(em) = sim.world_mut().get_entity_mut(e) {
            em.despawn();
            out.removed += 1;
        }
    }
}

fn master_root_sid(sim: &SimWorld, master_root: Entity) -> Option<u64> {
    sim.world().get::<StableId>(master_root).map(|s| s.0)
}

#[cfg(test)]
#[path = "instance_structure_tests.rs"]
mod tests;
