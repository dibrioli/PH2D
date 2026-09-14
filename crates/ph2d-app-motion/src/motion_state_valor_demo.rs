//! ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (`PH2D_GPU_COOK_DEMO=117`) — a cena do **ciclo 6**
//! ([doc 110](../../docs/Motion%20Nodes/110_ciclo_6_valor_e_pulso.md)).
//!
//! ## O que ela ensina, e por que são QUATRO quadrantes
//!
//! O grupo são **duas** famílias e elas respondem perguntas diferentes: `value.*` é **o NÚMERO** e
//! `pulse.*` é **o INSTANTE**. Uma cena que só mostrasse uma delas ensinaria que o grupo é metade
//! do que é.
//!
//! ```text
//!   EM CIMA   o mesmo pano, o mesmo LFO     só o LFO        →  respira LISO
//!                                           + Quantize      →  respira aos DEGRAUS
//!   EM BAIXO  o mesmo pano, o mesmo Beat    só o Beat       →  pisca a CADA batida
//!                                           + Counter       →  pisca a cada QUATRO
//! ```
//!
//! ⚠️⚠️ **Em cada par muda UM NÓ, e mais nada.** O pano, o tamanho, o período e a cor são os
//! mesmos; se a diferença estivesse num número, a cena ensinaria que o grupo é uma gaveta de
//! sliders. Ela é sobre o que um nó a MAIS faz ao número e ao instante que já lá estavam.
//!
//! ⚠️ **Ela precisa de Play** — o `value.lfo` e o `pulse.beat` leem o playhead, e parada ela é
//! quatro panos iguais.
//!
//! ⚠️ **O pano é PEQUENO** (`6 × 6`), ao contrário da `=116`. Aquela ensina um CUSTO e por isso
//! tem 102 400 peças; esta ensina uma LEI, e uma lei lê-se melhor em trinta e seis peças do que em
//! cem mil.
//!
//! ⚠️ **A metade de baixo corre na CPU e isso é DECLARADO** (doc 110 §8.6): o `motion.strobe` é o
//! consumidor de pulso que o artista alcança primeiro e ainda **não tem kernel**. *Uma cena de
//! ciclo mostra o que o grupo FAZ; a rota dele tem a `=116` para isso.*

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// O lado de cada pano. ⚠️ **Seis**: abaixo disto um pano deixa de se ler como um pano, e acima
/// dele os quatro quadrantes passam a tocar-se.
const LADO: f32 = 6.0;
/// O vão entre peças de um pano.
const VAO: f32 = 0.42;
/// Quanto os quadrantes se afastam do centro, em cada eixo.
const AFASTA_X: f32 = 1.9;
const AFASTA_Y: f32 = 1.55;
/// O período do oscilador, em segundos — lento o bastante para a respiração se seguir a olho.
const PERIODO: f32 = 2.4;
/// Quanto o tamanho varia. `1` seria a peça a desaparecer no vale da onda.
const AMPLITUDE: f32 = 0.55;
/// O degrau do `value.quantize`. ⚠️ **Grosso de propósito:** com o degrau fino a metade da direita
/// fica indistinguível da esquerda, e o par deixa de ensinar o que quer que seja.
const DEGRAU: f32 = 0.35;
/// Segundos por batida do metrónomo.
const BATIDA: f32 = 0.6;
/// De quantas em quantas batidas o contador transborda — o `carry` que a metade da direita usa.
const A_CADA: f32 = 4.0;

/// As legendas que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![
        Caption::new([-AFASTA_X, AFASTA_Y + 1.35], "O NUMERO: respira LISO"),
        Caption::new(
            [AFASTA_X, AFASTA_Y + 1.35],
            "+ Quantize: respira aos DEGRAUS",
        ),
        Caption::new(
            [-AFASTA_X, -AFASTA_Y - 1.35],
            "O INSTANTE: pisca a CADA batida",
        ),
        Caption::new(
            [AFASTA_X, -AFASTA_Y - 1.35],
            "+ Counter: pisca a cada QUATRO",
        ),
    ]
}

/// Liga `de` a `para`, com o `pre` a dizer se a aresta atravessa o tique.
fn liga(doc: &mut MotionDoc, de: (NodeId, u16), para: (NodeId, u16), pre: bool) -> Option<()> {
    doc.graph
        .connect(Edge {
            from: de,
            to: para,
            delayed: pre,
        })
        .ok()
}

/// Um pano de `LADO × LADO` centrado em `centro`.
fn pano(doc: &mut MotionDoc, centro: [f32; 2], x: f32, y: f32) -> NodeId {
    let g = doc.graph.add_node("motion.grid");
    doc.graph.set_param(g, "rows", LADO);
    doc.graph.set_param(g, "cols", LADO);
    doc.graph.set_param(g, "gap_x", VAO);
    doc.graph.set_param(g, "gap_y", VAO);
    let mv = doc.graph.add_node("motion.move");
    doc.graph.set_param(mv, "dx", centro[0]);
    doc.graph.set_param(mv, "dy", centro[1]);
    liga(doc, (g, 0), (mv, 0), false);
    doc.graph.set_pos(g, Pos { x, y });
    doc.graph.set_pos(mv, Pos { x: x + 170.0, y });
    mv
}

/// Monta a cena. Devolve os quatro sinks.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let mut sinks = Vec::new();

    // ── EM CIMA: o NÚMERO ────────────────────────────────────────────────────────────────
    for (k, com_degrau) in [(0usize, false), (1, true)].into_iter() {
        let lado = if k == 0 { -AFASTA_X } else { AFASTA_X };
        let y_grafo = if k == 0 { -260.0 } else { -60.0 };
        let base = pano(doc, [lado, AFASTA_Y], -520.0, y_grafo);
        let escala = doc.graph.add_node("motion.scale");
        let saida = doc.graph.add_node("motion.output");
        let lfo = doc.graph.add_node("value.lfo");
        doc.graph.set_param(lfo, "period", PERIODO);
        doc.graph.set_param(lfo, "amplitude", AMPLITUDE);
        // Sem o centro a onda desceria abaixo de zero e metade do ciclo seria tamanho negativo.
        doc.graph.set_param(lfo, "offset", 1.0);
        liga(doc, (base, 0), (escala, 0), false)?;
        liga(doc, (escala, 0), (saida, 0), false)?;
        doc.graph.set_pos(
            lfo,
            Pos {
                x: -180.0,
                y: y_grafo - 90.0,
            },
        );
        doc.graph.set_pos(
            escala,
            Pos {
                x: 20.0,
                y: y_grafo,
            },
        );
        doc.graph.set_pos(
            saida,
            Pos {
                x: 220.0,
                y: y_grafo,
            },
        );
        // ⭐ **A ÚNICA diferença do par**: um nó no meio do fio.
        let fonte = if com_degrau {
            let q = doc.graph.add_node("value.quantize");
            doc.graph.set_param(q, "step", DEGRAU);
            liga(doc, (lfo, 0), (q, 0), false)?;
            doc.graph.set_pos(
                q,
                Pos {
                    x: -20.0,
                    y: y_grafo - 90.0,
                },
            );
            q
        } else {
            lfo
        };
        doc.graph.drive_param(escala, "amount", (fonte, 0)).ok()?;
        sinks.push(saida);
    }

    // ── EM BAIXO: o INSTANTE ─────────────────────────────────────────────────────────────
    for (k, com_contador) in [(0usize, false), (1, true)].into_iter() {
        let lado = if k == 0 { -AFASTA_X } else { AFASTA_X };
        let y_grafo = if k == 0 { 160.0 } else { 380.0 };
        let base = pano(doc, [lado, -AFASTA_Y], -520.0, y_grafo);
        let batida = doc.graph.add_node("pulse.beat");
        doc.graph.set_param(batida, "period", BATIDA);
        liga(doc, (base, 0), (batida, 0), false)?;
        // ⚠️ O `pre` do metrónomo é o que faz dele um metrónomo: sem a memória do ciclo
        // anterior ele não sabe que a batida MUDOU, e dispara em todo o tique.
        liga(doc, (batida, 0), (batida, 1), true)?;
        doc.graph.set_pos(
            batida,
            Pos {
                x: -180.0,
                y: y_grafo + 90.0,
            },
        );

        // ⭐ **A ÚNICA diferença do par**: o pulso passa por um contador, e o que chega ao
        // flash é o **carry** — a porta que transborda de quatro em quatro.
        let (gatilho, porta) = if com_contador {
            let c = doc.graph.add_node("pulse.counter");
            doc.graph.set_param(c, "count_max", A_CADA);
            liga(doc, (batida, 0), (c, 0), false)?;
            liga(doc, (c, 0), (c, 1), true)?;
            doc.graph.set_pos(
                c,
                Pos {
                    x: -20.0,
                    y: y_grafo + 90.0,
                },
            );
            (c, 1u16)
        } else {
            (batida, 0u16)
        };

        let flash = doc.graph.add_node("motion.strobe");
        // Um clarão largo e curto: a peça incha e volta, e vê-se de longe.
        doc.graph.set_param(flash, "size_boost", 0.9);
        doc.graph.set_param(flash, "flash_amount", 1.0);
        doc.graph.set_param(flash, "flash_r", 1.0);
        doc.graph.set_param(flash, "flash_g", 0.85);
        doc.graph.set_param(flash, "flash_b", 0.2);
        doc.graph.set_param(flash, "decay", 0.12);
        let saida = doc.graph.add_node("motion.output");
        liga(doc, (base, 0), (flash, 0), false)?;
        liga(doc, (gatilho, porta), (flash, 1), false)?;
        // O `state` do flash é a memória do brilho a desvanecer.
        liga(doc, (flash, 0), (flash, 2), true)?;
        liga(doc, (flash, 0), (saida, 0), false)?;
        doc.graph.set_pos(
            flash,
            Pos {
                x: 20.0,
                y: y_grafo,
            },
        );
        doc.graph.set_pos(
            saida,
            Pos {
                x: 220.0,
                y: y_grafo,
            },
        );
        sinks.push(saida);
    }

    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_valor_demo_tests.rs"]
mod tests;
