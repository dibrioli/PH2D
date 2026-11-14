//! **O que a aba lê do mundo**, uma vez por quadro: as montagens (cada uma um objeto «Car 1»), as peças
//! nos papéis e as juntas com os números na unidade do Inspector (`build_joint_info`). A aba nunca guarda
//! o mundo: guarda este retrato e pede as mudanças pelo [`crate::Edit`].

use ph2d_ecs::{Entity, Name, SimWorld, StableId};
use ph2d_editor_core::screens::hero::InspectorJointInfo;
use ph2d_joint_templates::Template;
use ph2d_physics_ecs::{JointAssembly, PhysicsJoint};

/// Uma peça num papel.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub role: u16,
    /// `0` = o papel é o CENÁRIO.
    pub sid: u64,
    pub bits: u64,
    pub name: String,
    /// O centro da peça na cena, metros (a cota do entre-eixos).
    pub at: [f32; 2],
}

/// Uma junta da montagem.
#[derive(Clone, Debug, PartialEq)]
pub struct JointRow {
    pub bits: u64,
    /// O índice da regra no gabarito (`PhysicsJoint::role`).
    pub rule: u16,
    pub joint: PhysicsJoint,
    /// Os números na unidade do cartão (graus na dobradiça, metros no trilho).
    pub info: Option<InspectorJointInfo>,
}

/// Uma montagem na cena.
#[derive(Clone, Debug, PartialEq)]
pub struct AssemblyRow {
    pub sid: u64,
    pub bits: u64,
    pub name: String,
    pub template: &'static Template,
    pub parts: Vec<Part>,
    pub joints: Vec<JointRow>,
    pub linked: u32,
}

impl AssemblyRow {
    /// As peças de um papel, na ordem da fila.
    pub fn parts_of(&self, role: u16) -> impl Iterator<Item = &Part> {
        self.parts.iter().filter(move |p| p.role == role)
    }

    /// As juntas de uma regra.
    pub fn joints_of(&self, rule: u16) -> impl Iterator<Item = &JointRow> {
        self.joints.iter().filter(move |j| j.rule == rule)
    }

    /// Quantos papéis estão ocupados (um papel de várias conta uma vez).
    #[must_use]
    pub fn filled_roles(&self) -> usize {
        (0..self.template.roles.len())
            .filter(|&r| self.parts.iter().any(|p| usize::from(p.role) == r))
            .count()
    }

    #[must_use]
    pub fn is_linked(&self, group: u8) -> bool {
        group < 32 && self.linked & (1 << group) != 0
    }
}

/// O retrato do quadro.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub assemblies: Vec<AssemblyRow>,
    /// Os nomes dos objetos que podem ser peça (têm corpo físico), em ordem — a resposta da queda.
    pub bodies: Vec<String>,
}

impl Snapshot {
    #[must_use]
    pub fn find(&self, sid: u64) -> Option<&AssemblyRow> {
        self.assemblies.iter().find(|a| a.sid == sid)
    }

    /// A montagem de que uma junta faz parte (o pedido do Inspector às vezes nomeia a junta).
    #[must_use]
    pub fn owner_of_joint(&self, sid_or_bits: u64) -> Option<&AssemblyRow> {
        self.assemblies.iter().find(|a| {
            a.joints
                .iter()
                .any(|j| j.bits == sid_or_bits || j.joint.assembly == sid_or_bits)
        })
    }
}

fn name_of(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map_or_else(String::new, |n| n.as_str().to_owned())
}

/// Lê as montagens do mundo, pela ordem da Hierarchy (o `StableId`, estável entre quadros).
#[must_use]
pub fn read(sim: &mut SimWorld) -> Snapshot {
    let mut found: Vec<(Entity, u64, JointAssembly)> = {
        let w = sim.world_mut();
        let mut q = w.query::<(Entity, &StableId, &JointAssembly)>();
        q.iter(w).map(|(e, s, a)| (e, s.0, a.clone())).collect()
    };
    found.sort_by_key(|(_, sid, _)| *sid);
    let mut out = Vec::with_capacity(found.len());
    for (e, sid, asm) in found {
        let Some(template) = ph2d_joint_templates::by_id(asm.template) else {
            continue;
        };
        let parts = asm
            .parts
            .iter()
            .map(|p| {
                let ent = (p.body != 0)
                    .then(|| ph2d_ecs::entity_with_stable_id(sim.world(), StableId(p.body)))
                    .flatten();
                let at = ent
                    .and_then(|e| ph2d_ecs::world_transform(sim.world(), e))
                    .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
                Part {
                    role: p.role,
                    sid: p.body,
                    bits: ent.map_or(0, Entity::to_bits),
                    name: ent.map_or_else(String::new, |e| name_of(sim, e)),
                    at,
                }
            })
            .collect();
        let joints = ph2d_joint_assembly::joint_assembly::joints_of(sim.world_mut(), sid)
            .into_iter()
            .map(|(je, joint)| JointRow {
                bits: je.to_bits(),
                rule: joint.role,
                joint,
                info: ph2d_joint_assembly::joint_info::build_joint_info(sim, je.to_bits(), 0, 0),
            })
            .collect();
        out.push(AssemblyRow {
            sid,
            bits: e.to_bits(),
            name: name_of(sim, e),
            template,
            parts,
            joints,
            linked: asm.linked,
        });
    }
    let mut bodies: Vec<String> = {
        let w = sim.world_mut();
        let mut q = w.query::<(&Name, &ph2d_physics_ecs::RigidBody)>();
        q.iter(w).map(|(n, _)| n.as_str().to_owned()).collect()
    };
    bodies.sort_unstable();
    Snapshot {
        assemblies: out,
        bodies,
    }
}

/// O objeto da cena com este nome, se for um corpo que pode ser peça (tem `RigidBody`).
#[must_use]
pub fn body_named(sim: &mut SimWorld, name: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let e = q
        .iter(w)
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)?;
    w.get::<ph2d_physics_ecs::RigidBody>(e).map(|_| e)
}
