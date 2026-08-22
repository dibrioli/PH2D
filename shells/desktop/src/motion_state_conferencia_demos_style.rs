//! **TRÊS EXEMPLOS, UM POR LINHA** — a cena `=76` (doc 89, folha 14: as duas células
//! que a fechavam, mais o preenchimento que o traço apagava).
//!
//! | linha | esquerda | direita |
//! |---|---|---|
//! | **BORDA** | a forma chapada | a MESMA forma com **borda de outra cor** — miolo E borda |
//! | **APARADO** | o anel inteiro | só um **trecho** dele, e o trecho **CORRE** |
//! | **PICOTADO** | o contorno contínuo | o contorno **picotado** |
//!
//! ⚠️ **A linha do meio é a única que precisa de PLAY** — o `trim_offset` dela é conduzido
//! pelo relógio por um FIO (`Graph::drive_param`, doc 58). As outras duas leem-se paradas.
//!
//! ⚠️ **A colocação corre logo a seguir à fonte** (a lei que a `=73` pagou): todo
//! comportamento desta biblioteca é multiplicado pelo `falloff`, e um `motion.transform`
//! posto no fim da cadeia seria multiplicado por um campo. Aqui não há campo nenhum, e a
//! ordem é a mesma de propósito — para que a próxima pessoa não aprenda o hábito errado
//! deste arquivo.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_motion_shape::param as sp;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna, e de cada linha.
const COL_X: f32 = 3.4;
const ROW_Y: [f32; 3] = [3.1, 0.0, -3.1];
/// Onde os dois rótulos de coluna assentam.
const HEADER_Y: f32 = 5.8;
const LABEL_SIZE: f32 = 0.42;
/// O nome de cada linha, pintado no vão entre as duas colunas.
const ROW_LABELS: [&str; 3] = ["BORDA", "APARADO", "PICOTADO"];
const LABEL_RGB: [f32; 3] = [0.62, 0.64, 0.70];

/// O miolo da estrela da linha 0, e a borda dela — duas cores, e é isso que a linha diz.
const BODY: [f32; 4] = [0.35, 0.62, 1.0, 1.0];
const EDGE: [f32; 4] = [1.0, 0.66, 0.22, 1.0];
/// A largura da borda, em unidades de mundo. Grossa de propósito: a linha é sobre ela.
const STROKE_W: f32 = 0.10;
/// O branco do anel e o verde do retângulo picotado.
const RING: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const DASHED: [f32; 4] = [0.45, 0.92, 0.55, 1.0];

/// **A FRAÇÃO REVELADA** do anel da linha 1 — pouco mais de um quarto, para que se veja que
/// é um TRECHO e não um anel mais fino.
pub(crate) const TRIM_SPAN: f32 = 0.28;
/// Quanto tempo o trecho leva a dar uma volta completa, em segundos.
pub(crate) const LAP_SECS: f32 = 4.0;
/// O traço e o vão do picotado, em MÚLTIPLOS da largura (é assim que o `StrokeSpec` os fala).
pub(crate) const DASH: f32 = 2.5;
pub(crate) const DASH_GAP: f32 = 2.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = node(g, kind, ps, ey, x);
    let _ = wire(g, head, 0, n, 0);
    n
}

/// **A colocação, e ela corre logo a seguir à fonte.** Ver o aviso no topo do módulo.
fn place(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32, x: f32) -> NodeId {
    push(
        g,
        head,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        x,
    )
}

fn tint(g: &mut Graph, head: NodeId, rgba: [f32; 4], ey: f32, x: f32) -> NodeId {
    push(
        g,
        head,
        "motion.tint",
        &[
            ("r", rgba[0]),
            ("g", rgba[1]),
            ("b", rgba[2]),
            ("a", rgba[3]),
        ],
        ey,
        x,
    )
}

fn out_of(g: &mut Graph, tail: NodeId, ey: f32) -> Option<NodeId> {
    let out = node(g, "motion.output", &[], ey, 900.0);
    wire(g, tail, 0, out, 0)?;
    Some(out)
}

/// Os quatro params da cor de um traço, de uma vez.
fn stroke_rgba(c: [f32; 4]) -> [(&'static str, f32); 4] {
    [
        (sp::STROKE_R, c[0]),
        (sp::STROKE_G, c[1]),
        (sp::STROKE_B, c[2]),
        (sp::STROKE_A, c[3]),
    ]
}

/// Uma banda: a forma, já colocada e já pintada, com a saída dela.
fn band(
    g: &mut Graph,
    k: usize,
    right: bool,
    ps: &[(&str, f32)],
    fill: [f32; 4],
) -> Option<NodeId> {
    #[expect(clippy::cast_precision_loss, reason = "seis bandas")]
    let ey = (k * 2 + usize::from(right)) as f32 * 240.0;
    let src = node(g, "source.shape", ps, ey, 0.0);
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[k]];
    let placed = place(g, src, at, ey, 220.0);
    let painted = tint(g, placed, fill, ey, 440.0);
    let out = out_of(g, painted, ey)?;
    // ⚠️ **O relógio da linha do meio, à DIREITA.** Ele conduz o `trim_offset` por um FIO, e
    // é a única coisa desta cena que precisa de PLAY. Um param conduzido é a rota que
    // desaparecia em silêncio até 2026-08-21 (ver `motion_externals::driven_params`) — a
    // cena passa por ela de propósito.
    if k == 1 && right {
        let clock = node(g, "value.time", &[], ey + 120.0, 0.0);
        let lap = push(
            g,
            clock,
            "value.map_range",
            &[
                ("in_lo", 0.0),
                ("in_hi", LAP_SECS),
                ("out_lo", 0.0),
                ("out_hi", 1.0),
                // SEM trava: o offset dá a volta pela emenda (o `trim_contour` já o toma
                // módulo 1), e travá-lo pararia o trecho no fim da primeira volta.
                ("clamp", 0.0),
            ],
            ey + 120.0,
            180.0,
        );
        g.drive_param(src, sp::TRIM_OFFSET, (lap, 0)).ok()?;
    }
    Some(out)
}

/// Uma palavra no canvas.
fn label(g: &mut Graph, word: &str, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let t = g.add_node("source.text");
    g.set_pos(t, Pos { x: 0.0, y: ey });
    g.set_text_param(t, ph2d_node_source_text::TEXT_KEY, word);
    g.set_param(t, ph2d_node_source_text::param::SIZE, LABEL_SIZE);
    g.set_param(t, ph2d_node_source_text::param::ALIGN, 1.0);
    let placed = place(g, t, at, ey, 200.0);
    let painted = tint(
        g,
        placed,
        [LABEL_RGB[0], LABEL_RGB[1], LABEL_RGB[2], 1.0],
        ey,
        320.0,
    );
    out_of(g, painted, ey)
}

/// Os params da banda `(k, right)` — a tabela da cena, num sítio só.
fn params(k: usize, right: bool) -> (Vec<(&'static str, f32)>, [f32; 4]) {
    let mut ps: Vec<(&'static str, f32)> = Vec::new();
    match k {
        // BORDA — a mesma estrela; à direita ela ganha uma borda de outra cor.
        0 => {
            ps.push((sp::KIND, 5.0)); // Star
            ps.push((sp::SIZE, 1.15));
            ps.push((sp::SIDES, 5.0));
            if right {
                ps.push((sp::STROKE_WIDTH, STROKE_W));
                ps.extend(stroke_rgba(EDGE));
            }
            (ps, BODY)
        }
        // APARADO — um anel (miolo transparente, só a borda). À direita, um TRECHO dele.
        1 => {
            ps.push((sp::KIND, 0.0)); // Circle
            ps.push((sp::SIZE, 1.15));
            ps.push((sp::STROKE_WIDTH, STROKE_W));
            ps.extend(stroke_rgba(RING));
            if right {
                ps.push((sp::TRIM_END, TRIM_SPAN));
            }
            (ps, [0.0, 0.0, 0.0, 0.0])
        }
        // PICOTADO — o mesmo retângulo; à direita a linha dele é interrompida.
        _ => {
            ps.push((sp::KIND, 3.0)); // Rectangle
            ps.push((sp::SIZE, 1.3));
            ps.push((sp::ASPECT, 0.62));
            ps.push((sp::CORNER, 0.22));
            ps.push((sp::STROKE_WIDTH, STROKE_W));
            ps.extend(stroke_rgba(DASHED));
            if right {
                ps.push((sp::DASH, DASH));
                ps.push((sp::DASH_GAP, DASH_GAP));
            }
            (ps, [0.0, 0.0, 0.0, 0.0])
        }
    }
}

/// A cena `=76`, montada de uma vez.
pub(crate) fn build_style_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(6);
    for k in 0..ROW_LABELS.len() {
        for right in [false, true] {
            let (ps, fill) = params(k, right);
            sinks.push(band(g, k, right, &ps, fill)?);
        }
    }
    label(g, "ANTES", [-COL_X, HEADER_Y], 2000.0)?;
    label(g, "DEPOIS", [COL_X, HEADER_Y], 2140.0)?;
    for (k, word) in ROW_LABELS.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "três linhas")]
        let ey = 2280.0 + k as f32 * 140.0;
        label(g, word, [0.0, ROW_Y[k]], ey)?;
    }
    Some(sinks)
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (TRIM_SPAN * 100.0, LAP_SECS)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_style_tests.rs"]
mod tests;
