//! **ONDE ELE PARA QUANDO BOIA** — a sonda da linha de flutuação.
//!
//! A pergunta do Enio: *"não temos parâmetros para o quanto fica submerso quando
//! boia na superfície?"*. Antes de responder, meça — porque a submersão de
//! repouso **não é escolhida hoje, é DERIVADA**: o empuxo é `ρ_fluido · g · A_submersa`
//! e o peso é `m · g`, então o equilíbrio senta onde
//!
//! ```text
//!     A_submersa / A_total  =  m / (ρ_fluido · A_total)  =  1 / razão_de_densidades
//! ```
//!
//! Esta sonda pergunta ao PRODUTO três coisas que a aritmética não responde:
//!
//! 1. ele de facto **assenta**, ou sobe e sai da água?
//! 2. a fração submersa de repouso bate com a previsão?
//! 3. o **nado ligado** muda onde ele para?
//!
//! # ⚠️ E a resposta da 3 era SIM — foi esta sonda que achou o defeito
//!
//! O repouso do nado mirava **velocidade zero**, que é uma instrução diferente
//! de *"boie"*: o servo cancelava o empuxo a cada tique e o nadador congelava
//! onde estava. Medido antes da cura, poça `1,25×`, autoridade `44`:
//!
//! ```text
//!   nado 0  ->  80,1% submerso  (a linha da física)
//!   nado 4  -> 100,0% submerso  (afundado, e lá ficava)
//! ```
//!
//! Hoje o repouso procura a linha, e as duas colunas coincidem (`25,0 · 49,9 ·
//! 79,8` contra `25 · 50 · 80` previstos) — com um bónus que a tabela mostra na
//! coluna `osc`: o nadador **assenta** onde a boia oscila `0,43 m`.
//!
//! ⚠️ **A poça de razão `1,00` é o caso de fronteira, e a tabela o expõe:** o
//! nado **nunca arma** lá, porque a tesselação do collider lê `buoyed ≈ 0,996`
//! submerso por completo — logo abaixo do limiar de `1,0` (o viés de `0,64%` que
//! o `AreaBuoyancy` documenta). Um corpo de densidade neutra que precise nadar
//! pede um `swim_enter` um pouco abaixo de 1; e é ele que, submerso, fica onde
//! está sem nada fazer — a flutuação neutra sai de graça da razão, não de um
//! knob.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_the_float_line -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const DRAG: f32 = 0.6;

/// A altura total da cápsula — `2·(half_height + radius)`.
const TOTAL_H: f32 = 2.0 * (HALF_H + RADIUS);

/// A área da cápsula: o retângulo do meio mais o disco das duas calotas.
fn capsule_area() -> f32 {
    2.0 * RADIUS * (2.0 * HALF_H) + std::f32::consts::PI * RADIUS * RADIUS
}

fn pool(sim: &mut SimWorld, fluid: f32) {
    sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 6.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(fluid),
        AreaDrag(DRAG),
        // A superfície em `y = 0`.
        Transform::from_translation(Vec2::new(0.0, -6.0)),
    ));
}

fn player(sim: &mut SimWorld, kinematic: bool, swim: f32, density: f32) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            swim_speed: swim,
            swim_acceleration: 44.0,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, -2.0)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn y_of(sim: &SimWorld) -> f32 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation.y;
        }
    }
    panic!("o sujeito tem de existir");
}

/// Vinte segundos parado na poça. Devolve `(y médio do último terço, y mínimo e
/// máximo do último terço)` — a média porque o corpo OSCILA, os extremos porque
/// *"assentou"* é uma afirmação sobre a amplitude, não sobre um instante.
fn settles(kinematic: bool, swim: f32, density: f32, fluid: f32) -> (f32, f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim, fluid);
    let who = player(&mut sim, kinematic, swim, density);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());
    let (mut sum, mut n) = (0.0f64, 0u32);
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for t in 1..=1200u64 {
        bridge.dispatch(&mut sim, true, t);
        let y = y_of(&sim);
        if t > 800 {
            sum += f64::from(y);
            n += 1;
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32, lo, hi)
}

/// A fração da cápsula abaixo de `y = 0`, dado o centro em `y`.
///
/// ⚠️ Aproximação por FATIAS, deliberada: a sonda quer o número que o artista vê
/// (*"quanto do corpo está debaixo d'água"*), e re-derivar o recorte exato do
/// `buoyancy.rs` aqui seria a segunda cópia de uma pergunta que já tem dono.
fn submerged_fraction(y: f32) -> f32 {
    const SLICES: u32 = 4096;
    let mut wet = 0.0f32;
    let mut all = 0.0f32;
    for i in 0..SLICES {
        #[allow(clippy::cast_precision_loss)]
        let local = -TOTAL_H / 2.0 + TOTAL_H * (i as f32 + 0.5) / SLICES as f32;
        // Meia-largura da cápsula na altura local.
        let half_w = if local.abs() <= HALF_H {
            RADIUS
        } else {
            let d = local.abs() - HALF_H;
            (RADIUS * RADIUS - d * d).max(0.0).sqrt()
        };
        let strip = 2.0 * half_w;
        all += strip;
        if y + local < 0.0 {
            wet += strip;
        }
    }
    if all <= 0.0 { 0.0 } else { wet / all }
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_float_line() {
    let area = capsule_area();
    println!("\n=== A LINHA DE FLUTUACAO ===");
    println!(
        "capsula: altura {TOTAL_H:.3} m, area {area:.4} m^2 (retangulo + disco das calotas)\n"
    );

    println!(
        "| modo      | nado | dens | fluido | razao | y medio | osc    | submerso | previsto |"
    );
    println!(
        "|-----------|------|------|--------|-------|---------|--------|----------|----------|"
    );
    for kinematic in [false, true] {
        let modo = if kinematic { "kinematic" } else { "dynamic  " };
        for swim in [0.0f32, 4.0] {
            for (density, fluid) in [
                (1.0f32, 4.0f32),
                (1.0, 2.0),
                (1.0, 1.25),
                (1.0, 1.0),
                (2.0, 4.0),
            ] {
                let (y, lo, hi) = settles(kinematic, swim, density, fluid);
                let frac = submerged_fraction(y);
                let predicted = (density / fluid).min(1.0);
                let ratio = fluid / density;
                println!(
                    "| {modo} | {swim:4.1} | {density:4.1} | {fluid:6.2} | {ratio:5.2} | {y:7.3} | {:6.4} | {:7.1}% | {:7.1}% |",
                    hi - lo,
                    frac * 100.0,
                    predicted * 100.0
                );
            }
        }
    }
    println!();
}
