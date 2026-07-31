//! **O gerador de rig — a metade de shell** (W-Rig).
//!
//! O kernel já prova a TOPOLOGIA (`ph2d-physics-ecs/tests/rig.rs`). O que se
//! prova aqui é o que só o shell sabe: quem vira corpo, o que ele **não** pode
//! reescrever, e que clicar duas vezes não faz dois rigs.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::ChildOf;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}

/// Um nó com desenho — uma PARTE em potencial.
fn part(sim: &mut SimWorld, name: &str, y: f32, parent: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Name::new(name),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 0.4], [1.0, 1.0, 1.0, 1.0]),
            Transform::from_translation(Vec2::new(0.0, y)),
        ))
        .id();
    if let Some(p) = parent {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// Um nó SEM desenho — organização, não osso.
fn group(sim: &mut SimWorld, name: &str, parent: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Name::new(name),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    if let Some(p) = parent {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// Roda o gerador pela porta real e drena a fila, como o laço da shell faz.
fn rig(sim: &mut SimWorld, root: Entity) -> (usize, usize) {
    let reg = registry();
    let queue = EditorCommandQueue::default();
    let p = plan(sim, &[root.to_bits()]);
    let (bodies, joints, _) = apply(sim, &p, JointKind::Pin, &queue, &reg);
    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commit");
    (bodies, joints)
}

fn joint_pairs(sim: &mut SimWorld) -> Vec<(u64, u64)> {
    let mut q = sim.world_mut().query::<&PhysicsJoint>();
    let mut v: Vec<(u64, u64)> = q.iter(sim.world()).map(|j| (j.body_a, j.body_b)).collect();
    v.sort_unstable();
    v
}

/// Um boneco: tronco com cabeça e dois braços — a ÁRVORE que uma corrente não
/// expressa.
fn doll() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let torso = part(&mut sim, "Torso", 0.0, None);
    part(&mut sim, "Head", 0.5, Some(torso));
    part(&mut sim, "ArmL", -0.2, Some(torso));
    part(&mut sim, "ArmR", -0.2, Some(torso));
    (sim, torso)
}

/// **Um clique faz o rig inteiro** — quatro corpos e três joints, sobre uma
/// árvore que ramifica.
#[test]
fn rigging_a_branching_doll_makes_every_body_and_every_joint() {
    let (mut sim, torso) = doll();
    let (bodies, joints) = rig(&mut sim, torso);
    assert_eq!(bodies, 4, "cada parte tem de virar corpo");
    assert_eq!(joints, 3, "três filhos do tronco, três joints");
    for name in ["Torso", "Head", "ArmL", "ArmR"] {
        let e = named(&mut sim, name);
        assert!(
            sim.world().get::<RigidBody>(e).is_some(),
            "{name} ficou sem corpo"
        );
    }
}

/// **O collider sai da CAIXA DO SPRITE**, pela porta da §11 — não de uma segunda
/// regra que este arquivo inventasse.
#[test]
fn the_collider_of_a_generated_body_matches_the_sprite_box() {
    let (mut sim, torso) = doll();
    rig(&mut sim, torso);
    let head = named(&mut sim, "Head");
    let col = sim.world().get::<Collider>(head).expect("collider");
    match col.shape {
        ColliderShape::Cuboid { half_x, half_y } => {
            assert!((half_x - 0.2).abs() < 1e-4, "half_x = {half_x}");
            assert!((half_y - 0.2).abs() < 1e-4, "half_y = {half_y}");
        }
        other => panic!("o gerador não usou a caixa do sprite: {other:?}"),
    }
}

/// **Uma parte que JÁ tem corpo mantém o que foi autorado.** O `Add` reescreve o
/// `Collider` a partir do sprite, então passá-lo por cima apagaria a forma que o
/// artista escolheu — o gerador desfazendo trabalho no clique que acrescenta.
#[test]
fn a_part_that_already_has_a_body_keeps_its_authored_collider() {
    let (mut sim, torso) = doll();
    let head = named(&mut sim, "Head");
    sim.world_mut().entity_mut(head).insert((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.75 },
            ..Collider::default()
        },
    ));

    let (bodies, joints) = rig(&mut sim, torso);
    assert_eq!(bodies, 3, "a cabeça já era corpo e foi contada de novo");
    assert_eq!(joints, 3);

    let col = sim.world().get::<Collider>(head).expect("collider");
    assert!(
        matches!(col.shape, ColliderShape::Ball { radius } if (radius - 0.75).abs() < 1e-4),
        "a forma autorada foi reescrita pela caixa do sprite: {:?}",
        col.shape
    );
    assert_eq!(
        sim.world().get::<RigidBody>(head).expect("body").kind,
        BodyKind::Static,
        "o kind autorado foi reescrito"
    );
}

/// **Um grupo é TRANSPARENTE e não vira osso**: sem sprite ele não ganha corpo, e
/// o neto se liga ao avô. Dar-lhe corpo plantaria um collider invisível de meio
/// metro no meio do personagem (o fallback do `Add` para entidade sem sprite).
#[test]
fn a_group_gets_no_body_and_the_grandchild_joins_the_grandparent() {
    let mut sim = SimWorld::new();
    let torso = part(&mut sim, "Torso", 0.0, None);
    let g = group(&mut sim, "ArmsGroup", Some(torso));
    part(&mut sim, "ArmL", -0.2, Some(g));

    let (bodies, joints) = rig(&mut sim, torso);
    assert_eq!(bodies, 2, "o grupo ganhou corpo");
    assert_eq!(joints, 1);
    assert!(
        sim.world().get::<RigidBody>(g).is_none(),
        "o grupo virou osso"
    );

    let pairs = joint_pairs(&mut sim);
    assert_eq!(
        pairs,
        vec![(stable_name_id("Torso"), stable_name_id("ArmL"))],
        "o joint não pulou o grupo"
    );
}

/// **O PAI é o lado A.** O filho pende do pai, e é o lado A que o pivô segue
/// (W-AnchorFollow) — invertido, o dot de um braço seguiria a mão.
#[test]
fn the_parent_is_body_a() {
    let (mut sim, torso) = doll();
    rig(&mut sim, torso);
    let t = stable_name_id("Torso");
    for (a, _b) in joint_pairs(&mut sim) {
        assert_eq!(a, t, "um joint nasceu com o filho no lado A");
    }
}

/// **Clicar duas vezes não faz dois rigs.** Um par já ligado é pulado — sem isso
/// o solver ganharia duas restrições sobre o mesmo par, invisíveis fora de uma
/// segunda linha na Hierarquia.
#[test]
fn rigging_twice_makes_no_second_joint() {
    let (mut sim, torso) = doll();
    rig(&mut sim, torso);
    let first = joint_pairs(&mut sim);

    let (bodies, joints) = rig(&mut sim, torso);
    assert_eq!(bodies, 0, "o segundo clique criou corpos");
    assert_eq!(joints, 0, "o segundo clique criou joints");
    assert_eq!(joint_pairs(&mut sim), first);
}

/// **E re-rigar depois de acrescentar um membro liga SÓ o membro novo** — é o que
/// torna o gerador uma ferramenta em vez de um gesto de uma vez só.
#[test]
fn re_rigging_after_a_new_limb_joints_only_the_new_one() {
    let (mut sim, torso) = doll();
    rig(&mut sim, torso);

    part(&mut sim, "Tail", -0.6, Some(torso));
    let (bodies, joints) = rig(&mut sim, torso);
    assert_eq!(bodies, 1, "só a cauda precisava de corpo");
    assert_eq!(joints, 1, "só a cauda precisava de joint");
    assert_eq!(joint_pairs(&mut sim).len(), 4);
}

/// **Sem aresta, sem botão.** Dois irmãos selecionados não têm ancestral um do
/// outro, então o rig não faria nada — e um botão que não faz nada é pior que
/// botão nenhum.
#[test]
fn the_plan_is_not_offered_when_there_is_no_edge_to_make() {
    let (mut sim, torso) = doll();
    let arm_l = named(&mut sim, "ArmL");
    let arm_r = named(&mut sim, "ArmR");

    let p = plan(&mut sim, &[arm_l.to_bits(), arm_r.to_bits()]);
    assert!(!p.is_offered(), "o botão apareceria sobre dois irmãos");

    let p = plan(&mut sim, &[torso.to_bits()]);
    assert!(p.is_offered(), "o botão sumiu sobre um personagem inteiro");
    assert_eq!(p.parts.len(), 4);
    assert_eq!(p.edges.len(), 3);
}

/// Um objeto fora da subárvore não entra no rig — a expansão é do que foi
/// selecionado, não da cena.
#[test]
fn a_prop_beside_the_character_is_left_alone() {
    let (mut sim, torso) = doll();
    let prop = part(&mut sim, "Crate", -3.0, None);
    rig(&mut sim, torso);
    assert!(
        sim.world().get::<RigidBody>(prop).is_none(),
        "o gerador rigou um objeto que não estava na subárvore"
    );
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}
