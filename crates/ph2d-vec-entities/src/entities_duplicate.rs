//! ⭐ **O DUPLICAR de uma forma** — a cópia nasce ao lado da original, no mesmo pai (escolha do dono,
//! 04/10). Sobreviveu ao objecto-contentor (retirado pelo dono a 05/10, spec/06 F3 ▸ Vector 2.ª volta):
//! a lei é de qualquer pai — um grupo, uma moldura.

use ph2d_ecs::{ChildOf, Entity, RootOrder, SiblingOrder, SimWorld};
use ph2d_vec_scene::VecPathId;

use super::VecEntityMap;

/// ⭐ **A CÓPIA nasce IRMÃ da original** (escolha do dono, 04/10: *«duplicar e permanecer como filho
/// do mesmo pai»*) — as formas `copies` (acabadas de colar, ainda sem entidade) ganham entidade e
/// passam a filhas do pai de `source`, logo a seguir a ela entre as irmãs, no mesmo sítio do mundo.
/// Sem pai (`source` raiz) a cópia fica na raiz, como a original. Chamada no MESMO gesto (um passo de
/// undo).
pub fn place_beside(
    sim: &mut SimWorld,
    scene: &mut ph2d_vec_scene::VecScene,
    map: &mut VecEntityMap,
    copies: &[VecPathId],
    source: Entity,
) {
    let Some(parent) = sim.world().get::<ChildOf>(source).map(ChildOf::parent) else {
        return;
    };
    super::sync(sim, scene, map);
    crate::transform::settle_origins(sim, scene, map, &[]);
    let after = sim.world().get::<SiblingOrder>(source).map(|s| s.0);
    let roots: Vec<Entity> = copies
        .iter()
        .filter_map(|id| map.get(id))
        .map(|b| Entity::from_bits(*b))
        .filter(|e| sim.world().get::<ChildOf>(*e).is_none())
        .collect();
    if let Some(after) = after {
        let n = u32::try_from(roots.len()).unwrap_or(u32::MAX);
        let mut q = sim.world_mut().query::<(&ChildOf, &mut SiblingOrder)>();
        for (c, mut s) in q.iter_mut(sim.world_mut()) {
            if c.parent() == parent && s.0 > after {
                s.0 = s.0.saturating_add(n);
            }
        }
    }
    for (k, e) in roots.into_iter().enumerate() {
        if !crate::transform::reparent_keeping_world(sim, e, parent) {
            continue;
        }
        let mut em = sim.world_mut().entity_mut(e);
        em.remove::<RootOrder>();
        match after {
            Some(a) => {
                let k = u32::try_from(k).unwrap_or(u32::MAX);
                em.insert(SiblingOrder(a.saturating_add(1).saturating_add(k)));
            }
            None => {
                em.remove::<SiblingOrder>();
            }
        }
    }
}

#[cfg(test)]
#[path = "entities_duplicate_tests.rs"]
mod tests;
