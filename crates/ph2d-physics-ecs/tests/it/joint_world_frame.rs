//! **Num pino de MUNDO os números do cartão são os do OBJETO** (GEF02 onda 7, dono 10/11).
//!
//! O rapier mede o corpo B relativo ao A; num pino de mundo o A é o objeto e o B a âncora, então
//! sem a troca de referencial o objeto parava no batente OPOSTO, o motor girava ao contrário e o
//! Slider parava no outro fim. Medido antes da cura (tique 120): batentes `[−30°, +10°]` → −10°;
//! motor +1 rad/s → −114°; alvo +20° → −20°; Slider `[−0,5, +0,2]` → +0,5. A porta é
//! `ph2d_physics_ecs::bridge::joint_desc::no_outro_referencial`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, JointKind, JointWorldAnchor,
    MotorMode, PhysicsBridge, PhysicsJoint, RigidBody,
};

const MEIO: f32 = 0.75;

/// Um portão de `2 × MEIO` preso ao mundo pela ponta esquerda (o Slider, pelo centro).
fn portao(j: PhysicsJoint, gravidade: bool, empurrao: f32) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let g = sim
        .world_mut()
        .spawn((
            Name::new("Gate"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: MEIO,
                    half_y: 0.09,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
            InitialVelocity {
                linvel: [empurrao, 0.0],
                angvel: 0.0,
            },
        ))
        .id();
    if !gravidade {
        sim.world_mut().entity_mut(g).insert(GravityScale(0.0));
    }
    let x = if j.kind == JointKind::Slider {
        0.0
    } else {
        -MEIO
    };
    let pino = sim
        .world_mut()
        .spawn((
            Name::new("Pin"),
            PhysicsJoint {
                body_a: stable_name_id("Gate"),
                body_b: 0,
                motor_max_force: 1000.0,
                ..j
            },
            JointWorldAnchor,
            Transform::from_translation(Vec2::new(x, 5.0)),
        ))
        .id();
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    (sim, g, pino)
}

fn pin(limites: Option<[f32; 2]>) -> PhysicsJoint {
    PhysicsJoint {
        kind: JointKind::Pin,
        limits_enabled: limites.is_some(),
        limit_min: limites.map_or(0.0, |l| l[0].to_radians()),
        limit_max: limites.map_or(0.0, |l| l[1].to_radians()),
        ..PhysicsJoint::default()
    }
}

fn angulo(bridge: &PhysicsBridge, g: Entity) -> f32 {
    bridge.body_pose(g).expect("corpo").2.to_degrees()
}

fn corre(sim: &mut SimWorld, bridge: &mut PhysicsBridge, de: u64, ate: u64) {
    for t in de..=ate {
        bridge.dispatch(sim, true, t);
    }
}

/// ⭐ O portão cai e pára no MÍNIMO do cartão (−30°), não no simétrico do máximo (−10°); controlo:
/// sem gravidade e a girar para cima, ele pára no MÁXIMO (+10°).
#[test]
fn o_limite_minimo_do_pino_de_mundo_e_o_do_objeto() {
    let (mut sim, g, _) = portao(pin(Some([-30.0, 10.0])), true, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 120);
    let a = angulo(&bridge, g);
    assert!((a + 30.0).abs() < 0.5, "caiu até ao mínimo −30°: {a}");

    let mut j = pin(Some([-30.0, 10.0]));
    j.motor_enabled = true;
    j.motor_speed = 2.0;
    let (mut sim, g, _) = portao(j, false, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 120);
    let a = angulo(&bridge, g);
    assert!(
        (a - 10.0).abs() < 0.5,
        "controlo: subiu até ao máximo +10°: {a}"
    );
}

/// O motor fala o sentido do OBJETO: velocidade positiva gira o portão para cima (anti-horário), o
/// alvo +20° põe-no a +20°.
#[test]
fn o_motor_do_pino_de_mundo_gira_no_sentido_do_objeto() {
    let mut j = pin(None);
    j.motor_enabled = true;
    j.motor_speed = 1.0;
    let (mut sim, g, _) = portao(j, false, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 60);
    let a = angulo(&bridge, g);
    assert!(
        (40.0..70.0).contains(&a),
        "1 rad/s durante 1 s, para cima: {a}"
    );

    let mut j = pin(None);
    j.motor_enabled = true;
    j.motor_mode = MotorMode::Position;
    j.motor_target = 20f32.to_radians();
    let (mut sim, g, _) = portao(j, false, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 120);
    let a = angulo(&bridge, g);
    assert!((a - 20.0).abs() < 0.5, "o alvo +20°: {a}");
}

/// O Slider preso ao mundo, empurrado para a direita, pára no MÁXIMO do cartão (+0,2 m).
#[test]
fn o_slider_de_mundo_para_no_fim_do_objeto() {
    let j = PhysicsJoint {
        kind: JointKind::Slider,
        limits_enabled: true,
        limit_min: -0.5,
        limit_max: 0.2,
        ..PhysicsJoint::default()
    };
    let (mut sim, g, _) = portao(j, false, 3.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 60);
    let x = bridge.body_pose(g).expect("corpo").0;
    assert!((x - 0.2).abs() < 0.01, "o fim do objeto, +0,2 m: {x}");
}

/// ⭐⭐ **O caso do dono** (cena 66, Pin A): o portão repousa no mínimo e ADORMECE; a curva desce o
/// mínimo de −30° a −60° a meio do Play, e o portão segue-o até −60°.
#[test]
fn animar_o_minimo_do_pino_de_mundo_leva_o_portao_ate_la() {
    let (mut sim, g, pino) = portao(pin(Some([-30.0, 30.0])), true, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 240);
    assert!((angulo(&bridge, g) + 30.0).abs() < 0.5);
    for t in 241..=300u64 {
        let f = (t - 240) as f32 / 60.0;
        sim.world_mut()
            .get_mut::<PhysicsJoint>(pino)
            .expect("o pino")
            .limit_min = (-30.0 - 30.0 * f).to_radians();
        bridge.dispatch(&mut sim, true, t);
    }
    corre(&mut sim, &mut bridge, 301, 360);
    let a = angulo(&bridge, g);
    assert!(
        (a + 60.0).abs() < 0.5,
        "seguiu o mínimo animado até −60°: {a}"
    );
}

/// O desenho fala o cartão: no pino de mundo o arco é o autorado (`[min, max]`, parado no mundo) e a
/// agulha é o ângulo do OBJETO.
#[test]
fn o_arco_do_pino_de_mundo_mostra_os_numeros_do_cartao() {
    let (mut sim, g, _) = portao(pin(Some([-30.0, 10.0])), true, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 120);
    let v = bridge.joint_views().next().expect("a vista do pino");
    let [mn, mx] = v.limits.expect("limites");
    assert!(
        (mn.to_degrees() + 30.0).abs() < 1e-3 && (mx.to_degrees() - 10.0).abs() < 1e-3,
        "o arco é o do cartão: [{}, {}]",
        mn.to_degrees(),
        mx.to_degrees()
    );
    assert!(
        v.angle_a.abs() < 1e-6,
        "o arco parado no mundo: {}",
        v.angle_a
    );
    assert!(
        (v.angle_b.to_degrees() - angulo(&bridge, g)).abs() < 1e-3,
        "a agulha no objeto"
    );
}

/// O Custom fala o mesmo idioma: X e Y travados, a rotação com batentes `[−30°, +10°]`, o portão cai
/// até ao mínimo do cartão.
#[test]
fn o_custom_de_mundo_para_no_minimo_do_objeto() {
    use ph2d_physics_ecs::{AxisMode, CustomAxis};
    let mut j = PhysicsJoint {
        kind: JointKind::Custom,
        ..PhysicsJoint::default()
    };
    for eixo in [CustomAxis::X, CustomAxis::Y] {
        j.custom.axis_mut(eixo).mode = AxisMode::Locked;
    }
    let r = j.custom.axis_mut(CustomAxis::Rotation);
    r.mode = AxisMode::Limited;
    r.min = (-30f32).to_radians();
    r.max = 10f32.to_radians();
    let (mut sim, g, _) = portao(j, true, 0.0);
    let mut bridge = PhysicsBridge::new();
    corre(&mut sim, &mut bridge, 1, 120);
    let a = angulo(&bridge, g);
    assert!((a + 30.0).abs() < 0.5, "o mínimo do cartão, −30°: {a}");
}

/// A CORDA não troca de sentido: a distância entre as âncoras é a mesma vista de qualquer lado, então
/// o guincho de um pino de mundo é o mesmo guincho.
#[test]
fn a_corda_de_mundo_nao_troca_o_sentido_do_guincho() {
    let j = PhysicsJoint {
        kind: JointKind::Rope,
        motor_enabled: true,
        motor_speed: 0.5,
        motor_target: 1.0,
        max_length: 2.0,
        ..PhysicsJoint::default()
    };
    let d = |mundo| {
        ph2d_physics_ecs::joint_desc(&j, [0.0; 2], [0.0; 2], ([1.0, 0.0], [1.0, 0.0]), mundo)
            .expect("a corda tem descritor")
    };
    assert_eq!(d(true), d(false));
}

/// Com a cena PARADA no quadro 0 (o artista a editar o cartão) nenhum tique corre: o arco já tem de
/// mostrar os números do cartão.
#[test]
fn o_arco_do_pino_de_mundo_parado_mostra_o_cartao() {
    let (mut sim, _, _) = portao(pin(Some([-30.0, 10.0])), true, 0.0);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let v = bridge.joint_views().next().expect("a vista do pino");
    let [mn, mx] = v.limits.expect("limites");
    assert!(
        (mn.to_degrees() + 30.0).abs() < 1e-3 && (mx.to_degrees() - 10.0).abs() < 1e-3,
        "parado, o arco é o do cartão: [{}, {}]",
        mn.to_degrees(),
        mx.to_degrees()
    );
}
