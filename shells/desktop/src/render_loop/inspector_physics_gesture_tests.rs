//! **Os gestos COMPOSTOS da §11** — uma sequência de cliques produz uma coisa que
//! funciona?
//!
//! Separado do irmão `inspector_physics_tests` (que pergunta, por EDIT, se o clique
//! escreve o componente certo) quando a pergunta do Enio — *"como eu criaria uma zona
//! de água usando apenas a UI?"* — mostrou que as duas coisas são diferentes: **todo
//! edit pode ter gate e o gesto ainda não levar a lugar nenhum**. Uma row que só
//! aparece depois de outra, um default que atrapalha, um passo que exige um número que
//! o artista não tem como saber — nada disso um teste por-edit enxerga.
//!
//! Por isso o oráculo aqui nunca é "os componentes existem". É a CENA: o sprite está
//! deitado no chão um segundo depois, o corpo estático parou de cair, a caixa que caiu
//! na piscina está mais alta que a idêntica que caiu ao lado.

use super::inspector_physics_tests::{apply, sprite_scene};
use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor::PhysicsFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

use super::inspector_physics::build_physics_info;

/// O snapshot do §11 para `e`, com os fatos que só a shell tem em seus valores
/// NEUTROS (sem joins, sem rig, sem peças, sem gesto armado).
///
/// ⚠️ Existe porque cada chamada deletrava os mesmos sete defaults, e a lista
/// cresce: quando a W-PartFace acrescentou `part_count`, o `fmt` explodiu as
/// dezesseis chamadas em multi-linha e o arquivo passou o cap de 600 LOC
/// (555 → 653) **sem uma linha de teste nova**. Uma porta só, um lugar para o
/// oitavo argumento.
pub(super) fn snapshot(
    sim: &ph2d_ecs::SimWorld,
    e: ph2d_ecs::Entity,
) -> ph2d_editor::InspectorPhysicsInfo {
    build_physics_info(sim.world(), e.to_bits(), 0, 0, 0, false, 0, (0.0, 5.0), 0)
        .expect("§11 aparece para qualquer entidade com Transform")
}

/// **The whole feature, end to end.** Add on a plain sprite, then run the
/// clock: it has to fall and land on the floor.
#[test]
fn adding_a_body_from_the_inspector_makes_the_sprite_fall() {
    let (mut sim, e) = sprite_scene();

    // A floor to land on.
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
    ));

    let before = sim.world().get::<Transform>(e).unwrap().translation.y;
    apply(&mut sim, e, PhysicsFieldEdit::Add);

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let after = sim.world().get::<Transform>(e).unwrap().translation.y;
    assert!(
        after < before - 1.0,
        "the sprite never fell (y {before} -> {after}) — Add Physics Body reached the ECS but \
         the entity is not being simulated"
    );
    // Half-height 0.5 (the sprite is 1 m tall) resting on a floor whose top
    // is at y = 0.1.
    assert!(
        (after - 0.6).abs() < 0.15,
        "the sprite settled at y={after}, expected ~0.6 (floor top 0.1 + half height 0.5)"
    );
}

/// A body kind flip is a real change in the simulation, not just a tag: a
/// Static body must stop falling.
#[test]
fn making_a_body_static_stops_it_falling() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Kind(1)); // Static

    let before = sim.world().get::<Transform>(e).unwrap().translation.y;
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let after = sim.world().get::<Transform>(e).unwrap().translation.y;
    assert_eq!(
        after, before,
        "a Static body moved — the kind edit reached the component but not the solver"
    );
}

/// **A sequência do sinal de saída LEVA a algum lugar** (W-SignalLeave) — a
/// quarta condição de UI do módulo, aquela que as outras três não implicam.
///
/// ⚠️ **O que este gate cobre e o de seam não:** o seam prova que digitar na row
/// emite a ação; este prova que os BYTES que o commit escreve viram um componente
/// que o publicador lê. O trecho entre os dois é uma codificação postcard passada
/// pelo REGISTRO, e é exatamente ali que mora a armadilha que esta wave recusou:
/// `SignalOnHit`/`SignalOnLeave` são newtypes de `String` e hoje codificam igual,
/// então serializar a string e chamá-la de componente passaria HOJE e escreveria
/// lixo bem-formado no dia em que um dos dois ganhasse um campo.
///
/// O oráculo é a CENA: a porta abre quando o andarilho entra e fecha quando ele
/// sai — os dois nomes, na ordem, depois de a autoria ter passado pelo mesmo
/// caminho que o clique do artista usa.
#[test]
fn authoring_both_signal_names_makes_the_door_open_and_close() {
    use ph2d_ecs::scene::{
        ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
    };
    use ph2d_physics_ecs::{GravityScale, InitialVelocity, SignalOnHit, SignalOnLeave};

    let mut registry = ComponentRegistry::new();
    ph2d_physics_ecs::register_physics_components(&mut registry);

    let mut sim = SimWorld::new();
    let door = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        GravityScale(0.0),
        InitialVelocity {
            linvel: [4.0, 0.0],
            angvel: 0.0,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.1,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-3.0, 0.0)),
    ));

    // A autoria, pelo MESMO caminho do commit: cada tipo codificado pelo
    // `Serialize` DELE, e o `type_id` vindo do registro.
    let queue = EditorCommandQueue::new();
    for (data, type_name) in [
        (
            postcard::to_allocvec(&SignalOnHit("open".to_string())).expect("encode"),
            "ph2d::physics::SignalOnHit",
        ),
        (
            postcard::to_allocvec(&SignalOnLeave("close".to_string())).expect("encode"),
            "ph2d::physics::SignalOnLeave",
        ),
    ] {
        let entry = registry
            .get_by_name(type_name)
            .unwrap_or_else(|| panic!("{type_name} nao esta' registrado"));
        queue
            .push(EditorCommand::SetComponent {
                entity: door.to_bits(),
                type_id: entry.type_id,
                data,
            })
            .expect("queue");
    }
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("apply");

    let mut bridge = PhysicsBridge::new();
    let mut names = Vec::new();
    for t in 0..=180 {
        bridge.dispatch(&mut sim, true, t);
        for s in bridge.signal_events(&sim) {
            names.push(s.name);
        }
    }
    assert_eq!(
        names,
        vec!["open".to_string(), "close".to_string()],
        "autorar os dois nomes pelo caminho do commit tem de abrir E fechar a porta"
    );
}
