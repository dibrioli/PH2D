//! A ordem do cálculo entre entidades: quem lê a pose de outra corre depois dela.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, StableId};

use crate::Constraints;

/// As entidades com `T`, pela ordem do `iter_entities` (arquétipo a arquétipo), sem tocar nas outras.
pub(crate) fn com<T: Component>(world: &World) -> impl Iterator<Item = Entity> + '_ {
    let id = world.component_id::<T>();
    world
        .archetypes()
        .iter()
        .filter(move |a| id.is_some_and(|id| a.contains(id)))
        .flat_map(|a| a.entities().iter().map(|e| e.id()))
}

/// Os `T` pela ordem do `iter_entities`.
pub(crate) fn cada<'w, T: Component>(world: &'w World) -> Vec<(Entity, &'w T)> {
    let Some(mut q) = world.try_query::<&T>() else {
        return Vec::new();
    };
    com::<T>(world)
        .filter_map(|e| Some((e, q.get(world, e).ok()?)))
        .collect()
}

/// Índice `StableId → Entity` de uma passada (a resolução linear por alvo seria O(n²)).
pub(crate) fn indice(world: &World) -> BTreeMap<StableId, Entity> {
    cada::<StableId>(world)
        .into_iter()
        .map(|(e, s)| (*s, e))
        .collect()
}

fn pai(world: &World, e: Entity) -> Option<Entity> {
    world.get::<ChildOf>(e).map(ChildOf::parent)
}

/// `(ordem, em_ciclo)`: as entidades com pilha numa ordem topológica determinística (empate pelo
/// `Entity` em bits) e as que ficaram presas num ciclo — essas não se avaliam.
///
/// `e` depende de toda entidade com pilha que seja antepassada do seu pai ou de um alvo dela (ou o
/// próprio alvo): a pose mundial dessas entra na conta de `e`. E de toda ponta `E` de uma IK cuja
/// corrente esse caminho atravessa (a IK escreve os ossos acima de `E`), salvo se `e` é dessa corrente.
#[must_use]
pub fn ordem(world: &World) -> (Vec<Entity>, BTreeSet<Entity>) {
    ordem_com(world, &indice(world))
}

/// [`ordem`] com o índice `StableId → Entity` da passada.
pub(crate) fn ordem_com(
    world: &World,
    idx: &BTreeMap<StableId, Entity>,
) -> (Vec<Entity>, BTreeSet<Entity>) {
    // Índices densos pela ordem dos bits (a das rondas): a passada é por quadro.
    let mut com: Vec<Entity> = com::<Constraints>(world).collect();
    com.sort_unstable_by_key(|e| e.to_bits());
    com.dedup();
    let n = com.len();
    let ix = |e: Entity| com.binary_search_by_key(&e.to_bits(), |x| x.to_bits()).ok();
    let mut correntes: Vec<Vec<u64>> = vec![Vec::new(); n];
    let mut escrito_por: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (i, &e) in com.iter().enumerate() {
        if let Some(c) = world.get::<Constraints>(e) {
            for x in crate::ik::correntes(world, e, c) {
                correntes[i].push(x.to_bits());
                escrito_por.entry(x.to_bits()).or_default().push(i);
            }
        }
    }
    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &e) in com.iter().enumerate() {
        let mut de = Vec::new();
        let sobe = |mut x: Option<Entity>, de: &mut Vec<usize>| {
            let mut guarda = 0;
            while let Some(a) = x {
                if let Some(j) = ix(a) {
                    de.push(j);
                }
                for &ponta in escrito_por.get(&a.to_bits()).into_iter().flatten() {
                    if ponta != i && !correntes[ponta].contains(&e.to_bits()) {
                        de.push(ponta);
                    }
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
        de.sort_unstable();
        de.dedup();
        deps[i] = de;
    }
    // Kahn por camadas: a camada seguinte é quem ficou livre nesta, por bits (as rondas de sempre).
    let mut falta: Vec<usize> = deps.iter().map(Vec::len).collect();
    let mut quem_espera: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (k, d) in deps.iter().enumerate() {
        for &x in d {
            quem_espera[x].push(k);
        }
    }
    let mut feitos = Vec::with_capacity(n);
    let mut pronto = vec![false; n];
    let mut camada: Vec<usize> = (0..n).filter(|&k| falta[k] == 0).collect();
    while !camada.is_empty() {
        let mut seguinte = Vec::new();
        for k in camada {
            pronto[k] = true;
            feitos.push(com[k]);
            for &d in &quem_espera[k] {
                falta[d] -= 1;
                if falta[d] == 0 {
                    seguinte.push(d);
                }
            }
        }
        seguinte.sort_unstable();
        camada = seguinte;
    }
    let ciclo = (0..n).filter(|&k| !pronto[k]).map(|k| com[k]).collect();
    (feitos, ciclo)
}
