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
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor::PhysicsFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};
use ph2d_render::Sprite;

use super::inspector_physics::build_physics_info;

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

/// **Uma zona de ÁGUA é autorável só com gestos da UI** — o caminho de cliques inteiro,
/// do sprite pelado ao objeto que boia.
///
/// Enio perguntou, olhando o smoke `=26`: *"como eu conseguiria criar uma zona de água
/// como você fez usando apenas a UI?"*. Cada edit já tinha gate individual, mas o
/// **gesto composto** não tinha nenhum — e é ele que responde a pergunta. Um controle
/// pode estar vivo e a sequência ainda não levar a lugar nenhum (uma row que só aparece
/// depois de outra, um default que atrapalha, um passo que exige digitar um número que
/// o artista não tem como saber).
///
/// Por isso o oráculo não é "os componentes existem": é **a caixa que caiu na piscina
/// está mais alta que a idêntica que caiu ao lado**, depois de 3 s de relógio.
#[test]
fn a_water_zone_is_authorable_with_ui_gestures_alone() {
    let mut sim = SimWorld::new();
    // O artista posiciona um retângulo de 2 x 3 m (um sprite importado, que também é o
    // que dá à piscina a cor translúcida que ela deve ter).
    let pool = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            Sprite::atlas(0, [2.0, 3.0], [0.3, 0.5, 0.9, 0.3]),
        ))
        .id();
    assert!(
        !build_physics_info(sim.world(), pool.to_bits(), false, 5.0, 0)
            .expect("§11 aparece para qualquer entidade com Transform")
            .has_body,
        "num sprite pelado a seção mostra a face VAZIA — é ela que oferece o Add"
    );

    // 1. Add Physics Body. ⚠️ O collider nasce CASADO com o sprite (1.00 x 1.50 = metade
    //    de 2 x 3), então o artista não digita dimensão nenhuma para ter a piscina do
    //    tamanho que desenhou.
    apply(&mut sim, pool, PhysicsFieldEdit::Add);
    let i = build_physics_info(sim.world(), pool.to_bits(), false, 5.0, 0).unwrap();
    assert_eq!(
        (i.half_x, i.half_y),
        (1.0, 1.5),
        "o collider tem de nascer com a caixa do sprite, senão o passo 1 já exige números"
    );

    // 2. Kind = Static (uma piscina não cai). 3. Trigger = Sensor (dá para entrar nela).
    apply(&mut sim, pool, PhysicsFieldEdit::Kind(1));
    apply(&mut sim, pool, PhysicsFieldEdit::Sensor(true));
    let i = build_physics_info(sim.world(), pool.to_bits(), false, 5.0, 0).unwrap();
    assert!(
        i.is_sensor && i.kind_tag == 1,
        "é o Sensor que faz as rows Force/Drag serem oferecidas — sem ele o passo 4 não existe"
    );

    // 4. Force Y (o empuxo). 5. Drag (o meio).
    apply(&mut sim, pool, PhysicsFieldEdit::ForceY(3.5));
    apply(&mut sim, pool, PhysicsFieldEdit::AreaDrag(4.0));
    let i = build_physics_info(sim.world(), pool.to_bits(), false, 5.0, 0).unwrap();
    assert_eq!((i.force, i.area_drag), ([0.0, 3.5], 4.0));

    // E agora o oráculo: duas caixas idênticas, uma cai na piscina, a outra ao lado.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -2.5)),
    ));
    let drop_at = |sim: &mut SimWorld, x: f32| {
        sim.world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, 3.0)),
            ))
            .id()
    };
    let wet = drop_at(&mut sim, 0.0);
    let dry = drop_at(&mut sim, 5.0);

    let mut bridge = PhysicsBridge::new();
    for t in 1..=180 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = |sim: &SimWorld, e: Entity| sim.world().get::<Transform>(e).unwrap().translation.y;
    let (in_pool, in_air) = (y(&sim, wet), y(&sim, dry));
    assert!(
        in_pool > in_air + 1.0,
        "depois de 3 s a caixa na piscina tem de estar visivelmente mais alta que a \
         idêntica que caiu ao lado ({in_pool} vs {in_air}) — sem isso os cinco gestos \
         acima produziram uma piscina que não é água"
    );
}

/// **Uma poça com EMPUXO é autorável só com gestos da UI** — e o passo que a torna
/// água de verdade é um número só (W-Buoyancy).
///
/// O irmão acima (`a_water_zone_is_authorable_with_ui_gestures_alone`) monta a versão
/// do W-Area: `Force Y` constante. Este monta a de Arquimedes, e o oráculo é o que
/// **distingue as duas** — o corpo leve tem de **PARAR** perto da superfície em vez de
/// ser arremessado para fora, que era o defeito nomeado como aberto no W-AreaDrag.
#[test]
fn a_buoyant_pool_is_authorable_with_ui_gestures_alone() {
    let pool = {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, -1.5)),
                Sprite::atlas(0, [7.0, 3.0], [0.3, 0.5, 0.9, 0.3]),
            ))
            .id();
        (sim, e)
    };
    let (mut sim, pool) = pool;

    // Os mesmos quatro primeiros gestos do irmão — Add, Static, Sensor — e então UM
    // número: a densidade do fluido. Nenhuma força a tunar por objeto.
    apply(&mut sim, pool, PhysicsFieldEdit::Add);
    apply(&mut sim, pool, PhysicsFieldEdit::Kind(1));
    apply(&mut sim, pool, PhysicsFieldEdit::Sensor(true));
    apply(&mut sim, pool, PhysicsFieldEdit::AreaDensity(4.0));
    apply(&mut sim, pool, PhysicsFieldEdit::AreaDrag(1.5));
    let i = build_physics_info(sim.world(), pool.to_bits(), false, 5.0, 0).unwrap();
    assert_eq!(
        (i.area_density, i.area_drag),
        (4.0, 1.5),
        "as rows de área têm de aceitar os dois números num sensor"
    );

    // Chão, e duas caixas de MESMO tamanho e densidades diferentes largadas na poça.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -3.5)),
    ));
    let drop = |sim: &mut SimWorld, x: f32, density: f32| {
        sim.world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.25,
                        half_y: 0.25,
                    },
                    density,
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, 2.0)),
            ))
            .id()
    };
    let cork = drop(&mut sim, -2.0, 1.0);
    let stone = drop(&mut sim, 2.0, 12.0);

    let mut bridge = PhysicsBridge::new();
    for t in 1..=420 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = |sim: &SimWorld, e: Entity| sim.world().get::<Transform>(e).unwrap().translation.y;
    let (up, down) = (y(&sim, cork), y(&sim, stone));

    assert!(
        down < -2.0,
        "a caixa 3x mais densa que o fluido tem de ir ao fundo ({down})"
    );
    // ⚠️ O oráculo que separa Arquimedes de uma força constante: a cortiça PARA perto da
    // superfície (y = 0). Uma `Force Y` forte o bastante para levantá-la a lançaria para
    // fora da poça e ela continuaria subindo — é justamente o defeito que esta wave veio
    // fechar, e é por isso que o teto faz parte da asserção.
    assert!(
        up.abs() < 0.5,
        "a cortiça tem de PARAR na linha d'água, e não ser arremessada nem afundar ({up})"
    );
}
