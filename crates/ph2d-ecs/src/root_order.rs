//! `RootOrder` — explicit ordering index for root-level entities.
//!
//! M14.7 polish: root entities (no `ChildOf`) had no stable order in
//! [`crate::scene::build_hierarchy_snapshot`] beyond
//! `Entity::to_bits()`. The hierarchy panel's drag-reorder could
//! change the dispatch's `hierarchy_order`, but the host's snapshot
//! rebuild on the next frame restored the bits-sort — so dropping a
//! root sprite "above" another silently snapped back to id order.
//!
//! This component is read by the snapshot's root sort: roots are
//! ordered by `(RootOrder.0, entity.to_bits())`, with absent =
//! `u32::MAX` so new (untouched) entities collate after every
//! explicitly-ordered one. The editor's `pending_reparent` drain
//! assigns sequential indices when the user drops a root before /
//! after another root.
//!
//! Non-root entities (with `ChildOf`) ignore this component — sibling
//! order is already controlled by `Children`'s insertion order, which
//! the same drain rewrites via the re-insert-ChildOf trick.

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Entity, With, Without, World};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootOrder(pub u32);

/// Dá um `RootOrder` explícito a toda raiz que ainda não tem um, **preservando a ordem
/// que a Hierarquia já mostra**. Idempotente: rodar de novo é no-op.
///
/// # Por que isto existe (e por que é obrigatório, não higiene)
///
/// A raiz sem `RootOrder` colate em `u32::MAX`, e o sort de raízes de
/// [`crate::scene::build_hierarchy_snapshot`] desempata os empates por
/// **`Entity::to_bits()`** — o id de **ALOCAÇÃO** do bevy, que muda a cada spawn. Um
/// sprite importado nasce assim (`image_import.rs`).
///
/// Enquanto isso é só a ordem de uma lista, ninguém se machuca. Mas a **pilha de z do
/// vetor é a projeção dessa árvore** (ADR-0110), e um restore de undo **despawna e
/// re-spawna o mundo inteiro** — bits novos. Resultado medido: uma forma pendurada num
/// sprite trocava de posição na pilha a cada Ctrl+Z, e a captura do undo deixava de ser
/// ponto fixo dos sistemas (o passo espúrio que limpa o redo — BUGS #15).
///
/// O conserto não é escolher um desempate melhor: é **não ter empate**. Toda raiz passa a
/// ter um número explícito, o desempate por bits nunca dispara, e a ordem vira função do
/// CONTEÚDO — que sobrevive ao respawn (a mesma razão do `canonicalize` do undo).
///
/// As raízes sem ordem recebem números **depois** de todas as explícitas, na ordem em que
/// a árvore já as lista entre si — então a tela não muda; ela só para de escorregar.
pub fn assign_missing_root_order(world: &mut World) -> bool {
    let mut used = world.query_filtered::<&RootOrder, Without<crate::ChildOf>>();
    let next = used
        .iter(world)
        .map(|r| r.0)
        .filter(|&o| o != u32::MAX)
        .max()
        .map_or(0, |m| m.saturating_add(1));

    let mut missing = world
        .query_filtered::<Entity, (
            With<crate::Transform>,
            Without<crate::ChildOf>,
            Without<RootOrder>,
        )>()
        .iter(world)
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return false;
    }
    // A ordem que a árvore mostra HOJE entre as sem-ordem é a de `to_bits`. Congelá-la
    // aqui é o que faz a tela não piscar quando os números aparecem.
    missing.sort_unstable_by_key(|e| e.to_bits());
    for (i, e) in missing.into_iter().enumerate() {
        let order = next.saturating_add(u32::try_from(i).unwrap_or(u32::MAX));
        world.entity_mut(e).insert(RootOrder(order));
    }
    true
}
