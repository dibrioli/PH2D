//! A fase no mundo: parte do autorado, nunca grava, avisa o ciclo, devolve ao apagar.

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Canais, ChildOf as Filho, CopyTransform, Kind, LookAt};
use ph2d_constraints_ecs::{
    Constraints, Estado, Estados, bind_on_create, evaluate, register_constraint_components,
};
use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{
    ChildOf, Name, SimWorld, StableId, Transform, TransformPropagationState, WorklistBuf,
    world_transform,
};
use ph2d_physics_ecs::{BodyKind, RigidBody};
use ph2d_preview_drive::PreviewDrive;

fn em(x: f32, y: f32, rot_deg: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot_deg.to_radians();
    t
}

fn copia_local(alvo: u64, influencia: f64) -> Constraints {
    let mut c = Constraints::default();
    let id = c.push(
        Kind::CopyTransform(CopyTransform {
            canais: Canais::LOCATION,
            ..Default::default()
        }),
        Some(StableId(alvo)),
    );
    c.item_mut(id).unwrap().influence = influencia;
    c
}

fn x_de(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

#[test]
fn a_half_influence_does_not_creep_frame_after_frame() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    sim.world_mut().spawn((StableId(1), em(10.0, 0.0, 0.0)));
    let dono = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, copia_local(1, 0.5)))
        .id();
    for _ in 0..10 {
        evaluate(&mut sim, &mut drive);
        drive.settle();
        assert_eq!(x_de(&sim, dono), 5.0);
    }
}

#[test]
fn a_cycle_is_reported_and_never_evaluated() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let a = sim
        .world_mut()
        .spawn((StableId(1), em(3.0, 0.0, 0.0), copia_local(2, 1.0)))
        .id();
    let b = sim
        .world_mut()
        .spawn((StableId(2), em(7.0, 0.0, 0.0), copia_local(1, 1.0)))
        .id();
    evaluate(&mut sim, &mut drive);
    let est = sim.world().resource::<Estados>();
    assert_eq!(est.of(a, 0), Some(Estado::Ciclo));
    assert_eq!(est.of(b, 0), Some(Estado::Ciclo));
    assert_eq!((x_de(&sim, a), x_de(&sim, b)), (3.0, 7.0));
}

#[test]
fn a_constraint_never_writes_the_document() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    sim.world_mut().spawn((StableId(1), em(10.0, 4.0, 0.0)));
    let dono = sim
        .world_mut()
        .spawn((StableId(2), em(1.0, 1.0, 0.0), copia_local(1, 1.0)))
        .id();
    evaluate(&mut sim, &mut drive);
    assert_eq!(x_de(&sim, dono), 10.0);
    let vivo = drive.substitute(&mut sim);
    assert_eq!(x_de(&sim, dono), 1.0, "a fotografia vê o autorado");
    PreviewDrive::restore(&mut sim, &vivo);
    assert_eq!(x_de(&sim, dono), 10.0);
}

#[test]
fn removing_the_stack_gives_back_the_authored_pose() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    sim.world_mut().spawn((StableId(1), em(10.0, 0.0, 0.0)));
    let dono = sim
        .world_mut()
        .spawn((StableId(2), em(2.0, 0.0, 0.0), copia_local(1, 1.0)))
        .id();
    evaluate(&mut sim, &mut drive);
    assert_eq!(x_de(&sim, dono), 10.0);
    sim.world_mut().entity_mut(dono).remove::<Constraints>();
    evaluate(&mut sim, &mut drive);
    assert_eq!(x_de(&sim, dono), 2.0);
    sim.world_mut().entity_mut(dono).insert(copia_local(1, 1.0));
    evaluate(&mut sim, &mut drive);
    sim.world_mut().get_mut::<Constraints>(dono).unwrap().items[0].enabled = false;
    evaluate(&mut sim, &mut drive);
    assert_eq!(x_de(&sim, dono), 2.0, "desligar também devolve");
}

#[test]
fn a_dynamic_body_refuses_the_constraint() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    sim.world_mut().spawn((StableId(1), em(10.0, 0.0, 0.0)));
    let corpo = RigidBody {
        kind: BodyKind::Dynamic,
    };
    let dono = sim
        .world_mut()
        .spawn((StableId(2), em(2.0, 0.0, 0.0), corpo, copia_local(1, 1.0)))
        .id();
    evaluate(&mut sim, &mut drive);
    assert_eq!(x_de(&sim, dono), 2.0);
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, 0),
        Some(Estado::CorpoDinamico)
    );
}

#[test]
fn child_of_never_jumps_when_created_inside_a_hierarchy() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mut avo_t = em(1.0, -2.0, -20.0);
    avo_t.scale.x = 1.2;
    avo_t.scale.y = 1.2;
    let avo = sim.world_mut().spawn((StableId(9), avo_t)).id();
    let alvo = sim
        .world_mut()
        .spawn((StableId(1), em(5.0, 0.0, 37.0), ChildOf(avo)))
        .id();
    let pai_do_dono = sim
        .world_mut()
        .spawn((StableId(8), em(-3.0, 2.0, 15.0)))
        .id();
    let dono = sim
        .world_mut()
        .spawn((StableId(2), em(2.0, 1.0, 30.0), ChildOf(pai_do_dono)))
        .id();
    let antes = world_transform(sim.world(), dono).unwrap();
    let mut kind = Kind::ChildOf(Filho::default());
    bind_on_create(&sim, &drive, dono, &mut kind, Some(alvo)).unwrap();
    let mut c = Constraints::default();
    c.push(kind, Some(StableId(1)));
    sim.world_mut().entity_mut(dono).insert(c);
    evaluate(&mut sim, &mut drive);
    let depois = world_transform(sim.world(), dono).unwrap();
    assert!(
        (depois.translation - antes.translation).length() < 1e-4,
        "{antes:?}\n{depois:?}"
    );
    assert!((depois.rotation - antes.rotation).abs() < 1e-5);
    sim.world_mut()
        .get_mut::<Transform>(avo)
        .unwrap()
        .translation
        .x += 3.0;
    evaluate(&mut sim, &mut drive);
    let seguiu = world_transform(sim.world(), dono).unwrap();
    assert!(
        (seguiu.translation.x - antes.translation.x - 3.0).abs() < 1e-4,
        "segue o avô do alvo"
    );
}

#[test]
fn whoever_reads_another_runs_after_it() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    sim.world_mut().spawn((StableId(1), em(10.0, 0.0, 0.0)));
    // C copia B, B copia A — C spawnado ANTES de B, para que a ordem por bits estivesse errada.
    let c = sim
        .world_mut()
        .spawn((StableId(3), Transform::IDENTITY, copia_local(2, 1.0)))
        .id();
    let b = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, copia_local(1, 1.0)))
        .id();
    evaluate(&mut sim, &mut drive);
    assert_eq!((x_de(&sim, b), x_de(&sim, c)), (10.0, 10.0));
}

#[test]
fn look_at_without_a_live_target_says_so() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mut c = Constraints::default();
    c.push(Kind::LookAt(LookAt::default()), Some(StableId(77)));
    let dono = sim
        .world_mut()
        .spawn((StableId(2), em(0.0, 0.0, 12.0), c))
        .id();
    evaluate(&mut sim, &mut drive);
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, 0),
        Some(Estado::SemAlvo)
    );
}

#[test]
fn the_stack_survives_snapshot_and_restore() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_constraint_components(&mut reg);
    let pilha = copia_local(1, 0.25);
    let mut a = SimWorld::new();
    a.world_mut()
        .spawn((Name::new("dono"), Transform::IDENTITY, pilha.clone()));
    let mut state = TransformPropagationState::new(a.world_mut());
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(
        a.world_mut(),
        &mut state,
        &mut WorklistBuf::new(),
        &reg,
        &mut snap,
    )
    .unwrap();
    let mut b = SimWorld::new();
    let back = snapshot_to_world(b.world_mut(), &snap, &reg).unwrap();
    let dono = back
        .iter()
        .copied()
        .find(|&e| {
            b.world()
                .get::<Name>(e)
                .is_some_and(|n| n.as_str() == "dono")
        })
        .unwrap();
    assert_eq!(b.world().get::<Constraints>(dono), Some(&pilha));
}
