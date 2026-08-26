//! **Os gestos compostos de ZONA** — irmão do [`super::inspector_physics_gesture_tests`],
//! separado dele em W-SignalLeave pelo cap de 600 LOC da shell.
//!
//! ⚠️ **O corte é por ASSUNTO, não por tamanho**, e é o MESMO que esta linha já
//! desenhou em `components/area.rs` e `inspector_physics_area.rs`: *o que este
//! CORPO é* contra *o que esta ÁREA faz a outros*. Os cinco gates aqui perguntam
//! todos a mesma coisa — **uma zona é autorável só com gestos da UI?** — e o pai
//! fica com o corpo e o que ele GRITA.
//!
//! O oráculo é o mesmo em toda parte e nunca é "os componentes existem": é a
//! CENA (a caixa que caiu na piscina está mais alta que a idêntica ao lado).

use super::inspector_physics_tests::apply;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor::PhysicsFieldEdit;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsSettings, RigidBody,
};
use ph2d_render::Sprite;

use super::inspector_physics_gesture_tests::{attach, snapshot};
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
    // ⚠️ **O 1.º passo mudou de PORTA na F3** (ADR-0166): a §11 já não se pinta sobre um sprite
    // pelado — a face vazia dela era a única rota, e hoje a rota é o `+` do cabeçalho.
    assert!(
        super::inspector_physics::build_physics_info(
            sim.world(),
            pool.to_bits(),
            0,
            0,
            0,
            false,
            0,
            (0.0, 5.0),
            0
        )
        .is_none(),
        "um sprite pelado nao tem §11 — a poda da F3"
    );

    // 1. O `+` → **Rigid Body**. ⚠️ O collider nasce CASADO com o sprite (1.00 x 1.50 = metade de
    //    2 x 3), então o artista não digita dimensão nenhuma para ter a piscina do tamanho que
    //    desenhou — e agora isso é o *seed* que a paleta corre, não um braço do botão que morreu.
    attach(&mut sim, pool, "ph2d::physics::RigidBody");
    attach(&mut sim, pool, "ph2d::physics::Collider");
    let i = snapshot(&sim, pool);
    assert_eq!(
        (i.half_x, i.half_y),
        (1.0, 1.5),
        "o collider tem de nascer com a caixa do sprite, senão o passo 1 já exige números"
    );

    // 2. Kind = Static (uma piscina não cai). 3. Trigger = Sensor (dá para entrar nela).
    apply(&mut sim, pool, PhysicsFieldEdit::Kind(1));
    apply(&mut sim, pool, PhysicsFieldEdit::Sensor(true));
    let i = snapshot(&sim, pool);
    assert!(
        i.is_sensor && i.kind_tag == 1,
        "é o Sensor que faz as rows Force/Drag serem oferecidas — sem ele o passo 4 não existe"
    );

    // 4. Force Y (o empuxo). 5. Drag (o meio).
    apply(&mut sim, pool, PhysicsFieldEdit::ForceY(3.5));
    apply(&mut sim, pool, PhysicsFieldEdit::AreaDrag(4.0));
    let i = snapshot(&sim, pool);
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

/// **Uma MESA GIRATÓRIA é autorável só com gestos da UI** (W-AreaTorque) — e o passo
/// que a faz girar é um número só.
///
/// O oráculo nunca é "o componente `AreaTorque` existe": é a CENA. Uma caixa dentro da
/// zona GIRA (a rotação cresce), a idêntica ao lado fica parada. Sem gravidade de mundo,
/// para que a rampa de ângulo venha inteira do torque da zona e a caixa não caia para
/// fora do sensor.
#[test]
fn a_spin_zone_is_authorable_with_ui_gestures_alone() {
    let mut sim = SimWorld::new();
    // O artista posiciona um retângulo de 4 x 4 m (a mesa).
    let table = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            Sprite::atlas(0, [4.0, 4.0], [0.6, 0.4, 0.9, 0.3]),
        ))
        .id();

    // 1. Add. 2. Static (a mesa não cai). 3. Sensor (dá para entrar nela).
    apply(&mut sim, table, PhysicsFieldEdit::Add);
    apply(&mut sim, table, PhysicsFieldEdit::Kind(1));
    apply(&mut sim, table, PhysicsFieldEdit::Sensor(true));
    let i = snapshot(&sim, table);
    assert!(
        i.is_sensor && i.kind_tag == 1,
        "é o Sensor que faz a row Torque ser oferecida — sem ele o passo 4 não existe"
    );

    // 4. Torque (o giro) — um número só, e o sinal é o sentido. ⚠️ 0.5 gira a caixa 1x1
    //    ~86 graus em 60 ticks: um giro claro, mas sub-revolução, para que a `rotation`
    //    (que dá a volta em ±pi) não wrape e o oráculo seja legível.
    apply(&mut sim, table, PhysicsFieldEdit::AreaTorque(0.5));
    let i = snapshot(&sim, table);
    assert_eq!(
        i.area_torque, 0.5,
        "a row Torque tem de anexar o `AreaTorque` com o valor autorado"
    );

    // O oráculo: duas caixas 1x1 idênticas, uma dentro da mesa, a outra fora.
    let box_at = |sim: &mut SimWorld, x: f32| {
        sim.world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.5,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, 0.0)),
            ))
            .id()
    };
    let inside = box_at(&mut sim, 0.0);
    let outside = box_at(&mut sim, 8.0);

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    });
    for t in 1..=60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let rot = |sim: &SimWorld, e: Entity| sim.world().get::<Transform>(e).unwrap().rotation;
    let (spun, still) = (rot(&sim, inside), rot(&sim, outside));
    assert!(
        spun.abs() > 0.5,
        "depois de 1 s a caixa na mesa tem de ter girado visivelmente (rotação {spun}) — \
         sem isso os quatro gestos acima produziram uma mesa que não gira"
    );
    assert!(
        still.abs() < 1e-4,
        "a caixa fora da mesa não pode girar (rotação {still})"
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
    let i = snapshot(&sim, pool);
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

/// **Um TRONCO boiando, só com gestos da UI** — e o passo que NÃO se dá.
///
/// Pedido do Enio depois do smoke `=27`. O gesto é curto (Add + Density), e a lição
/// está no que **não** se faz: converter para **Capsule**.
///
/// ⚠️ **Uma cápsula é Y-alinhada por design** (`ShapeDesc::Capsule` documenta o porquê:
/// um eixo configurável seria uma segunda forma de dizer o que o `Transform` já diz), e
/// um tronco DEITADO é largo. A conversão então toma `radius = min(hx, hy)` — a regra
/// honesta *"a cápsula nunca fica mais larga que a caixa"* — e para uma caixa larga isso
/// dá `half_height = 0`: **um círculo inscrito na altura**, que joga fora todo o
/// comprimento. Medido: um tronco de 2,4 × 0,5 vira uma bola de raio 0,25, e aí ele
/// **não endireita mais** — um círculo não tem orientação para endireitar.
///
/// Não é bug da conversão, é geometria; e o contorno (tecla B) mostra a bola, então o
/// artista VÊ. O que este gate garante é que o caminho certo — deixar a caixa que o Add
/// já casou com o sprite — produz um tronco que boia E se endireita.
#[test]
fn a_floating_log_is_authorable_with_ui_gestures_alone() {
    let mut sim = SimWorld::new();
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

    // A poça, pelos gestos que o irmão acima já pina.
    let pool = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, -1.5)),
            Sprite::atlas(0, [8.0, 3.0], [0.3, 0.5, 0.9, 0.3]),
        ))
        .id();
    for edit in [
        PhysicsFieldEdit::Add,
        PhysicsFieldEdit::Kind(1),
        PhysicsFieldEdit::Sensor(true),
        PhysicsFieldEdit::AreaDensity(4.0),
        PhysicsFieldEdit::AreaDrag(1.5),
    ] {
        apply(&mut sim, pool, edit);
    }

    // O tronco: um sprite comprido, posicionado INCLINADO a 1 rad — é assim que o
    // artista o larga, e é o que torna o auto-endireitamento visível.
    let log = sim
        .world_mut()
        .spawn((
            Transform {
                translation: Vec2::new(0.0, 2.0),
                rotation: 1.0,
                scale: Vec2::new(1.0, 1.0),
                skew_x: 0.0,
                skew_y: 0.0,
            },
            Sprite::atlas(0, [2.4, 0.5], [0.55, 0.35, 0.2, 1.0]),
        ))
        .id();

    // Gesto 1: Add. O collider nasce 1,20 × 0,25 — a caixa do sprite, e já a forma
    // certa para um tronco deitado.
    apply(&mut sim, log, PhysicsFieldEdit::Add);
    let i = snapshot(&sim, log);
    assert_eq!(
        (i.shape_tag, i.half_x, i.half_y),
        (1, 1.2, 0.25),
        "o Add tem de casar o collider com o sprite comprido — é o que faz o passo \
         seguinte ser o ÚLTIMO"
    );
    // Gesto 2: Density abaixo da do fluido. Madeira contra água, e nada mais.
    apply(&mut sim, log, PhysicsFieldEdit::Density(1.0));

    let mut bridge = PhysicsBridge::new();
    for t in 1..=600 {
        bridge.dispatch(&mut sim, true, t);
    }
    let tr = *sim.world().get::<Transform>(log).unwrap();
    assert!(
        tr.translation.y.abs() < 0.4,
        "o tronco tem de PARAR na linha d'água (y ≈ 0), e ficou em {}",
        tr.translation.y
    );
    assert!(
        tr.rotation.sin().abs() < 0.15,
        "largado a 1 rad (sin 0,84), o tronco tem de se ENDIREITAR — |sin| ficou em {}",
        tr.rotation.sin().abs()
    );

    // ⚠️ E o controle que nomeia a armadilha: o MESMO tronco convertido para Capsule
    // vira um círculo, e um círculo não endireita. Sem esta metade, o gate acima
    // passaria e a recomendação "não converta" seria prosa sem prova.
    apply(&mut sim, log, PhysicsFieldEdit::Shape(2));
    let i = snapshot(&sim, log);
    assert_eq!(
        (i.radius, i.cap_half_height),
        (0.25, 0.0),
        "converter um tronco DEITADO para cápsula dá o círculo inscrito na altura — a \
         cápsula é Y-alinhada, então o comprimento não tem onde caber"
    );
}

/// **Uma rajada que enfraquece na margem é autorável só com gestos da UI** — o falloff
/// (W-AreaFalloff), e o passo que a torna um campo com centro é UM número.
///
/// Os quatro primeiros gestos são os mesmos dos irmãos acima (Add · Static · Sensor · a
/// força), e é justamente por isso que o gate importa: até aqui a sequência produzia um
/// bloco de vento que empurra igual em toda a sua extensão. O 5º passo — `Falloff` — é o
/// que a torna uma RAJADA, e o oráculo é a cena: dois corpos idênticos, na mesma linha do
/// mesmo vento, um no olho e outro na margem, param em lugares diferentes.
#[test]
fn a_gust_that_fades_at_the_edge_is_authorable_with_ui_gestures_alone() {
    let mut sim = SimWorld::new();
    // O artista deita um retângulo de 24 x 8 m — a coluna de vento.
    let gust = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            Sprite::atlas(0, [24.0, 8.0], [0.9, 0.7, 0.3, 0.3]),
        ))
        .id();

    // 1. Add. 2. Static (o vento não cai). 3. Sensor (dá para entrar nele). 4. A força.
    apply(&mut sim, gust, PhysicsFieldEdit::Add);
    apply(&mut sim, gust, PhysicsFieldEdit::Kind(1));
    apply(&mut sim, gust, PhysicsFieldEdit::Sensor(true));
    apply(&mut sim, gust, PhysicsFieldEdit::ForceX(4.0));
    let i = snapshot(&sim, gust);
    assert!(
        i.is_sensor && i.force[0] == 4.0,
        "é o Sensor que faz as rows de área serem oferecidas — sem ele o passo 5 não existe"
    );

    // 5. O falloff. Um número em 0..=1, e a régua é a própria zona: o artista não tem de
    //    saber nenhum raio, que é exatamente o que um "alcance" em metros exigiria dele.
    apply(&mut sim, gust, PhysicsFieldEdit::AreaFalloff(1.0));
    let i = snapshot(&sim, gust);
    assert_eq!(
        i.area_falloff, 1.0,
        "a row Falloff tem de anexar o `AreaFalloff` com a fração autorada"
    );

    // O oráculo é a CENA: duas bolas idênticas na mesma linha do vento.
    let ball_at = |sim: &mut SimWorld, x: f32| {
        sim.world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.5 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, 0.0)),
            ))
            .id()
    };
    let eye = ball_at(&mut sim, 0.0);
    let edge = ball_at(&mut sim, 10.8);

    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    });
    for t in 1..=12 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x = |sim: &SimWorld, e: Entity| sim.world().get::<Transform>(e).unwrap().translation.x;
    let (moved_eye, moved_edge) = (x(&sim, eye) - 0.0, x(&sim, edge) - 10.8);
    assert!(
        moved_eye > moved_edge * 3.0,
        "os cinco gestos têm de produzir uma rajada com CENTRO: a bola do olho andou \
         {moved_eye} e a da margem {moved_edge}"
    );
    assert!(
        moved_edge > 0.0,
        "a margem ainda é dentro do vento — a bola de lá tem de andar um pouco \
         ({moved_edge})"
    );
}
