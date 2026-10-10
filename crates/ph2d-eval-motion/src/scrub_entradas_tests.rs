//! As ENTRADAS publicadas no scrub (ciclo 19, doc 128 §3.3): um valor que o jogo publicou num
//! tique tem de ser o MESMO quando o scrub volta a esse tique — senão o replay diverge do que se
//! viu a tocar.
//!
//! O arnês: um acumulador sequencial que soma, a cada tique, o número publicado em `$input:g`.
//! A trajectória depende de TODO o histórico da entrada, logo um replay com o valor de agora (e
//! não o da época) aparece no número.

use crate::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const ENTRADA: &str = "$input:g";

static ACC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.acumula"),
    name: "motion.test.acumula",
    inputs: &[PortSpec {
        name: "state",
        ty: INST,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: ph2d_nodegraph::effect::Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Acumula;
impl NodeOp for Acumula {
    fn manifest(&self) -> &'static NodeManifest {
        &ACC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let prev = match ctx.input(0).get("P") {
            Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
            _ => 0.0,
        };
        let g = match ctx.external(ENTRADA).get("v") {
            Some(Column::Scalar(v)) if !v.is_empty() => v[0],
            _ => 0.0,
        };
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[prev + g, 0.0]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == ACC_MAN.id).then_some(&Acumula as &dyn NodeOp)
    }
}

fn grafo() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let c = g.add_node("motion.test.acumula");
    g.connect(Edge {
        from: (c, 0),
        to: (c, 0),
        delayed: true,
    })
    .unwrap();
    (g, c)
}

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SIZE: [f32; 2] = [1.0, 1.0];
const DT: f64 = 1.0 / 60.0;

fn publica(pump: &mut MotionCookPump, g: f32) {
    pump.cook
        .set_external(ENTRADA, Stream::new(1).with("v", Column::Scalar(vec![g])));
}

/// O que o jogo publica no tique `t` desta corrida: 1 até ao 19, 5 do 20 em diante.
fn entrada_da_corrida(t: u64) -> f32 {
    if t < 20 { 1.0 } else { 5.0 }
}

/// Toca para a frente, como o editor: publica a entrada do quadro e coze o tique. Devolve o
/// que se viu em cada tique.
fn toca(
    pump: &mut MotionCookPump,
    g: &Graph,
    c: NodeId,
    de: u64,
    ate: u64,
    e: impl Fn(u64) -> f32,
) -> Vec<f32> {
    let scopes = TimeScopes::new();
    (de..=ate)
        .map(|t| {
            publica(pump, e(t));
            pump.advance_or_scrub_scoped(g, &Ops, &[c], t, |k| k as f64 * DT, UV, SIZE, &scopes);
            pump.instances[0].world_pos[0]
        })
        .collect()
}

fn scrub(pump: &mut MotionCookPump, g: &Graph, c: NodeId, t: u64) -> f32 {
    let scopes = TimeScopes::new();
    pump.advance_or_scrub_scoped(g, &Ops, &[c], t, |k| k as f64 * DT, UV, SIZE, &scopes);
    pump.instances[0].world_pos[0]
}

/// ⭐ A lei: voltar a um tique já tocado mostra o que se viu nele, com a entrada de agora ainda
/// publicada (o jogo não volta atrás com o editor).
#[test]
fn a_scrub_replays_the_inputs_that_were_published_at_each_tick() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    let visto = toca(&mut pump, &g, c, 0, 40, entrada_da_corrida);

    for alvo in [10u64, 25, 3, 39, 20, 19] {
        // O editor publica o valor de AGORA antes do quadro, como o jogo faria.
        publica(&mut pump, entrada_da_corrida(40));
        let x = scrub(&mut pump, &g, c, alvo);
        assert_eq!(
            x, visto[alvo as usize],
            "scrub para {alvo}: viu-se {} ao tocar e o replay deu {x}",
            visto[alvo as usize]
        );
    }
}

/// ⭐ E um RAMO novo: voltar ao 25, mudar a entrada e tocar de novo reescreve o futuro — um
/// scrub ao 30 mostra o ramo NOVO, nunca o checkpoint velho do primeiro ramo.
#[test]
fn playing_on_after_a_scrub_with_a_new_input_writes_a_new_future() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    toca(&mut pump, &g, c, 0, 40, entrada_da_corrida);

    publica(&mut pump, 5.0);
    scrub(&mut pump, &g, c, 25);
    let ramo = |t: u64| if t <= 25 { entrada_da_corrida(t) } else { 2.0 };
    let novo = toca(&mut pump, &g, c, 26, 35, ramo);

    for alvo in [30u64, 33, 27] {
        publica(&mut pump, 2.0);
        let x = scrub(&mut pump, &g, c, alvo);
        assert_eq!(
            x,
            novo[(alvo - 26) as usize],
            "scrub para {alvo} depois do ramo novo: o replay deu {x}"
        );
    }
    // E o ramo novo, fechado à mão: até ao 19 soma 1 (20), do 20 ao 25 soma 5 (+30), do 26 ao
    // 30 soma 2 (+10).
    publica(&mut pump, 2.0);
    assert_eq!(scrub(&mut pump, &g, c, 30), 60.0);
}
