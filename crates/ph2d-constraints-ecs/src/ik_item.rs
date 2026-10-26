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
    world
        .get::<Constraints>(osso)?
        .items
        .iter()
        .find_map(|it| match it.kind {
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

/// A IK `chain = 1` gravada (o *Look At* do painel do esqueleto antes de 25/10) vira um Look At no
/// MESMO lugar da pilha — id, alvo, força e desvio iguais: «olhar para o alvo» tem uma porta só, e a
/// IK começa nos 2 ossos (num objecto a IK é `NaoEOsso` e fica). Idempotente; devolve quantos
/// converteu. Corre a cada quadro com [`converter_ik_goals`]: o ficheiro, a colagem e o `Chain = 1`
/// do painel chegam todos aqui.
pub fn converter_ik_de_um_osso(world: &mut World) -> usize {
    let mut q =
        world.query_filtered::<&mut Constraints, bevy_ecs::query::With<ph2d_skeleton_ecs::Bone>>();
    let mut n = 0;
    for mut c in q.iter_mut(world) {
        for it in &mut c.items {
            if let Kind::Ik(ik) = it.kind
                && ik.chain == 1
            {
                it.kind = Kind::LookAt(ph2d_constraints::LookAt { offset: ik.offset });
                n += 1;
            }
        }
    }
    n
}

/// ⭐ **A âncora de um osso** — o item que persegue um alvo: a IK (alcança, de 2 ossos para cima) ou o
/// Look At (olha). O losango, o arrasto do alvo, o realce, «governado» e o painel do esqueleto leem
/// esta; quem só quer a IK lê [`ik_de`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ancora {
    Ik(Ik),
    LookAt(ph2d_constraints::LookAt),
}

impl Ancora {
    #[must_use]
    pub fn ik(self) -> Option<Ik> {
        match self {
            Self::Ik(ik) => Some(ik),
            Self::LookAt(_) => None,
        }
    }

    /// O desvio do olhar (radianos): o do Look At, ou o da IK (lido só quando ela resolve 1 osso).
    #[must_use]
    pub fn desvio(self) -> f64 {
        match self {
            Self::Ik(ik) => ik.offset,
            Self::LookAt(l) => l.offset,
        }
    }

    pub fn set_desvio(&mut self, rad: f64) {
        match self {
            Self::Ik(ik) => ik.offset = rad,
            Self::LookAt(l) => l.offset = rad,
        }
    }

    fn of(kind: Kind) -> Option<Self> {
        match kind {
            Kind::Ik(ik) => Some(Self::Ik(ik)),
            Kind::LookAt(l) => Some(Self::LookAt(l)),
            _ => None,
        }
    }

    fn kind(self) -> Kind {
        match self {
            Self::Ik(ik) => Kind::Ik(ik),
            Self::LookAt(l) => Kind::LookAt(l),
        }
    }
}

/// A (primeira) âncora do osso: `(id, item, âncora)`. `None` também para quem não é osso — o Look At
/// de um objecto é uma constraint do cartão, não uma âncora do esqueleto.
#[must_use]
pub fn ancora_de(world: &World, osso: Entity) -> Option<(u32, Item, Ancora)> {
    world.get::<ph2d_skeleton_ecs::Bone>(osso)?;
    world
        .get::<Constraints>(osso)?
        .items
        .iter()
        .find_map(|it| Some((it.id, *it, Ancora::of(it.kind)?)))
}

/// Todos os ossos com âncora, por bits.
#[must_use]
pub fn ancoras(world: &World) -> Vec<(Entity, u32, Item, Ancora)> {
    let mut v: Vec<_> = world
        .iter_entities()
        .filter_map(|e| {
            let (id, it, a) = ancora_de(world, e.id())?;
            Some((e.id(), id, it, a))
        })
        .collect();
    v.sort_by_key(|(e, ..)| e.to_bits());
    v
}

/// Edita a âncora do osso; `false` se ele não tem uma.
pub fn com_ancora(world: &mut World, osso: Entity, f: impl FnOnce(&mut Item, &mut Ancora)) -> bool {
    if world.get::<ph2d_skeleton_ecs::Bone>(osso).is_none() {
        return false;
    }
    let Some(mut c) = world.get_mut::<Constraints>(osso) else {
        return false;
    };
    let Some(it) = c.items.iter_mut().find(|i| Ancora::of(i.kind).is_some()) else {
        return false;
    };
    let Some(mut a) = Ancora::of(it.kind) else {
        return false;
    };
    f(it, &mut a);
    it.kind = a.kind();
    true
}

/// Tira as âncoras do osso (e a pilha, se fica vazia). `true` se havia alguma.
pub fn tirar_ancora(world: &mut World, osso: Entity) -> bool {
    let Some(mut c) = world.get_mut::<Constraints>(osso) else {
        return false;
    };
    let n = c.items.len();
    c.items.retain(|i| Ancora::of(i.kind).is_none());
    let tirou = c.items.len() != n;
    if c.items.is_empty() {
        world.entity_mut(osso).remove::<Constraints>();
    }
    tirou
}
