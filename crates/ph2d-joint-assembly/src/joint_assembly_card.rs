//! **O que o cartão Joint sabe de uma junta de montagem** (Joint Editor W2): o título pelo papel, o
//! grupo «iguais entre si» e a edição em grupo. A montagem é a fonte (ADR-0201): a junta só aponta.

use ph2d_ecs::{Entity, Name, SimWorld, StableId, World};
use ph2d_editor_core::JointFieldEdit;
use ph2d_i18n::tr_with;
use ph2d_joint_templates::{JointRule, Template};
use ph2d_physics_ecs::{JointAssembly, PhysicsJoint};

/// O título, o «Same as …» e se o grupo está ligado.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardContext {
    pub title: String,
    pub same_as: String,
    pub linked: bool,
}

fn assembly_of(world: &World, id: u64) -> Option<Entity> {
    (id != 0)
        .then(|| ph2d_ecs::entity_with_stable_id(world, StableId(id)))
        .flatten()
}

fn rule_of(
    world: &World,
    joint: &PhysicsJoint,
) -> Option<(Entity, &'static Template, &'static JointRule)> {
    let asm = assembly_of(world, joint.assembly)?;
    let t = ph2d_joint_templates::by_id(world.get::<JointAssembly>(asm)?.template)?;
    Some((asm, t, t.joints.get(usize::from(joint.role))?))
}

/// O cartão de uma junta: «Front wheel · Car 1» e os papéis do grupo dela.
#[must_use]
pub fn card_context(world: &World, joint: &PhysicsJoint) -> CardContext {
    let Some((asm, t, rule)) = rule_of(world, joint) else {
        return CardContext::default();
    };
    let name = world.get::<Name>(asm).map_or("", |n| n.as_str());
    let title = tr_with(
        "panel.inspector.joint.role_title",
        &[("role", &rule.name), ("assembly", &name)],
    );
    // Os papéis das OUTRAS regras do grupo, e o da própria quando ela se repete (as tábuas).
    let me = usize::from(joint.role);
    let mut names: Vec<&str> = Vec::new();
    for (i, r) in t.joints.iter().enumerate() {
        let repeats = i == me && r.link == ph2d_joint_templates::Link::Chain;
        if r.group == rule.group && (i != me || repeats) && !names.contains(&r.name) {
            names.push(r.name);
        }
    }
    let linked = world
        .get::<JointAssembly>(asm)
        .is_some_and(|a| a.is_linked(rule.group));
    CardContext {
        title,
        same_as: names.join(", "),
        linked,
    }
}

/// **O bloco «Joints» do corpo `body_bits`** (Joint Editor W2): toda junta que o nomeia (lado A ou B),
/// pelo título do papel ou, fora de montagem, pelo `Name` do objeto-junta. Ordem estável: montagem,
/// papel, bits.
#[must_use]
pub fn body_joints(
    world: &mut World,
    body_bits: Option<u64>,
) -> Vec<ph2d_editor_core::screens::hero::InspectorBodyJoint> {
    // A junta nomeia o corpo pela IDENTIDADE (`StableId`, `name_refs::resolve_body_names`).
    let Some(id) = body_bits
        .and_then(|b| ph2d_ecs::stable_id_of(world, Entity::from_bits(b)))
        .map(|s| s.0)
    else {
        return Vec::new();
    };
    let mut q = world.query::<(Entity, &PhysicsJoint)>();
    let mut found: Vec<(Entity, PhysicsJoint)> = q
        .iter(world)
        .filter(|(_, j)| j.body_a == id || j.body_b == id)
        .map(|(e, j)| (e, *j))
        .collect();
    found.sort_by_key(|(e, j)| (j.assembly, j.role, e.to_bits()));
    found
        .into_iter()
        .map(|(e, j)| {
            let title = card_context(world, &j).title;
            let label = if title.is_empty() {
                world
                    .get::<Name>(e)
                    .map_or_else(String::new, |n| n.as_str().to_owned())
            } else {
                title
            };
            ph2d_editor_core::screens::hero::InspectorBodyJoint {
                joint_bits: e.to_bits(),
                label,
            }
        })
        .collect()
}

/// As OUTRAS juntas do grupo ligado desta (vazio fora de montagem ou com o grupo desligado).
#[must_use]
pub fn linked_siblings(world: &mut World, joint_e: Entity) -> Vec<Entity> {
    let Some(&joint) = world.get::<PhysicsJoint>(joint_e) else {
        return Vec::new();
    };
    let Some((asm, t, rule)) = rule_of(world, &joint) else {
        return Vec::new();
    };
    if !world
        .get::<JointAssembly>(asm)
        .is_some_and(|a| a.is_linked(rule.group))
    {
        return Vec::new();
    }
    crate::joint_assembly::joints_of(world, joint.assembly)
        .into_iter()
        .filter(|(e, j)| {
            *e != joint_e
                && t.joints
                    .get(usize::from(j.role))
                    .is_some_and(|r| r.group == rule.group)
        })
        .map(|(e, _)| e)
        .collect()
}

/// **A edição que vale para o grupo** — afinação. Não: o tipo (o gabarito pede-o), o Drive (a roda
/// de trás puxa, a da frente não), o Active, e os gestos sobre o objeto (corpos, troca, apagar…).
#[must_use]
pub fn edit_is_shared(edit: JointFieldEdit) -> bool {
    use JointFieldEdit as E;
    match edit {
        E::LimitsEnabled(_)
        | E::LimitMin(_)
        | E::LimitMax(_)
        | E::MotorMode(_)
        | E::MotorSpeed(_)
        | E::MotorTarget(_)
        | E::MotorMaxForce(_)
        | E::RestLength(_)
        | E::Stiffness(_)
        | E::Damping(_)
        | E::MaxLength(_)
        | E::BreakEnabled(_)
        | E::BreakForce(_)
        | E::BreakTorque(_)
        | E::Soft(_)
        | E::CollideConnected(_)
        | E::OneChain(_)
        | E::AxisMode(..)
        | E::AxisMin(..)
        | E::AxisMax(..)
        | E::MotorAxis(_) => true,
        E::Kind(_)
        | E::MotorEnabled(_)
        | E::Active(_)
        | E::PickBodyA
        | E::PickBodyB
        | E::CopyProperties
        | E::PasteProperties
        | E::Swap
        | E::AddWheel
        | E::AnchorToWorld(_)
        | E::Remove
        | E::Linked(_) => false,
    }
}

/// As edições do quadro, cada uma estendida às irmãs do grupo ligado. Sem repetir um par.
#[must_use]
pub fn with_linked(
    sim: &mut SimWorld,
    edits: &[(u64, JointFieldEdit)],
) -> Vec<(u64, JointFieldEdit)> {
    let mut out: Vec<(u64, JointFieldEdit)> = Vec::with_capacity(edits.len());
    for &(bits, edit) in edits {
        if !out.contains(&(bits, edit)) {
            out.push((bits, edit));
        }
        if edit_is_shared(edit) {
            for s in linked_siblings(sim.world_mut(), Entity::from_bits(bits)) {
                let pair = (s.to_bits(), edit);
                if !out.contains(&pair) {
                    out.push(pair);
                }
            }
        }
    }
    out
}

/// «Same as …»: liga ou desliga o grupo desta junta na montagem. Devolve se mudou.
pub fn set_linked(world: &mut World, joint_e: Entity, on: bool) -> bool {
    let Some(&joint) = world.get::<PhysicsJoint>(joint_e) else {
        return false;
    };
    let Some((asm, _, rule)) = rule_of(world, &joint) else {
        return false;
    };
    let Some(mut a) = world.get_mut::<JointAssembly>(asm) else {
        return false;
    };
    let bit = 1_u32.checked_shl(u32::from(rule.group)).unwrap_or(0);
    let next = if on { a.linked | bit } else { a.linked & !bit };
    if next == a.linked {
        return false;
    }
    a.linked = next;
    true
}
