//! ⭐⭐ **O OBJECTO VETORIAL** (spec/06 F3 ▸ Vector) — o contentor das formas, e a regra das SOLTAS.
//!
//! Escolha do dono (04/10): *«as shapes são filhas do objeto vetorial vazio»*, e *«cada uma ganha o
//! seu objecto»*. Toda forma vive dentro de um [`VecObject`]: a que nasce com um Edit aberto entra no
//! objecto do Edit; a que aparece sem ele (projecto antigo, cena de teste, colar fora de um Edit)
//! ganha um objecto NOVO só para ela.
//!
//! ⚠️ [`adopt_loose`] corre depois do `sync` e do `settle_origins`, no passe do desenho E na rede da
//! captura (`vec_tree_settle`): a forma entra no objecto no MESMO passo de undo em que nasceu.

use ph2d_ecs::{
    ChildOf, Entity, Name, RootOrder, SiblingOrder, SimWorld, Transform, VecObject, VecPathRef,
};
use ph2d_vec_scene::VecPathId;

use super::{MAX_DEPTH, VecEntityMap, next_root_order};

/// O objecto vetorial de `e`: ele mesmo, se é um; senão o ancestral mais próximo que o é.
#[must_use]
pub fn object_of(sim: &SimWorld, e: Entity) -> Option<Entity> {
    let w = sim.world();
    let mut cur = e;
    for _ in 0..MAX_DEPTH {
        if w.get::<VecObject>(cur).is_some() {
            return Some(cur);
        }
        cur = w.get::<ChildOf>(cur)?.parent();
    }
    None
}

/// O pai de `e`, se ele é um objecto vetorial.
#[must_use]
pub fn object_parent(sim: &SimWorld, e: Entity) -> Option<Entity> {
    let w = sim.world();
    let p = w.get::<ChildOf>(e)?.parent();
    w.get::<VecObject>(p).is_some().then_some(p)
}

/// ⭐ **O ancestral de topo DENTRO do objecto** — sobe até ao filho directo de um [`VecObject`], ou
/// até à raiz fora de qualquer um. É a fronteira que agrupar, desagrupar e o clique respeitam: dentro
/// de um objecto, o «topo» de uma forma nunca é o próprio objecto.
#[must_use]
pub fn top_within_object(sim: &SimWorld, e: Entity) -> Entity {
    let w = sim.world();
    let mut cur = e;
    for _ in 0..MAX_DEPTH {
        let Some(p) = w.get::<ChildOf>(cur).map(ChildOf::parent) else {
            break;
        };
        if w.get::<VecObject>(p).is_some() {
            break;
        }
        cur = p;
    }
    cur
}

/// ⭐ **Cria um objecto vetorial VAZIO** na raiz, em `at` (mundo), com o nome único de `name`.
pub fn spawn_object(sim: &mut SimWorld, name: &str, at: ph2d_core::Vec2) -> Entity {
    let name = ph2d_unique_name::unique_name(sim, name);
    let order = next_root_order(sim);
    let pose = Transform {
        translation: at,
        ..Transform::default()
    };
    sim.world_mut()
        .spawn((pose, Name::new(name), RootOrder(order), VecObject))
        .id()
}

/// Teto de nós de uma sub-árvore (defesa contra save corrompido, não limite de produto).
const MAX_NODES: usize = 4096;

/// A entidade é de OUTRA família (desenha por si e não é vetor) — ou é um objecto vetorial.
fn not_vector(w: &ph2d_ecs::World, e: Entity) -> bool {
    w.get::<VecObject>(e).is_some()
        || w.get::<ph2d_render::Sprite>(e).is_some()
        || w.get::<ph2d_ecs::FlipObjectRef>(e).is_some()
        || w.get::<ph2d_ecs::Sculpt3dPieceRef>(e).is_some()
        || w.get::<ph2d_ecs::PaintedDoc>(e).is_some()
}

/// `p` e tudo debaixo dele é VETOR: formas, o contentor de um envelope, um grupo de formas. É por
/// onde a cabeça solta sobe — uma moldura leva os filhos, um envelope leva a gaiola inteira.
fn vector_only(w: &ph2d_ecs::World, p: Entity) -> bool {
    let mut stack = vec![p];
    let mut seen = 0;
    while let Some(e) = stack.pop() {
        seen += 1;
        if seen > MAX_NODES || not_vector(w, e) {
            return false;
        }
        if let Some(kids) = w.get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    true
}

/// As CABEÇAS soltas: o ancestral mais alto de cada forma sem objecto por cima cuja sub-árvore só
/// tem vetor (uma moldura leva os filhos; um envelope, a gaiola). Uma cabeça com QUALQUER forma em
/// gesto debaixo dela espera — levá-la a meio do traço deslocaria a forma de baixo do cursor.
/// Ordem do mapa (`BTreeMap`) — determinística.
fn loose_heads(sim: &SimWorld, map: &VecEntityMap, drawing: &[VecPathId]) -> Vec<Entity> {
    let w = sim.world();
    let in_gesture = |root: Entity| {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if w.get::<VecPathRef>(e)
                .is_some_and(|r| drawing.contains(&r.0))
            {
                return true;
            }
            if let Some(kids) = w.get::<ph2d_ecs::Children>(e) {
                stack.extend(kids.iter().copied());
            }
        }
        false
    };
    let mut heads: Vec<Entity> = Vec::new();
    for bits in map.values() {
        let e = Entity::from_bits(*bits);
        if w.get_entity(e).is_err() || object_of(sim, e).is_some() {
            continue;
        }
        let mut head = e;
        for _ in 0..MAX_DEPTH {
            match w.get::<ChildOf>(head).map(ChildOf::parent) {
                Some(p) if vector_only(w, p) => head = p,
                _ => break,
            }
        }
        if !in_gesture(head) && !heads.contains(&head) {
            heads.push(head);
        }
    }
    heads
}

/// ⭐⭐ **A regra das SOLTAS** — devolve quantas cabeças mudaram de sítio.
///
/// - `into` = o objecto do Edit aberto: a cabeça RAIZ entra nele mantendo o mundo — à frente, ou
///   ao FUNDO se nasceu atrás dele (o balde vai para o fundo do que preenche).
/// - Sem Edit, e para a cabeça que já vive debaixo de outra coisa (um grupo, um sprite): ela é
///   EMBRULHADA no lugar — um objecto novo toma-lhe o sítio (a ordem e a translação).
///
/// `name` é a base do nome de um objecto novo.
pub fn adopt_loose(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    into: Option<Entity>,
    drawing: &[VecPathId],
    name: &str,
) -> usize {
    let into = into.filter(|o| sim.world().get::<VecObject>(*o).is_some());
    let heads = loose_heads(sim, map, drawing);
    for &h in &heads {
        let root = sim.world().get::<ChildOf>(h).is_none();
        match into {
            Some(o) if root => adopt_into(sim, h, o),
            _ => {
                wrap(sim, h, name);
            }
        }
    }
    heads.len()
}

/// A cabeça raiz `h` passa a filha de `o`, no mesmo sítio do mundo.
fn adopt_into(sim: &mut SimWorld, h: Entity, o: Entity) {
    let w = sim.world();
    let behind = match (w.get::<RootOrder>(h), w.get::<RootOrder>(o)) {
        (Some(a), Some(b)) => a.0 < b.0,
        _ => false,
    };
    if !crate::transform::reparent_keeping_world(sim, h, o) {
        return;
    }
    sim.world_mut()
        .entity_mut(h)
        .remove::<(RootOrder, SiblingOrder)>();
    if behind {
        send_to_back(sim, o, h);
    }
}

/// `h` (já filha de `o`) passa a primeira entre as irmãs.
fn send_to_back(sim: &mut SimWorld, o: Entity, h: Entity) {
    let kids: Vec<Entity> = sim
        .world()
        .get::<ph2d_ecs::Children>(o)
        .map(|c| c.iter().copied().filter(|k| *k != h).collect())
        .unwrap_or_default();
    let w = sim.world_mut();
    for k in kids {
        if let Some(mut s) = w.get_mut::<SiblingOrder>(k) {
            s.0 = s.0.saturating_add(1);
        }
    }
    w.entity_mut(h).insert(SiblingOrder(0));
}

/// ⭐ **A CÓPIA nasce IRMÃ da original** (escolha do dono, 04/10: *«duplicar e permanecer como filho
/// do mesmo pai»*) — as formas `copies` (acabadas de colar, ainda sem entidade) ganham entidade e
/// passam a filhas do pai de `source`, logo a seguir a ela entre as irmãs, no mesmo sítio do mundo.
/// Sem pai (`source` raiz) nada muda: a regra das soltas decide. Chamada no MESMO gesto (um passo de
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

/// ⭐ **`top` passa a SER um objecto vetorial** — um grupo comum ganha o marcador; uma forma é
/// embrulhada num objecto novo. É a porta de quem cria várias formas de UMA vez (o SVG importado
/// entra num só objecto, escolha do dono 04/10). Devolve o objecto.
pub fn enclose(sim: &mut SimWorld, top: Entity, name: &str) -> Entity {
    let w = sim.world();
    if w.get::<VecObject>(top).is_some() {
        return top;
    }
    if w.get::<VecPathRef>(top).is_some() {
        return wrap(sim, top, name);
    }
    sim.world_mut().entity_mut(top).insert(VecObject);
    top
}

/// Um objecto novo toma o sítio de `head` (o pai, a ordem e a translação) e `head` passa a filha
/// dele, na origem — o mundo não se mexe.
fn wrap(sim: &mut SimWorld, head: Entity, name: &str) -> Entity {
    let w = sim.world();
    let pose = w.get::<Transform>(head).copied().unwrap_or_default();
    let parent = w.get::<ChildOf>(head).map(ChildOf::parent);
    let root_order = w.get::<RootOrder>(head).copied();
    let sibling = w.get::<SiblingOrder>(head).copied();
    let root_order = match (parent, root_order) {
        (None, None) => Some(RootOrder(next_root_order(sim))),
        (_, o) => o,
    };
    let name = ph2d_unique_name::unique_name(sim, name);
    let at = Transform {
        translation: pose.translation,
        ..Transform::default()
    };
    let mut obj = sim.world_mut().spawn((at, Name::new(name), VecObject));
    match parent {
        Some(p) => {
            obj.insert(ChildOf(p));
            if let Some(s) = sibling {
                obj.insert(s);
            }
        }
        None => {
            if let Some(r) = root_order {
                obj.insert(r);
            }
        }
    }
    let obj = obj.id();
    let local = Transform {
        translation: ph2d_core::Vec2::ZERO,
        ..pose
    };
    sim.world_mut()
        .entity_mut(head)
        .remove::<(RootOrder, SiblingOrder)>()
        .insert((local, ChildOf(obj)));
    obj
}

#[cfg(test)]
#[path = "entities_object_tests.rs"]
mod tests;
