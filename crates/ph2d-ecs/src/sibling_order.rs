//! **`SiblingOrder` — a ordem entre irmãos vira DADO** ([ADR-0164](../../../docs/architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) §1, plano F1).
//!
//! # O bug pré-existente que ele fecha (classe BUGS #15)
//!
//! Medido na auditoria de 2026-08-21 (doc 01 §8, item 12): **nenhum componente guardava a
//! ordem dos filhos**. Ela vivia na ordem de INSERÇÃO da lista `Children` do bevy — memória de
//! runtime, não dado do documento. As três consequências, todas visíveis:
//!
//! 1. **Reordenar irmãos não era desfazível** — o Ctrl+Z não tinha o que repor, porque a
//!    ordem nunca entrou no snapshot.
//! 2. **Não sobrevivia a um restore** — o `snapshot_to_world` re-insere `ChildOf` na ordem
//!    das linhas, e o `canonicalize` do undo reordenava as linhas por CONTEÚDO.
//! 3. **Não sobrevivia ao save/load** pela mesma razão.
//!
//! É o gémeo exato do [`crate::RootOrder`], que já resolveu isto para as RAÍZES em 2026-07 e
//! cujo doc-comment escreve a lei que vale para os dois: *"O conserto não é escolher um
//! desempate melhor: é **não ter empate**."* As raízes tinham número explícito e os filhos
//! não — esta wave fecha a outra metade.
//!
//! # Quem lê
//!
//! O [`crate::scene::world_to_snapshot`] ordena os filhos por este número ao descer a árvore,
//! e o `snapshot_to_world` reconstrói a lista nessa ordem. ⇒ a ordem passa a ser função do
//! **conteúdo**, e sobrevive ao respawn — a mesma propriedade que o `canonicalize` do undo
//! exige de tudo o resto.
//!
//! ⚠️ **Ausente colate no fim** (`u32::MAX`), como no `RootOrder`: um filho recém-criado
//! aparece depois dos que já têm ordem, e a varredura idempotente dá-lhe um número no quadro
//! seguinte sem a tela piscar.

use bevy_ecs::component::Component;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::prelude::{Entity, With, World};
use serde::{Deserialize, Serialize};

/// A posição deste filho entre os irmãos. Menor desenha/lista primeiro.
///
/// ⚠️ **Só faz sentido com `ChildOf`.** Numa raiz quem manda é o [`crate::RootOrder`] — são
/// duas perguntas diferentes com o mesmo formato, e misturá-las daria a uma raiz duas ordens.
#[derive(
    Component, Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct SiblingOrder(pub u32);

impl SiblingOrder {
    /// O valor em que um filho sem ordem colate: **depois** de todos os explícitos.
    pub const UNSET: u32 = u32::MAX;
}

/// A chave de ordenação de um filho: o número explícito, ou o fim da fila.
///
/// ⚠️ O desempate é o `Entity::index()` e **não** o `to_bits()` — medido nesta wave, o
/// `to_bits` do bevy 0.18 inverte a ordem de criação (ver o cabeçalho de
/// [`crate::stable_id`]). Ele só morde entre filhos que ainda não têm número, e só até a
/// varredura correr.
#[must_use]
pub fn sibling_key(world: &World, child: Entity) -> (u32, bevy_ecs::entity::EntityIndex) {
    let order = world
        .get::<SiblingOrder>(child)
        .map_or(SiblingOrder::UNSET, |s| s.0);
    (order, child.index())
}

/// ⭐⭐⭐ **A chave de ordenação de uma RAIZ** — a gémea da de cima, e pela mesma razão.
///
/// ⛔⛔ **Ela existe porque a lei estava escrita DUAS vezes e as duas cópias divergiram** (report do
/// Enio, 2026-08-27: *«com uso de Undo muitas vezes o z-order muda sem o comando do usuário, e
/// sprites que estão mais abaixo na hierarquia passam a ser desenhadas atrás de sprites que estão
/// mais acima»*).
///
/// A **lista** (`build_hierarchy_snapshot`) e o **canvas** (`propagate_transforms`, que é quem dá o
/// `draw_order`) faziam a mesma pergunta e respondiam diferente:
///
/// | | raízes | filhos |
/// |---|---|---|
/// | a lista | `(RootOrder, bits)` | **`sibling_key`** (ADR-0164 F1) |
/// | o canvas | `(RootOrder, bits)` | ⛔ **a ordem de INSERÇÃO da lista `Children`** |
///
/// A ordem de inserção é memória de runtime que o respawn do undo reconstrói do zero — por
/// `StableId`, que não é o `SiblingOrder`. ⇒ **ao primeiro Ctrl+Z a ordem de desenho passa a ser
/// outra**, e a lista continua a mostrar a que o artista autorou. A F1 curou este defeito na lista
/// e ninguém voltou a perguntar o que o desenho ainda usava. *Uma lei escrita em dois sítios ainda
/// não é uma lei — só uma PORTA é.*
///
/// ⚠️ O desempate é o `Entity::index()`, como no [`sibling_key`], e só morde entre raízes que ainda
/// não têm número — `assign_missing_root_order` corre no passe de quadro e apaga o empate.
#[must_use]
pub fn root_key(world: &World, root: Entity) -> (u32, bevy_ecs::entity::EntityIndex) {
    let order = world
        .get::<crate::RootOrder>(root)
        .map_or(u32::MAX, |r| r.0);
    (order, root.index())
}

/// **Dá um `SiblingOrder` a todo filho que ainda não tem um, preservando a ordem que a
/// Hierarquia já mostra.** Idempotente; devolve `false` quando não havia nada a fazer.
///
/// Gémea de [`crate::assign_missing_root_order`]. Corre no mesmo sítio do passe de quadro, e
/// pela mesma razão: a ordem tem de existir como DADO **antes** da captura do fim do quadro,
/// senão o primeiro Ctrl+Z depois de criar um filho não tem o que repor.
pub fn assign_missing_sibling_order(world: &mut World) -> bool {
    // Os pais que têm filhos. Recolhidos primeiro porque o laço muta o mundo.
    let parents: Vec<Entity> = world
        .query_filtered::<Entity, With<Children>>()
        .iter(world)
        .collect();

    let mut changed = false;
    for parent in parents {
        let Some(children) = world.get::<Children>(parent) else {
            continue;
        };
        let kids: Vec<Entity> = children.iter().copied().collect();
        // O maior número já atribuído entre estes irmãos — o novo entra depois.
        let next = kids
            .iter()
            .filter_map(|&c| world.get::<SiblingOrder>(c).map(|s| s.0))
            .filter(|&o| o != SiblingOrder::UNSET)
            .max()
            .map_or(0, |m| m.saturating_add(1));

        // Congela a ordem que a árvore mostra HOJE entre os sem-número: a lista `Children`
        // é a ordem de inserção, que é exatamente o que o artista está a ver.
        let missing: Vec<Entity> = kids
            .iter()
            .copied()
            .filter(|&c| world.get::<SiblingOrder>(c).is_none())
            .collect();
        if missing.is_empty() {
            continue;
        }
        changed = true;
        for (i, c) in missing.into_iter().enumerate() {
            let order = next.saturating_add(u32::try_from(i).unwrap_or(SiblingOrder::UNSET));
            world.entity_mut(c).insert(SiblingOrder(order));
        }
    }
    changed
}

/// **Escreve a ordem pedida como DADO** — a porta que torna um reordenar desfazível.
///
/// `desired` é a lista completa de filhos de `parent`, na ordem que o artista quer. Cada um
/// recebe o seu índice; quem não for filho de `parent` é ignorado.
///
/// ⚠️ **Escreve com `set_if_neq` na prática** (compara antes de inserir): o undo deste editor
/// regista por DIFF, e reescrever o mesmo número em todo filho marcaria o arquétipo de cada um
/// como mudado — um passo espúrio por quadro, que é a doença que o `RootOrder` foi construído
/// para curar. É a mesma precaução que o [`crate::children_order::reinsert_children_in_order`]
/// já tomava, aqui pelo mesmo motivo.
pub fn set_sibling_order(world: &mut World, parent: Entity, desired: &[Entity]) -> bool {
    let mut changed = false;
    for (i, &child) in desired.iter().enumerate() {
        let is_child = world.get::<ChildOf>(child).map(|c| c.0) == Some(parent);
        if !is_child {
            continue;
        }
        let want = SiblingOrder(u32::try_from(i).unwrap_or(SiblingOrder::UNSET));
        if world.get::<SiblingOrder>(child) == Some(&want) {
            continue;
        }
        world.entity_mut(child).insert(want);
        changed = true;
    }
    changed
}

/// Os filhos de `parent`, **na ordem do documento**.
///
/// É esta função que o snapshot usa para descer a árvore, e é ela que faz a ordem ser função
/// do dado em vez da memória de inserção do bevy.
#[must_use]
pub fn ordered_children(world: &World, parent: Entity) -> Vec<Entity> {
    let mut kids: Vec<Entity> = world
        .get::<Children>(parent)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    kids.sort_by_key(|&c| sibling_key(world, c));
    kids
}

#[cfg(test)]
#[path = "sibling_order_tests.rs"]
mod tests;
