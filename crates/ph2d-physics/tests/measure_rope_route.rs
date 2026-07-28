//! **Quantas passadas o ponto fixo do LADO precisa** — a medição por trás do
//! `MAX_SIDE_PASSES` (W-Pulley W1).
//!
//! `cargo test -p ph2d-physics --test measure_rope_route -- --ignored --nocapture`
//!
//! O cap não é um palpite de segurança: ele é o teto do caso patológico, e o
//! número que importa é quantas passadas uma montagem SÃ de fato consome. Esta
//! sonda conta, sobre montagens de 1 a 6 roldanas em posições variadas.

use ph2d_physics::world::rope_route::{RopeWheel, Tangent, route};

/// Uma cópia INSTRUMENTADA do laço de `resolve_sides`, que devolve quantas
/// passadas foram precisas.
///
/// ⚠️ É uma segunda cópia da regra, e de propósito — a porta do produto não
/// conta nada, e fazê-la contar seria instrumentar o caminho quente para uma
/// pergunta que só se faz uma vez. Ela é curta e o gate de acordo
/// (`the_resolved_sides_agree_with_the_route_they_produce`) prova a propriedade
/// que importa pelos DOIS caminhos.
fn passes_to_settle(a: [f32; 2], b: [f32; 2], wheels: &mut [RopeWheel]) -> usize {
    let mut scratch: Vec<Tangent> = Vec::new();
    for i in 0..wheels.len() {
        let prev = if i == 0 { a } else { wheels[i - 1].centre };
        let next = if i + 1 == wheels.len() {
            b
        } else {
            wheels[i + 1].centre
        };
        let c = wheels[i].centre;
        let (u, v) = (
            [c[0] - prev[0], c[1] - prev[1]],
            [next[0] - c[0], next[1] - c[1]],
        );
        let cross = u[0] * v[1] - u[1] * v[0];
        wheels[i].side = if cross >= 0.0 { 1 } else { -1 };
    }
    for pass in 1..=16 {
        if route(a, b, wheels, &mut scratch).is_none() {
            return pass;
        }
        let mut changed = false;
        for i in 0..wheels.len() {
            let (u_in, u_out) = (scratch[i].dir, scratch[i + 1].dir);
            let cross = u_in[0] * u_out[1] - u_in[1] * u_out[0];
            let s = if cross > 0.0 {
                1
            } else if cross < 0.0 {
                -1
            } else {
                wheels[i].side
            };
            if s != wheels[i].side {
                wheels[i].side = s;
                changed = true;
            }
        }
        if !changed {
            return pass;
        }
    }
    99
}

#[test]
#[ignore = "measurement, not a gate"]
fn how_many_passes_the_side_fixpoint_needs() {
    println!("\n=== passadas até o lado assentar ===");
    let mut worst = 0;
    for n in 1..=6_usize {
        for spread in [1.0_f32, 3.0, 8.0] {
            let mut wheels: Vec<RopeWheel> = (0..n)
                .map(|i| {
                    let t = (i as f32 + 1.0) / (n as f32 + 1.0);
                    RopeWheel {
                        // Zigue-zague: as roldanas alternam acima e abaixo da
                        // linha das âncoras, que é onde o lado de fato muda.
                        centre: [
                            -6.0 + 12.0 * t,
                            spread * if i % 2 == 0 { 1.0 } else { -1.0 },
                        ],
                        radius: 0.2 + 0.3 * (i % 3) as f32,
                        side: 1,
                        id: 0,
                        break_force: f32::INFINITY,
                    }
                })
                .collect();
            let p = passes_to_settle([-8.0, 0.0], [8.0, 0.0], &mut wheels);
            worst = worst.max(p);
            println!("  {n} roldana(s), espalhamento {spread:>4.1}: {p} passada(s)");
        }
    }
    println!("  PIOR CASO: {worst} passada(s)");
}
