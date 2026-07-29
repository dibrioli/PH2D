//! **O TAMBOR DIFERENCIAL, medido** (W-Pulley W4) — a vantagem mecânica
//! CONTÍNUA, e a prova de que ela cai dos dois raios em vez de ser digitada.
//!
//! Roda pelo caminho do PRODUTO (`PhysicsWorld::step`), nunca por uma segunda
//! cópia do laço: uma sonda que re-implementa o que mede fica cega à porta.
//!
//! `cargo test -p ph2d-physics --test measure_pulley_differential -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// **O sarilho diferencial:** um eixo com DOIS diâmetros, o contrapeso pendurado
/// no lado por onde a corda ENTRA e a carga no lado por onde ela SAI.
///
/// ```text
///            tambor (0, 8)   r_entra = R  ·  r_sai = r
///                 _.-""-._
///        .-------(   ( )   )-------.
///        |        "-.__.-"         |
///   contrapeso (-1, 5)        carga (+1, 5)
/// ```
///
/// Girar o eixo de `dθ` recolhe `R·dθ` de um lado e paga `r·dθ` do outro, então
/// `r·Δl_entra + R·Δl_sai = 0` — e é só isso. A vantagem é `R/r`, o quociente de
/// duas circunferências que o artista desenha.
///
/// `r_out = None` é o **CONTROLE**: a mesma montagem com uma roldana comum, onde
/// a vantagem é 1 porque a tensão de uma corda é uniforme (§3 do plano).
fn windlass(
    load: f32,
    counter: f32,
    r_in: f32,
    r_out: Option<f32>,
) -> (PhysicsWorld, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    const BODY_R: f32 = 0.2;
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (a, _) = w.add_dynamic_circle(-1.0, 5.0, BODY_R, counter / area);
    let (b, _) = w.add_dynamic_circle(1.0, 5.0, BODY_R, load / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, 8.0],
        radius: r_in,
        radius_out: r_out,
        id: 1,
        ..RopeWheel::default()
    }];
    // O lado por onde a corda passa, pela MESMA porta que a ponte usa em autoria.
    let mut scratch = Vec::new();
    rope_route::resolve_sides([-1.0, 5.0], [1.0, 5.0], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: a,
        body_b: b,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, a, b)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).map_or(f32::NAN, |b| b.translation().y)
}

/// Quanto a CARGA anda em 1 s.
fn load_travel(load: f32, counter: f32, r_in: f32, r_out: Option<f32>) -> f32 {
    let (mut w, _, b) = windlass(load, counter, r_in, r_out);
    let y0 = y(&w, b);
    for _ in 0..60 {
        w.step();
    }
    y(&w, b) - y0
}

/// **A vantagem é o quociente dos raios** — e o oráculo é a TABELA, não uma
/// bissecção.
///
/// ⚠️ **A primeira versão bisseccionava a carga de equilíbrio e a medição a
/// derrubou:** o sistema NÃO é monótono na carga. Com a carga muito acima do
/// equilíbrio o contrapeso leve é arremessado para cima, ALCANÇA o tambor, e a
/// rota degenera (o caso que o W1 nomeou) — então `desce` volta a virar `sobe` lá
/// em cima e a bissecção caminhava direto para o teto do intervalo. A tabela
/// pergunta no lugar certo: a carga PREVISTA e as duas vizinhas.
#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_advantage_against_the_radii() {
    println!("\n=== O CONTRAPESO É 1 kg; QUE CARGA ELE SEGURA ===");
    println!("r_entra = 0,50 fixo; só o r_sai muda. Previsto: carga = R/r.");
    println!(
        "{:>8} {:>7} | {:>10} {:>10} {:>10} | {:>10} {:>10} {:>10}",
        "r_sai", "R/r", "-20%", "PREVISTO", "+20%", "dy(-20%)", "dy(prev)", "dy(+20%)"
    );
    for r_out in [0.50_f32, 0.25, 0.125, 0.10] {
        let gear = 0.5 / r_out;
        let d = |m: f32| load_travel(m, 1.0, 0.5, Some(r_out));
        let (lo, mid, hi) = (gear * 0.8, gear, gear * 1.2);
        println!(
            "{r_out:>8.3} {gear:>7.2} | {lo:>10.2} {mid:>10.2} {hi:>10.2} | \
             {:>10.4} {:>10.4} {:>10.4}",
            d(lo),
            d(mid),
            d(hi)
        );
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_control_is_one_to_one() {
    println!("\n=== O CONTROLE: roldana COMUM, mesma montagem ===");
    println!(
        "{:>10} | {:>12} {:>12}",
        "carga (kg)", "dy da carga", "veredito"
    );
    for load in [0.5_f32, 0.9, 1.0, 1.1, 2.0] {
        let d = load_travel(load, 1.0, 0.5, None);
        let verdict = if d.abs() < 0.05 {
            "EQUILIBRIO"
        } else if d > 0.0 {
            "sobe"
        } else {
            "desce"
        };
        println!("{load:>10.2} | {d:>12.4} {verdict:>12}");
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_cost() {
    for r_out in [None, Some(0.25_f32)] {
        let label = if r_out.is_some() {
            "DIFERENCIAL"
        } else {
            "comum     "
        };
        println!("{label}: {:.3} ms/tique (50 sarilhos)", cost_of(r_out));
    }
}

/// O custo de 50 cordas, com e sem tambor diferencial — a engrenagem é uma
/// MULTIPLICAÇÃO por trecho, então o controle existe para dizer isso com um
/// número em vez de com uma frase.
fn cost_of(r_out: Option<f32>) -> f64 {
    let mut w = PhysicsWorld::new();
    const BODY_R: f32 = 0.2;
    let mut pulleys = Vec::new();
    let mut wheels = Vec::new();
    for i in 0..50 {
        let x = i as f32 * 3.0;
        let (a, _) = w.add_dynamic_circle(x - 1.0, 5.0, BODY_R, 1.0);
        let (b, _) = w.add_dynamic_circle(x + 1.0, 5.0, BODY_R, 1.0);
        wheels.push(RopeWheel {
            centre: [x, 8.0],
            radius: 0.5,
            radius_out: r_out,
            side: -1,
            id: i as u64 + 1,
            ..RopeWheel::default()
        });
        pulleys.push(PulleyDesc {
            id: i as u64 + 1,
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: i,
            wheel_count: 1,
            total_length: 10.0,
            motor_rate: 0.0,
            break_force: f32::INFINITY,
        });
    }
    w.set_pulleys(pulleys, wheels);
    // Uma corrida de aquecimento, senão o primeiro tique paga o first-touch das
    // arenas e a diferença medida é a do alocador.
    for _ in 0..10 {
        w.step();
    }
    let t = std::time::Instant::now();
    for _ in 0..60 {
        w.step();
    }
    t.elapsed().as_secs_f64() * 1000.0 / 60.0
}

#[test]
#[ignore = "measurement, not a gate"]
fn probe_raw_travel() {
    println!("\n=== TRAVESSIA CRUA, r_entra 0,50 / r_sai 0,25 (engrenagem 2) ===");
    println!(
        "{:>10} | {:>12} {:>12}",
        "carga", "dy carga", "dy contrapeso"
    );
    for load in [0.5_f32, 1.0, 2.0, 3.0, 5.0, 10.0, 40.0] {
        let (mut w, a, b) = windlass(load, 1.0, 0.5, Some(0.25));
        let (ya, yb) = (y(&w, a), y(&w, b));
        for _ in 0..60 {
            w.step();
        }
        println!(
            "{load:>10.2} | {:>12.4} {:>12.4}",
            y(&w, b) - yb,
            y(&w, a) - ya
        );
    }
    // E a rota, crua.
    let (w, _, _) = windlass(2.0, 1.0, 0.5, Some(0.25));
    let d = w.pulleys()[0];
    println!("L0 (pesado) = {:.4}", d.total_length);
    let arena = w.pulley_wheels();
    println!(
        "roldana: r_in {:.3} r_out {:.3} gear {:.3} side {}",
        arena[0].radius,
        arena[0].radius_out(),
        arena[0].gear(),
        arena[0].side
    );
}

#[test]
#[ignore = "measurement, not a gate"]
fn probe_break_thresholds() {
    println!("\n=== TENSAO DE PICO e travessia, carga 4 kg / contrapeso 1 kg ===");
    println!(
        "{:>12} {:>7} | {:>12} {:>14}",
        "r_sai", "gear", "pico (N)", "dy carga (30 t)"
    );
    for r_out in [None, Some(0.25_f32), Some(0.125_f32)] {
        let (mut w, _, b) = windlass(4.0, 1.0, 0.5, r_out);
        let y0 = y(&w, b);
        let mut peak = 0.0_f32;
        for _ in 0..30 {
            w.step();
            peak = peak.max(w.pulley_tension(1));
        }
        let gear = r_out.map_or(1.0, |r| 0.5 / r);
        println!(
            "{:>12} {gear:>7.2} | {peak:>12.3} {:>14.4}",
            r_out.map_or("comum".to_string(), |r| format!("{r:.3}")),
            y(&w, b) - y0
        );
    }
}
