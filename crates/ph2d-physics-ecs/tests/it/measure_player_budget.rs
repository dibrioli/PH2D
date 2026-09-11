//! **O QUE UM PERSONAGEM CUSTA** — o item 1 da §7 do plano 07, e o único da
//! tabela de números que ainda não tinha sido perguntado.
//!
//! O modo cinemático troca o par *cast + solver* por um **`move_shape`** (o
//! sweep do controlador do rapier) por player por tique, e a §7 diz, com todas
//! as letras, que esse custo se mede *contra o cast+solver de hoje*. Ninguém o
//! tinha medido — e ele decide se o modo que este módulo **recomenda para
//! plataforma** tem teto de contagem.
//!
//! # ⚠️ Três decisões de fixture, e cada uma tem um modo de falha
//!
//! 1. **Os players não se veem.** Cada um tem chão próprio, longe dos outros —
//!    senão eu mediria colisão player-contra-player, que é `O(N²)` e não é a
//!    pergunta. O gate de FORMA abaixo é o que prova que a fixture é linear.
//! 2. **Eles ANDAM** (`drive = 1`). Um personagem parado toma ramos mais curtos
//!    (o freio da caminhada satura, o motor devolve boost zero), e medir o
//!    repouso responderia *quanto custa não fazer nada*.
//! 3. **A janela é medida depois de assentar**, e o redutor é a **MEDIANA** —
//!    esta máquina é compartilhada, e o mínimo pegaria o tique que não emitiu
//!    trabalho enquanto o máximo pegaria o escalonador do SO.
//!
//! ⚠️ **A RAZÃO é o número que sobrevive à máquina**; o absoluto contra o
//! orçamento do HR-4 (1,5 ms para physics rígidos a 60 Hz) vale para ESTA
//! máquina e está aqui para dar ordem de grandeza, não para virar um teto.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_player_budget
//! -- --ignored --nocapture`

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    PlayerMode, RigidBody,
};

/// O orçamento que o HR-4 dá aos rígidos, a 60 Hz.
const HR4_RIGID_MS: f64 = 1.5;

/// Quanto os corredores ficam afastados — folgado o bastante para o `broad
/// phase` nunca pôr dois players no mesmo par.
const LANE_PITCH: f32 = 60.0;

fn scene(count: u32, kinematic: bool) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    for i in 0..count {
        let x = i as f32 * LANE_PITCH;
        sim.world_mut().spawn((
            Name::new(format!("Floor {i}")),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 25.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ));
        let mut e = sim.world_mut().spawn((
            Name::new(format!("Player {i}")),
            RigidBody {
                kind: if kinematic {
                    BodyKind::Kinematic
                } else {
                    BodyKind::Dynamic
                },
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
                float_height: 0.9,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(x - 20.0, 1.4)),
        ));
        if kinematic {
            e.insert(PlayerMode::Kinematic);
        }
    }
    (sim, PhysicsBridge::new())
}

/// A mediana do custo de um `dispatch`, em milissegundos, com todos a ANDAR.
fn ms_per_tick(count: u32, kinematic: bool) -> f64 {
    let (mut sim, mut bridge) = scene(count, kinematic);
    // Assenta (sem entrada), depois arma o dedo em todos.
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let ids: Vec<_> = {
        let mut q = sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &PlatformPlayer)>();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    for id in &ids {
        bridge.set_player_input(
            *id,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
    }
    let mut samples: Vec<f64> = Vec::with_capacity(120);
    for t in 61..=180u64 {
        let t0 = Instant::now();
        bridge.dispatch(&mut sim, true, t);
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

#[test]
#[ignore = "sonda"]
fn measure_what_a_player_costs() {
    println!("\n=== O QUE UM PERSONAGEM CUSTA POR TIQUE (todos a ANDAR) ===");
    println!("orcamento do HR-4 para rigidos, 60 Hz: {HR4_RIGID_MS} ms\n");
    println!(
        "{:>7}  {:>12}  {:>12}  {:>8}  {:>14}  {:>14}",
        "N", "dinamico ms", "cinematico ms", "razao", "din us/player", "cin us/player"
    );
    for n in [1u32, 10, 50, 100, 200] {
        let d = ms_per_tick(n, false);
        let k = ms_per_tick(n, true);
        println!(
            "{n:>7}  {d:>12.4}  {k:>12.4}  {:>8.2}x  {:>14.2}  {:>14.2}",
            k / d,
            d * 1000.0 / f64::from(n),
            k * 1000.0 / f64::from(n)
        );
    }
    // Os dois numeros DERIVADOS, que sao a pergunta da §7.1 propriamente dita:
    // o que o `move_shape` custa A MAIS que o cast+solver, e quantos cabem.
    let d = ms_per_tick(200, false) * 1000.0 / 200.0;
    let k = ms_per_tick(200, true) * 1000.0 / 200.0;
    println!(
        "\nA §7.1 propriamente dita (medido a N = 200, onde o custo fixo do\n\
         dispatch ja' esta amortizado):\n\
         \x20 o `move_shape` custa {:.2} us/player A MAIS que o cast+solver ({:+.0}%)\n\
         \x20 e cabem ~{:.0} personagens cinematicos no orcamento de {HR4_RIGID_MS} ms\n\
         \x20 (contra ~{:.0} dinamicos) NESTA maquina.",
        k - d,
        (k / d - 1.0) * 100.0,
        HR4_RIGID_MS * 1000.0 / k,
        HR4_RIGID_MS * 1000.0 / d,
    );
    println!(
        "\nLEITURA: a coluna `us/player` tem de ser PLANA em N (a fixture separa\n\
         os corredores, entao o custo e' linear); se ela subir, a fixture esta\n\
         a medir colisao entre players e nao o custo de um. A linha N = 1 e' a\n\
         excecao esperada: ali o custo FIXO de um dispatch nao esta amortizado.\n"
    );
}

/// **O CUSTO DE UM PERSONAGEM É LINEAR EM N** — a fixture mede um player, não
/// a interação entre eles.
///
/// ⚠️ **É gate de FORMA, não de relógio**, e por isso ele sobrevive a esta
/// máquina compartilhada: uma razão entre duas contagens é imune à deriva que
/// um limite em milissegundos apanharia. Se ele falhar, o número que a sonda
/// imprime deixou de ser *"o que um personagem custa"* e passou a ser *"o que
/// N personagens no mesmo corredor custam"*.
#[test]
fn the_cost_of_a_player_is_linear_in_their_number() {
    for kinematic in [false, true] {
        let one = ms_per_tick(10, kinematic);
        let many = ms_per_tick(100, kinematic);
        let ratio = many / one;
        assert!(
            ratio > 3.0,
            "a fixture tem de CONTER o fenomeno: 10x os players deu {ratio:.2}x \
             o custo (kinematic = {kinematic}), e um relogio cego nao mede forma"
        );
        // ⚠️ **O teto é MEDIDO, não escolhido.** Três corridas dão 6,49-6,79×
        // (dinâmico) e 7,42-7,71× (cinemático) — abaixo de 10 porque a N = 10 o
        // custo FIXO de um dispatch ainda pesa. O colapso dos corredores
        // (`LANE_PITCH` de 60 para 0,5) mede **22,08×**, e um teto de 20 deixava
        // 10% de folga entre o verdadeiro e o defeito: apertado demais para uma
        // máquina compartilhada. Em 12 a distância é 1,6× para cada lado.
        assert!(
            ratio < 12.0,
            "10x os players tem de custar ~7x (medido 6,5-7,7), nao {ratio:.2}x \
             (kinematic = {kinematic}) — os corredores deixaram de ser disjuntos"
        );
    }
}
