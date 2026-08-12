//! **SUBSTEPS** (doc 89, folha 13 — o ultimo P1). A folha mediu que encadear `sim.step` duas
//! vezes e um no-op EXATO (`dt = playhead - sim_t = 0` no segundo). Esta sonda pergunta a
//! outra metade: **o `dt` do passo sai de `playhead - sim_t` por ELEMENTO**, entao subdividir
//! o PLAYHEAD entre `cook`/`advance_tick` deveria dar substeps sem tocar no `sim.step`.
//!
//! Rodar: cargo test -p ph2d-node-registry-init --release --test measure_substeps -- --ignored --nocapture

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// grid -> zone(init) ; zone =pre=> wind -> step -> zone(state)
fn falling_zone(reg: &NodeRegistry) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    g.set_param(seed, "gap_x", 1.0);
    g.set_param(seed, "gap_y", 1.0);

    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    // ⚠️ `force.wind` fala `angle`/`strength` — nao `dir_x`/`dir_y`. Um param que o manifesto
    // nao declara passa CALADO por `set_param`, e a 1ª versao desta sonda mediu o eixo errado.
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "strength", 40.0);
    g.set_param(wind, "gust", 0.0); // rajada zero: a aceleracao fica CONSTANTE, com analitica
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 1.0);

    wire(&mut g, seed, 0, zone, 0, false);
    wire(&mut g, zone, 0, wind, 0, true); // o `pre`
    wire(&mut g, wind, 0, step, 0, false);
    wire(&mut g, step, 0, zone, 1, false);
    // O `validate` e o que recusa um param inventado, em vez de o deixar passar calado.
    g.validate(reg).expect("bem-tipado");
    (g, zone)
}

fn p_of(s: &Stream) -> [f32; 2] {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0],
        _ => [f32::NAN, f32::NAN],
    }
}

/// Corre `frames` quadros a 60 fps, cada quadro subdividido em `sub` passadas.
fn run(g: &Graph, reg: &NodeRegistry, zone: NodeId, frames: u64, sub: u32) -> [f32; 2] {
    let mut cook = Cook::new();
    let mut last = [f32::NAN; 2];
    for k in 0..frames {
        let t0 = k as f64 / 60.0;
        for s in 0..sub {
            // O playhead que o passo LE avanca uma fracao do quadro por passada.
            let t = t0 + (s as f64 + 1.0) / (sub as f64 * 60.0);
            last = p_of(cook.cook(g, reg, zone, t).expect("cooks")[0].as_stream());
            cook.advance_tick(g, reg, t).expect("tick");
        }
    }
    last
}

#[test]
#[ignore = "measurement probe; run with --ignored --nocapture"]
fn measure_whether_subdividing_the_playhead_substeps_the_zone() {
    let reg = registry();
    let (g, zone) = falling_zone(&reg);

    println!("\n=== QUEDA sob vento constante, 60 quadros (1 s), por nº de passadas ===");
    println!("sub   P.x final     erro vs a analitica");
    // v(t) = a*t, y(t) = -a*t^2/2 com a = 40 (o vento e aceleracao pura no `accel`).
    let exact = 40.0f32 * 1.0 * 1.0 / 2.0;
    for sub in [1u32, 2, 4, 8, 16] {
        let p = run(&g, &reg, zone, 60, sub);
        println!("{sub:3}   {:10.4}    {:8.4}", p[0], p[0] - exact);
    }
    println!("analitica x(1s) = {exact:.4}  (Euler semi-implicito converge POR CIMA)");
}

/// **O que N passadas GLOBAIS custam** — a pergunta que decide se o substep e um numero do
/// mundo (subdividir o playhead no pump) ou uma propriedade por-zona (o cook aprender a
/// iterar um sub-DAG, foundational). Se um no CARO fora da zona paga N x, o global e caro.
#[test]
#[ignore = "measurement probe; run with --ignored --nocapture"]
fn measure_what_n_global_passes_cost() {
    use std::time::Instant;
    let reg = registry();

    // Uma zona pequena + um vizinho CARO que nao tem nada com ela (20k linhas deformadas).
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    g.set_param(seed, "gap_x", 1.0);
    g.set_param(seed, "gap_y", 1.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "strength", 40.0);
    g.set_param(wind, "gust", 0.0);
    let step = g.add_node("sim.step");
    wire(&mut g, seed, 0, zone, 0, false);
    wire(&mut g, zone, 0, wind, 0, true);
    wire(&mut g, wind, 0, step, 0, false);
    wire(&mut g, step, 0, zone, 1, false);

    let big = g.add_node("motion.grid");
    g.set_param(big, "rows", 141.0);
    g.set_param(big, "cols", 141.0);
    g.set_param(big, "gap_x", 1.0);
    g.set_param(big, "gap_y", 1.0);
    let bend = g.add_node("motion.bend");
    g.set_param(bend, "angle", 0.7);
    wire(&mut g, big, 0, bend, 0, false);
    // ⚠️ Um vizinho que NAO le o relogio e memoizado, e a 1ª versao desta sonda mediu
    // 0,014 ms para 20k linhas -- o memo, nao o trabalho. O LFO o torna dependente do
    // tempo, que e a condicao para ele de fato re-cozinhar a cada passada.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 2.0);
    wire(&mut g, lfo, 0, bend, 1, false);
    let merge = g.add_node("motion.combine");
    wire(&mut g, zone, 0, merge, 0, false);
    wire(&mut g, bend, 0, merge, 1, false);
    g.validate(&reg).expect("bem-tipado");

    println!("\n=== CUSTO de N passadas GLOBAIS (zona de 1 + vizinho de 20k) ===");
    println!("sub    ms/quadro    razao vs sub=1    linhas");
    let mut base = 0.0f64;
    for sub in [1u32, 2, 4, 8, 16] {
        let mut cook = Cook::new();
        // aquece
        for k in 0..5 {
            let t = k as f64 / 60.0;
            cook.cook(&g, &reg, merge, t).unwrap();
            cook.advance_tick(&g, &reg, t).unwrap();
        }
        let frames = 60u64;
        let t0 = Instant::now();
        let mut rows = 0usize;
        for k in 0..frames {
            let f0 = k as f64 / 60.0;
            for s in 0..sub {
                let t = f0 + (s as f64 + 1.0) / (sub as f64 * 60.0);
                rows = cook.cook(&g, &reg, merge, t).unwrap()[0]
                    .as_stream()
                    .count();
                cook.advance_tick(&g, &reg, t).unwrap();
            }
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / frames as f64;
        if sub == 1 {
            base = ms;
        }
        println!("{sub:3}   {ms:9.3}    {:11.2}x    {rows:6}", ms / base);
    }
}

/// **O TETO do `substeps` é do RELÓGIO DE PAREDE, e sai daqui** (§0: meça antes de limitar).
/// Uma zona com N partículas, custo por QUADRO contra o orçamento de 60 fps (16,7 ms).
#[test]
#[ignore = "measurement probe; run with --ignored --nocapture"]
fn measure_what_a_substep_costs_per_frame() {
    use std::time::Instant;
    let reg = registry();
    println!("\n=== CUSTO por QUADRO de uma zona substepada (orcamento 60fps = 16,67 ms) ===");
    println!("particulas   sub=1     sub=4     sub=8     sub=16    sub=32    sub=64");
    for side in [16u32, 64, 128] {
        let mut g = Graph::new();
        let seed = g.add_node("motion.grid");
        g.set_param(seed, "rows", side as f32);
        g.set_param(seed, "cols", side as f32);
        let zone = g.add_node("sim.zone");
        let wind = g.add_node("force.wind");
        g.set_param(wind, "strength", 40.0);
        g.set_param(wind, "gust", 0.0);
        let step = g.add_node("sim.step");
        wire(&mut g, seed, 0, zone, 0, false);
        wire(&mut g, zone, 0, wind, 0, true);
        wire(&mut g, wind, 0, step, 0, false);
        wire(&mut g, step, 0, zone, 1, false);
        g.validate(&reg).expect("bem-tipado");

        print!("{:>8}     ", side * side);
        for sub in [1u32, 4, 8, 16, 32, 64] {
            let mut cook = Cook::new();
            // aquece: o 1o quadro aloca as colunas
            for k in 0..5u64 {
                let t = (k + 1) as f64 / 60.0;
                cook.substep(&g, &reg, zone, k as f64 / 60.0, t, sub).ok();
                cook.cook(&g, &reg, zone, t).ok();
                cook.advance_tick(&g, &reg, t).ok();
            }
            let t0 = Instant::now();
            const FRAMES: u64 = 20;
            for k in 5..5 + FRAMES {
                let t = (k + 1) as f64 / 60.0;
                cook.substep(&g, &reg, zone, k as f64 / 60.0, t, sub).ok();
                cook.cook(&g, &reg, zone, t).ok();
                cook.advance_tick(&g, &reg, t).ok();
            }
            print!(
                "{:>7.3}   ",
                t0.elapsed().as_secs_f64() * 1e3 / FRAMES as f64
            );
        }
        println!();
    }
}
