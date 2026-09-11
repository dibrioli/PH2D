//! **SONDA — o `motion.integrate` já é sub-passável pelo motor que existe?**
//!
//! A folha 17 linha 76 marca `P1` pedindo *sub-steps / o timestep exposto* no `motion.integrate`,
//! e lista quatro rotas tentadas e recusadas — a (c) sendo *"cozinhar N vezes por tick — o
//! `ticks_owed` do `motion_bridge` é o relógio do `Playhead`, não um knob do grafo"* e a (d)
//! *"`sim.zone` roda o miolo **uma** vez por tick"*.
//!
//! ⚠️ **As duas caíram em 2026-08-12, e a célula não foi reconferida** (§0 do `CLAUDE.md`: *quem
//! move o número que tornava algo inalcançável tem de reconferir a nota*). A folha 13 linha 59
//! registra o motor que aterrou: [`Cook::substep`] subdivide o **playhead** dentro de um tique e
//! re-cozinha o cone do alvo a cada fatia, e a declaração é uma **convenção de manifesto** — um nó
//! que oferece um param `substeps` diz *"o meu interior sub-tica"*. Nada nesse motor sabe o que é
//! uma zona.
//!
//! Então a pergunta desta sonda é estreita e mensurável: **entregar o `motion.integrate` a esse
//! bracket integra, ou só re-cozinha?** Ela mede a cadeia real (`grid → integrate`, com
//! `integrate =pre=> force.wind =fwd=> integrate.forces`) contra a analítica `a·T²/2` da queda sob
//! aceleração constante, em `1/2/4/8/16` sub-passadas.
//!
//! ⚠️ **E ela mede uma segunda coisa, que é a que decide o DESENHO:** a rota local — um laço de
//! `N` passos dentro do `eval`, com a aceleração que já está lá — contra a rota do RELÓGIO, que
//! re-cozinha a cadeia de `force.*` a cada fatia. Só a segunda é o que a referência chama
//! *substep*; a primeira não volta a perguntar quanta força há. A fixture que as separa é uma
//! força que **varia no tempo** (`force.wind` com `gust`), porque com aceleração constante as duas
//! respondem igual e a viagem é cega à diferença.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_integrate_substeps -- --ignored --nocapture`.

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

/// A cadeia canônica do integrador: `grid → integrate.rest` e o laço
/// `integrate.out =pre=> force.wind =fwd=> integrate.forces`.
///
/// `gust` é a rajada senoidal do `force.wind` — a **zero** a aceleração é constante e a resposta
/// exata é `a·T²/2`; acima de zero ela varia dentro do tique, que é o caso em que re-cozinhar a
/// força importa.
fn falling_chain(g: &mut Graph, strength: f32, gust: f32, gust_freq: f32) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    g.set_param(seed, "gap_x", 1.0);
    g.set_param(seed, "gap_y", 1.0);

    let integ = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "strength", strength);
    g.set_param(wind, "gust", gust);
    g.set_param(wind, "gust_freq", gust_freq);

    wire(g, seed, 0, integ, 0, false);
    wire(g, integ, 0, wind, 0, true);
    wire(g, wind, 0, integ, 1, false);
    integ
}

fn px(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
        _ => f32::NAN,
    }
}

/// Corre `frames` quadros a 60 fps, pedindo `sub` sub-passadas ao alvo em cada um.
fn run(g: &Graph, reg: &NodeRegistry, target: NodeId, frames: u64, sub: u32) -> f32 {
    let mut cook = Cook::new();
    let mut last = f32::NAN;
    for k in 0..frames {
        let t = (k + 1) as f64 / 60.0;
        cook.substep(g, reg, target, k as f64 / 60.0, t, sub)
            .expect("substeps");
        last = px(cook.cook(g, reg, target, t).expect("cooks")[0].as_stream());
        cook.advance_tick(g, reg, t).expect("tick");
    }
    last
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_clock_bracket_does_to_the_integrator() {
    let reg = registry();
    let a = 40.0f32;

    let mut g = Graph::new();
    let integ = falling_chain(&mut g, a, 0.0, 1.0);
    g.validate(&reg).expect("bem-tipado");

    let exact = a / 2.0; // a·T²/2, T = 1 s
    eprintln!("\n[integrate-substep] queda de 1 s sob aceleracao CONSTANTE a = {a}");
    eprintln!("  analitica a*T^2/2 = {exact:.6}\n");
    eprintln!(
        "  {:>4}  {:>12}  {:>12}  {:>8}",
        "sub", "P final", "erro", "razao"
    );
    let mut prev = f32::INFINITY;
    for sub in [1u32, 2, 4, 8, 16, 32] {
        let p = run(&g, &reg, integ, 60, sub);
        let err = (p - exact).abs();
        let ratio = if prev.is_finite() {
            format!("{:.3}x", prev / err)
        } else {
            "-".into()
        };
        eprintln!("  {sub:>4}  {p:>12.6}  {err:>12.6}  {ratio:>8}");
        prev = err;
    }

    eprintln!(
        "\n  LEITURA: se as linhas forem IGUAIS, o bracket e' um no-op para este no'.
  Se o erro cair pela METADE a cada dobra, ele INTEGRA — Euler de 1a ordem, a mesma
  assinatura que a `sim.zone` mede na folha 13."
    );
}

/// **O TETO, e o recurso é o RELÓGIO DE PAREDE** (§0: meça antes de limitar). Custo por QUADRO da
/// cadeia substepada contra o orçamento de 60 fps (16,67 ms).
///
/// ⚠️ Rode em `--release` e com a máquina CALMA — nenhuma leitura de relógio desta workstation
/// vale acima de `load ~5`.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_a_substep_costs_the_integrator_per_frame() {
    let reg = registry();
    eprintln!("\n[integrate-substep] custo por QUADRO (ms), orcamento de 60 fps = 16,67 ms\n");
    eprint!("  {:>10}", "elementos");
    for sub in [1u32, 8, 16, 32, 64] {
        eprint!("  {:>9}", format!("sub={sub}"));
    }
    eprintln!();
    for n in [256usize, 4096, 16384] {
        eprint!("  {n:>10}");
        for sub in [1u32, 8, 16, 32, 64] {
            let mut g = Graph::new();
            let seed = g.add_node("motion.grid");
            g.set_param(seed, "rows", 1.0);
            g.set_param(seed, "cols", n as f32);
            let integ = g.add_node("motion.integrate");
            let wind = g.add_node("force.wind");
            g.set_param(wind, "angle", 0.0);
            g.set_param(wind, "strength", 40.0);
            g.set_param(wind, "gust", 0.0);
            wire(&mut g, seed, 0, integ, 0, false);
            wire(&mut g, integ, 0, wind, 0, true);
            wire(&mut g, wind, 0, integ, 1, false);
            g.validate(&reg).expect("bem-tipado");

            const FRAMES: u64 = 20;
            let mut cook = Cook::new();
            // Um quadro de aquecimento fora do relógio: o 1º semeia e aloca.
            cook.cook(&g, &reg, integ, 0.0).expect("coza");
            cook.advance_tick(&g, &reg, 0.0).expect("tick");
            let t0 = std::time::Instant::now();
            for k in 0..FRAMES {
                let t = (k + 2) as f64 / 60.0;
                cook.substep(&g, &reg, integ, (k + 1) as f64 / 60.0, t, sub)
                    .expect("sub");
                cook.cook(&g, &reg, integ, t).expect("coza");
                cook.advance_tick(&g, &reg, t).expect("tick");
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / FRAMES as f64;
            eprint!("  {ms:>9.3}");
        }
        eprintln!();
    }
    eprintln!(
        "\n  LEITURA: o numero que decide o teto DIGITAVEL e' onde uma cena pesada come o
  quadro; a faixa CONFORTAVEL do arrasto para onde o erro ja' nao se ve'."
    );
}

/// A metade que decide o DESENHO: com a força a variar dentro do tique, sub-passar o RELÓGIO
/// re-pergunta quanta força há; um laço local dentro do `eval` não.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_re_cooking_the_force_chain_change_the_answer() {
    let reg = registry();
    let a = 40.0f32;

    eprintln!("\n[integrate-substep] a forca VARIA no tique (force.wind com gust, freq 40)");
    eprintln!("  {:>6}  {:>4}  {:>12}", "gust", "sub", "P final");
    for gust in [0.0f32, 0.9] {
        let mut g = Graph::new();
        let integ = falling_chain(&mut g, a, gust, 40.0);
        g.validate(&reg).expect("bem-tipado");
        for sub in [1u32, 4, 16] {
            let p = run(&g, &reg, integ, 60, sub);
            eprintln!("  {gust:>6.1}  {sub:>4}  {p:>12.6}");
        }
    }

    eprintln!(
        "\n  LEITURA: com gust = 0 a aceleracao e' constante e as sub-passadas so' refinam a
  aritmetica. Com gust > 0 a aceleracao muda DENTRO do tique — e ai' a diferenca
  entre as linhas e' o que um laco local (que congela `accel`) NAO consegue dar."
    );
}

/// **A CENA: onde é que 1 sub-passo QUEBRA e 8 aguentam?** O regime em que a diferença vive é o
/// do passo grande — um atrator forte é o caso canônico (Euler explícito tem limite de
/// estabilidade `dt²·k < 4`, e sub-passar divide `dt` por `n`, ou seja compra `n²` de margem).
///
/// Varre a força e imprime o RAIO máximo alcançado em 3 s: um corpo que oscila num poço fica
/// limitado; um que ganha energia foge.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn where_one_substep_breaks_and_eight_hold() {
    let reg = registry();
    eprintln!("\n[integrate-substep] raio MAXIMO em 3 s (comeca a 4,0 do centro)\n");
    eprint!("  {:>9}", "strength");
    for sub in [1u32, 2, 4, 8, 16, 32] {
        eprint!("  {:>9}", format!("sub={sub}"));
    }
    eprintln!();
    for strength in [800.0f32, 3200.0, 12800.0] {
        eprint!("  {strength:>9.0}");
        for sub in [1u32, 2, 4, 8, 16, 32] {
            let mut g = Graph::new();
            let seed = g.add_node("motion.grid");
            g.set_param(seed, "rows", 1.0);
            g.set_param(seed, "cols", 1.0);
            let mv = g.add_node("motion.move");
            g.set_param(mv, "dx", 4.0);
            wire(&mut g, seed, 0, mv, 0, false);
            let integ = g.add_node("motion.integrate");
            let att = g.add_node("force.attractor");
            g.set_param(att, "target_x", 0.0);
            g.set_param(att, "target_y", 0.0);
            g.set_param(att, "strength", strength);
            g.set_param(att, "radius", 40.0);
            wire(&mut g, mv, 0, integ, 0, false);
            wire(&mut g, integ, 0, att, 0, true);
            wire(&mut g, att, 0, integ, 1, false);
            g.validate(&reg).expect("bem-tipado");

            let mut cook = Cook::new();
            let mut worst = 0.0f32;
            for k in 0..180u64 {
                let t = (k + 1) as f64 / 60.0;
                if let Some(fs) = cook.prev_playhead() {
                    cook.substep(&g, &reg, integ, fs, t, sub).expect("sub");
                }
                let s = cook.cook(&g, &reg, integ, t).expect("coza")[0].as_stream();
                if let Some(Column::Vec2(v)) = s.get("P")
                    && let Some(q) = v.first()
                {
                    worst = worst.max((q[0] * q[0] + q[1] * q[1]).sqrt());
                }
                cook.advance_tick(&g, &reg, t).expect("tick");
            }
            eprint!("  {worst:>12.3}");
        }
        eprintln!();
    }
    eprintln!(
        "\n  LEITURA: 4,0 = o corpo nunca passa do ponto de partida (oscilacao sadia).
  Muito acima disso = ele ganhou energia e esta' a fugir do poco."
    );
}
