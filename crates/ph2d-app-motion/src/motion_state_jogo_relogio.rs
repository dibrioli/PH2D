//! ⭐ **O RELÓGIO DO GRUPO DO JOGO** (ciclo 19, passo 5 —
//! [doc 128](../../../docs/Motion%20Nodes/128_ciclo_19_o_jogo_manda.md) §5.4).
//!
//! As perguntas: quanto custa o `Follow` por linha; o que o `Driven` poupa quando a entrada está
//! PARADA (um instante parado = memo) contra quando mexe; quanto a fita guarda por tique e quanto
//! custa voltar 300 tiques; e o quadro inteiro da cena `=134`. Tudo na CPU — é onde este grupo
//! corre (o valor vem de um external; um escopo de tempo leva o grafo à CPU, doc 98 §2) —, os
//! casos intercalados quadro a quadro, mínimo e mediana.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_game_group
//! ```
use crate::motion_state::MotionState;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::external::{INPUT_VALUE, input_of};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::time::Instant;

const AMOSTRAS: usize = 31;
const LADO: f32 = 181.0;

fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16, pre: bool) {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: pre,
    })
    .expect("liga");
}

/// `grid 181²` → um campo de valor por linha (`value.time` pela ordem) → `Follow` com laço `pre`.
fn grafo_do_follow(m: &MotionState) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grade = g.add_node("motion.grid");
    g.set_param(grade, "rows", LADO);
    g.set_param(grade, "cols", LADO);
    let tempo = g.add_node("value.time");
    g.set_param(tempo, "delay", 2.0);
    liga(&mut g, grade, tempo, 0, false);
    let seguir = g.add_node("value.follow");
    g.set_param(seguir, ph2d_node_value_follow::MODE, 2.0);
    liga(&mut g, tempo, seguir, 0, false);
    liga(&mut g, seguir, seguir, 1, true);
    g.validate(&m.registry).expect("valida");
    (g, seguir)
}

/// `grid 181²` → `value.time` → `drive Size` → `Time Remap (Driven)` ← `value.input`.
fn grafo_do_driven(m: &MotionState) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grade = g.add_node("motion.grid");
    g.set_param(grade, "rows", LADO);
    g.set_param(grade, "cols", LADO);
    let tempo = g.add_node("value.time");
    g.set_param(tempo, "delay", 2.0);
    liga(&mut g, grade, tempo, 0, false);
    let tamanho = g.add_node("motion.drive");
    g.set_param(tamanho, "channel", 3.0);
    g.set_param(tamanho, "mode", 1.0);
    liga(&mut g, grade, tamanho, 0, false);
    liga(&mut g, tempo, tamanho, 1, false);
    let remap = g.add_node("motion.time_remap");
    g.set_param(remap, "mode", 6.0);
    g.set_param(remap, "duration", 2.2);
    liga(&mut g, tamanho, remap, 0, false);
    let vida = g.add_node("value.input");
    g.set_text_param(vida, "name", "vida");
    g.drive_param(remap, "drive", (vida, 0)).expect("dirige");
    g.validate(&m.registry).expect("valida");
    (g, remap)
}

fn publica(c: &mut Cook, v: f32) {
    c.set_external(
        input_of("vida"),
        Stream::new(1).with(INPUT_VALUE, Column::Scalar(vec![v])),
    );
}

#[test]
#[ignore = "sonda de medição: corre com --release --ignored --nocapture"]
fn measure_the_game_group() {
    let m = MotionState::new();
    let reg = &m.registry;
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load {}", carga.trim());

    // ── 1. o Follow, por linha (32 761), nos três modos.
    let (mut g, seguir) = grafo_do_follow(&m);
    let mut linhas = vec![];
    for (rotulo, modo) in [
        ("follow Linear", 0.0),
        ("follow Smooth", 1.0),
        ("follow Spring", 2.0),
    ] {
        g.set_param(seguir, ph2d_node_value_follow::MODE, modo);
        let mut c = Cook::new();
        let mut v = vec![];
        for k in 0..(AMOSTRAS + 10) {
            let t = k as f64 / 60.0;
            let t0 = Instant::now();
            c.cook(&g, reg, seguir, t).expect("coze");
            c.advance_tick(&g, reg, t).expect("avanca");
            if k >= 10 {
                v.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        let (mn, md) = min_med(&mut v);
        linhas.push(format!("{rotulo:<26}│ {mn:>8.3} / {md:>8.3} ms"));
    }

    // ── 2. o Driven: a entrada PARADA (memo) contra a entrada a MEXER (recoze o escopo).
    let (g, remap) = grafo_do_driven(&m);
    let scopes = |c: &Cook, g: &Graph| {
        let _ = c;
        ph2d_node_motion_time_remap::time_scopes(g, reg)
    };
    let (mut parada, mut mexe) = (Cook::new(), Cook::new());
    let (mut vp, mut vm) = (vec![], vec![]);
    for k in 0..(AMOSTRAS + 10) {
        let t = k as f64 / 60.0;
        publica(&mut parada, 0.5);
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        publica(&mut mexe, (k as f32 / 40.0).fract());
        let s = scopes(&parada, &g);
        let t0 = Instant::now();
        parada.cook_scoped(&g, reg, remap, t, &s).expect("coze");
        let a = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = Instant::now();
        mexe.cook_scoped(&g, reg, remap, t, &s).expect("coze");
        let b = t1.elapsed().as_secs_f64() * 1e3;
        if k >= 10 {
            vp.push(a);
            vm.push(b);
        }
    }
    let (mn, md) = min_med(&mut vp);
    linhas.push(format!(
        "{:<26}│ {mn:>8.3} / {md:>8.3} ms",
        "driven, entrada parada"
    ));
    let (mn, md) = min_med(&mut vm);
    linhas.push(format!(
        "{:<26}│ {mn:>8.3} / {md:>8.3} ms",
        "driven, entrada a mexer"
    ));

    // ── 3. a cena =134 inteira, a fita por tique e um scrub de 300 tiques.
    let mut r = MotionState::new();
    r.doc = ph2d_motion_doc::MotionDoc::new();
    let c = super::monta(&mut r.doc, &r.registry).expect("a cena monta");
    let sinks = vec![c.barra, c.cruza, c.galaxia];
    r.sinks = sinks.clone();
    r.pump.define_a_lei(true);
    r.pump.mark_dirty();
    let mut quadro = vec![];
    for t in 0..=420u64 {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        r.entradas.jogo.set(
            "vida",
            Escritor::Script,
            Valor::Numero((t as f32 / 120.0).fract()),
        );
        r.entradas.jogo.set(
            "estado",
            Escritor::Script,
            Valor::Numero(f32::from(u8::from((t / 90) % 2 == 1))),
        );
        if t % 60 == 0 {
            r.entradas.jogo.fire("boom", Escritor::Script);
        }
        let t0 = Instant::now();
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut r, t as f64 / 60.0);
        let s = ph2d_node_motion_time_remap::time_scopes(&r.doc.graph, &r.registry);
        r.pump.advance_or_scrub_scoped(
            &r.doc.graph,
            &r.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            r.default_uv_rect,
            r.default_size,
            &s,
        );
        if t >= 60 {
            quadro.push(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    let (mn, md) = min_med(&mut quadro);
    linhas.push(format!(
        "{:<26}│ {mn:>8.3} / {md:>8.3} ms",
        "cena =134, um quadro"
    ));
    let bytes = r.pump.fita().bytes();
    linhas.push(format!(
        "{:<26}│ {bytes} B em 421 tiques = {:.1} B/tique",
        "a fita (3 entradas a mexer)",
        bytes as f64 / 421.0
    ));
    let mut volta = vec![];
    for alvo in [120u64, 300, 60, 400, 10] {
        let s = ph2d_node_motion_time_remap::time_scopes(&r.doc.graph, &r.registry);
        let t0 = Instant::now();
        r.pump.advance_or_scrub_scoped(
            &r.doc.graph,
            &r.registry,
            &sinks,
            alvo,
            |t| t as f64 / 60.0,
            r.default_uv_rect,
            r.default_size,
            &s,
        );
        volta.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let (mn, md) = min_med(&mut volta);
    linhas.push(format!(
        "{:<26}│ {mn:>8.3} / {md:>8.3} ms",
        "scrub (5 alvos)"
    ));

    eprintln!("  caso                      │   mínimo / mediana");
    for l in linhas {
        eprintln!("  {l}");
    }
}
