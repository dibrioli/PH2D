//! Os três interruptores de uma linha da Hierarquia (olho, cadeado, grupo), virados sobre a entidade.

use crate::{Entity, GroupedChildren, Locked, Visibility, World};

/// Que interruptor da linha.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RowFlag {
    Visible,
    Locked,
    Grouped,
}

/// Vira `flag` em `entity` (uma entidade que já não existe não faz nada).
pub fn flip(world: &mut World, entity: Entity, flag: RowFlag) {
    let Ok(mut e) = world.get_entity_mut(entity) else {
        return;
    };
    match flag {
        RowFlag::Visible => {
            let hidden = e.get::<Visibility>().is_some_and(|v| v.hidden);
            e.insert(Visibility { hidden: !hidden });
        }
        RowFlag::Locked if e.contains::<Locked>() => {
            e.remove::<Locked>();
        }
        RowFlag::Locked => {
            e.insert(Locked);
        }
        RowFlag::Grouped if e.contains::<GroupedChildren>() => {
            e.remove::<GroupedChildren>();
        }
        RowFlag::Grouped => {
            e.insert(GroupedChildren);
        }
    }
}

/// Aplica TODOS os pedidos de um quadro (o arrasto que pinta vários num quadro): `bits_de` resolve
/// a linha para a entidade; uma linha sem entidade fica de fora.
pub fn aplica<R>(
    world: &mut World,
    visible: impl IntoIterator<Item = R>,
    locked: impl IntoIterator<Item = R>,
    grouped: impl IntoIterator<Item = R>,
    bits_de: impl Fn(R) -> Option<u64>,
) {
    let pedidos = (visible.into_iter().map(|r| (r, RowFlag::Visible)))
        .chain(locked.into_iter().map(|r| (r, RowFlag::Locked)))
        .chain(grouped.into_iter().map(|r| (r, RowFlag::Grouped)));
    for (row, flag) in pedidos {
        if let Some(bits) = bits_de(row) {
            flip(world, Entity::from_bits(bits), flag);
        }
    }
}
