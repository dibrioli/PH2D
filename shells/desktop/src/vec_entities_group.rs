//! **AGRUPAR e DESAGRUPAR** — o verbo que muda a árvore (ADR-0110).
//!
//! ⚠️ **Irmão de [`super`] por ASSUNTO e pelo tecto de 600 LOC da shell:** lá mora o que a ponte
//! MANTÉM (a identidade `path ⟺ entidade`, a ordem, o que a árvore esconde); aqui, o gesto que
//! reparenta. O grupo é o MESMO que os sprites usam — por isso ele aceita qualquer mistura de
//! tipos, e não há um tipo de nó especial.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecPathRef};

use super::{next_root_order, top_ancestor};

/// Agrupa as entidades `members` sob uma **entidade comum nova** (nome, `Transform`,
/// `RootOrder`), preservando a ordem. É o mesmo grupo que os sprites usam — por isso
/// ele aceita qualquer mistura de tipos (ADR-0110). Devolve o grupo, ou `None` se
/// sobrar menos de 2 ancestrais de topo distintos.
///
/// Agrupar normaliza para os ancestrais de topo: pegar um filho traz o grupo dele
/// junto (aninhamento), não o filho solto — a convenção de qualquer editor.
pub(crate) fn group_entities(sim: &mut SimWorld, members: &[u64], name: String) -> Option<u64> {
    let mut tops: Vec<Entity> = Vec::new();
    for &bits in members {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let t = top_ancestor(sim, e);
        if !tops.contains(&t) {
            tops.push(t);
        }
    }
    if tops.len() < 2 {
        return None;
    }
    let order = next_root_order(sim);
    let group = sim
        .world_mut()
        .spawn((Transform::default(), Name::new(name), RootOrder(order)))
        .id();
    // `Children` preserva a ordem de inserção → os membros entram na ordem de z.
    for t in tops {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(t) {
            e.remove::<RootOrder>();
            e.insert(ChildOf(group));
        }
    }
    Some(group.to_bits())
}

/// Dissolve os grupos de topo tocados por `members`. Um "grupo" aqui é o que a
/// árvore chama de grupo: uma entidade **sem geometria própria** (nem `VecPathRef`
/// nem sprite) que tem filhos. Um sprite com filhos é um pai, não um grupo — e
/// dissolvê-lo apagaria um objeto. Devolve quantos grupos sumiram.
pub(crate) fn ungroup_entities(sim: &mut SimWorld, members: &[u64]) -> usize {
    let mut tops: Vec<Entity> = Vec::new();
    for &bits in members {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let t = top_ancestor(sim, e);
        if t != e && !tops.contains(&t) && is_plain_group(sim, t) {
            tops.push(t);
        }
    }
    let mut order = next_root_order(sim);
    for g in &tops {
        let parent = sim.world().get::<ChildOf>(*g).map(|c| c.parent());
        let kids: Vec<Entity> = sim
            .world()
            .get::<ph2d_ecs::Children>(*g)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();
        for k in kids {
            if let Ok(mut e) = sim.world_mut().get_entity_mut(k) {
                match parent {
                    Some(p) => {
                        e.insert(ChildOf(p));
                    }
                    None => {
                        e.remove::<ChildOf>();
                        e.insert(RootOrder(order));
                        order = order.saturating_add(1);
                    }
                }
            }
        }
        if let Ok(e) = sim.world_mut().get_entity_mut(*g) {
            e.despawn();
        }
    }
    tops.len()
}

/// A entidade é um grupo puro: sem geometria própria, mas com filhos. Um sprite ou
/// um path com filhos NÃO é um grupo — dissolvê-lo apagaria um objeto.
fn is_plain_group(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    w.get::<VecPathRef>(e).is_none()
        && w.get::<ph2d_render::Sprite>(e).is_none()
        && w.get::<ph2d_ecs::Children>(e)
            .is_some_and(|c| !c.is_empty())
}
