//! **A SONDA da quina** (W10) — mede antes de o número ser escrito.
//!
//! Três perguntas, e nenhuma delas tem resposta por raciocínio:
//!
//! 1. **Que sobreposição a assistência de fato salva?** O `corner_reach` é uma
//!    distância, e o que o artista precisa saber é *até que profundidade de
//!    encosto o pulo passa* — não uma fração da largura do corpo escolhida no
//!    papel.
//! 2. **Onde ela PARA de salvar?** Um teto de verdade tem de continuar a barrar,
//!    senão a assistência é teletransporte.
//! 3. **Quanto custam os raios?** São [`CORNER_SAMPLES`] + 2 por tique de
//!    subida, e o módulo tem orçamento (HR-4).
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_corner -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, PlayerInput, RigidBody,
};
use ph2d_platformer::CORNER_SAMPLES;
use scene_fixture::{FLOAT_HEIGHT, pose, scene};

/// A meia-largura da cápsula da fixture — `radius`, porque a caixa dela é
/// `2·radius` de largura.
const HALF_W: f32 = 0.2;

/// Monta a cena e devolve `(sim, bridge, player)`: o chão da fixture, uma
/// beirada cuja borda ESQUERDA está em `edge`, e o personagem em x = 0.
///
/// Com `edge = HALF_W − clip` a beirada cobre `clip` metros do lado direito da
/// cabeça — a quina canônica, e o escape é para a esquerda.
fn ledge_scene(clip: f32, reach: f32) -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    let edge = HALF_W - clip;
    // A beirada: 4 m de comprimento a partir de `edge`, com a face de baixo
    // 1,2 m acima da cabeça do personagem em repouso.
    let half_x = 2.0;
    let under = FLOAT_HEIGHT + 0.5 + 1.2;
    sim.world_mut().spawn((
        Name::new("Ledge"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(edge + half_x, under + 0.5)),
    ));
    let mut q = sim
        .world_mut()
        .try_query::<(&Name, &mut PlatformPlayer)>()
        .expect("a query do player");
    for (n, mut p) in q.iter_mut(sim.world_mut()) {
        if n.as_str() == "Player" {
            p.corner_reach = reach;
        }
    }
    (sim, bridge, player)
}

/// Pula e devolve `(altura máxima alcançada, deslocamento horizontal total)`.
fn jump_and_watch(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    player: ph2d_ecs::Entity,
) -> (f32, f32) {
    let (x0, y0) = pose(sim);
    let mut tick = 0_u64;
    // Um tique parado para o BVH indexar (o contrato do `world::cast`).
    tick += 1;
    bridge.dispatch(sim, true, tick);
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: false,
            dash: false,
            grab: false,
        },
    );
    let mut peak = y0;
    let mut drift = 0.0_f32;
    for k in 0..90 {
        tick += 1;
        bridge.dispatch(sim, true, tick);
        if k == 2 {
            // Solta o botão: a altura variável não interessa a esta sonda, e
            // segurar re-dispararia o pulo no pouso.
            bridge.set_player_input(player, PlayerInput::default());
        }
        let (x, y) = pose(sim);
        peak = peak.max(y);
        drift = x - x0;
    }
    (peak - y0, drift)
}

/// **Até que sobreposição a assistência salva o pulo, e onde ela para.**
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_the_corner_assist_saves() {
    // A face de baixo da beirada, medida do repouso do personagem.
    let under_rel = 0.5 + 1.2;
    println!("\n=== A QUINA: o que o alcance salva (beirada a {under_rel:.2} m da cabeca) ===");
    println!(
        "{:>8} | {:>14} | {:>14} | {:>10}",
        "encosto", "pico SEM", "pico COM", "desvio x"
    );
    for step in 0..=10 {
        let clip = 0.02 * step as f32;
        let (mut s0, mut b0, p0) = ledge_scene(clip, 0.0);
        let (peak_off, _) = jump_and_watch(&mut s0, &mut b0, p0);
        let (mut s1, mut b1, p1) = ledge_scene(clip, 0.12);
        let (peak_on, drift) = jump_and_watch(&mut s1, &mut b1, p1);
        println!("{clip:>8.3} | {peak_off:>14.3} | {peak_on:>14.3} | {drift:>10.3}");
    }
    println!(
        "(a meia-largura do corpo e' {HALF_W}, entao 'encosto' 0,20 e' a cabeca INTEIRA tapada)"
    );
}

/// **Quanto custa perguntar.** Os raios só correm subindo, e só com alcance.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_the_probe_costs() {
    println!(
        "\n=== O CUSTO do LEQUE ({CORNER_SAMPLES} raios + 2 laterais por tique de subida) ==="
    );
    // ⚠️ **O rótulo abaixo mede o LEQUE, não o sensor inteiro.** Desde a
    // `W-Ceiling` a linha `reach 0` NÃO significa *"sensor nenhum"*: o fato do
    // teto (`head_blocked`) é uma varredura sem knob e roda nas duas linhas, por
    // ser um FATO e não uma assistência. A diferença aqui é, e sempre foi, o
    // preço da ASSISTÊNCIA — e deixá-la rotulada como *"o sensor inteiro"*
    // faria a próxima leitura precificar a coisa errada.
    println!("     (a varredura do FATO roda nas DUAS linhas -- ela nao tem knob)");
    let mut rows = Vec::new();
    for (name, reach) in [
        ("so' o fato (reach 0)", 0.0_f32),
        ("fato + leque (0,12)", 0.12),
    ] {
        // A mesma cena, o mesmo pulo — só o alcance muda.
        let (mut sim, mut bridge, player) = ledge_scene(0.05, reach);
        // Aquece.
        let mut tick = 0_u64;
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 0.0,
                jump: true,
                down: false,
                dash: false,
                grab: false,
            },
        );
        let t = Instant::now();
        for _ in 0..600 {
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
            // Re-pula sempre que possível, para MAXIMIZAR os tiques de subida.
        }
        let per = t.elapsed().as_secs_f64() * 1000.0 / 600.0;
        rows.push((name, per));
    }
    for (n, ms) in &rows {
        println!("{n:>22}: {ms:.4} ms/tique");
    }
    let delta = rows[1].1 - rows[0].1;
    println!("            diferenca: {delta:+.4} ms/tique (SO' o leque)");
}

/// **A CHAMINÉ** — a geometria da cena de smoke, medida antes de ser escrita.
///
/// Duas prateleiras deixam um vão de `gap`; o personagem pula de um desvio `off`
/// do centro. A pergunta é *quanto o alcance ALARGA a janela em que o pulo
/// passa* — que é o que o artista vai de fato julgar.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_chimney_window() {
    for gap in [0.5_f32, 0.6, 0.7] {
        let slack = (gap - 2.0 * HALF_W) * 0.5;
        println!("\n=== CHAMINE de {gap:.2} m (folga geometrica {slack:.3} m de cada lado) ===");
        println!("{:>8} | {:>12} | {:>12}", "desvio", "reach 0", "reach 0,12");
        for step in 0..=8 {
            let off = 0.03 * step as f32;
            let mut row = [0.0_f32; 2];
            for (slot, reach) in [0.0_f32, 0.12].iter().enumerate() {
                let (mut sim, bridge, player) = scene(0.0, 0.0);
                let under = FLOAT_HEIGHT + 0.5 + 1.2;
                for (name, cx) in [
                    ("ShelfL", -(gap * 0.5) - 3.0 - off),
                    ("ShelfR", (gap * 0.5) + 3.0 - off),
                ] {
                    sim.world_mut().spawn((
                        Name::new(name),
                        RigidBody {
                            kind: BodyKind::Static,
                        },
                        Collider {
                            shape: ColliderShape::Cuboid {
                                half_x: 3.0,
                                half_y: 0.5,
                            },
                            ..Collider::default()
                        },
                        Transform::from_translation(Vec2::new(cx, under + 0.5)),
                    ));
                }
                let mut q = sim
                    .world_mut()
                    .try_query::<(&Name, &mut PlatformPlayer)>()
                    .expect("a query do player");
                for (n, mut p) in q.iter_mut(sim.world_mut()) {
                    if n.as_str() == "Player" {
                        p.corner_reach = *reach;
                    }
                }
                let mut r = (sim, bridge, player);
                row[slot] = jump_and_watch(&mut r.0, &mut r.1, r.2).0;
            }
            // ⚠️ O pulo desta sonda e' CORTADO (o botao e' solto no 3o tique),
            // entao ele nunca atravessa a chamine — o oraculo e' o PICO: 0,833 e'
            // o pulo livre, e qualquer coisa abaixo disso e' a cabeca a bater.
            let mark = |p: f32| if p > 0.83 { "livre" } else { "bate " };
            println!(
                "{off:>8.2} | {:>6.3} {} | {:>6.3} {}",
                row[0],
                mark(row[0]),
                row[1],
                mark(row[1])
            );
        }
    }
}
