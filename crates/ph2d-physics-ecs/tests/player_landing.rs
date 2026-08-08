//! **O POUSO PARA DE ESCORREGAR** (W-Landing).
//!
//! Report do Enio (2026-08-07): *"quando o player está pousando de seu pulo, a
//! desaceleração ao encostar no chão é muito lenta e fica artificial"*.
//!
//! Medido antes da cura (`measure_landing::measure_landing_profile`): toque no
//! tique 39, repouso no **69** — `0,500 s` —, e o perfil **não é uma parada, é
//! um decaimento**: `−0,258 · −0,229 · −0,204 · −0,181 …`, razão **0,888 por
//! tique**. Ele nunca chega; aproxima-se para sempre. E **não afunda**
//! (`0,9023` contra o repouso `0,900`), então não é quique — é arrasto.
//!
//! # ⚠️ A causa é aritmética, e o número previsto bate com o medido
//!
//! Com `spring_damping` no teto — o que shipa — o boost apaga a velocidade
//! relativa INTEIRA a cada tique, então o único movimento que sobra é o que a
//! mola produz em UM tique, e o resto decai por
//!
//! ```text
//! 1 − k·dt²  =  1 − 400/3600  =  0,889          (medido: 0,888)
//! ```
//!
//! ⇒ **quem manda no pouso é a RIGIDEZ, e o amortecimento não.**
//!
//! # ⚠️ E o caminho óbvio é o pior
//!
//! Baixar o amortecimento também acelera o pouso (`0,50` dá os mesmos
//! `0,133 s`) — mas ele é o knob que a W11c pôs no teto para **zerar a deriva de
//! rampa**, e a lei publicada cobra `0,153 · sen θ · (1 − d)`. A rigidez compra
//! o mesmo pouso por **zero** de deriva, e isso está medido nos dois eixos aqui.

#[path = "platform_water_scene.rs"]
mod water;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::RideConfig;
use water::{DT, FLOAT, subject_tuned};

/// A queda destes gates.
const DROP: f32 = 3.0;

fn tuned(strength: f32) -> PlatformPlayer {
    PlatformPlayer {
        float_height: FLOAT,
        spring_strength: strength,
        ..PlatformPlayer::default()
    }
}

/// `(segundos do toque ao repouso, quanto afundou abaixo do repouso em cm)`.
fn land(strength: f32) -> (f32, f32) {
    let mut sim = SimWorld::new();
    water::floor(&mut sim, 0.0);
    let _ = subject_tuned(&mut sim, true, DROP, Some(tuned(strength)));
    let mut bridge = PhysicsBridge::new();

    let reach = FLOAT + PlatformPlayer::default().cling_distance;
    let mut prev = DROP;
    let (mut touch, mut settled) = (None, None);
    let mut lowest = f32::INFINITY;
    for t in 1..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        let y = water::y_of(&sim, "Subject");
        let vy = (y - prev) / DT;
        prev = y;
        lowest = lowest.min(y);
        if touch.is_none() && y <= reach {
            touch = Some(t);
        }
        if touch.is_some() && settled.is_none() && (y - FLOAT).abs() < 0.01 && vy.abs() < 0.05 {
            settled = Some(t);
        }
    }
    (
        (settled.unwrap_or(600) - touch.unwrap()) as f32 * DT,
        (FLOAT - lowest) * 100.0,
    )
}

/// **O pouso do DEFAULT assenta depressa.**
#[test]
fn the_shipping_landing_settles_quickly() {
    let (secs, _) = land(PlatformPlayer::default().spring_strength);
    assert!(
        secs < 0.20,
        "o pouso do default tem de assentar em menos de 0,20 s: {secs:.3} s \
         (antes da cura: 0,500)"
    );
}

/// **E ele NÃO chega lá afundando.**
///
/// ⚠️ **Este gate é o que impede a cura barata:** baixar o amortecimento também
/// acelera o pouso, e paga com o personagem a mergulhar no chão (medido:
/// `15,5 cm` em `d = 0,25`). Sem ele, a cura errada passaria no gate acima.
#[test]
fn a_quick_landing_does_not_dip_into_the_floor() {
    let (_, dip_cm) = land(PlatformPlayer::default().spring_strength);
    assert!(
        dip_cm < 1.0,
        "o pouso nao pode afundar no chao: {dip_cm:.1} cm abaixo do repouso"
    );
}

/// **E a deriva de rampa continua ZERO.**
///
/// ⚠️ É o que separa esta cura da tentadora — a W11c pôs o amortecimento no teto
/// exatamente para este número, e a rigidez não pode gastá-lo.
#[test]
fn a_stiffer_leg_does_not_bring_the_ramp_drift_back() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 200.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform {
            rotation: 30.0_f32.to_radians(),
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    let top = 0.5 / 30.0_f32.to_radians().cos();
    let _ = subject_tuned(
        &mut sim,
        true,
        top + FLOAT,
        Some(tuned(PlatformPlayer::default().spring_strength)),
    );
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x0 = water::xy_of(&sim, "Subject").0;
    for t in 61..=660u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let drift = (water::xy_of(&sim, "Subject").0 - x0).abs();
    assert!(
        drift < 0.005,
        "a deriva de rampa em 10 s tem de continuar zero: {drift:.4} m"
    );
}

/// **O TETO da rigidez é `1/dt²`, e ele é um FATO da discretização.**
///
/// Acima dele a razão `1 − k·dt²` fica **negativa**: a mola passa do alvo em vez
/// de chegar nele, e o personagem volta a afundar. Medido: `3600 → 0,033 s` e
/// zero afundamento (a resposta *deadbeat*), `4000 → 2,5 cm`, `5000 → 8,9 cm`.
///
/// É o irmão exato do [`RideConfig::MAX_DAMPING`], que existe porque acima dele
/// *"o boost inverte a velocidade em vez de matá-la, e o personagem pipoca"*.
#[test]
fn the_stiffness_ceiling_is_where_the_spring_overshoots() {
    let at = RideConfig::MAX_SPRING_STRENGTH;
    assert!(
        (at - 1.0 / (DT * DT)).abs() < 1.0,
        "o teto tem de SER 1/dt^2, nao um numero parecido: {at} contra {}",
        1.0 / (DT * DT)
    );

    let (_, dip_at) = land(at);
    assert!(
        dip_at < 0.5,
        "no teto a mola ainda CHEGA no alvo: afundou {dip_at:.1} cm"
    );

    // ⚠️ O clamp da porta é o que torna o teto real: pedir o dobro tem de dar o
    // MESMO pouso, e não o ringing medido em 5000 (8,9 cm).
    let (secs_at, _) = land(at);
    let (secs_over, dip_over) = land(at * 2.0);
    assert!(
        (secs_over - secs_at).abs() < 1.0e-4 && dip_over < 0.5,
        "pedir acima do teto tem de ser clampado: {secs_at:.3} s contra \
         {secs_over:.3} s, afundou {dip_over:.1} cm"
    );
}
