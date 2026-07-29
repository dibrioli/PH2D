//! **O que a POLIA custa** (W-Pulley) — o item do §10 do plano que pedia *"custo
//! por sub-passo × nº de roldanas, contra o HR-4"*, e que nunca tinha número.
//!
//! O HR-4 dá **1,5 ms** a *Physics rígidos (rapier)* num frame de 16,6 ms. A
//! pergunta que importa não é "a polia é rápida" — é **quantas roldanas o artista
//! pode acrescentar antes de a corda comer o orçamento**, porque o botão `Add
//! Wheel` não tem cap e o §0 do CLAUDE.md proíbe escrever um sem medir.
//!
//! Roda pelo caminho do PRODUTO (`PhysicsWorld::step`, com o passe onde ele de
//! fato mora): uma sonda que re-implementa o laço fica cega à porta.
//!
//! `cargo test -p ph2d-physics --test measure_pulley_budget -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;
use std::time::Instant;

/// O orçamento do HR-4 para *Physics rígidos*, ms num frame de 60 Hz.
const HR4_RIGID_MS: f64 = 1.5;

/// `ropes` cordas, cada uma com `wheels_each` roldanas em arco sobre os dois
/// corpos dela. As roldanas ficam ACIMA e ESPALHADAS, nunca sobrepostas — uma
/// rota degenerada sai por `continue` e mediria o custo de recusar, não o de
/// resolver.
fn scene(ropes: usize, wheels_each: usize) -> (PhysicsWorld, Vec<PulleyDesc>) {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    let mut descs = Vec::new();
    let mut arena: Vec<RopeWheel> = Vec::new();
    for k in 0..ropes {
        let x0 = k as f32 * 12.0;
        let (a, _) = w.add_dynamic_circle(x0 - 1.5, 2.0, R, 3.0 / area);
        let (b, _) = w.add_dynamic_circle(x0 + 1.5, 2.0, R, 1.0 / area);
        let start = u32::try_from(arena.len()).expect("arena cabe em u32");
        for i in 0..wheels_each {
            // Um arco de roldanas de x0−1,5 a x0+1,5, todas em y = 6, com folga
            // entre elas: `2·radius` de espaçamento mínimo garante que nenhuma
            // engole a vizinha.
            let t = if wheels_each == 1 {
                0.5
            } else {
                i as f32 / (wheels_each - 1) as f32
            };
            arena.push(RopeWheel {
                centre: [x0 - 1.5 + 3.0 * t, 6.0],
                radius: 0.3,
                side: 1,
                id: u64::try_from(k + 1).expect("id cabe"),
                break_force: f32::INFINITY,
                ..RopeWheel::default()
            });
        }
        descs.push(PulleyDesc {
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: start,
            wheel_count: u32::try_from(wheels_each).expect("conta cabe"),
            id: u64::try_from(k + 1).expect("id cabe"),
            // Longa o bastante para a corda nascer FOLGADA em qualquer contagem:
            // o custo do passe não depende de ela estar esticada, e uma corda
            // esticada demais no tique 0 mediria o transiente.
            total_length: 40.0,
            motor_rate: 0.0,
            break_force: f32::INFINITY,
        });
    }
    // Os lados vêm da MESMA porta que a ponte usa.
    for d in &descs {
        let s = d.wheel_start as usize;
        let n = d.wheel_count as usize;
        let (a, b) = (
            [(d.body_a.into_raw_parts().0 as f32) * 0.0 - 1.5, 2.0],
            [1.5, 2.0],
        );
        ph2d_physics::world::rope_route::resolve_sides(a, b, &mut arena[s..s + n], &mut Vec::new());
    }
    w.set_pulleys(descs.clone(), arena);
    (w, descs)
}

/// **O controle: os MESMOS corpos, sem corda nenhuma.**
///
/// ⚠️ Construído pela MESMA porta e cronometrado pelo MESMO laço dos casos com
/// corda. A 1ª versão desta sonda reusava UM mundo entre as corridas do controle e
/// reconstruía a cena nos casos com corda — o controle saiu **mais CARO** que uma
/// corda de 2 roldanas (0,0063 contra 0,0034) e a coluna de delta veio NEGATIVA.
/// O controle atropelado pelo experimento, pela quinta vez nesta linha.
fn bare(ropes: usize) -> PhysicsWorld {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    for k in 0..ropes {
        let x0 = k as f32 * 12.0;
        w.add_dynamic_circle(x0 - 1.5, 2.0, R, 3.0 / area);
        w.add_dynamic_circle(x0 + 1.5, 2.0, R, 1.0 / area);
    }
    w
}

/// ms por tique de `step()`, mediana de `runs` corridas de 60 tiques, **com o mundo
/// reconstruído a cada corrida**.
///
/// ⚠️ **Mediana, e a 1ª corrida descartada:** buffers recém-alocados pagam
/// *first-touch* e o alocador tem memória entre chamadas — a lição que a sonda do
/// pen-up do Painter pagou com um número que não reproduzia.
fn ms_of(runs: usize, build: impl Fn() -> PhysicsWorld) -> f64 {
    let mut samples = Vec::new();
    for run in 0..=runs {
        let mut w = build();
        let t0 = Instant::now();
        for _ in 0..60 {
            w.step();
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / 60.0;
        if run > 0 {
            samples.push(ms);
        }
    }
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

fn ms_per_tick(ropes: usize, wheels_each: usize, runs: usize) -> f64 {
    ms_of(runs, || scene(ropes, wheels_each).0)
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_a_rope_costs_per_wheel() {
    println!("\n=== UMA corda, custo por tique × nº de roldanas ===");
    println!(
        "{:>8} | {:>12} | {:>12} | {:>9}",
        "roldanas", "ms/tique", "delta/roldana", "% HR-4"
    );
    let control = ms_of(4, || bare(1));
    println!(
        "{:>8} | {control:>12.4} | {:>12} | {:>8.2}%",
        "(sem corda)",
        "-",
        100.0 * control / HR4_RIGID_MS
    );
    for n in [1_usize, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
        let ms = ms_per_tick(1, n, 4);
        println!(
            "{n:>8} | {ms:>12.4} | {:>12.5} | {:>8.2}%",
            (ms - control) / n as f64,
            100.0 * ms / HR4_RIGID_MS
        );
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_how_many_ropes_fit_in_the_budget() {
    println!("\n=== MUITAS cordas de 2 roldanas (a montagem comum) ===");
    println!("{:>7} | {:>12} | {:>9}", "cordas", "ms/tique", "% HR-4");
    for k in [1_usize, 4, 16, 64, 128, 256] {
        let ms = ms_per_tick(k, 2, 3);
        println!("{k:>7} | {ms:>12.4} | {:>8.2}%", 100.0 * ms / HR4_RIGID_MS);
    }
    println!("\n=== E de 8 roldanas (uma talha composta) ===");
    println!("{:>7} | {:>12} | {:>9}", "cordas", "ms/tique", "% HR-4");
    for k in [1_usize, 4, 16, 64] {
        let ms = ms_per_tick(k, 8, 3);
        println!("{k:>7} | {ms:>12.4} | {:>8.2}%", 100.0 * ms / HR4_RIGID_MS);
    }
    println!();
}
