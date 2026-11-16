//! **Quantas peças tem a fila de uma montagem feita** (plano §W4, 2.º ponto): o campo «Planks»/«Links»
//! numa ponte ou corrente JÁ montada refaz as cópias — e N peças saem de UMA solta. As cópias são da
//! última da fila pela porta do Duplicate (`deep_copy_subtree` com o registo da app: a imagem que o
//! artista deu à tábua vai junto), um passo adiante cada; as que sobram saem do fim. O papel preso à
//! ÚLTIMA (o poste da direita) anda com ela, como no «Make the parts» (`joint_factory::shift`). As
//! juntas que nascem levam os números das irmãs da mesma regra: o que o artista afinou fica.

use std::collections::BTreeMap;

use ph2d_core::Vec2;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, Name, SimWorld, StableId, Transform};
use ph2d_joint_templates::Link;
use ph2d_physics_ecs::{AssemblyPart, JointAssembly, PhysicsJoint};

use ph2d_joint_assembly::joint_assembly::{joints_of, sync_joints};

/// O que mudou numa fila.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowChange {
    /// O papel de várias, e quantas peças ele tinha e tem.
    pub role: u16,
    pub before: u16,
    pub after: u16,
    /// As peças feitas agora (`bits`).
    pub made: Vec<u64>,
}

fn entity(sim: &SimWorld, id: u64) -> Option<Entity> {
    (id != 0)
        .then(|| ph2d_ecs::entity_with_stable_id(sim.world(), StableId(id)))
        .flatten()
}

fn at(sim: &SimWorld, e: Entity) -> Option<[f32; 2]> {
    ph2d_ecs::world_transform(sim.world(), e).map(|t| [t.translation.x, t.translation.y])
}

/// Põe `e` no ponto de MUNDO `to` (a peça mora na pasta da montagem).
fn put(sim: &mut SimWorld, e: Entity, to: [f32; 2]) {
    let Some(now) = ph2d_ecs::world_transform(sim.world(), e) else {
        return;
    };
    let want = Transform {
        translation: Vec2::new(to[0], to[1]),
        ..now
    };
    let parent = ph2d_ecs::parent_world_transform(sim.world(), e);
    if let Some(local) = Transform::inverse_compose(parent, want)
        && let Some(mut t) = sim.world_mut().get_mut::<Transform>(e)
    {
        t.translation = local.translation;
    }
}

/// O nome da `i`-ésima peça da fila: o do gabarito com o sufixo que a última tem («Plank 9 (1)»); uma
/// peça do artista com outro nome dá «Board (1)», «Board (2)»….
fn name_for(sim: &mut SimWorld, last: &str, factory_last: &str, factory: &str) -> String {
    let base = last
        .strip_prefix(factory_last)
        .map_or_else(|| last.to_owned(), |suffix| format!("{factory}{suffix}"));
    ph2d_unique_name::unique_name(sim, &base)
}

/// **Deixa a fila da montagem `assembly` com `n` peças** (`n ≥ 1`). `None` = não é uma montagem com fila,
/// a fila está vazia, ou já tem `n`. Um passo de desfazer (o mesmo quadro).
pub fn set_row_count(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    assembly: Entity,
    n: u16,
) -> Option<RowChange> {
    let asm = sim.world().get::<JointAssembly>(assembly)?.clone();
    let t = ph2d_joint_templates::by_id(asm.template)?;
    let ri = t.roles.iter().position(|r| r.many.is_some())?;
    let (role, many) = (u16::try_from(ri).ok()?, t.roles[ri].many?);
    let row: Vec<u64> = asm.bodies_of(role).filter(|&b| b != 0).collect();
    let k = u16::try_from(row.len()).ok()?;
    let n = n.clamp(1, crate::joint_factory::MAX_ROW_PARTS);
    if k == 0 || n == k {
        return None;
    }
    let ents: Vec<Entity> = row.iter().filter_map(|&b| entity(sim, b)).collect();
    let last = *ents.last()?;
    let last_at = at(sim, last)?;
    // O passo é o da cena (as duas últimas), e o do gabarito com uma peça só.
    let step = match ents[..] {
        [.., prev, _] => {
            at(sim, prev).map_or(many.step, |p| [last_at[0] - p[0], last_at[1] - p[1]])
        }
        _ => many.step,
    };
    let asm_id = ph2d_ecs::stable_id_of(sim.world(), assembly).map_or(0, |s| s.0);
    let mut tuned: BTreeMap<u16, PhysicsJoint> = BTreeMap::new();
    for (_, j) in joints_of(sim.world_mut(), asm_id) {
        tuned.entry(j.role).or_insert(j);
    }
    // O papel preso à ÚLTIMA anda com ela.
    let d = f32::from(n) - f32::from(k);
    let followers: Vec<u64> = t
        .joints
        .iter()
        .filter(|r| r.link == Link::ToLast && r.b == role)
        .flat_map(|r| asm.bodies_of(r.a).collect::<Vec<_>>())
        .collect();
    for id in followers {
        if let Some(e) = entity(sim, id)
            && let Some(p) = at(sim, e)
        {
            put(sim, e, [p[0] + step[0] * d, p[1] + step[1] * d]);
        }
    }
    let mut change = RowChange {
        role,
        before: k,
        after: n,
        made: Vec::new(),
    };
    let mut fresh = Vec::new();
    if n > k {
        let last_name = sim
            .world()
            .get::<Name>(last)
            .map_or_else(String::new, |s| s.as_str().to_owned());
        let factory_last = t.part_name(role, k - 1);
        for i in k..n {
            let Ok(copy) = ph2d_ecs::instantiate::deep_copy_subtree(
                sim.world_mut(),
                registry,
                last,
                Some(assembly),
            ) else {
                break;
            };
            let e = copy.root;
            // A junta que mora debaixo da peça copiada é da original: a da cópia nasce no `sync`.
            for c in copy.copies() {
                if c != e && sim.world().get::<PhysicsJoint>(c).is_some() {
                    sim.world_mut().despawn(c);
                }
            }
            let k_ahead = f32::from(i - k + 1);
            put(
                sim,
                e,
                [
                    last_at[0] + step[0] * k_ahead,
                    last_at[1] + step[1] * k_ahead,
                ],
            );
            let name = name_for(sim, &last_name, &factory_last, &t.part_name(role, i));
            if let Some(mut nm) = sim.world_mut().get_mut::<Name>(e) {
                *nm = Name::new(name);
            }
            let id = ph2d_ecs::stable_id_of(sim.world(), e).map_or(0, |s| s.0);
            if let Some(mut a) = sim.world_mut().get_mut::<JointAssembly>(assembly) {
                a.parts.push(AssemblyPart { role, body: id });
            }
            change.made.push(e.to_bits());
        }
        // Na Hierarchy, as cópias logo a seguir à última (antes do poste da direita).
        let copies: Vec<Entity> = change.made.iter().map(|&b| Entity::from_bits(b)).collect();
        let mut order = ph2d_ecs::sibling_order::ordered_children(sim.world(), assembly);
        order.retain(|c| !copies.contains(c));
        let at = order
            .iter()
            .position(|&c| c == last)
            .map_or(order.len(), |p| p + 1);
        order.splice(at..at, copies);
        ph2d_ecs::sibling_order::set_sibling_order(sim.world_mut(), assembly, &order);
    } else {
        let gone: Vec<u64> = row[usize::from(n)..].to_vec();
        if let Some(mut a) = sim.world_mut().get_mut::<JointAssembly>(assembly) {
            a.parts
                .retain(|p| !(p.role == role && gone.contains(&p.body)));
        }
        fresh = sync_joints(sim, assembly);
        for id in gone {
            if let Some(e) = entity(sim, id) {
                sim.world_mut().despawn(e);
            }
        }
    }
    // As juntas novas (as da fila e a da ponta, que mudou de par) com os números das irmãs.
    fresh.extend(sync_joints(sim, assembly));
    for j in fresh {
        let Some(&cur) = sim.world().get::<PhysicsJoint>(j) else {
            continue;
        };
        if let Some(&tune) = tuned.get(&cur.role)
            && let Some(mut w) = sim.world_mut().get_mut::<PhysicsJoint>(j)
        {
            *w = PhysicsJoint {
                body_a: cur.body_a,
                body_b: cur.body_b,
                local_a: cur.local_a,
                local_b: cur.local_b,
                anchored: cur.anchored,
                assembly: cur.assembly,
                role: cur.role,
                ..tune
            };
        }
    }
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    Some(change)
}
