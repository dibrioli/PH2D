//! **As edições da aba no mundo.** Cada uma passa pela porta que o resto do app já usa: a montagem por
//! `joint_assembly::place_part`, os números por `joint::joint_with_edit` (o funil do cartão Joint), o
//! «Same as» por `joint_assembly_card::set_linked`. O desfazer vem do diff do projeto: uma edição = um passo.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, StableId, Transform};
use ph2d_editor_core::JointFieldEdit;
use ph2d_i18n::tr_with;
use ph2d_joint_assembly::joint_assembly;
use ph2d_physics_ecs::PhysicsJoint;

use crate::doc::Edit;
use crate::model::body_named;

/// O que uma edição deixou para a aba: a montagem nova a abrir, e a frase do pé.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub made: Option<u64>,
    pub line: Option<String>,
    /// O objeto a selecionar na cena (`bits`).
    pub select: Option<u64>,
    /// Os objetos feitos agora (`bits`), todos selecionados; o primeiro é o principal.
    pub picked: Vec<u64>,
}

fn sid_of(sim: &SimWorld, e: Entity) -> u64 {
    ph2d_ecs::stable_id_of(sim.world(), e).map_or(0, |s| s.0)
}

fn assembly_entity(sim: &SimWorld, sid: u64) -> Option<Entity> {
    (sid != 0)
        .then(|| ph2d_ecs::entity_with_stable_id(sim.world(), StableId(sid)))
        .flatten()
}

/// Aplica uma edição. Uma montagem nova só nasce quando a primeira peça entra de facto (nunca vazia).
pub fn apply(sim: &mut SimWorld, edit: Edit) -> Outcome {
    let mut out = Outcome::default();
    match edit {
        Edit::Place {
            assembly,
            template,
            role,
            bodies,
        } => {
            let Some(t) = ph2d_joint_templates::by_id(template) else {
                return out;
            };
            let parts: Vec<Entity> = bodies.iter().filter_map(|n| body_named(sim, n)).collect();
            if parts.is_empty() {
                return out;
            }
            let (asm, new) = match assembly_entity(sim, assembly) {
                Some(e) => (e, false),
                None => (joint_assembly::new_assembly(sim, t), true),
            };
            let mut made = 0_usize;
            for p in &parts {
                if let Ok(j) = joint_assembly::place_part(sim, asm, role, Some(*p)) {
                    made += j.len();
                } else {
                    break;
                }
            }
            if new
                && sim
                    .world()
                    .get::<ph2d_physics_ecs::JointAssembly>(asm)
                    .is_some_and(|a| a.parts.is_empty())
            {
                // A peça foi recusada: nada de montagem vazia na Hierarchy.
                joint_assembly::dissolve(sim, asm);
                return out;
            }
            let name = sim
                .world()
                .get::<ph2d_ecs::Name>(asm)
                .map_or_else(String::new, |n| n.as_str().to_owned());
            let role_name = t.role(role).map_or("", |r| r.name);
            out.line = Some(tr_with(
                "joint_editor.placed",
                &[
                    ("n", &parts.len()),
                    ("role", &role_name),
                    ("assembly", &name),
                    ("joints", &made),
                ],
            ));
            if new {
                out.made = Some(sid_of(sim, asm));
            }
        }
        Edit::PlaceWorld {
            assembly,
            template,
            role,
        } => {
            let Some(t) = ph2d_joint_templates::by_id(template) else {
                return out;
            };
            let (asm, new) = match assembly_entity(sim, assembly) {
                Some(e) => (e, false),
                None => (joint_assembly::new_assembly(sim, t), true),
            };
            if joint_assembly::place_part(sim, asm, role, None).is_err() && new {
                joint_assembly::dissolve(sim, asm);
                return out;
            }
            let name = sim
                .world()
                .get::<ph2d_ecs::Name>(asm)
                .map_or_else(String::new, |n| n.as_str().to_owned());
            let role_name = t.role(role).map_or("", |r| r.name);
            out.line = Some(tr_with(
                "joint_editor.placed_world",
                &[("role", &role_name), ("assembly", &name)],
            ));
            if new {
                out.made = Some(sid_of(sim, asm));
            }
        }
        Edit::Remove { assembly } => {
            if let Some(e) = assembly_entity(sim, assembly) {
                joint_assembly::dissolve(sim, e);
            }
        }
        Edit::Joints { joints, edit } => {
            for bits in joints {
                write_joint(sim, bits, edit);
            }
        }
        Edit::Linked { joint, on } => {
            let _ = ph2d_joint_assembly::joint_assembly_card::set_linked(
                sim.world_mut(),
                Entity::from_bits(joint),
                on,
            );
        }
        Edit::Select { bits } => out.select = Some(bits),
        Edit::Make {
            template,
            counts,
            at,
            edits,
        } => make(sim, template, &counts, at, &edits, &mut out),
        Edit::Wheelbase { assembly, metres } => {
            if let Some(e) = assembly_entity(sim, assembly) {
                set_wheelbase(sim, e, metres);
            }
        }
    }
    out
}

/// **«Make the parts»** (plano §W4): as peças de fábrica com o meio delas em `at`, montadas numa montagem
/// nova; reais, nomeadas, todas selecionadas e anunciadas. Um passo de desfazer (o mesmo quadro).
fn make(
    sim: &mut SimWorld,
    template: u16,
    counts: &[u16],
    at: [f32; 2],
    edits: &[(u16, JointFieldEdit)],
    out: &mut Outcome,
) {
    use ph2d_joint_assembly::joint_factory::{make_parts, middle};
    let Some(t) = ph2d_joint_templates::by_id(template) else {
        return;
    };
    let mid = middle(t, counts);
    let parts = make_parts(sim, t, [at[0] - mid[0], at[1] - mid[1]], counts, false);
    let bodies: Vec<Entity> = parts.iter().filter_map(|p| p.1).collect();
    let name = |e: Entity| {
        sim.world()
            .get::<ph2d_ecs::Name>(e)
            .map_or_else(String::new, |n| n.as_str().to_owned())
    };
    // Uma fila diz-se pela primeira e a última («Plank 1 … Plank 12»): o aviso cabe em duas linhas.
    let mut names: Vec<String> = Vec::new();
    for (i, _) in t.roles.iter().enumerate() {
        let row: Vec<Entity> = parts
            .iter()
            .filter(|p| usize::from(p.0) == i)
            .filter_map(|p| p.1)
            .collect();
        match row[..] {
            [] => {}
            [one] => names.push(name(one)),
            [first, .., last] => names.push(format!("{} \u{2026} {}", name(first), name(last))),
        }
    }
    // Um gabarito que recusa as próprias peças é um defeito de catálogo (o gate `every_template_…`):
    // as peças ficam, soltas e anunciadas.
    if let Ok(a) = joint_assembly::assemble(sim, t, &parts) {
        let sid = sid_of(sim, a.assembly);
        // Os números escolhidos no painel de criação, regra a regra, pela porta do cartão Joint.
        let joints = joint_assembly::joints_of(sim.world_mut(), sid);
        for &(rule, edit) in edits {
            for (e, _) in joints.iter().filter(|(_, j)| j.role == rule) {
                write_joint(sim, e.to_bits(), edit);
            }
        }
        out.made = Some(sid);
    }
    out.line = Some(tr_with(
        "joint_editor.made",
        &[("n", &bodies.len()), ("names", &names.join(", "))],
    ));
    out.picked = bodies.iter().map(|e| e.to_bits()).collect();
}

/// O funil do cartão Joint, escrito no lugar (o quadro da aba tem o mundo na mão).
fn write_joint(sim: &mut SimWorld, bits: u64, edit: JointFieldEdit) {
    let e = Entity::from_bits(bits);
    let Some(&cur) = sim.world().get::<PhysicsJoint>(e) else {
        return;
    };
    if let Some(next) = ph2d_joint_assembly::joint_info::joint_with_edit(cur, edit)
        && next != cur
        && let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e)
    {
        *j = next;
    }
}

/// **O entre-eixos**: a roda da frente (a última regra de roda) anda ao longo do corpo até ficar a `metres`
/// da de trás; a junta dela vai junto (a âncora no corpo, a entidade-junta). Só peças na raiz da cena.
fn set_wheelbase(sim: &mut SimWorld, asm: Entity, metres: f32) {
    let asm_id = sid_of(sim, asm);
    let joints = joint_assembly::joints_of(sim.world_mut(), asm_id);
    let wheels: Vec<&(Entity, PhysicsJoint)> = joints
        .iter()
        .filter(|(_, j)| j.kind == ph2d_physics_ecs::JointKind::Wheel)
        .collect();
    let [rear, front] = wheels[..] else {
        return;
    };
    let ent = |id: u64| ph2d_ecs::entity_with_stable_id(sim.world(), StableId(id));
    let (Some(body), Some(rw), Some(fw)) =
        (ent(front.1.body_a), ent(rear.1.body_b), ent(front.1.body_b))
    else {
        return;
    };
    let tf = |e: Entity| ph2d_ecs::world_transform(sim.world(), e);
    let (Some(b), Some(r), Some(f)) = (tf(body), tf(rw), tf(fw)) else {
        return;
    };
    let now = libm::hypotf(
        f.translation.x - r.translation.x,
        f.translation.y - r.translation.y,
    );
    let delta = metres.max(0.0) - now;
    if delta.abs() <= f32::EPSILON {
        return;
    }
    let (sin, cos) = libm::sincosf(b.rotation);
    let (dx, dy) = (cos * delta, sin * delta);
    // Em MUNDO: a roda e a junta moram em pastas (a montagem, o chassi), e o passo é no mundo.
    let to = Transform {
        translation: Vec2::new(f.translation.x + dx, f.translation.y + dy),
        ..f
    };
    let parent = ph2d_ecs::parent_world_transform(sim.world(), fw);
    let Some(local) = Transform::inverse_compose(parent, to) else {
        return;
    };
    let pivot = ph2d_physics_ecs::joint_world_pose(sim.world(), front.0);
    let w = sim.world_mut();
    if let Some(mut t) = w.get_mut::<Transform>(fw) {
        t.translation = local.translation;
    }
    if let Some(p) = pivot {
        let at = [p.translation.x + dx, p.translation.y + dy];
        ph2d_physics_ecs::set_joint_world_pose(w, front.0, at, None);
    }
    if let Some(mut j) = w.get_mut::<PhysicsJoint>(front.0) {
        // A âncora no corpo é medida no referencial dele: o eixo x do corpo É a direcção do passo.
        j.local_a[0] += delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Edit;
    use ph2d_core::Vec2;
    use ph2d_ecs::Name;
    use ph2d_physics_ecs::{BodyKind, ColliderShape, JointAssembly, RigidBody, nasce_com_forma};

    fn body(sim: &mut SimWorld, name: &str, at: [f32; 2], shape: ColliderShape) -> Entity {
        let w = sim.world_mut();
        let e = w
            .spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Name::new(name),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
            ))
            .id();
        let _ = nasce_com_forma(w, e, shape);
        ph2d_ecs::assign_missing_stable_ids(w);
        e
    }

    fn car_parts(sim: &mut SimWorld) {
        body(
            sim,
            "Kart",
            [0.0, 0.55],
            ColliderShape::Cuboid {
                half_x: 1.2,
                half_y: 0.25,
            },
        );
        body(
            sim,
            "Back",
            [-0.7, 0.3],
            ColliderShape::Ball { radius: 0.3 },
        );
        body(
            sim,
            "Front",
            [0.7, 0.3],
            ColliderShape::Ball { radius: 0.3 },
        );
    }

    fn place(sim: &mut SimWorld, assembly: u64, role: u16, name: &str) -> Outcome {
        apply(
            sim,
            Edit::Place {
                assembly,
                template: 1,
                role,
                bodies: vec![name.to_owned()],
            },
        )
    }

    /// ⭐ **A primeira peça faz nascer a montagem; a segunda faz nascer a junta, com o papel certo.**
    ///
    /// Mutações que devem sangrar: não criar a montagem quando `assembly == 0` · passar o papel errado.
    #[test]
    fn largar_o_corpo_e_a_roda_cria_a_montagem_e_a_junta_da_roda() {
        let mut sim = SimWorld::new();
        car_parts(&mut sim);
        let o = place(&mut sim, 0, 0, "Kart");
        let asm = o.made.expect("a montagem nasce com a 1.ª peça");
        let o2 = place(&mut sim, asm, 2, "Front");
        assert!(o2.made.is_none(), "a 2.ª peça não faz outra montagem");
        let joints = joint_assembly::joints_of(sim.world_mut(), asm);
        assert_eq!(joints.len(), 1, "uma junta: corpo + roda da frente");
        assert_eq!(joints[0].1.role, 1, "a regra da roda da frente");
        assert_eq!(joints[0].1.kind, ph2d_physics_ecs::JointKind::Wheel);
    }

    /// ⛔ Uma queda recusada (o nome não é um corpo) não deixa montagem vazia na Hierarchy.
    #[test]
    fn uma_queda_recusada_nao_deixa_montagem_vazia() {
        let mut sim = SimWorld::new();
        car_parts(&mut sim);
        let o = place(&mut sim, 0, 0, "Nobody");
        assert!(o.made.is_none());
        let w = sim.world_mut();
        assert_eq!(w.query::<&JointAssembly>().iter(w).count(), 0);
    }

    /// ⭐ O entre-eixos move a roda da frente e a âncora dela; a de trás fica.
    #[test]
    fn o_entre_eixos_move_a_roda_da_frente_e_a_junta_dela() {
        let mut sim = SimWorld::new();
        car_parts(&mut sim);
        let asm = place(&mut sim, 0, 0, "Kart").made.expect("montagem");
        place(&mut sim, asm, 1, "Back");
        place(&mut sim, asm, 2, "Front");
        let e = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(asm)).expect("e");
        set_wheelbase(&mut sim, e, 2.0);
        let x = |sim: &mut SimWorld, n: &str| {
            let e = body_named(sim, n).expect("peça");
            ph2d_ecs::world_transform(sim.world(), e)
                .expect("t")
                .translation
                .x
        };
        assert!(
            (x(&mut sim, "Front") - 1.3).abs() < 1e-4,
            "a roda da frente foi para 1,3"
        );
        assert!((x(&mut sim, "Back") + 0.7).abs() < 1e-4, "a de trás ficou");
        let joints = joint_assembly::joints_of(sim.world_mut(), asm);
        let front = joints
            .iter()
            .find(|(_, j)| j.role == 1)
            .expect("a da frente");
        assert!(
            (front.1.local_a[0] - 1.3).abs() < 1e-3,
            "a âncora no corpo andou"
        );
    }
}
