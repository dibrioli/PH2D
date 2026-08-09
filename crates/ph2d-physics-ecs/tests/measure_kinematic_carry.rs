//! **QUANTO A PLATAFORMA LEVA** — a sonda que localiza a contagem dupla do K7.
//!
//! A sonda da W-KinPure mediu, de passagem, uma plataforma a andar **4,0000 m**
//! e um personagem cinemático a ser levado **7,9194** — quase o dobro, enquanto
//! o dinâmico media exacto. O gate que vigiava isto (`..._rides_a_moving_
//! platform`) pede `travelled > 3.0`: uma **barra de um lado só**, tão contente
//! com 8 quanto com 4, e é por isso que o defeito viveu.
//!
//! Esta sonda separa o que a barra não separava, em três eixos:
//!
//! 1. **a DIREÇÃO da plataforma** — o vagão horizontal e o elevador vertical,
//!    porque a lei da caminhada só age ao longo da TANGENTE do chão: se a
//!    contagem dupla vier dela, o elevador tem de sair EXACTO;
//! 2. **o MODO** — o dinâmico é o controle, e ele mede a mesma lei sem passar
//!    pelo integrador cinemático;
//! 3. **o que ele POSSUI** — a plataforma pára de repente e mede-se o que ele
//!    continua a andar. É o teste do doc-comment do `kinematic_advance`, que
//!    afirma que a velocidade do chão *não entra na velocidade guardada*.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --test measure_kinematic_carry
//! --release -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod platform;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerMode,
    RigidBody,
};

use platform::FLOAT_HEIGHT;

/// Quantos tiques a plataforma anda, e a que velocidade.
const RIDE_TICKS: u64 = 120;
const PLATFORM_SPEED: f32 = 2.0;

/// ⚠️ **A plataforma é LONGA de propósito.** A 1ª versão usava meia-largura 3,0
/// e o personagem — levado 7,02 m por um vagão que andou 4,00 — **saiu de cima
/// dela**; a coluna "após-parar" media então um deslize sem chão, não o que ele
/// possui. É o controle atropelado pelo experimento, a 5ª vez nesta linha.
const WAGON_HALF: f32 = 30.0;

/// A cena: uma plataforma dirigida pela CENA e um personagem em cima dela.
///
/// `traction` é o knob **do artista** (`PlatformPlayer::acceleration`): é a
/// ablação que decide se a lei da caminhada é a segunda contagem.
fn scene(kinematic: bool, traction: f32) -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    let wagon = sim
        .world_mut()
        .spawn((
            Name::new("Wagon"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: WAGON_HALF,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let who = sim
        .world_mut()
        .spawn((
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
                float_height: FLOAT_HEIGHT,
                acceleration: traction,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.25 + FLOAT_HEIGHT)),
        ))
        .id();
    if kinematic {
        let mut e = sim.world_mut().entity_mut(who);
        e.insert(PlayerMode::Kinematic);
        if let Some(mut rb) = e.get_mut::<RigidBody>() {
            rb.kind = BodyKind::Kinematic;
        }
    }
    let bridge = PhysicsBridge::new();
    (sim, bridge, wagon, who)
}

fn nudge(sim: &mut SimWorld, wagon: Entity, d: Vec2) {
    let mut e = sim.world_mut().entity_mut(wagon);
    if let Some(mut tr) = e.get_mut::<Transform>() {
        tr.translation.x += d.x;
        tr.translation.y += d.y;
    }
}

fn player_pos(sim: &SimWorld, who: Entity) -> Vec2 {
    sim.world()
        .get::<Transform>(who)
        .map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.y))
}

fn wagon_pos(sim: &SimWorld, wagon: Entity) -> Vec2 {
    sim.world()
        .get::<Transform>(wagon)
        .map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.y))
}

/// Uma corrida: assenta, anda `RIDE_TICKS` com a plataforma, PÁRA a plataforma,
/// e deixa correr mais meio segundo.
///
/// Devolve `(plataforma, levado, deriva_depois_de_parar)`.
fn ride(kinematic: bool, axis: Vec2, traction: f32) -> (f32, f32, f32) {
    let (mut sim, mut bridge, wagon, who) = scene(kinematic, traction);
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p0 = player_pos(&sim, who);
    let w0 = wagon_pos(&sim, wagon);
    let step = Vec2::new(
        axis.x * PLATFORM_SPEED / 60.0,
        axis.y * PLATFORM_SPEED / 60.0,
    );
    for t in 61..=(60 + RIDE_TICKS) {
        nudge(&mut sim, wagon, step);
        bridge.dispatch(&mut sim, true, t);
    }
    let p1 = player_pos(&sim, who);
    let w1 = wagon_pos(&sim, wagon);
    // A plataforma PÁRA. O que ele continuar a andar é o que ele POSSUI.
    for t in (61 + RIDE_TICKS)..=(90 + RIDE_TICKS) {
        bridge.dispatch(&mut sim, true, t);
    }
    let p2 = player_pos(&sim, who);
    let along = |a: Vec2, b: Vec2| (b.x - a.x) * axis.x + (b.y - a.y) * axis.y;
    (along(w0, w1), along(p0, p1), along(p1, p2))
}

#[test]
#[ignore = "sonda de medicao"]
fn measure_the_carry() {
    println!("\n=== QUANTO A PLATAFORMA LEVA ===");
    println!("plataforma a {PLATFORM_SPEED} m/s por {RIDE_TICKS} tiques, e depois PARA\n");
    let full = ph2d_physics_ecs::PlatformPlayer::default().acceleration;
    println!("eixo         modo         tracao  plataforma(m)  levado(m)  razao   apos-parar(m)");
    for (name, axis) in [
        ("horizontal", Vec2::new(1.0, 0.0)),
        ("vertical  ", Vec2::new(0.0, 1.0)),
    ] {
        for (mode, kin) in [("dinamico  ", false), ("CINEMATICO", true)] {
            for (tag, tr) in [("cheia", full), ("ZERO ", 0.0)] {
                let (plat, got, coast) = ride(kin, axis, tr);
                let ratio = if plat.abs() > 1e-6 { got / plat } else { 0.0 };
                println!(
                    "{name}   {mode}   {tag}   {plat:>10.4}   {got:>9.4}   {ratio:>5.2}x  {coast:>10.4}"
                );
            }
        }
    }
    println!(
        "\nLEITURA: razao ~1.00x e' carregar; ~2.00x e' contar duas vezes.\n\
         A coluna 'apos-parar' diz se a velocidade GUARDADA tem a do chao\n\
         dentro dela -- e o doc do `kinematic_advance` afirma que NAO tem.\n\
         TRACAO ZERO desliga a lei da caminhada pela porta do ARTISTA: se a\n\
         2a contagem for dela, a razao cai para 1,00x nessa linha.\n"
    );
}
