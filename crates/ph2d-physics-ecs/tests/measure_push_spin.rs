//! **O CAIXOTE GIRA DEMAIS QUANDO O CINEMÁTICO O EMPURRA** — o report do Enio
//! de 2026-08-09 (*"se o caixote não tiver com rotação travada, o contato com o
//! player kinematic causa rotação exagerada"*).
//!
//! ⚠️ **A wave já sabia e escondeu na fixture:** a cena `=102` trava a rotação
//! dos caixotes com um comentário que o diz — *"sem isto os caixotes TOMBAM ao
//! serem empurrados e a cena passa a medir tombo, não empurrão"*. Era a decisão
//! certa para aquela cena (ela mede o EMPURRÃO) e a errada para o módulo: um
//! defeito contornado por uma fixture não é um defeito nomeado.
//!
//! # O que se mede, e por quê assim
//!
//! ⚠️ **O ângulo WRAPA em ±π** (`Transform.rotation`), então o que se soma é o
//! delta DESENROLADO tique a tique — a lição que o `W-AreaTorque` pagou, e na
//! qual eu tropecei na 1ª versão desta sonda: ela tomava `max |rotation|` e
//! devolvia `3,12 · 3,14 · 3,12`, ou seja **π saturado** em três linhas, um
//! número que só diz *"deu voltas"*.
//!
//! ⚠️ **E o controle é o modo DINÂMICO**, empurrando o MESMO caixote: sem ele
//! *"girou 74 radianos"* não distingue *o empurrão cinemático é violento* de
//! *um caixote empurrado acima do centro de massa gira, e isso é física*.
//!
//! # O que ela mediu (2026-08-09)
//!
//! ```text
//!   caixote      dinamico    CINEMATICO    alavanca
//!      0.30        0.3175       74.2906       0.600   <- ~12 voltas
//!      0.50        0.0043       12.8937       0.400
//!      0.80        0.0007        0.0418       0.100
//!      1.00        0.0006        0.0021      -0.100
//! ```
//!
//! **O giro segue a ALAVANCA** (a altura do contato acima do centro de massa do
//! caixote), e some quando ela some. ⇒ o mecanismo é o `at` do
//! `apply_impulse_at_point`: o empurrão inteiro do tique entra como **UM
//! impulso num ponto alto**, e `r × F` faz o resto.
//!
//! ⚠️ **A metade LINEAR está calibrada e não é o problema** — a W-KinPush a
//! mediu em `16,54` contra os `16,55` do dinâmico. O que ninguém escolheu foi o
//! TORQUE que veio junto: o dinâmico empurra com uma força sustentada por
//! sub-passo e o cinemático com uma martelada por tique.
//!
//! ⚠️ **A cura NÃO foi construída, e a razão não é preguiça:** a porta
//! (`apply_player_reaction`) é COMPARTILHADA com a reação vertical da W6, onde o
//! ponto é a razão de a jangada INCLINAR sob o personagem — uma feature que a
//! cena `=103` demonstra. Tirar o ponto ali é tirar a inclinação; mantê-lo e
//! filtrar só o empurrão lateral é uma decisão de produto (*um caixote empurrado
//! no peito TOMBA na vida real*), e a §8.2 do plano 07 a escreve com as opções
//! e o preço de cada uma.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_push_spin
//! -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    PlayerMode, RigidBody,
};

const HALF_HEIGHT: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOOR_TOP: f32 = 0.25;
const FLOAT: f32 = 0.9;
/// Onde o caixote fica. O personagem nasce à esquerda e anda para a direita.
const CRATE_X: f32 = 2.0;

/// `(andou, pico de |omega| em rad/s, altura do contato acima do centro do
/// caixote)`.
///
/// `crate_half` é a meia-extensão do caixote: é ela que decide onde o meio da
/// cápsula bate, e portanto o braço de alavanca.
fn push(kinematic: bool, crate_half: f32) -> (f32, f32, f32) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: FLOOR_TOP,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // ⚠️ **SEM `LockRotation`** — é exatamente o que a cena `=102` trava, e é o
    // caso que o Enio reportou.
    let boxy = sim
        .world_mut()
        .spawn((
            Name::new("Crate"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: crate_half,
                    half_y: crate_half,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(CRATE_X, FLOOR_TOP + crate_half)),
        ))
        .id();
    let mut e = sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
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
            float_height: FLOAT,
            speed: 3.0,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP + FLOAT)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    let who = e.id();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=90u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x0 = crate_x(&sim, boxy);
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    // O centro da cápsula em repouso, e o do caixote: a diferença de altura é o
    // braço de alavanca que um empurrão horizontal converte em torque.
    let lever = (FLOOR_TOP + FLOAT - (HALF_HEIGHT + RADIUS) + (HALF_HEIGHT + RADIUS))
        - (FLOOR_TOP + crate_half);
    // ⚠️ **O ângulo é DESENROLADO tique a tique**, e isto é a correção de um
    // erro meu: a 1ª versão desta sonda tomava `max |rotation|` — exatamente o
    // que o aviso do módulo proíbe — e devolvia `3,12 · 3,14 · 3,12` em três
    // linhas, ou seja **π saturado**, que é o mesmo que dizer *"deu voltas"* e
    // não ordena nada. O total desenrolado ordena.
    let mut total = 0.0_f32;
    let mut prev = spin(&sim, boxy);
    for t in 91..=270u64 {
        bridge.dispatch(&mut sim, true, t);
        let now = spin(&sim, boxy);
        let mut d = now - prev;
        while d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        while d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        total += d.abs();
        prev = now;
    }
    (crate_x(&sim, boxy) - x0, total, lever)
}

fn crate_x(sim: &SimWorld, who: Entity) -> f32 {
    sim.world()
        .get::<Transform>(who)
        .map_or(0.0, |t| t.translation.x)
}

/// O ângulo do caixote, cru — quem o desenrola é o laço que o consome.
fn spin(sim: &SimWorld, who: Entity) -> f32 {
    sim.world()
        .get::<Transform>(who)
        .map_or(0.0, |t| t.rotation)
}

#[test]
#[ignore = "sonda"]
fn measure_how_much_the_crate_spins() {
    println!("\n=== O CAIXOTE SEM ROTACAO TRAVADA, EMPURRADO ===");
    println!("(a capsula tem meia-extensao 0,50 e o centro dela fica a 0,90 do chao)\n");
    println!(
        "{:>12}  {:>12}  {:>12}  {:>14}  {:>10}",
        "caixote", "modo", "andou (m)", "girou (rad)", "alavanca"
    );
    // ⚠️ **A partir de 0,3**: com meia-extensão 0,2 o topo do caixote fica
    // exactamente na sola da cápsula e o personagem SOBE nele em vez de o
    // empurrar — a fixture mediria uma escalada e chamaria isso de empurrão.
    for half in [0.3_f32, 0.5, 0.8, 1.0] {
        for (tag, kin) in [("dinamico", false), ("CINEMATICO", true)] {
            let (ran, peak, lever) = push(kin, half);
            println!("{half:>12.2}  {tag:>12}  {ran:>12.4}  {peak:>14.4}  {lever:>10.3}");
        }
    }
    println!(
        "\nLEITURA: se o CINEMATICO girar muito mais que o dinamico para o mesmo\n\
         deslocamento, o empurrao esta' a ser aplicado no lugar errado ou com a\n\
         magnitude errada. Se os dois girarem parecido, girar E' a fisica de\n\
         empurrar um caixote acima do centro de massa dele, e o que a cena =102\n\
         escondeu foi o normal.\n"
    );
}

/// **O CAIXOTE AINDA RODOPIA, E ESTE É O NÚMERO DELE** (report do Enio,
/// 2026-08-09).
///
/// ⚠️ **Um gate VERDE sobre um defeito**, o precedente do
/// `the_documented_hardening_is_still_there_and_this_is_its_number` do Painter e
/// do irmão do salto da descida: ele não diz que o produto está certo — ele
/// impede o número de CRESCER em silêncio e obriga a cura a atualizá-lo.
///
/// ⚠️ **A afirmação é uma RAZÃO contra o controle dinâmico, não um valor**: um
/// caixote a dar doze voltas é caótico, e pinar `74,29` seria um gate que
/// qualquer mudança sem relação derruba. A razão é robusta e diz a mesma coisa.
#[test]
fn the_pushed_crate_still_tumbles_and_this_is_its_number() {
    let (ran_k, spun_k, _) = push(true, 0.3);
    let (ran_d, spun_d, _) = push(false, 0.3);
    assert!(
        ran_k > 1.0 && ran_d > 1.0,
        "a fixture tem de CONTER o empurrao nos dois modos: {ran_k:.2} e {ran_d:.2} m"
    );
    assert!(
        spun_d < 1.0,
        "o CONTROLE mal gira ({spun_d:.4} rad) -- se ele passar a girar, a \
         fixture deixou de isolar o modo"
    );
    assert!(
        spun_k > 50.0 * spun_d.max(1.0e-3),
        "o cinematico girava 234x o dinamico (74,29 contra 0,3175) e agora gira \
         {:.1}x ({spun_k:.4} contra {spun_d:.4}) -- se DIMINUIU, alguem curou o \
         defeito e este gate tem de ser reescrito",
        spun_k / spun_d.max(1.0e-3)
    );
}
