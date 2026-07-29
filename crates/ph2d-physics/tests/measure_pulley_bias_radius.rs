//! **O `PULLEY_BIAS`, re-medido COM RAIO** — o item do §10 do plano que dizia
//! *"`PULLEY_BIAS` de novo (com raio, a geometria mudou)"*.
//!
//! As tabelas que o `PULLEY_BIAS` cita foram medidas no modelo de **PONTO**
//! (`measure_pulley.rs::point_wheels`, `radius: 0.0`) — e o arquivo é explícito
//! sobre isso, *"é ele que as mantém comparáveis"*. Quando o raio chegou, a
//! geometria da rota mudou: o comprimento passou a incluir os **ARCOS**, e os
//! pontos de tangência **deslizam** quando as âncoras se movem.
//!
//! A pergunta é se a escolha de `β = 0,20` sobrevive a isso. O cabeçalho do módulo
//! da rota tem uma previsão forte — *a derivada de `L` em relação à âncora é
//! exatamente o versor daquele último trecho, porque a variação do arco CANCELA a
//! do trecho (teorema do envelope)* —, então o Jacobiano não deveria mudar. **Uma
//! previsão não é uma medição**, e o §10 pediu a medição.
//!
//! `cargo test -p ph2d-physics --test measure_pulley_bias_radius -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// As duas roldanas COM RAIO, na mesma posição das do modelo de ponto.
fn round_wheels(radius: f32) -> Vec<RopeWheel> {
    let mut wheels = vec![
        RopeWheel {
            centre: [-1.0, 4.0],
            radius,
            side: 1,
            id: 1,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [1.0, 4.0],
            radius,
            side: 1,
            id: 1,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        },
    ];
    // Pela MESMA porta que a ponte usa — cravar os lados faria a fixture descrever
    // uma corda que o produto não monta.
    rope_route::resolve_sides([-1.0, 2.0], [1.0, 2.0], &mut wheels, &mut Vec::new());
    wheels
}

/// A balança de Atwood do `measure_pulley.rs`, com roldanas de raio `radius`.
///
/// ⚠️ **O `total_length` é a rota MEDIDA em repouso**, não o `4.0` cravado da
/// versão de ponto: com raio a corda é mais longa (os arcos entram), e uma corda
/// curta demais nasceria violada — o transiente mediria o piso, não o β.
fn atwood(mass: f32, radius: f32) -> (PhysicsWorld, PulleyDesc) {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    let (a, _) = w.add_dynamic_circle(-1.0, 2.0, R, mass / area);
    let (b, _) = w.add_dynamic_circle(1.0, 2.0, R, mass / area);
    let wheels = round_wheels(radius);
    let total_length = rope_route::route([-1.0, 2.0], [1.0, 2.0], &wheels, &mut Vec::new())
        .expect("a rota de repouso resolve")
        .length;
    let desc = PulleyDesc {
        body_a: a,
        body_b: b,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 2,
        id: 1,
        total_length,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels);
    (w, desc)
}

fn stretch(w: &PhysicsWorld, d: &PulleyDesc) -> f32 {
    w.pulley_span(d).unwrap_or(f32::NAN) - d.total_length
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_bias_with_radius() {
    for radius in [0.0_f32, 0.3, 1.0] {
        println!(
            "\n=== O ESTICAMENTO EM REGIME por beta — roldanas de RAIO {radius:.2} \
             (Atwood 1 kg x 1 kg, 2 s) ==="
        );
        println!(
            "{:>6} | {:>12} | {:>12} | {:>10}",
            "beta", "regime (m)", "pico (m)", "tremor"
        );
        for beta in [0.05_f32, 0.1, 0.2, 0.4, 0.8, 1.0, 1.5, 2.0] {
            let (mut w, d) = atwood(1.0, radius);
            w.set_pulley_bias(beta);
            let (mut peak, mut lo, mut hi) = (0.0_f32, f32::INFINITY, f32::NEG_INFINITY);
            for tick in 0..120 {
                w.step();
                let s = stretch(&w, &d);
                peak = peak.max(s);
                if tick >= 90 {
                    lo = lo.min(s);
                    hi = hi.max(s);
                }
            }
            println!(
                "{beta:>6.2} | {:>12.4} | {peak:>12.4} | {:>10.5}",
                stretch(&w, &d),
                hi - lo
            );
        }
    }
    println!("\n  (o teto legitimo e o do rapier: normalized_allowed_linear_error = 1,3 mm)");
}

#[test]
#[ignore = "measurement, not a gate"]
fn what_the_arc_adds_to_the_length() {
    println!("\n=== Quanto o ARCO acrescenta ao comprimento da corda ===");
    println!("{:>7} | {:>12} | {:>12}", "raio", "rota (m)", "delta");
    let base = rope_route::route([-1.0, 2.0], [1.0, 2.0], &round_wheels(0.0), &mut Vec::new())
        .expect("rota de raio zero")
        .length;
    for radius in [0.0_f32, 0.1, 0.3, 0.5, 1.0] {
        let l = rope_route::route(
            [-1.0, 2.0],
            [1.0, 2.0],
            &round_wheels(radius),
            &mut Vec::new(),
        )
        .expect("a rota resolve")
        .length;
        println!("{radius:>7.2} | {l:>12.4} | {:>12.4}", l - base);
    }
    println!();
}
