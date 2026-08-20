//! **A CHUVA RALA** — a cena `=67` (doc 89, folha 01: a `probability` do `motion.emitter`).
//!
//! Dois jactos com o **mesmo `rate`**. O da esquerda deixa nascer toda a gente; o da direita
//! deixa nascer **40%**.
//!
//! ⚠️ **O que o par prova é que isto NÃO é o `rate` mais baixo.** Baixar o `rate` para 40%
//! afasta as partículas **regularmente** — o jacto fica ralo e certinho. A probabilidade
//! deixa o ritmo intacto e tira partículas **onde calha**: os buracos são irregulares, que é
//! a diferença entre um chuveiro e uma chuva.
//!
//! ⚠️ **Contagem modesta de propósito.** Com a probabilidade abaixo de `1` o emitter **recusa
//! o device** (a contagem passa a depender de dados — ver o `applicable` do kernel dele), e a
//! cena `=5` corre **1,2 milhões** de partículas. Aqui são ~600 por jacto: a cena existe para
//! mostrar a LEI, e a que existe para mostrar o tecto do hardware é a outra.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Partículas por segundo — o MESMO nos dois jactos, e é isso que o par afirma.
const RATE: f32 = 200.0;
/// Quanto tempo cada uma vive.
const LIFE: f32 = 3.0;
/// A fracção que nasce no jacto da direita.
const THIN: f32 = 0.4;
/// O vão entre os dois jactos.
const GAP_X: f32 = 7.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Um jacto: emitter → integrate (com gravidade) → tinta por idade → saída.
fn jet(g: &mut Graph, probability: f32, x: f32, ey: f32) -> Option<NodeId> {
    let em = g.add_node("motion.emitter");
    g.set_pos(em, Pos { x: 0.0, y: ey });
    g.set_param(em, "rate", RATE);
    g.set_param(em, "life", LIFE);
    g.set_param(em, "max", 4096.0);
    g.set_param(em, "probability", probability);
    g.set_param(em, "size", 0.16);
    g.set_param(em, "speed", 11.0);
    g.set_param(em, "speed_random", 0.15);
    g.set_param(em, "angle", 90.0); // para cima, neste mundo Y-up
    g.set_param(em, "spread", 26.0);
    g.set_param(em, "x", x);
    g.set_param(em, "y", -7.0);
    g.set_param(em, "seed", 7.0); // ⚠️ O MESMO seed nos dois: o par isola a probabilidade

    let gravity = g.add_node("force.wind");
    g.set_pos(
        gravity,
        Pos {
            x: 220.0,
            y: ey + 90.0,
        },
    );
    g.set_param(gravity, "angle", 270.0);
    g.set_param(gravity, "strength", 22.0);
    g.set_param(gravity, "gust", 0.0);

    let ig = g.add_node("motion.integrate");
    g.set_pos(ig, Pos { x: 440.0, y: ey });
    wire(g, em, 0, gravity, 0)?;
    wire(g, gravity, 0, ig, 0)?;

    // A cor conta a IDADE (os ids sobem do mais velho para o mais novo), então o jacto sai
    // quente na boca e frio nas pontas — e as falhas aparecem como buracos no degradê.
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 660.0, y: ey });
    g.set_param(tint, "mode", 1.0);
    g.set_param(tint, "r", 1.0);
    g.set_param(tint, "g", 0.84);
    g.set_param(tint, "b", 0.42);
    g.set_param(tint, "r2", 0.2);
    g.set_param(tint, "g2", 0.38);
    g.set_param(tint, "b2", 0.95);
    wire(g, ig, 0, tint, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 880.0, y: ey });
    wire(g, tint, 0, out, 0)?;
    Some(out)
}

/// Monta a cena. Devolve os sinks: `[cheio, ralo]`.
pub(crate) fn build_drizzle_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![jet(g, 1.0, -GAP_X, 0.0)?, jet(g, THIN, GAP_X, 300.0)?])
}

/// Os rótulos dos dois jactos, na ordem em que a cena os monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CHEIO -- Probability 1: toda a gente nasce",
        "RALO -- Probability 0,4: o MESMO rate, e 60% nao nascem",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (RATE, THIN)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_drizzle_tests.rs"]
mod tests;
