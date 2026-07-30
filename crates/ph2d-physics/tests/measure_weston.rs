//! **A TALHA DE WESTON, medida** (W-Pulley, W-Weston) — e a nota que esta sonda
//! carregava antes estava **errada nas duas metades**.
//!
//! ⚠️ **O que a versão anterior desta sonda afirmava:** *"a Weston não é
//! expressável — é topologia, e pediria uma SEGUNDA restrição por corda"*. A
//! primeira metade era verdade e a segunda **não**, e a diferença decide uma
//! wave: a eliminação da rotação do eixo entre os DOIS contatos deixa **uma**
//! restrição escalar, e ela é um orçamento **pesado** — exatamente o tipo que a
//! rota já soma. O peso é `R/(R−r)`.
//!
//! ⚠️ **E a objeção geométrica também caiu:** *"duas roldanas concêntricas são
//! recusadas pela rota"* vale para pares **consecutivos**, e num par de Weston
//! eles nunca são consecutivos — a cadernal está no meio.
//!
//! # O que esta sonda mede
//!
//! Três colunas, e a terceira é o CONTROLE:
//!
//! 1. a **Weston** de verdade (o eixo composto atravessado duas vezes, com a
//!    cadernal móvel abraçada entre os contatos);
//! 2. o **tambor adjacente** do W4 com `r_saída = R − r` — o rig que produz o
//!    MESMO orçamento e que já shipava, e é por isso que ele serve de oráculo
//!    independente da lei nova;
//! 3. o par **recusado** (`r ≥ R`), onde os dois contatos voltam a ser roldanas
//!    comuns e sobra a vantagem da cadernal sozinha.
//!
//! Roda pelo caminho do PRODUTO (`PhysicsWorld::step`), nunca por uma segunda
//! cópia do laço.
//!
//! `cargo test -p ph2d-physics --test measure_weston -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};

/// O diâmetro de ENTRADA do eixo composto.
///
/// ⚠️ **Potência de dois**, e os raios de retorno derivados dela por `1 − 1/2^k`,
/// para que o peso `R/(R−r)` seja EXATO em `f32`: uma tabela com folga de 20% não
/// notaria o erro, mas o gate que compara razões notaria — e a sonda e o gate têm
/// de falar dos mesmos números.
const R_IN: f32 = 0.5;

/// O número do eixo que os dois contatos compartilham. Qualquer não-zero serve;
/// no produto é o `stable_name_id` do nome da roldana.
const AXLE: u64 = 7;

/// O raio de RETORNO que produz o peso `gear` — a inversa de `R/(R−r)`.
fn return_radius(gear: f32) -> f32 {
    R_IN * (1.0 - 1.0 / gear)
}

/// **A talha de WESTON:** um eixo composto no teto, a corda saindo pelo diâmetro
/// grande, abraçando a cadernal MÓVEL que carrega a carga, e voltando pelo
/// pequeno.
///
/// ```text
///        morta (-0.8, 8)   eixo composto (0, 8)   R = 0.5 · r = retorno
///                    \        / \
///                     \      /   contrapeso (0.8, 6)
///                      \    /
///                    cadernal MOVEL (0, 4)  [montada no BLOCO]
/// ```
///
/// A corda anda **do contrapeso** (ponta A) para a ponta MORTA (B): entra no
/// contato de raio `R`, desce até a cadernal, abraça, **volta ao MESMO eixo** pelo
/// raio `r`, e o que sobra até a ponta morta é o ramo SOLTO (peso zero).
fn weston(
    load: f32,
    counter: f32,
    gear: f32,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(-0.8, 8.0, 0.1, 0.1);
    let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, load / area);
    let (haul, _) = w.add_dynamic_circle(0.8, 6.0, BODY_R, counter / area);
    let mut wheels = vec![
        // O contato de IDA, pelo diâmetro grande.
        RopeWheel {
            centre: [0.0, 8.0],
            radius: R_IN,
            axle: AXLE,
            id: 1,
            ..RopeWheel::default()
        },
        // A cadernal móvel — o eixo no CENTRO do bloco, então a corda não lhe faz
        // torque.
        RopeWheel {
            centre: [0.0, 4.0],
            body: Some(block),
            local: [0.0, 0.0],
            radius: 0.15,
            id: 2,
            ..RopeWheel::default()
        },
        // O contato de RETORNO — **o mesmo eixo**, pelo diâmetro pequeno. Mesmo
        // centro do primeiro: é isso que um eixo composto é, e a rota o aceita
        // porque os dois nunca são consecutivos.
        RopeWheel {
            centre: [0.0, 8.0],
            radius: return_radius(gear),
            axle: AXLE,
            id: 1,
            ..RopeWheel::default()
        },
    ];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.8, 6.0], [-0.8, 8.0], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: haul,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 3,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block, haul)
}

/// **O ORÁCULO INDEPENDENTE:** o tambor ADJACENTE do W4 com `r_saída = R − r`.
///
/// Ele produz os MESMOS pesos de orçamento (`1` no ramo de esforço, `R/(R−r)` nos
/// dois ramos da cadernal) por um caminho de código que **já shipava** e que a lei
/// nova não toca. Se as duas colunas discordarem, é a lei nova que está errada.
fn adjacent(
    load: f32,
    counter: f32,
    gear: f32,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(-0.8, 8.0, 0.1, 0.1);
    let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, load / area);
    let (haul, _) = w.add_dynamic_circle(0.8, 6.0, BODY_R, counter / area);
    let mut wheels = vec![
        RopeWheel {
            centre: [0.0, 8.0],
            radius: R_IN,
            radius_out: Some(R_IN / gear),
            id: 1,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [0.0, 4.0],
            body: Some(block),
            local: [0.0, 0.0],
            radius: 0.15,
            id: 2,
            ..RopeWheel::default()
        },
    ];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.8, 6.0], [-0.8, 8.0], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: haul,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 2,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block, haul)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).map_or(f32::NAN, |b| b.translation().y)
}

/// Quanto o bloco anda em 1 s, com um contrapeso de 1 kg.
fn travel(
    rig: fn(f32, f32, f32) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle, RigidBodyHandle),
    load: f32,
    gear: f32,
) -> f32 {
    let (mut w, _, block, _) = rig(load, 1.0, gear);
    let y0 = y(&w, block);
    for _ in 0..60 {
        w.step();
    }
    y(&w, block) - y0
}

fn verdict(d: f32) -> &'static str {
    if d.abs() < 0.02 {
        "parado"
    } else if d > 0.0 {
        "SOBE"
    } else {
        "desce"
    }
}

/// **A WESTON entrega `2R/(R−r)`?** — a tabela de previsão, nos dois lados de
/// cada linha.
///
/// ⚠️ **Previsão com bracket, nunca bisseção.** O sistema **não é monotônico na
/// carga** (muito acima do equilíbrio o contrapeso leve é arremessado até o eixo e
/// *"desce"* volta a virar *"sobe"*), e uma busca binária sobre isso **já mentiu
/// nesta linha**: ela imprimiu *"vantagem 199,99"* a partir do próprio piso.
#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_weston_advantage() {
    println!("\n=== A TALHA DE WESTON: 2R/(R-r) (contrapeso de 1 kg) ===");
    println!("R = {R_IN}; o raio de retorno e derivado do peso alvo.");
    println!(
        "{:>6} | {:>9} | {:>10} | {:>24} | {:>24}",
        "peso", "r retorno", "previsto", "carga -20% (sobe?)", "carga +20% (desce?)"
    );
    for gear in [2.0_f32, 4.0, 8.0, 16.0] {
        let predicted = 2.0 * gear;
        let (lo, hi) = (predicted * 0.8, predicted * 1.2);
        let (dlo, dhi) = (travel(weston, lo, gear), travel(weston, hi, gear));
        println!(
            "{gear:>6.1} | {:>9.4} | {predicted:>10.2} | {:>24} | {:>24}",
            return_radius(gear),
            format!("{lo:.2} kg {:.3} {}", dlo, verdict(dlo)),
            format!("{hi:.2} kg {:.3} {}", dhi, verdict(dhi))
        );
    }
    println!("\nSe o par de eixo nao pesasse a corda, toda linha equilibraria em 2 kg");
    println!("-- e a coluna '-20%' desceria a partir da primeira.");
}

/// **A Weston e o tambor adjacente concordam?** — o mesmo orçamento por dois
/// caminhos de código, e um deles já shipava.
#[test]
#[ignore = "measurement, not a gate"]
fn the_weston_agrees_with_the_adjacent_drum_that_shipped() {
    println!("\n=== WESTON x TAMBOR ADJACENTE (mesmo orcamento, dois caminhos) ===");
    println!(
        "{:>6} | {:>10} | {:>13} | {:>13} | {:>10}",
        "peso", "previsto", "weston (m)", "adjacente (m)", "delta"
    );
    for gear in [2.0_f32, 4.0, 8.0, 16.0] {
        let load = 2.0 * gear;
        let (dw, da) = (travel(weston, load, gear), travel(adjacent, load, gear));
        println!(
            "{gear:>6.1} | {:>10.2} | {dw:>13.5} | {da:>13.5} | {:>10.5}",
            load,
            (dw - da).abs()
        );
    }
    println!("\nNo equilibrio previsto os dois rigs tem de ficar quase PARADOS, e");
    println!("o delta entre eles e o que separa a lei nova da que ja shipava.");
}

/// **Onde a precisão de `f32` come a máquina** — o único recurso que pode pedir um
/// teto, e ele é MEDIDO em vez de escolhido.
///
/// `C = Σ w·l − L₀`, e com peso `w` grande o `L₀` é da ordem de `w·l`: a resolução
/// absoluta de `C` degrada como `w·l·6e−8`. A pergunta é a partir de que peso isso
/// aparece no PRODUTO.
#[test]
#[ignore = "measurement, not a gate"]
fn sweep_where_the_precision_eats_the_weston() {
    println!("\n=== O TETO: ate onde o peso da Weston sobrevive ao f32 ===");
    println!(
        "{:>8} | {:>10} | {:>10} | {:>12} | {:>12} | {:>10}",
        "peso", "r retorno", "L0 (m)", "sobe -20%", "desce +20%", "tensao (N)"
    );
    for gear in [
        2.0_f32, 8.0, 32.0, 128.0, 512.0, 2048.0, 8192.0, 32768.0, 131_072.0,
    ] {
        let predicted = 2.0 * gear;
        let (lo, hi) = (predicted * 0.8, predicted * 1.2);
        let (dlo, dhi) = (travel(weston, lo, gear), travel(weston, hi, gear));
        let (mut w, d, _, _) = weston(predicted, 1.0, gear);
        for _ in 0..30 {
            w.step();
        }
        println!(
            "{gear:>8.0} | {:>10.6} | {:>10.2} | {:>12} | {:>12} | {:>10.3}",
            return_radius(gear),
            d.total_length,
            format!("{:.4} {}", dlo, verdict(dlo)),
            format!("{:.4} {}", dhi, verdict(dhi)),
            w.pulley_tension(d.id)
        );
    }
    println!("\nA linha em que 'sobe' e 'desce' param de discordar e onde a maquina");
    println!("deixa de ser dirigivel -- e e o numero que um teto pode citar.");
}

/// **O par RECUSADO** — `r ≥ R` não é uma Weston que este orçamento saiba segurar,
/// e a recusa devolve duas roldanas comuns (vantagem 2, a da cadernal sozinha).
#[test]
#[ignore = "measurement, not a gate"]
fn a_refused_pair_falls_back_to_the_tackle_alone() {
    println!("\n=== O PAR RECUSADO (r >= R): sobra a cadernal ===");
    println!(
        "{:>10} | {:>8} | {:>10} | {:>22} | {:>22}",
        "r retorno", "par?", "previsto", "carga -20% (sobe?)", "carga +20% (desce?)"
    );
    // `r = R` (travada), `r > R` (invertida) e `r = 0` (não há o que enrolar).
    for r_ret in [R_IN, R_IN * 1.5, 0.0] {
        let wheels = [
            RopeWheel {
                radius: R_IN,
                axle: AXLE,
                ..RopeWheel::default()
            },
            RopeWheel {
                radius: 0.15,
                ..RopeWheel::default()
            },
            RopeWheel {
                radius: r_ret,
                axle: AXLE,
                ..RopeWheel::default()
            },
        ];
        let paired = rope_route::axle_pair(&wheels, 0).is_some();
        // A previsão é a da cadernal SOZINHA: 2.
        let (lo, hi) = (2.0 * 0.8, 2.0 * 1.2);
        // Monta à mão para não passar por `return_radius`, que é a inversa do peso.
        let build = |load: f32| -> f32 {
            const BODY_R: f32 = 0.2;
            let mut w = PhysicsWorld::new();
            let area = std::f32::consts::PI * BODY_R * BODY_R;
            let (dead, _) = w.add_static_cuboid(-0.8, 8.0, 0.1, 0.1);
            let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, load / area);
            let (haul, _) = w.add_dynamic_circle(0.8, 6.0, BODY_R, 1.0 / area);
            let mut ws = vec![
                RopeWheel {
                    centre: [0.0, 8.0],
                    radius: R_IN,
                    axle: AXLE,
                    id: 1,
                    ..RopeWheel::default()
                },
                RopeWheel {
                    centre: [0.0, 4.0],
                    body: Some(block),
                    local: [0.0, 0.0],
                    radius: 0.15,
                    id: 2,
                    ..RopeWheel::default()
                },
                RopeWheel {
                    centre: [0.0, 8.0],
                    radius: r_ret,
                    axle: AXLE,
                    id: 1,
                    ..RopeWheel::default()
                },
            ];
            let mut scratch = Vec::new();
            rope_route::resolve_sides([0.8, 6.0], [-0.8, 8.0], &mut ws, &mut scratch);
            let mut d = PulleyDesc {
                id: 1,
                body_a: haul,
                body_b: dead,
                local_a: [0.0, 0.0],
                local_b: [0.0, 0.0],
                wheel_start: 0,
                wheel_count: 3,
                total_length: 0.0,
                motor_rate: 0.0,
                break_force: f32::INFINITY,
            };
            w.set_pulleys(vec![d], ws.clone());
            let Some(span) = w.pulley_span(&d) else {
                return f32::NAN;
            };
            d.total_length = span;
            w.set_pulleys(vec![d], ws);
            let y0 = y(&w, block);
            for _ in 0..60 {
                w.step();
            }
            y(&w, block) - y0
        };
        let (dlo, dhi) = (build(lo), build(hi));
        println!(
            "{r_ret:>10.4} | {:>8} | {:>10.2} | {:>22} | {:>22}",
            if paired { "SIM" } else { "nao" },
            2.0,
            format!("{lo:.2} kg {:.3} {}", dlo, verdict(dlo)),
            format!("{hi:.2} kg {:.3} {}", dhi, verdict(dhi))
        );
    }
    println!("\nSem par, os dois contatos sao roldanas comuns: sobra o enlace da");
    println!("cadernal, que vale 2 em qualquer diametro.");
}

/// **A taxa de içamento de um eixo DIRIGIDO** — e de que a sensibilidade
/// `Δcomprimento/Δaltura` de fato é feita.
///
/// A previsão de papel é `ω(R−r)/2`: o orçamento encurta a `ω·R`, o trecho abraçado
/// é pesado por `R/(R−r)`, e a carga pende de dois ramos. Esta sonda pergunta ao
/// produto se a razão é essa — e imprime a sensibilidade medida ao lado, que é o
/// número que explica qualquer diferença.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_driven_hoist_rate() {
    const OMEGA: f32 = 1.0;
    const BODY_R: f32 = 0.2;
    const SHEAVE_Y: f32 = 30.0;
    println!("\n=== A TAXA de um eixo composto DIRIGIDO (pontas presas) ===");
    println!(
        "{:>6} | {:>10} | {:>10} | {:>10} | {:>12} | {:>12}",
        "peso", "r ret", "previsto", "medido", "medido/prev", "dL/dh"
    );
    for gear in [1.0_f32, 2.0, 4.0, 8.0] {
        let ret = if gear == 1.0 {
            0.0
        } else {
            return_radius(gear)
        };
        let mut w = PhysicsWorld::new();
        let area = std::f32::consts::PI * BODY_R * BODY_R;
        let (dead, _) = w.add_static_cuboid(-0.8, SHEAVE_Y, 0.1, 0.1);
        let (effort, _) = w.add_static_cuboid(0.8, SHEAVE_Y - 2.0, 0.1, 0.1);
        let (block, _) = w.add_dynamic_circle(0.0, 4.0, BODY_R, 1.0 / area);
        let axle = if gear == 1.0 { 0 } else { AXLE };
        let mut wheels = vec![
            RopeWheel {
                centre: [0.0, SHEAVE_Y],
                radius: R_IN,
                axle,
                id: 1,
                ..RopeWheel::default()
            },
            RopeWheel {
                centre: [0.0, 4.0],
                body: Some(block),
                local: [0.0, 0.0],
                radius: 0.15,
                id: 2,
                ..RopeWheel::default()
            },
            RopeWheel {
                centre: [0.0, SHEAVE_Y],
                radius: ret,
                axle,
                id: 1,
                ..RopeWheel::default()
            },
        ];
        let mut scratch = Vec::new();
        let ea = [0.8, SHEAVE_Y - 2.0];
        rope_route::resolve_sides(ea, [-0.8, SHEAVE_Y], &mut wheels, &mut scratch);
        let mut d = PulleyDesc {
            id: 1,
            body_a: effort,
            body_b: dead,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 3,
            total_length: 0.0,
            motor_rate: OMEGA * R_IN,
            break_force: f32::INFINITY,
        };
        w.set_pulleys(vec![d], wheels.clone());
        d.total_length = w.pulley_span(&d).expect("rota sã");
        w.set_pulleys(vec![d], wheels);
        for _ in 0..60 {
            w.step();
        }
        let (y0, l0) = (y(&w, block), w.pulley_span(&d).unwrap_or(f32::NAN));
        for _ in 0..60 {
            w.step();
        }
        let (y1, l1) = (y(&w, block), w.pulley_span(&d).unwrap_or(f32::NAN));
        let measured = y1 - y0;
        let predicted = if gear == 1.0 {
            OMEGA * R_IN / 2.0
        } else {
            OMEGA * (R_IN - ret) / 2.0
        };
        println!(
            "{gear:>6.1} | {ret:>10.4} | {predicted:>10.4} | {measured:>10.4} | {:>12.4} | {:>12.3}",
            measured / predicted,
            (l0 - l1) / measured
        );
    }
    println!("\n`dL/dh` e a sensibilidade do orcamento a altura da carga: 2 sem peso,");
    println!("2*peso com. Se ela nao for isso, a diferenca esta na GEOMETRIA da rota.");
}
