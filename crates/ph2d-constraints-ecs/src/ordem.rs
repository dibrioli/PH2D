//! A ordem do cálculo entre entidades: quem lê a pose de outra corre depois dela.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, StableId};

use crate::Constraints;

/// Índice `StableId → Entity` de uma passada (a resolução linear por alvo seria O(n²)).
pub(crate) fn indice(world: &World) -> BTreeMap<StableId, Entity> {
    world
        .iter_entities()
        .filter_map(|e| Some((*e.get::<StableId>()?, e.id())))
        .collect()
}

fn pai(world: &World, e: Entity) -> Option<Entity> {
    world.get::<ChildOf>(e).map(ChildOf::parent)
}

/// `(ordem, em_ciclo)`: as entidades com pilha numa ordem topológica determinística (empate pelo
/// `Entity` em bits) e as que ficaram presas num ciclo — essas não se avaliam.
///
/// `e` depende de toda entidade com pilha que seja antepassada do seu pai ou de um alvo dela (ou o
/// próprio alvo): a pose mundial dessas entra na conta de `e`.
#[must_use]
pub fn ordem(world: &World) -> (Vec<Entity>, BTreeSet<Entity>) {
    let idx = indice(world);
    let com: BTreeMap<u64, Entity> = world
        .iter_entities()
        .filter(|e| e.contains::<Constraints>())
        .map(|e| (e.id().to_bits(), e.id()))
        .collect();
    let tem = |e: Entity| com.contains_key(&e.to_bits());
    let mut deps: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
    for &e in com.values() {
        let mut de = BTreeSet::new();
        let sobe = |mut x: Option<Entity>, de: &mut BTreeSet<u64>| {
            let mut guarda = 0;
            while let Some(a) = x {
                if tem(a) {
                    de.insert(a.to_bits());
                }
                x = pai(world, a);
                guarda += 1;
                if guarda > 4096 {
                    break;
                }
            }
        };
        sobe(pai(world, e), &mut de);
        if let Some(c) = world.get::<Constraints>(e) {
            for it in c.items.iter().filter(|i| i.enabled && i.influence > 0.0) {
                if let Some(t) = it.target.and_then(|s| idx.get(&s).copied()) {
                    sobe(Some(t), &mut de);
                }
            }
        }
        deps.insert(e.to_bits(), de);
    }
    let mut feitos = Vec::with_capacity(com.len());
    let mut prontos: BTreeSet<u64> = BTreeSet::new();
    loop {
        let livres: Vec<u64> = deps
            .iter()
            .filter(|(k, d)| !prontos.contains(k) && d.iter().all(|x| prontos.contains(x)))
            .map(|(k, _)| *k)
            .collect();
        if livres.is_empty() {
            break;
        }
        for k in livres {
            prontos.insert(k);
            feitos.push(com[&k]);
        }
    }
    let ciclo = com
        .iter()
        .filter(|(k, _)| !prontos.contains(k))
        .map(|(_, e)| *e)
        .collect();
    (feitos, ciclo)
}
