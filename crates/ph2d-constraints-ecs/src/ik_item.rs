//! ⭐ **O item IK de um osso** — a porta única de quem o lê e escreve (o painel do esqueleto, o gesto de
//! «Add IK», a timeline, o espelho, o cartão). O `ph2d_skeleton_ecs::IkGoal` passou a ser só a forma
//! de um ficheiro anterior à W2: [`converter_ik_goals`] torna-o neste item
//! (`docs/BlenderConstraints/02_W2_ossos.md` §1).

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_constraints::{Ik, Kind};
use ph2d_ecs::StableId;
use ph2d_skeleton_ecs::IkGoal;

use crate::{Constraints, Item};

/// O (primeiro) item IK do osso: `(id, item, ik)`.
#[must_use]
pub fn ik_de(world: &World, osso: Entity) -> Option<(u32, Item, Ik)> {
    world.get::<Constraints>(osso)?.items.iter().find_map(|it| match it.kind {
        Kind::Ik(ik) => Some((it.id, *it, ik)),
        _ => None,
    })
}

/// Todos os ossos com item IK, por bits (determinístico).
#[must_use]
pub fn iks(world: &World) -> Vec<(Entity, u32, Item, Ik)> {
    let mut v: Vec<_> = world
        .iter_entities()
        .filter_map(|e| {
            let (id, it, ik) = ik_de(world, e.id())?;
            Some((e.id(), id, it, ik))
        })
        .collect();
    v.sort_by_key(|(e, ..)| e.to_bits());
    v
}

/// Edita o item IK do osso; `false` se ele não tem um.
pub fn com_ik(world: &mut World, osso: Entity, f: impl FnOnce(&mut Item, &mut Ik)) -> bool {
    let Some(mut c) = world.get_mut::<Constraints>(osso) else {
        return false;
    };
    let Some(it) = c.items.iter_mut().find(|i| matches!(i.kind, Kind::Ik(_))) else {
        return false;
    };
    let Kind::Ik(mut ik) = it.kind else {
        return false;
    };
    f(it, &mut ik);
    it.kind = Kind::Ik(ik);
    true
}

/// Acrescenta um item IK no FIM da pilha do osso (cria a pilha se falta) e devolve o id.
pub fn por_ik(world: &mut World, osso: Entity, alvo: StableId, ik: Ik, forca: f64) -> u32 {
    let alvo = (alvo != StableId::NONE).then_some(alvo);
    let mut e = world.entity_mut(osso);
    if e.get::<Constraints>().is_none() {
        e.insert(Constraints::default());
    }
    let mut c = e.get_mut::<Constraints>().expect("acabada de pôr");
    let id = c.push(Kind::Ik(ik), alvo);
    if let Some(it) = c.item_mut(id) {
        it.influence = forca.clamp(0.0, 1.0);
    }
    id
}

/// Tira os itens IK do osso (e a pilha, se fica vazia). `true` se havia algum.
pub fn tirar_ik(world: &mut World, osso: Entity) -> bool {
    let Some(mut c) = world.get_mut::<Constraints>(osso) else {
        return false;
    };
    let n = c.items.len();
    c.items.retain(|i| !matches!(i.kind, Kind::Ik(_)));
    let tirou = c.items.len() != n;
    if c.items.is_empty() {
        world.entity_mut(osso).remove::<Constraints>();
    }
    tirou
}

/// O `IkGoal` gravado (ficheiro, prefab ou colagem anteriores à W2) vira o item IK, no fim da pilha
/// — onde o passe antigo corria. Idempotente; devolve quantos converteu.
pub fn converter_ik_goals(world: &mut World) -> usize {
    let velhos: Vec<(Entity, IkGoal)> = world
        .iter_entities()
        .filter_map(|e| Some((e.id(), *e.get::<IkGoal>()?)))
        .collect();
    for &(e, g) in &velhos {
        world.entity_mut(e).remove::<IkGoal>();
        let ik = Ik {
            chain: g.chain,
            softness: g.softness,
            bend: g.bend,
            offset: g.offset,
            stretch: false,
        };
        por_ik(world, e, g.target, ik, g.mix);
    }
    velhos.len()
}
