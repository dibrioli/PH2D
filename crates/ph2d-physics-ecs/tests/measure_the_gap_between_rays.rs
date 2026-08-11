//! **O QUE PASSA ENTRE DUAS AMOSTRAS** — a sonda da lacuna que a `W-ShapeCast`
//! fecha (plano 08 §4.3).
//!
//! Dois sensores do player leem o mundo com **três raios** e cada um nomeia a
//! mesma limitação por escrito:
//!
//! - `crouch::headroom_offsets` — *"um teto mais estreito que meia largura,
//!   entre duas amostras, não é visto … a cura para os três seria a mesma (um
//!   shape cast, que este wrapper ainda não tem)"*;
//! - `wall::wall_offsets` — *"uma fresta mais estreita que meia altura, entre
//!   duas amostras, segue invisível"*.
//!
//! ⚠️ **E o do agachar carrega uma AFIRMAÇÃO sobre a direção do erro** (o doc do
//! `probe_headroom`): *"o erro possível é ficar agachado onde caberia, nunca
//! levantar-se para dentro da pedra"*. Ela é verdade sobre a **caixa envolvente
//! contra a cápsula** — e esta sonda existe para perguntar se ela sobrevive a um
//! obstáculo que cabe ENTRE os raios, que é a outra metade da mesma pergunta.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --test measure_the_gap_between_rays -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// Meia-altura do segmento reto da cápsula do player.
const HALF_H: f32 = 0.3;
/// Raio da cápsula. A caixa envolvente mede `radius` de meia-largura.
const RADIUS: f32 = 0.2;
/// A altura de flutuação DE PÉ.
const FLOAT_HEIGHT: f32 = 1.1;
/// A altura de flutuação AGACHADO.
const CROUCH_HEIGHT: f32 = 0.6;

/// Um chão comprido, um player, e **um PILAR estreito** pendurado em `pillar_x`
/// com meia-largura `pillar_half`, cuja face de baixo fica em `bottom`.
fn rig(pillar_x: f32, pillar_half: f32, bottom: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 60.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));

    const PILLAR_HALF_Y: f32 = 1.0;
    sim.world_mut().spawn((
        Name::new("Pillar"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: pillar_half,
                half_y: PILLAR_HALF_Y,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(pillar_x, bottom + PILLAR_HALF_Y)),
    ));

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: HALF_H,
                    radius: RADIUS,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                crouch_height: CROUCH_HEIGHT,
                corner_reach: 0.0,
                lift_momentum: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, CROUCH_HEIGHT)),
        ))
        .id();

    (sim, PhysicsBridge::new(), player)
}

fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}

fn down() -> PlayerInput {
    PlayerInput {
        down: true,
        ..PlayerInput::default()
    }
}

/// Agacha por `hold` tiques, solta, e corre mais `free`. Devolve `(topo, x)`.
///
/// ⚠️ **O `x` é metade da resposta**, e sem ele a tabela mente: um pilar
/// estreito é uma coisa de que o solver consegue **escorregar para o lado**, e
/// *subiu porque a pedra é invisível* e *subiu porque fugiu dela* são vereditos
/// diferentes sobre o mesmo número de altura.
fn crouch_then_release(
    pillar_x: f32,
    pillar_half: f32,
    bottom: f32,
    hold: u64,
    free: u64,
) -> (f32, f32) {
    let (mut sim, mut bridge, player) = rig(pillar_x, pillar_half, bottom);
    let mut t = 0;
    for _ in 0..hold {
        bridge.set_player_input(player, down());
        t += 1;
        bridge.dispatch(&mut sim, true, t);
    }
    for _ in 0..free {
        bridge.set_player_input(player, PlayerInput::default());
        t += 1;
        bridge.dispatch(&mut sim, true, t);
    }
    let (x, y) = pose(&sim);
    (y + HALF_H + RADIUS, x)
}

/// **O TETO QUE PASSA ENTRE OS RAIOS.**
///
/// A caixa do corpo mede `RADIUS` de meia-largura, então os três raios do
/// headroom nascem em `-0,20 · 0,00 · +0,20`. Um pilar estreito posto em `+0,10`
/// cai exactamente no meio de duas amostras — e o corpo, ali, ainda mede
/// `0,3 + sqrt(0,2² − 0,1²) = 0,473` de meia-altura.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_ceiling_that_fits_between_the_rays() {
    // A face de baixo do pilar, logo acima da cabeça AGACHADA (0,6 + 0,5 = 1,1)
    // e bem abaixo da cabeça DE PÉ (1,1 + 0,5 = 1,6).
    const BOTTOM: f32 = 1.25;
    eprintln!("\n== O TETO ENTRE OS RAIOS (raios em -0.20, 0.00, +0.20) ==");
    eprintln!("(pilar com a face de baixo em {BOTTOM:.2}; de pe' a cabeca chegaria a 1.60)\n");
    eprintln!("| pilar x | meia-larg | cobre um raio? | topo da cabeca | x final | veredito |");
    eprintln!("|---------|-----------|----------------|----------------|---------|----------|");
    for (x, half) in [
        (0.0_f32, 0.05_f32), // sobre o raio do MEIO -> visto
        (0.20, 0.05),        // sobre o raio da BORDA -> visto
        (0.10, 0.04),        // ENTRE dois raios -> ?
        (0.10, 0.02),        // mais estreito ainda
        (-0.10, 0.04),       // o outro vao
        (0.0, 0.30),         // CONTROLE: um teto largo, visto por todos
    ] {
        let covers = [-RADIUS, 0.0, RADIUS].iter().any(|r| (r - x).abs() <= half);
        let (top, fx) = crouch_then_release(x, half, BOTTOM, 60, 120);
        // Ele ainda está SOB o pilar? (a caixa do corpo contra a do pilar)
        let under = (fx - x).abs() < RADIUS + half;
        let verdict = if top <= BOTTOM {
            "ficou baixo"
        } else if under {
            "NA PEDRA"
        } else {
            "fugiu de lado"
        };
        eprintln!(
            "| {x:7.2} | {half:9.2} | {:14} | {top:14.3} | {fx:7.3} | {verdict:8} |",
            if covers { "sim" } else { "NAO" },
        );
    }
    eprintln!(
        "\nLEITURA: 'NA PEDRA' significa que a cabeca subiu ATRAVES do pilar e o solver\n\
         a segura la' dentro -- exactamente o erro que o doc do probe_headroom declara\n\
         impossivel. 'fugiu de lado' e' o solver a expulsar o corpo de sob uma pedra\n\
         estreita demais para o segurar: sobe, mas nao onde o artista o deixou.\n"
    );
}

/// **A PAREDE COM FRESTA** — o outro sensor de três amostras.
///
/// Os raios do flanco nascem em `0 · −h · +h` (a cintura e as duas bordas da
/// caixa). Uma parede feita de duas lajes com um vão no meio pode deixar os três
/// raios passarem enquanto o corpo ainda encosta.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_wall_with_a_slot() {
    eprintln!("\n== a fresta da parede: quantos dos 3 raios do flanco a atravessam ==");
    eprintln!("(raios em 0.00, -0.50, +0.50 relativos ao centro do corpo)\n");
    for slot in [0.10_f32, 0.30, 0.60, 1.10] {
        let hits = [0.0_f32, -0.5, 0.5]
            .iter()
            .filter(|o| o.abs() > slot * 0.5)
            .count();
        eprintln!("  fresta de {slot:4.2} m centrada na cintura -> {hits} de 3 raios veem parede");
    }
    eprintln!();
}
