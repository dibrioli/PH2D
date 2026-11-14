//! **Montar um gabarito de juntas na cena** (ADR-0201; plano `docs/JointEditor/01_plano.md`).
//!
//! Uma porta só: [`place_part`] põe UM objeto num papel e [`sync_joints`] deixa as juntas da
//! montagem iguais ao que o gabarito pede para as peças que já lá estão — cria as que faltam,
//! apaga as que deixaram de ter par (a última tábua mudou), e não toca nas que já existem (os
//! números que o artista afinou ficam). [`assemble`] é a mesma porta, peça a peça.
//!
//! A junta nasce pelas portas de sempre ([`create_joint_at`], [`create_world_pin_at`]): nome,
//! identidade, âncoras e roldanas saem de lá, e aqui só entram os números do gabarito.

use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform, World};
use ph2d_joint_templates::{Anchor, JointRule, Link, Role, Side, Spot, Template};
use ph2d_physics_ecs::{
    AssemblyPart, ColliderPose, JointAssembly, JointKind, PhysicsJoint, PulleyWheel,
};

use crate::joint_create::create_joint_at;
use crate::joint_world::create_world_pin_at;

/// Porque uma peça não entrou no papel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlaceError {
    /// A entidade não é uma montagem, ou o gabarito dela não existe nesta versão.
    NotAnAssembly,
    UnknownRole,
    /// O papel é de uma peça só e já tem a dele.
    RoleTaken,
    /// O papel não pode ficar por conta do cenário.
    WorldNotAllowed,
    /// O objeto já ocupa um papel desta montagem.
    AlreadyPlaced,
    /// O objeto não existe (ou não tem `Transform`).
    NoSuchObject,
}

/// O que [`assemble`] fez.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assembled {
    pub assembly: Entity,
    pub joints: Vec<Entity>,
}

/// Uma montagem VAZIA do gabarito: um objeto real, nomeado («Car 1») e na Hierarchy.
pub fn new_assembly(sim: &mut SimWorld, template: &Template) -> Entity {
    let name = ph2d_unique_name::unique_name(sim, &format!("{} 1", template.name));
    let e = sim
        .world_mut()
        .spawn((
            Name::new(name),
            JointAssembly {
                template: template.id.0,
                ..JointAssembly::default()
            },
            Transform::IDENTITY,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    e
}

/// Põe `body` no papel `role` (`None` = o papel fica por conta do CENÁRIO) e cria as juntas que
/// passaram a ter as duas peças. Devolve as juntas novas.
pub fn place_part(
    sim: &mut SimWorld,
    assembly: Entity,
    role: u16,
    body: Option<Entity>,
) -> Result<Vec<Entity>, PlaceError> {
    let (template, asm) = read(sim.world(), assembly).ok_or(PlaceError::NotAnAssembly)?;
    let spec = template.role(role).ok_or(PlaceError::UnknownRole)?;
    if spec.many.is_none() && asm.bodies_of(role).next().is_some() {
        return Err(PlaceError::RoleTaken);
    }
    let id = match body {
        None if spec.may_be_world => 0,
        None => return Err(PlaceError::WorldNotAllowed),
        Some(e) => {
            let at = ph2d_ecs::world_transform(sim.world(), e).ok_or(PlaceError::NoSuchObject)?;
            // A junta cita a peça pela identidade, e uma peça sem nome não tem como ser lida.
            crate::joint_create::ensure_named(sim, e, "Body").ok_or(PlaceError::NoSuchObject)?;
            ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
            let id = ph2d_ecs::stable_id_of(sim.world(), e).map_or(0, |s| s.0);
            if id == 0 {
                return Err(PlaceError::NoSuchObject);
            }
            if asm.parts.iter().any(|p| p.body == id) {
                return Err(PlaceError::AlreadyPlaced);
            }
            // A montagem mora onde a primeira peça está: o ponto dela na cena deixa de ser a origem.
            if asm.parts.iter().all(|p| p.body == 0)
                && let Some(mut t) = sim.world_mut().get_mut::<Transform>(assembly)
            {
                t.translation = at.translation;
            }
            id
        }
    };
    if let Some(mut a) = sim.world_mut().get_mut::<JointAssembly>(assembly) {
        a.parts.push(AssemblyPart { role, body: id });
    }
    Ok(sync_joints(sim, assembly))
}

/// Uma montagem inteira de uma vez: `parts` é `(papel, objeto)`, `None` = o cenário. Para ao
/// primeiro erro e não deixa meia montagem para trás.
pub fn assemble(
    sim: &mut SimWorld,
    template: &Template,
    parts: &[(u16, Option<Entity>)],
) -> Result<Assembled, PlaceError> {
    let assembly = new_assembly(sim, template);
    let mut joints = Vec::new();
    for &(role, body) in parts {
        match place_part(sim, assembly, role, body) {
            Ok(mut made) => joints.append(&mut made),
            Err(e) => {
                dissolve(sim, assembly);
                return Err(e);
            }
        }
    }
    // Uma junta criada cedo pode ter sido trocada por outra (a ponta da fila): só as vivas contam.
    joints.retain(|&j| sim.world().get::<PhysicsJoint>(j).is_some());
    Ok(Assembled { assembly, joints })
}

/// Apaga a montagem e as juntas dela. As peças ficam.
pub fn dissolve(sim: &mut SimWorld, assembly: Entity) {
    let id = ph2d_ecs::stable_id_of(sim.world(), assembly).map_or(0, |s| s.0);
    for (j, _) in joints_of(sim.world_mut(), id) {
        despawn_joint(sim.world_mut(), j);
    }
    sim.world_mut().despawn(assembly);
}

/// As juntas de uma montagem, pela identidade dela.
pub fn joints_of(world: &mut World, assembly_id: u64) -> Vec<(Entity, PhysicsJoint)> {
    if assembly_id == 0 {
        return Vec::new();
    }
    let mut q = world.query::<(Entity, &PhysicsJoint)>();
    let mut out: Vec<_> = q
        .iter(world)
        .filter(|(_, j)| j.assembly == assembly_id)
        .map(|(e, j)| (e, *j))
        .collect();
    // A ordem do `Query` é a do arquétipo; a das regras é a que a tela e os testes leem.
    out.sort_by_key(|(e, j)| (j.role, e.to_bits()));
    out
}

/// Deixa as juntas da montagem iguais ao que o gabarito pede para as peças presentes. Devolve as
/// juntas criadas.
pub fn sync_joints(sim: &mut SimWorld, assembly: Entity) -> Vec<Entity> {
    let Some((template, asm)) = read(sim.world(), assembly) else {
        return Vec::new();
    };
    let asm_id = ph2d_ecs::stable_id_of(sim.world(), assembly).map_or(0, |s| s.0);
    if asm_id == 0 {
        return Vec::new();
    }
    let wanted = wanted_pairs(template, &asm);
    let mut have: Vec<(Entity, PairKey)> = joints_of(sim.world_mut(), asm_id)
        .into_iter()
        .map(|(e, j)| (e, PairKey::new(j.role, j.body_a, j.body_b)))
        .collect();
    for (e, key) in &have {
        if !wanted.iter().any(|w| w.key() == *key) {
            despawn_joint(sim.world_mut(), *e);
        }
    }
    have.retain(|(_, key)| wanted.iter().any(|w| w.key() == *key));
    let mut made = Vec::new();
    for pair in wanted {
        if have.iter().any(|(_, k)| *k == pair.key()) {
            continue;
        }
        if let Some(j) = create(sim, template, &pair, asm_id) {
            made.push(j);
        }
    }
    made
}

fn read(world: &World, assembly: Entity) -> Option<(&'static Template, JointAssembly)> {
    let asm = world.get::<JointAssembly>(assembly)?.clone();
    Some((ph2d_joint_templates::by_id(asm.template)?, asm))
}

fn despawn_joint(world: &mut World, joint: Entity) {
    // As roldanas de uma polia são objetos da corda: sem ela ficariam a apontar para ninguém.
    let rope = ph2d_ecs::stable_id_of(world, joint).map_or(0, |s| s.0);
    if rope != 0 {
        let mut q = world.query::<(Entity, &PulleyWheel)>();
        let wheels: Vec<Entity> = q
            .iter(world)
            .filter(|(_, w)| w.rope == rope)
            .map(|(e, _)| e)
            .collect();
        for w in wheels {
            world.despawn(w);
        }
    }
    world.despawn(joint);
}

/// Uma ponta de uma junta pedida: a peça (`0` = o cenário), o papel e o lugar dela na fila.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct End {
    body: u64,
    role: u16,
    index: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pair {
    rule: u16,
    a: End,
    b: End,
}

/// A identidade de uma junta dentro da montagem: a regra e as duas peças, sem ordem (a junta de
/// mundo guarda a peça em `body_a`, seja qual for o lado dela no gabarito).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct PairKey(u16, u64, u64);

impl PairKey {
    fn new(rule: u16, x: u64, y: u64) -> Self {
        Self(rule, x.min(y), x.max(y))
    }
}

impl Pair {
    fn key(&self) -> PairKey {
        PairKey::new(self.rule, self.a.body, self.b.body)
    }
}

/// Os pares que o gabarito pede para as peças presentes, na ordem das regras.
fn wanted_pairs(template: &Template, asm: &JointAssembly) -> Vec<Pair> {
    let ends = |role: u16| -> Vec<End> {
        asm.bodies_of(role)
            .enumerate()
            .map(|(i, body)| End {
                body,
                role,
                index: u16::try_from(i).unwrap_or(u16::MAX),
            })
            .collect()
    };
    let mut out = Vec::new();
    for (i, r) in template.joints.iter().enumerate() {
        let rule = u16::try_from(i).unwrap_or(u16::MAX);
        let (a, b) = (ends(r.a), ends(r.b));
        let mut push = |a: End, b: End| {
            // Duas pontas no cenário não são junta nenhuma, nem uma peça consigo mesma.
            if (a.body != 0 || b.body != 0) && a.body != b.body {
                out.push(Pair { rule, a, b });
            }
        };
        match r.link {
            Link::Pair | Link::ToFirst => {
                if let (Some(&a), Some(&b)) = (a.first(), b.first()) {
                    push(a, b);
                }
            }
            Link::ToLast => {
                if let (Some(&a), Some(&b)) = (a.first(), b.last()) {
                    push(a, b);
                }
            }
            Link::Chain => {
                for w in a.windows(2) {
                    push(w[0], w[1]);
                }
            }
        }
    }
    out
}

/// A caixa de uma peça na cena: o centro, a rotação e as meias-extensões da silhueta.
#[derive(Copy, Clone, Debug)]
struct PartBox {
    centre: [f32; 2],
    rotation: f32,
    half: [f32; 2],
}

impl PartBox {
    fn point(&self, s: Spot) -> [f32; 2] {
        let (sin, cos) = libm::sincosf(self.rotation);
        let (x, y) = (s.u * self.half[0], s.v * self.half[1]);
        [
            self.centre[0] + cos * x - sin * y,
            self.centre[1] + sin * x + cos * y,
        ]
    }
}

/// A caixa de um objeto: a silhueta da forma de colisão (a mesma de `seam_between`); sem forma, um
/// ponto no centro do objeto.
fn part_box(world: &World, e: Entity) -> Option<PartBox> {
    let body = ph2d_ecs::world_transform(world, e)?;
    let silhouette = ph2d_physics_ecs::forma_para_editar(world, e).and_then(|f| {
        let col = world.get::<ph2d_physics_ecs::Collider>(f)?;
        let t = ph2d_ecs::world_transform(world, f)?;
        Some(ColliderPose::resolve(
            &ph2d_physics_ecs::forma_resolvida(world, f, col),
            &t,
        ))
    });
    Some(match silhouette {
        Some(p) => {
            let (sin, cos) = libm::sincosf(p.rotation);
            PartBox {
                centre: p.centre,
                rotation: p.rotation,
                half: [p.reach([cos, sin]), p.reach([-sin, cos])],
            }
        }
        None => PartBox {
            centre: [body.translation.x, body.translation.y],
            rotation: body.rotation,
            half: [0.0, 0.0],
        },
    })
}

/// O centro da forma-guia de uma ponta, no desenho do gabarito.
fn guide_at(role: &Role, index: u16) -> [f32; 2] {
    let step = role.many.map_or([0.0, 0.0], |m| m.step);
    let i = f32::from(index);
    [role.at[0] + step[0] * i, role.at[1] + step[1] * i]
}

fn create(sim: &mut SimWorld, template: &Template, pair: &Pair, asm_id: u64) -> Option<Entity> {
    let rule: &JointRule = template.rule(pair.rule)?;
    let (role_a, role_b) = (template.role(pair.a.role)?, template.role(pair.b.role)?);
    let entity = |world: &World, id: u64| {
        (id != 0)
            .then(|| ph2d_ecs::entity_with_stable_id(world, StableId(id)))
            .flatten()
    };
    let (ea, eb) = (
        entity(sim.world(), pair.a.body),
        entity(sim.world(), pair.b.body),
    );
    let real = |e: Option<Entity>| e.and_then(|e| part_box(sim.world(), e));
    let (box_a, box_b) = (real(ea), real(eb));
    // A ponta que é o CENÁRIO não tem silhueta: vale a forma-guia dela, posta em relação à peça
    // real como o desenho do gabarito as põe.
    let virtual_box = |me: (&Role, End), other: (&Role, End), other_box: PartBox| {
        let (m, o) = (guide_at(me.0, me.1.index), guide_at(other.0, other.1.index));
        PartBox {
            centre: [
                other_box.centre[0] + m[0] - o[0],
                other_box.centre[1] + m[1] - o[1],
            ],
            rotation: 0.0,
            half: me.0.guide.half(),
        }
    };
    let (box_a, box_b) = match (box_a, box_b) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) if pair.b.body == 0 => {
            (a, virtual_box((role_b, pair.b), (role_a, pair.a), a))
        }
        (None, Some(b)) if pair.a.body == 0 => {
            (virtual_box((role_a, pair.a), (role_b, pair.b), b), b)
        }
        // Uma peça citada que já não existe (apagada): sem ela não há junta a criar.
        _ => return None,
    };
    let point = |s: Spot| match s.on {
        Side::A => box_a.point(s),
        Side::B => box_b.point(s),
    };
    let ends = match rule.anchor {
        Anchor::At(s) => Some((point(s), point(s))),
        Anchor::Ends(a, b) => Some((point(a), point(b))),
        Anchor::Seam => None,
    };
    let joint = match (ea, eb, ends) {
        (Some(a), Some(b), at) => create_joint_at(sim, a.to_bits(), b.to_bits(), rule.kind, at)?,
        // No pino de mundo, `press` é o ponto NA peça e `anchor` o ponto no cenário.
        (Some(a), None, Some((pa, pb))) => {
            create_world_pin_at(sim, a.to_bits(), rule.kind, pa, pb)?
        }
        (None, Some(b), Some((pa, pb))) => {
            create_world_pin_at(sim, b.to_bits(), rule.kind, pb, pa)?
        }
        // «Onde as silhuetas se encontram» precisa de duas silhuetas (gate do catálogo).
        _ => return None,
    };
    let gap = ends.map_or(0.0, |(a, b)| libm::hypotf(b[0] - a[0], b[1] - a[1]));
    let made = *sim.world().get::<PhysicsJoint>(joint)?;
    let mut fab = rule.joint(gap);
    if rule.kind == JointKind::Pulley {
        // O comprimento da corda de uma polia é o que a criação mediu pelas roldanas.
        fab.max_length = made.max_length;
    }
    let world = sim.world_mut();
    *world.get_mut::<PhysicsJoint>(joint)? = PhysicsJoint {
        body_a: made.body_a,
        body_b: made.body_b,
        local_a: made.local_a,
        local_b: made.local_b,
        anchored: made.anchored,
        assembly: asm_id,
        role: pair.rule,
        ..fab
    };
    if let Some(axis) = rule.axis {
        world.get_mut::<Transform>(joint)?.rotation = axis;
    }
    Some(joint)
}

#[cfg(test)]
#[path = "joint_assembly_tests.rs"]
mod tests;
