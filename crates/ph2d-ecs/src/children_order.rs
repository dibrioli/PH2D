//! **A ORDEM dos filhos de uma entidade** — a única porta que a reescreve.
//!
//! # Por que uma porta, e não duas linhas em cada chamador
//!
//! O `bevy_ecs` não tem um "põe este filho no índice `k`": a lista `Children` preserva a ordem de
//! INSERÇÃO, e a única forma de a reordenar é remover e re-inserir o `ChildOf` de cada filho na
//! sequência desejada. Isso é um detalhe do motor, não do produto — e enquanto ele viveu escrito à
//! mão dentro do drop da Hierarquia era uma resposta que o segundo consumidor teria de adivinhar.
//!
//! Hoje há dois: o **drop na Hierarquia** (que constrói a ordem a partir de *antes/depois deste
//! irmão*) e o **arrasto no CANVAS dentro de um fluxo** (que a constrói a partir de um índice).
//! Cada um sabe a ORDEM que quer; nenhum precisa saber como o `bevy` a guarda.

use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;

use crate::Entity;

/// Reescreve a lista de filhos de `parent` para que ela seja exactamente `desired`.
///
/// ⚠️ **`desired` tem de ser a lista COMPLETA**: quem ficar de fora é re-inserido no fim pelo
/// simples facto de nunca ter sido removido, e a ordem final deixaria de ser a que o chamador
/// pediu. Entidades que já não são filhas de `parent` são ignoradas.
///
/// No-op quando a ordem já é a pedida — a checagem existe porque o undo deste editor regista por
/// DIFF, e remover-e-inserir todo o `ChildOf` marcaria o arquétipo de cada filho como mudado.
pub fn reinsert_children_in_order(world: &mut World, parent: Entity, desired: &[Entity]) -> bool {
    let current: Vec<Entity> = world
        .get::<Children>(parent)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    if current == desired {
        return false;
    }
    for &child in desired {
        if !current.contains(&child) {
            continue;
        }
        if let Ok(mut entry) = world.get_entity_mut(child) {
            entry.remove::<ChildOf>();
            entry.insert(ChildOf(parent));
        }
    }
    true
}

#[cfg(test)]
#[path = "children_order_tests.rs"]
mod tests;
