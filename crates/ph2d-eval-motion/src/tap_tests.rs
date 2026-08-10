//! Gates das TOMADAS — o que impede a marcha de andar duas vezes.
//!
//! ⚠️ O gate que carrega o arquivo **conta EVALS, não milissegundos**: *"o prefixo compartilhado
//! é simulado uma vez"* é uma propriedade sobre quantas vezes um nó foi avaliado, e um relógio
//! responderia a pergunta errada (e flakaria sob carga). É o mesmo oráculo que o irmão
//! `Boundaries` já usa para a mesma afirmação.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::Edge;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::cell::Cell;

const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

// Quantas vezes a FONTE foi avaliada — o oráculo.
//
// ⚠️ **POR THREAD, e não um atômico global.** Os testes correm em paralelo e um contador
// compartilhado lê a soma dos vizinhos: a primeira versão deste arquivo mediu `3` onde a
// resposta é `1`. Esta crate não tem `rayon`, então todo eval acontece na thread que chamou o
// pump — o que torna a poluição estruturalmente impossível, em vez de exigir uma trava que
// alguém esquece de segurar (a cicatriz que o `ph2d-painter-brush` pagou).
thread_local! {
    static SOURCE_EVALS: Cell<usize> = const { Cell::new(0) };
}

fn reset_evals() {
    SOURCE_EVALS.with(|c| c.set(0));
}

fn evals() -> usize {
    SOURCE_EVALS.with(Cell::get)
}

struct Source;
static SOURCE_MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("tap.source"),
    name: "tap.source",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
impl NodeOp for Source {
    fn manifest(&self) -> &'static NodeManifest {
        &SOURCE_MANIFEST
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        SOURCE_EVALS.with(|c| c.set(c.get() + 1));
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[1.0, 2.0]])));
    }
}

struct Pass;
static PASS_MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("tap.pass"),
    name: "tap.pass",
    inputs: &[PortSpec {
        name: "in",
        ty: INST,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
impl NodeOp for Pass {
    fn manifest(&self) -> &'static NodeManifest {
        &PASS_MANIFEST
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let s = ctx.input(0).clone();
        ctx.emit(s);
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, id: NodeTypeId) -> Option<&dyn NodeOp> {
        static SOURCE: Source = Source;
        static PASS: Pass = Pass;
        if id == SOURCE_MANIFEST.id {
            Some(&SOURCE)
        } else if id == PASS_MANIFEST.id {
            Some(&PASS)
        } else {
            None
        }
    }
}

/// `fonte → meio → sink`, com o MEIO como tomada: os dois compartilham a fonte.
fn chain() -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("tap.source");
    let mid = g.add_node("tap.pass");
    let sink = g.add_node("tap.pass");
    for (a, b) in [(src, mid), (mid, sink)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("bem tipado");
    }
    (g, mid, sink)
}

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SIZE: [f32; 2] = [1.0, 1.0];

/// **O prefixo compartilhado é simulado UMA vez.** A tomada cavalga a marcha dos sinks e bate
/// no memo — se ela fosse uma segunda marcha, a fonte seria avaliada duas vezes por tique **e o
/// relógio andaria duas vezes**, que é o defeito SILENCIOSO que o `Boundaries` já documenta.
#[test]
fn a_tomada_cavalga_a_marcha_dos_sinks() {
    let (g, mid, sink) = chain();
    let scopes = TimeScopes::new();
    reset_evals();
    let mut pump = MotionCookPump::new();
    pump.advance_or_scrub_with_taps_scoped(
        &g,
        &Ops,
        &[sink],
        &[mid],
        0,
        |t| t as f64,
        UV,
        SIZE,
        &scopes,
    );
    assert_eq!(
        evals(),
        1,
        "a fonte foi avaliada UMA vez, apesar de sink e tomada a compartilharem"
    );
    assert_eq!(pump.tap_streams().len(), 1, "e a tomada voltou");
    assert_eq!(pump.tap_streams()[0].0, mid);
    assert_eq!(pump.last_cooked_tick(), Some(0), "um tique, nao dois");
}

/// **Sem tomadas o mundo é o de antes** — a lista vazia não cozinha nada e não guarda nada, e a
/// irmã sem tomadas delega para cá com `&[]`, logo não há um segundo caminho para divergir.
#[test]
fn a_lista_vazia_e_o_mundo_anterior() {
    let (g, _mid, sink) = chain();
    let scopes = TimeScopes::new();
    reset_evals();
    let mut pump = MotionCookPump::new();
    pump.advance_or_scrub_scoped(&g, &Ops, &[sink], 0, |t| t as f64, UV, SIZE, &scopes);
    assert_eq!(evals(), 1);
    assert!(
        pump.tap_streams().is_empty(),
        "nada cozinhou e nada foi guardado"
    );
}

/// **Um SCRUB para trás cozinha as tomadas pelo MESMO caminho.** As duas rotas compartilham o
/// `cook_target_into` de propósito — uma tomada que só existisse no play seria a divergência
/// play-vs-scrub que a doc do pump nomeia como a armadilha clássica de determinismo.
#[test]
fn o_scrub_tambem_entrega_as_tomadas() {
    let (g, mid, sink) = chain();
    let scopes = TimeScopes::new();
    let mut pump = MotionCookPump::new();
    for t in 0..=4 {
        pump.advance_or_scrub_with_taps_scoped(
            &g,
            &Ops,
            &[sink],
            &[mid],
            t,
            |t| t as f64,
            UV,
            SIZE,
            &scopes,
        );
    }
    // De volta ao tique 1 — o caminho de scrub (restaura o checkpoint e re-simula).
    pump.advance_or_scrub_with_taps_scoped(
        &g,
        &Ops,
        &[sink],
        &[mid],
        1,
        |t| t as f64,
        UV,
        SIZE,
        &scopes,
    );
    assert_eq!(
        pump.tap_streams().len(),
        1,
        "a tomada volta no scrub, nao so no play"
    );
    assert_eq!(pump.tap_streams()[0].0, mid);
}
