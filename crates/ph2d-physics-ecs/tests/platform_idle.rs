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

/// **O que shipa não deixa resíduo NENHUM — e baixar o knob é o que o traz de
/// volta, por uma lei que este gate pina.**
///
/// ⚠️ **Este gate já afirmou o CONTRÁRIO, e a inversão é a história do módulo.**
/// Enquanto o default era `0,50` ele existia para pinar um resíduo *nomeado*
/// (0,0383 m em 10 s a 30°), porque o teto — que sempre zerou a deriva — custava
/// **metade do peso** do personagem e não podia ser o default. A wave
/// `gravity_hold` mudou a coluna do preço (95% → 91%, quatro pontos), o smoke de
/// 05/08 reportou a subida pela segunda vez, e a troca inverteu de lado.
///
/// ## A escada, e cada degrau tem um dono
///
/// | quando | resíduo do default | o que mudou |
/// |---|---|---|
/// | antes da W11 | 0,3295 m | o amortecedor no `up`: **nenhum** valor do knob o removia |
/// | eixo na NORMAL | 0,1644 m | `super::damping_axis` |
/// | W11b | 0,0383 m | o cancelamento da gravidade passa a ser INTEGRADO como ela |
/// | **hoje** | **0,0000 m** | o default subiu ao teto, que a W11b tornou pagável |
///
/// ⚠️ **As duas metades são load-bearing e medem coisas diferentes.** A primeira
/// diz que o default não viaja — e ela sozinha ficaria verde numa lei que
/// congelasse o personagem, ou no dia em que alguém *"simplificasse"* a deriva
/// removendo o knob. A segunda mede o knob **abaixado** e exige o resíduo de
/// volta: é ela que prova que o zero de cima foi **comprado** pelo
/// amortecimento, e não é vácuo.
#[test]
fn the_shipped_default_leaves_nothing_and_lowering_the_knob_is_what_costs() {
    let d = idle_travel(30.0, 10, RideConfig::STARTING_POINT.spring_damping);
    assert!(
        d < 1.0e-3,
        "o default tem de ficar PARADO: viajou {d:.4} m em 10 s a 30°"
    );
    // E a metade que impede o zero de cima de passar por vácuo: com o knob em
    // meio curso o resíduo volta, e volta na ordem que a lei
    // `0,153 · sen θ · (1 − d)` prevê (0,0383 m a 30°, d = 0,5).
    let half = idle_travel(30.0, 10, 0.5 * RideConfig::MAX_DAMPING);
    assert!(
        (0.02..0.06).contains(&half),
        "com meio amortecimento o residuo mudou de ordem: {half:.4} m"
    );
}

/// ⚠️ **E o TETO continua a zerá-lo — agora por um preço que se pode pagar.**
///
/// O gate irmão do de cima, e a razão de ele existir separado: o que mudou na
/// W11 não foi só o resíduo, foi **quanto custa removê-lo**. O amortecimento no
/// teto sempre deu deriva exactamente zero; o que o impedia de ser o default era
/// a coluna do PESO, e ela mudou de casa.
///
/// | `spring_damping` | erro de repouso | peso transmitido |
/// |---|---|---|
/// | 0,50 | 5,75 → **1,15 mm** | 77% → **95%** |
/// | **1,00** (o teto, e o que shipa desde 05/08) | 11,50 → **2,30 mm** | 53% → **91%** |
///
/// O peso é `(9,81 − k·erro)/9,81` com `k = spring_strength`: a perna paira
/// ACIMA do pedido, o offset fica negativo, e o que a mola deixa de empurrar é o
/// que o chão deixa de sentir. Cortar o erro por cinco corta a perda por cinco.
///
/// **Mutação que deve sangrar:** agrupar o `gravity_hold` de volta no motor.
#[test]
fn the_ceiling_now_costs_a_tenth_of_the_weight_not_a_half() {
    let (mut sim, mut bridge) = rig(0.0, MAX_DAMPING);
    for t in 1..=600 {
        bridge.dispatch(&mut sim, true, t);
    }
    // A folga que a perna de facto segura, no plano, contra a pedida.
    let held = pose(&sim).1 - 0.5;
    let err = held - FLOAT;
    assert!(
        (0.0005..0.005).contains(&err),
        "o erro de repouso no teto mudou de ordem: {:.3} mm",
        err * 1000.0
    );
    // E o peso que sobra para o chão, pela mesma aritmética que a tabela usa.
    let weight = (9.81 - RideConfig::STARTING_POINT.spring_strength * err) / 9.81;
    assert!(
        weight > 0.85,
        "no teto o personagem tem de pesar quase tudo: {:.0}%",
        weight * 100.0
    );
}

/// **E o CONTROLE: o plano já estava certo, e continua** — perguntado ao
/// `STARTING_POINT` e não a um literal, para seguir o default aonde ele for.
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
