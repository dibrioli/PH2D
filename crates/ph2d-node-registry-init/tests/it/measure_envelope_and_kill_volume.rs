//! **SONDA — o ENVELOPE de nascimento e o KILL VOLUME já são exprimíveis?**
//!
//! Duas células da folha 13, ambas com a coluna *"exprimível?"* a dizer **SIM** e uma cadeia
//! escrita — e nenhuma das duas foi corrida. ⚠️ *Uma cadeia escrita numa célula é uma
//! hipótese; só a corrida a torna uma refutação.* Esta sonda corre as duas.
//!
//! ```text
//! envelope:     value.time → value.step → dirige `rate` do sim.spawn
//! kill volume:  motion.falloff(Circle, invert) → motion.cull(Falloff)
//! ```
//!
//! ⚠️ **O que decide o envelope não é «o rate desce»: é a HISTÓRIA.** O `born_in` do
//! `sim.spawn` usa o `rate` de AGORA nos dois termos do `floor` — foi isso que refutou a
//! cadeia da *probabilidade* (sonda irmã) —, então a pergunta é se um `rate` que vai a zero
//! **para** de emitir ou **re-deriva** um passado que nunca aconteceu.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_envelope_and_kill_volume -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Quantos quadros a 60 fps.
const TICKS: usize = 300;
/// Onde o envelope corta, em segundos.
const CUT_S: f32 = 2.0;
/// A taxa de emissao, em nascimentos por segundo.
const RATE: f32 = 30.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .expect("wire");
}

/// Liga a saída de `from` ao PARAM `param` de `to` (a rota dos params dirigidos).
fn drive(g: &mut Graph, from: NodeId, to: NodeId, param: &str) {
    g.drive_param(to, param, (from, 0)).expect("dirige");
}

/// Corre `ticks` quadros e devolve quantas linhas existiam em cada um.
fn run(g: &Graph, reg: &NodeRegistry, out: NodeId, ticks: usize) -> Vec<usize> {
    let mut cook = Cook::new();
    let mut counts = Vec::with_capacity(ticks);
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        let s: Stream = cook.cook(g, reg, out, t).expect("cooks")[0]
            .as_stream()
            .clone();
        counts.push(s.count());
        cook.advance_tick(g, reg, t).expect("advances");
    }
    counts
}

/// ⚠️ **A 1.ª versão desta sonda mediu a MINHA cadeia, não o produto — duas vezes.**
///
/// 1. Ela lia a contagem de UM tique como se fosse um acumulado: o `sim.spawn` emite os
///    RECÉM-NASCIDOS daquele tique, então a `30/s` a 60 fps a coluna certa é `0` ou `1` e o
///    «não cresceu» era a leitura, não o produto.
/// 2. E o `value.math(Mul)` tinha a segunda entrada **solta**, que vale `0` — a taxa dirigida
///    era zero desde o instante zero, e o «envelope» que eu media era um interruptor
///    permanentemente desligado. *O CONTROLO é que denuncia isto: ele tem de emitir.*
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_a_driven_rate_stop_the_births_or_rewrite_the_past() {
    let reg = registry();
    eprintln!("\n[envelope] `value.time` -> `value.step(invert)` -> `map_range` -> `rate`\n");
    eprintln!(
        "  {:>8}  {:>12}  {:>12}  a emissao parou depois do corte?",
        "corte", "nascidos ate'", "nascidos depois"
    );
    let cut_tick = (CUT_S * 60.0) as usize;
    for cut in [f32::INFINITY, CUT_S] {
        let mut g = Graph::new();
        let src = g.add_node("motion.grid");
        g.set_param(src, "rows", 1.0);
        g.set_param(src, "cols", 1.0);
        let spawn = g.add_node("sim.spawn");
        g.set_param(spawn, "rate", RATE);
        wire(&mut g, src, 0, spawn, 0);
        if cut.is_finite() {
            // O relógio, o degrau invertido (1 ANTES do corte, 0 depois) e a régua que o
            // leva de `[0,1]` para `[0, RATE]` — um `value.math` precisaria de uma segunda
            // entrada, e foi ela solta que fez a 1.ª versão medir um zero constante.
            let clock = g.add_node("value.time");
            let step = g.add_node("value.step");
            g.set_param(step, "threshold", cut);
            g.set_param(step, "invert", 1.0);
            wire(&mut g, clock, 0, step, 0);
            let scale = g.add_node("value.map_range");
            g.set_param(scale, "in_lo", 0.0);
            g.set_param(scale, "in_hi", 1.0);
            g.set_param(scale, "out_lo", 0.0);
            g.set_param(scale, "out_hi", RATE);
            wire(&mut g, step, 0, scale, 0);
            drive(&mut g, scale, spawn, "rate");
        }
        g.validate(&reg).expect("bem-tipado");
        let c = run(&g, &reg, spawn, TICKS);
        let before: usize = c.iter().take(cut_tick).sum();
        let after: usize = c.iter().skip(cut_tick).sum();
        let parou = if after == 0 { "SIM" } else { "nao" };
        let rotulo = if cut.is_finite() {
            format!("{cut:.0}s")
        } else {
            "nenhum".to_string()
        };
        eprintln!("  {rotulo:>8}  {before:>12}  {after:>12}  {parou}");
    }
    eprintln!(
        "\n  LEITURA: o CONTROLO («nenhum») tem de nascer nos dois intervalos -- se ele nao
  nascer, a sonda esta' errada e nao o produto. Com corte, `nascidos depois = 0` diz que
  o envelope existe por composicao; um numero > 0 diz que o `rate` dirigido nao para a
  emissao, e a celula fica de pe' pelo motivo do `born_in`."
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_falloff_plus_cull_already_kill_by_place() {
    let reg = registry();
    eprintln!("\n[kill volume] `motion.falloff(Circle, invert)` -> `motion.cull(Falloff)`\n");
    eprintln!(
        "  {:>10}  {:>8}  {:>8}  quantos sobreviveram",
        "raio", "entram", "saem"
    );
    // Uma grelha 11x11 de -5 a 5; o volume é um disco no centro.
    for radius in [0.0f32, 1.5, 3.0, 9.0] {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 11.0);
        g.set_param(grid, "cols", 11.0);
        g.set_param(grid, "gap_x", 1.0);
        g.set_param(grid, "gap_y", 1.0);
        let fall = g.add_node("motion.falloff");
        g.set_param(fall, "shape", 0.0); // Circle
        g.set_param(fall, "radius", radius);
        g.set_param(fall, "invert", 1.0);
        wire(&mut g, grid, 0, fall, 0);
        let cull = g.add_node("motion.cull");
        g.set_param(cull, "mode", 1.0); // Falloff
        wire(&mut g, fall, 0, cull, 0);
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let inp = cook.cook(&g, &reg, grid, 0.0).expect("coza")[0]
            .as_stream()
            .count();
        let out = cook.cook(&g, &reg, cull, 0.0).expect("coza")[0]
            .as_stream()
            .count();
        eprintln!("  {radius:>10.1}  {inp:>8}  {out:>8}");
    }
    eprintln!(
        "\n  LEITURA: se o numero de sobreviventes CAIR com o raio, matar-por-lugar existe
  em dois nos que ja' existem, e a celula (e a linha 96 do doc 63, que a marcava P1)
  fecham por refutacao. Se nao cair, o `invert` nao faz o que a celula supoe."
    );
}
