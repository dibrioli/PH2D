//! **A montagem é a PASTA das peças** (W4h/W4i, dono 15/11): onde mora cada peça e cada junta,
//! o que apagar, mover, desmontar, copiar e trancar a pasta faz ao conjunto.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_joint_templates::{JointKind, by_name};
use ph2d_physics_ecs::{PhysicsJoint, RigidBody};

use super::{built, ground, part, parts_of, pose};
use ph2d_joint_assembly::joint_assembly::{assemble, joints_of};

fn parent(sim: &SimWorld, e: Entity) -> Option<Entity> {
    sim.world()
        .get::<ph2d_ecs::ChildOf>(e)
        .map(ph2d_ecs::ChildOf::parent)
}

fn by_sid(sim: &SimWorld, id: u64) -> Entity {
    ph2d_ecs::entity_with_stable_id(sim.world(), ph2d_ecs::StableId(id)).expect("by id")
}

/// ⭐ **A montagem é a PASTA do conjunto** (dono 15/11): as peças moram dentro de «Car 1», cada
/// junta dentro do corpo A dela, e nada saiu do lugar ao entrar na pasta.
/// **Mutações que sangram:** `place_part` sem o `adopt` · `joint_home` a escolher o corpo B ·
/// `adopt_keeping_world` sem o `inverse_compose`.
#[test]
fn the_assembly_is_the_folder_of_its_parts_and_each_joint_lives_in_body_a() {
    let car = by_name(JointKind::Wheel, "Car").expect("car");
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let parts = parts_of(&mut sim, car, false);
    let at: Vec<Transform> = parts
        .iter()
        .map(|(_, e)| pose(&sim, e.expect("part")))
        .collect();
    let made = assemble(&mut sim, car, &parts).expect("assembles");
    for ((_, e), was) in parts.iter().zip(&at) {
        let e = e.expect("part");
        assert_eq!(
            parent(&sim, e),
            Some(made.assembly),
            "the part is inside the assembly"
        );
        let now = pose(&sim, e);
        assert!(
            (now.translation - was.translation).length() < 1e-5,
            "entering the folder moved nothing: {was:?} → {now:?}"
        );
    }
    assert_eq!(made.joints.len(), 2);
    for &j in &made.joints {
        let pj = *sim.world().get::<PhysicsJoint>(j).expect("joint");
        assert_eq!(
            parent(&sim, j),
            Some(by_sid(&sim, pj.body_a)),
            "the joint lives in body A"
        );
    }
}

/// **Apagar a montagem apaga o carro inteiro** (dono 15/11) — a pasta leva as peças, e as juntas
/// vão dentro delas.
#[test]
fn deleting_the_assembly_deletes_the_whole_machine() {
    let (mut sim, _parts, made) = built(JointKind::Wheel, "Car", false);
    sim.world_mut().despawn(made.assembly);
    let w = sim.world_mut();
    assert_eq!(
        w.query::<&PhysicsJoint>().iter(w).count(),
        0,
        "no joint left"
    );
    assert_eq!(
        w.query::<&RigidBody>().iter(w).count(),
        1,
        "only the ground is left"
    );
}

/// **Tirar uma peça da pasta não desfaz as juntas** (dono 15/11): a junta vai junto com o corpo A,
/// e a montagem continua a contá-la.
#[test]
fn moving_a_part_out_of_the_folder_keeps_its_joints() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    assert!(ph2d_physics_ecs::adopt_keeping_world(
        sim.world_mut(),
        chassis,
        None
    ));
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
        .expect("id")
        .0;
    let left = joints_of(sim.world_mut(), id);
    assert_eq!(left.len(), 2, "both wheels still hang from the chassis");
    for (j, _) in left {
        assert_eq!(parent(&sim, j), Some(chassis));
    }
}

/// **Mover a pasta leva o conjunto inteiro, e as juntas seguem as peças** — o pivô de cada junta
/// continua no ponto do corpo A, lido pela porta de MUNDO.
#[test]
fn moving_the_folder_moves_the_machine_and_its_pivots() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    let before: Vec<[f32; 2]> = made
        .joints
        .iter()
        .map(|&j| {
            let t = ph2d_physics_ecs::joint_world_pose(sim.world(), j).expect("pose");
            [t.translation.x, t.translation.y]
        })
        .collect();
    let was = pose(&sim, chassis).translation;
    sim.world_mut()
        .get_mut::<Transform>(made.assembly)
        .expect("folder")
        .translation += Vec2::new(5.0, 2.0);
    let moved = pose(&sim, chassis).translation - was;
    assert!(
        (moved - Vec2::new(5.0, 2.0)).length() < 1e-5,
        "the chassis moved with the folder"
    );
    for (&j, b) in made.joints.iter().zip(&before) {
        let t = ph2d_physics_ecs::joint_world_pose(sim.world(), j).expect("pose");
        assert!((t.translation.x - b[0] - 5.0).abs() < 1e-5);
        assert!((t.translation.y - b[1] - 2.0).abs() < 1e-5);
    }
}

/// **Toda junta de todo gabarito mora na casa certa**: dentro do corpo A, ou — num pino de
/// cenário, cujo prego não pode andar com a peça — dentro da pasta da montagem.
#[test]
fn every_joint_of_every_template_lives_in_its_home() {
    let mut pins = 0;
    for t in ph2d_joint_templates::ALL {
        for scenery_is_world in [false, true] {
            let label = format!("{} (world: {scenery_is_world})", t.name);
            let mut sim = SimWorld::new();
            let parts = parts_of(&mut sim, t, scenery_is_world);
            let made = assemble(&mut sim, t, &parts).unwrap_or_else(|e| panic!("{label}: {e:?}"));
            let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
                .expect("id")
                .0;
            for (j, pj) in joints_of(sim.world_mut(), id) {
                let pin = pj.body_b == 0
                    || sim
                        .world()
                        .get::<ph2d_physics_ecs::JointWorldAnchor>(j)
                        .is_some();
                let home = if pin {
                    made.assembly
                } else {
                    by_sid(&sim, pj.body_a)
                };
                pins += usize::from(pin);
                assert_eq!(parent(&sim, j), Some(home), "{label}: home of a joint");
            }
        }
    }
    assert!(pins > 0, "the catalogue has scenery pins to test");
}

/// **Desmontar devolve as peças à raiz, no mesmo lugar** — a aba «Remove» não apaga o que o
/// artista desenhou.
#[test]
fn dissolving_puts_the_parts_back_at_the_root_where_they_were() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let at: Vec<Transform> = parts
        .iter()
        .map(|(_, e)| pose(&sim, e.expect("p")))
        .collect();
    ph2d_joint_assembly::joint_assembly::dissolve(&mut sim, made.assembly);
    for ((_, e), was) in parts.iter().zip(&at) {
        let e = e.expect("p");
        assert_eq!(parent(&sim, e), None, "back at the root");
        assert!((pose(&sim, e).translation - was.translation).length() < 1e-5);
    }
    let w = sim.world_mut();
    assert_eq!(w.query::<&PhysicsJoint>().iter(w).count(), 0);
}

/// **Duplicar só o chassi não engorda a montagem** — as juntas filhas vão na cópia, presas às rodas
/// de antes (a lei da cópia: o que ficou de fora continua o mesmo alvo), mas como juntas AVULSAS:
/// «Car 1» continua com as duas dela.
#[test]
fn duplicating_only_the_chassis_leaves_the_assembly_with_its_own_joints() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    let reg = super::super::test_registry();
    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &reg, chassis, Some(made.assembly))
        .expect("copy");
    ph2d_physics_ecs::remap_joint_refs(sim.world_mut(), &copy.copies(), &copy.stable_ids);
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
        .expect("id")
        .0;
    assert_eq!(
        joints_of(sim.world_mut(), id).len(),
        2,
        "the assembly keeps its two"
    );
    let new_id = ph2d_ecs::stable_id_of(sim.world(), copy.root)
        .expect("id")
        .0;
    let w = sim.world_mut();
    let copied: Vec<PhysicsJoint> = w
        .query::<&PhysicsJoint>()
        .iter(w)
        .filter(|j| j.body_a == new_id)
        .copied()
        .collect();
    assert_eq!(copied.len(), 2, "the copy carries its child joints");
    assert!(copied.iter().all(|j| j.assembly == 0), "as loose joints");
}

/// ⭐ **O cadeado de «Car 1» tranca o carro** (dono 15/11) — a pasta não tem arte própria, então o
/// cadeado dela vale para o conteúdo; destrancada, as peças voltam a mexer-se. A marca vem do
/// próprio `JointAssembly` (`#[require]`), sem nada gravado a mais.
/// **Mutações que sangram:** tirar o `#[require]` · `locked_by_a_folder` sem o `Locked`.
#[test]
fn locking_the_assembly_locks_the_whole_machine() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    let joint = made.joints[0];
    assert!(
        sim.world()
            .get::<ph2d_ecs::LockCoversChildren>(made.assembly)
            .is_some()
    );
    assert!(
        !ph2d_ecs::is_locked_for_edit(sim.world(), chassis),
        "unlocked moves"
    );
    assert_eq!(ph2d_ecs::pick_target(sim.world(), chassis), Some(chassis));
    sim.world_mut()
        .entity_mut(made.assembly)
        .insert(ph2d_ecs::Locked);
    for e in [chassis, joint] {
        assert!(
            ph2d_ecs::is_locked_for_edit(sim.world(), e),
            "the lock covers what is inside"
        );
    }
    assert_eq!(
        ph2d_ecs::pick_target(sim.world(), chassis),
        None,
        "a click cannot take it"
    );
    // Um objecto COM arte continua com a regra de 26/05: o cadeado trava só ele (o vazio é pasta).
    let w = sim.world_mut();
    let plain = w
        .spawn((
            Transform::IDENTITY,
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ph2d_ecs::Locked,
        ))
        .id();
    let kid = w
        .spawn((Transform::IDENTITY, ph2d_ecs::ChildOf(plain)))
        .id();
    assert!(!ph2d_ecs::is_locked_for_edit(sim.world(), kid));
}
