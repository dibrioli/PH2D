//! **O REPOUSO** (W11) — o personagem parado fica parado, na rampa também.
//!
//! Report do Enio no smoke da W10: *"nas rampas, se parado, a depender do Float
//! Height ele pode subir a rampa sozinho bem devagar"*. Medido antes de uma
//! linha ser escrita: numa rampa de 30° ele subia a **3,3 cm/s, para sempre** —
//! um regime permanente, não um transiente.
//!
//! # ⚠️ Por que o gate mora AQUI e não na lei
//!
//! A lei pura não tem como produzir o defeito: ele é um **acordo entre três
//! coisas** — o freio da caminhada (que remove a componente TANGENTE), a perna
//! (que remove a componente que ela consegue ver) e o INTEGRADOR do `rapier`
//! (que aplica a gravidade ao longo do tique enquanto a perna a cancela com um
//! impulso no topo dele). Só a simulação de verdade compõe os três.
//!
//! A causa está medida por ablação na sonda `measure_idle` e escrita em
//! [`BUGS_physics.md`](../../../docs/Physics/BUGS_physics.md) §3.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::RideConfig;
use scene_fixture::pose;

/// O teto medido do amortecimento — o valor em que a perna remove a
/// aproximação INTEIRA num tique ([`RideConfig::MAX_DAMPING`]).
const MAX_DAMPING: f32 = RideConfig::MAX_DAMPING;

/// A altura de flutuação destes gates.
///
/// ⚠️ **0,9 e não 0,5, e a fixture depende disso:** o mínimo geométrico desta
/// cápsula no PLANO já é `0,5` ([`ph2d_platformer::RideConfig::min_float_height`]),
/// então em `0,5` ela **encosta** na rampa e quem responde passa a ser o solver
/// de contato. Um gate montado ali mediria o atrito do `rapier`, não a perna.
const FLOAT: f32 = 0.9;

/// Chão inclinado + player em cima, sem entrada nenhuma.
fn rig(slope_deg: f32, damping: f32) -> (SimWorld, PhysicsBridge) {
    let slope = slope_deg.to_radians();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            // Largo o bastante para nenhuma medição encontrar uma beirada — a
            // sonda irmã já reportou uma QUEDA como se fosse velocidade.
            shape: ColliderShape::Cuboid {
                half_x: 200.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            spring_damping: damping,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.5 / slope.cos() + FLOAT)),
    ));
    (sim, PhysicsBridge::new())
}

/// Quantos segundos a perna tem para assentar antes de a medição começar.
///
/// ⚠️ **A fixture SEM isto media a coisa errada, e ela nasceu assim:** o
/// personagem é largado na altura pedida e a perna o assenta 11,5 mm acima dela
/// (o erro de repouso da tabela do `STARTING_POINT`), então uma medição desde o
/// tique 1 reportava **0,0115 m de viagem no chão PLANO** — o assentamento, não
/// uma deriva. O defeito é um REGIME PERMANENTE; o que o gate tem de ver é o que
/// sobra depois de a perna se acomodar.
const SETTLE_SECS: u64 = 2;

/// Quanto o personagem VIAJOU em `secs` segundos sem ninguém tocar nele,
/// **contados depois de a perna assentar**.
fn idle_travel(slope_deg: f32, secs: u64, damping: f32) -> f32 {
    let (mut sim, mut bridge) = rig(slope_deg, damping);
    for t in 1..=SETTLE_SECS * 60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let start = pose(&sim);
    for t in SETTLE_SECS * 60 + 1..=(SETTLE_SECS + secs) * 60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let end = pose(&sim);
    ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt()
}

/// **O GATE DA WAVE.** Com a perna amortecendo no eixo certo, o personagem
/// parado numa ladeira caminhável **fica parado** — em qualquer inclinação.
///
/// ## Medido (2026-08-04, 10 s, `float_height = 0,9`, `spring_damping` no teto)
///
/// | rampa | com o amortecedor no `up` | com ele na NORMAL |
/// |---|---|---|
/// | 10° | 0,132 m | **0,000 m** |
/// | 20° | 0,244 m | **0,000 m** |
/// | 30° | 0,328 m | **0,000 m** |
///
/// ⚠️ **O amortecimento é posto no TETO aqui, e não é conveniência de fixture:**
/// é a única forma de a lei remover a componente inteira, e a tabela do
/// [`ph2d_platformer::RideConfig::STARTING_POINT`] diz por que o DEFAULT não
/// vive lá (no teto o personagem pesa metade). O que este gate prova é que a
/// lei **consegue** ficar parada — antes do eixo corrigido, nenhum valor do knob
/// conseguia (0,3276 contra 0,3295 do controle).
///
/// ⚠️ **O oráculo é a DISTÂNCIA PERCORRIDA, não a posição final**, e a diferença
/// não é estilo: um personagem que sobe e volta terminaria no lugar e o gate
/// diria que está tudo bem. O que o artista vê é a viagem.
///
/// **Mutação que deve sangrar:** o eixo do amortecedor de volta ao `up`.
#[test]
fn a_full_damper_holds_the_player_still_on_a_walkable_ramp() {
    for &slope in &[10.0_f32, 20.0, 30.0, 40.0] {
        let d = idle_travel(slope, 10, MAX_DAMPING);
        assert!(
            d < 1.0e-3,
            "parado numa rampa de {slope:.0}° o personagem viajou {d:.4} m em 10 s"
        );
    }
}

/// ⚠️ **A quietude não se desfaz com o TEMPO** — o defeito era um regime
/// permanente (3,3 cm/s **para sempre**), não um transiente de assentamento, e
/// um gate de 10 s não distingue os dois sozinho.
#[test]
fn the_stillness_holds_for_a_minute_not_just_for_ten_seconds() {
    let d = idle_travel(30.0, 60, MAX_DAMPING);
    assert!(d < 1.0e-3, "um minuto parado numa rampa de 30°: {d:.4} m");
}

/// **E o que shipa hoje deixa um resíduo NOMEADO, não zero.**
///
/// Com o `spring_damping` default a lei remove metade da componente e o resto
/// vira viagem. O gate pina o número dos dois lados: o teto **superior** é o que
/// o eixo corrigido comprou (era 0,33 m com o amortecedor no `up`), e o
/// **inferior** impede que ele passe por vácuo no dia em que alguém subir o
/// default sem ler a coluna do peso — nesse dia é este gate que pergunta se a
/// troca foi deliberada.
#[test]
fn the_shipped_default_leaves_a_measured_residue() {
    let d = idle_travel(30.0, 10, RideConfig::STARTING_POINT.spring_damping);
    assert!(
        (0.05..0.25).contains(&d),
        "o residuo do default mudou de ordem: {d:.4} m em 10 s a 30°"
    );
}

/// **E o CONTROLE: o plano já estava certo, e continua** — com o default que
/// shipa, não com o teto.
///
/// Sem esta metade, *"não viaja"* seria verdade também numa lei que congelasse
/// o personagem — e o gate não distinguiria a correção de um freio de mão.
#[test]
fn the_flat_ground_control_is_still_perfectly_still() {
    let shipped = RideConfig::STARTING_POINT.spring_damping;
    assert!(idle_travel(0.0, 10, shipped) < 1.0e-6);
}

/// **A perna continua a segurar a altura pedida** — o gate que impede a cura de
/// ser *"o personagem parou de flutuar"*.
///
/// ⚠️ O erro de repouso é NOMEADO e não é zero: a perna paira alguns milímetros
/// acima do pedido, e o número é o preço medido do amortecimento (a tabela do
/// [`ph2d_platformer::RideConfig::STARTING_POINT`]). O bar de 20 mm existe para
/// pinar a ordem de grandeza, não para esconder o erro.
#[test]
fn the_leg_still_holds_the_height_it_was_asked_for() {
    for &slope in &[0.0_f32, 20.0, 30.0] {
        let (mut sim, mut bridge) = rig(slope, MAX_DAMPING);
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
        }
        let (x, y) = pose(&sim);
        // A folga VERTICAL até o topo do chão sob o personagem — a mesma que o
        // raio do sensor mede.
        let held = y - (0.5 / slope.to_radians().cos() + x * slope.to_radians().tan());
        assert!(
            (held - FLOAT).abs() < 0.02,
            "rampa {slope:.0}°: a perna segurou {held:.4} em vez de {FLOAT}"
        );
    }
}
