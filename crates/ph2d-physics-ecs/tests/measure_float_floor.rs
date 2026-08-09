//! **ONDE A PERNA FICA CURTA DEMAIS** — o piso geométrico da altura de
//! flutuação, e por que ele NÃO é uma constante.
//!
//! A sonda do repouso (`measure_idle`) mede, numa rampa de 30°, o personagem a
//! **descer 0,59 m em 10 s com amplitude de 0,27 m** — no `float_height = 0,50`,
//! que é o **default que shipa**. De 0,60 para cima ela mede 0,0000 em tudo.
//!
//! A suspeita tem forma fechada e é o que esta sonda existe para confirmar ou
//! matar: o pé de uma cápsula fica mais longe da origem do raio quando a
//! superfície se inclina, então o mínimo geométrico **cresce com a rampa** —
//!
//! ```text
//! float_min(θ) = half_height + radius / cos θ
//! ```
//!
//! — e a lane do `physics_ecs_c9` já traz esse número escrito à mão para a rampa
//! dela (*"o piso geométrico desta cápsula com folga (`half + radius / cos 45°`
//! ≈ 0,82): sem ela o personagem nasceria tangente e a rampa o faria penetrar"*).
//! O que ninguém tinha perguntado é se o **DEFAULT** o respeita.
//!
//! # O que ela mediu (2026-08-09), e o veredito é *não há defeito*
//!
//! ```text
//!   rampa   previsto   onset medido   |  no default 0,50
//!      10°     0.5031        0.5100   |  deriva  +0.0200  amp 0.0035
//!      20°     0.5128        0.5200   |  deriva  +0.0393  amp 0.0134
//!      30°     0.5309        0.5400   |  deriva  -0.6823  amp 0.3406
//!      40°     0.5611        0.5700   |  deriva  -5.1431  amp 3.3004
//!      45°     0.5828        0.5900   |  deriva  -9.5340  amp 6.7303
//! ```
//!
//! O onset segue a previsão com **um passo de varredura** de folga em toda
//! linha ⇒ o piso é geométrico, confirmado. E o **gesto do artista já o
//! honra**: `apply_player_edit(Add)` chama `RideConfig::min_float_height` e
//! multiplica por **1,2**, o que a 45° dá `0,6994` contra o onset de `0,59` —
//! **19% de folga**, medida em vez de escolhida.
//!
//! ⚠️ **A forma da falha abaixo do piso é o que torna essa margem
//! load-bearing, e ela não estava escrita em lugar nenhum:** não é uma deriva
//! que cresce devagar — a 45° o personagem **cai 9,5 m oscilando 6,7 m**. Um
//! piso que se erra por um centímetro não custa um centímetro.
//!
//! ⚠️ **E o `RideConfig::STARTING_POINT` fica em `0,50` de propósito** — ele é
//! o ponto de partida do MODELO, e quem o veste com a geometria de um corpo
//! concreto é o gesto. A linha `float 0,50` da tabela de rampa do
//! `measure_idle` mostra a patologia sem dizer isso, e foi ela que me mandou
//! nesta caçada: *um número medido fora do piso do seu próprio domínio lê como
//! defeito de produto*.
//!
//! ⚠️ Esta sonda **não conserta nada** — ela mede pelas portas do produto, para
//! que a decisão venha de um número.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_float_floor
//! -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};
use scene_fixture::pose;

/// A cápsula do player em toda cena e todo gate deste módulo.
const HALF_HEIGHT: f32 = 0.3;
const RADIUS: f32 = 0.2;

/// O piso geométrico previsto: onde o PÉ da cápsula toca a superfície inclinada.
///
/// ⚠️ **Ela PERGUNTA à porta do produto** (`RideConfig::min_float_height`) em vez
/// de reescrever a forma fechada — e a 1ª versão deste arquivo a reescrevia, o
/// que fazia do gate um espelho de uma SEGUNDA cópia: ele continuaria verde no
/// dia em que a fórmula do produto se movesse, que é precisamente o dia que ele
/// existe para pegar. A forma fechada fica no doc do módulo, para o leitor.
fn predicted_floor(slope_deg: f32) -> f32 {
    ph2d_platformer::RideConfig::min_float_height(HALF_HEIGHT, RADIUS, slope_deg.to_radians().cos())
}

fn rig(slope_deg: f32, float_height: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let slope = slope_deg.to_radians();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    let who = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: HALF_HEIGHT,
                    radius: RADIUS,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.25 + float_height + 0.4)),
        ))
        .id();
    let bridge = PhysicsBridge::new();
    (sim, bridge, who)
}

/// Devolve `(deriva ao longo da rampa, amplitude de `y` na janela assentada)`.
fn idle(slope_deg: f32, float_height: f32) -> (f32, f32) {
    let (mut sim, mut bridge, who) = rig(slope_deg, float_height);
    // Assenta.
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p0 = pose(&sim);
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for t in 121..=720u64 {
        bridge.dispatch(&mut sim, true, t);
        let y = pose(&sim).1;
        lo = lo.min(y);
        hi = hi.max(y);
    }
    let p1 = pose(&sim);
    let _ = who;
    let along =
        (p1.0 - p0.0) * slope_deg.to_radians().cos() + (p1.1 - p0.1) * slope_deg.to_radians().sin();
    (along, hi - lo)
}

/// **A fronteira, varrida finamente** — e a previsão ao lado dela.
#[test]
#[ignore = "sonda"]
fn measure_where_the_leg_runs_short() {
    println!("\n=== O PISO DA PERNA: onde a rampa comeca a bater na capsula ===");
    println!("capsula half {HALF_HEIGHT} / radius {RADIUS}; previsao = half + radius/cos(theta)\n");
    println!("  rampa   previsto   |  float: onde a deriva/oscilacao ACABA  | default 0,50");
    for slope in [0.0f32, 10.0, 20.0, 30.0, 40.0, 45.0] {
        let pred = predicted_floor(slope);
        // Varredura fina de 0,45 a 0,90 em passos de 0,01.
        let mut onset = f32::NAN;
        let mut f = 0.45f32;
        while f <= 0.9005 {
            let (drift, amp) = idle(slope, f);
            if drift.abs() < 1.0e-3 && amp < 1.0e-3 {
                onset = f;
                break;
            }
            f += 0.01;
        }
        let (d50, a50) = idle(slope, 0.5);
        println!(
            "   {slope:>4.0}°   {pred:>8.4}   |            {onset:>8.4}               | \
             deriva {d50:>8.4}  amp {a50:>7.4}"
        );
    }
    println!(
        "\nLEITURA: se a coluna do meio SEGUIR a previsao, o piso e' geometrico\n\
         e o default de 0,50 (`RideConfig::STARTING_POINT`) fica ABAIXO dele em\n\
         toda rampa que o limite autorado de 45 graus permite.\n"
    );
}

/// **O PISO PREVISTO É O PISO QUE O SIMULADOR MOSTRA** — a fórmula é medida
/// contra o comportamento, não contra a tabela dela mesma.
///
/// ⚠️ **Por que este gate não é redundante com o `ride_tests`:** aquele confere
/// `min_float_height` contra a **tabela do próprio doc** — um espelho da
/// fórmula, que continuaria verde se as duas se movessem juntas. Este pergunta
/// ao **SIMULADOR**, que não conhece fórmula nenhuma.
///
/// ⚠️ **E a 1ª versão dele não tinha dentes, com este mesmo nome:** ela afirmava
/// que `piso × 1,2` fica QUIETO, e a margem de 20% **absorve a fórmula errada**
/// — com o piso colapsado numa constante (sem o `cos`), a 45° ele dá
/// `0,50 × 1,2 = 0,60` contra um onset de `0,59`, e o gate passava. Um gate cujo
/// nome fala de PISO tem de medir o piso: o que ele afirma agora é que o
/// **onset do simulador SEGUE a fórmula**, varrido.
#[test]
fn the_predicted_floor_is_the_floor_the_simulator_shows() {
    // Um passo de varredura (0,01) mais o viés medido (~0,008) mais folga.
    const TOL: f32 = 0.03;
    // A margem que o gesto do artista aplica (`fitted_float`, na shell).
    const MARGIN: f32 = 1.2;

    for slope in [20.0f32, 30.0, 45.0] {
        let floor = predicted_floor(slope);

        // Onde a patologia ACABA, perguntado ao simulador em passos de 1 cm.
        let mut onset = f32::NAN;
        let mut f = 0.45f32;
        while f <= 0.9005 {
            let (drift, amp) = idle(slope, f);
            if drift.abs() < 1.0e-3 && amp < 1.0e-3 {
                onset = f;
                break;
            }
            f += 0.01;
        }
        assert!(
            onset.is_finite(),
            "a fixture tem de CONTER o fenomeno: a {slope}° a varredura nao \
             achou onde a patologia acaba"
        );
        assert!(
            (onset - floor).abs() <= TOL,
            "a {slope}° o simulador para de brigar em {onset:.4} e a formula do \
             produto prevê {floor:.4} — a formula deixou de descrever o piso"
        );
        // E a margem do gesto o limpa: é ela que faz de um personagem novo um
        // personagem quieto, e o número dela é medido em vez de escolhido.
        assert!(
            floor * MARGIN > onset,
            "a {slope}° a margem de {MARGIN} sobre o piso ({:.4}) nao limpa o \
             onset medido ({onset:.4})",
            floor * MARGIN
        );
    }
}
