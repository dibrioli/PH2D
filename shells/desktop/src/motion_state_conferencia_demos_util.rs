//! **O VOCABULÁRIO DA UTILIDADE** (`PH2D_GPU_COOK_DEMO=81`) — a cena do grupo de
//! 2026-08-22 (doc 89, folha 08): o que o mixer, o sort e o make_point passaram a
//! saber dizer.
//!
//! ## Esta cena NÃO é um gráfico de perfil — são FIGURAS no plano
//!
//! ⚠️ E a diferença é deliberada. As irmãs `=41`/`=78`/`=79`/`=80` desenham *que
//! número sai daqui*, e a resposta certa para elas é uma curva. Três dos quatro
//! pares aqui respondem *que FORMA sai daqui* — uma reta contra uma tenda, uma lente
//! contra um círculo, uma diagonal contra uma espiral —, e uma dessas respostas
//! espremida num gráfico de altura deixaria de ser a resposta.
//!
//! O idioma é o da `=76`/`=77`: **ESQUERDA = como era · DIREITA = o que mudou**,
//! quatro linhas.
//!
//! ## As quatro linhas
//!
//! 1. **MISTURA** — duas lanes cruzadas (uma sobe, outra desce). `Avg` dá a RETA do
//!    meio; `Min` dá a TENDA que abraça a mais baixa das duas.
//! 2. **A FORMA** — as mesmas duas lanes, uma redonda e outra reta. Misturadas, o
//!    resultado é uma LENTE: uma terceira forma que nenhuma das duas tinha. Com a
//!    geometria presa a uma lane, o círculo volta inteiro.
//! 3. **A ORDEM** — o posto de cada peça vira altura. O `shift` **roda** a escada:
//!    o degrau mais alto muda de sítio, e nenhuma peça se perde.
//! 4. **O PONTO** — os MESMOS dois números. Em `Cartesian` são `(x, y)` e desenham
//!    uma diagonal; em `Polar` são `(raio, volta)` e desenham uma ESPIRAL.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Quantas peças cada figura tem.
pub(crate) const COUNT: f32 = 40.0;
/// O tamanho das peças.
const DOT: f32 = 0.22;
/// A distância vertical entre linhas.
const ROW_GAP: f32 = 2.35;
/// A que distância do centro cada coluna vive.
const COL_X: f32 = 2.55;
/// O raio das figuras — escolhido para caber na célula (`ROW_GAP/2`) com folga.
const R: f32 = 0.95;

/// De quanto o `shift` roda a escada da linha da ORDEM.
pub(crate) const SHIFT: f32 = 12.0;

/// Que linha é esta, e qual metade.
#[derive(Clone, Copy)]
enum Cell {
    /// Duas lanes cruzadas, reduzidas pelo `mode` dado.
    Mix { mode: f32 },
    /// As mesmas duas lanes, com a geometria presa (ou não) a uma delas.
    Geom { from: f32 },
    /// O posto de cada peça vira altura, com a ordem rodada por `shift`.
    Order { shift: f32 },
    /// Os mesmos dois números lidos em `Cartesian` ou em `Polar`.
    Point { polar: bool },
}

struct Row {
    label: &'static str,
    left: Cell,
    right: Cell,
}

static ROWS_TABLE: &[Row] = &[
    Row {
        label: "MISTURA   Avg da' a RETA do meio -- Min da' a TENDA",
        left: Cell::Mix { mode: 0.0 },
        right: Cell::Mix { mode: 5.0 },
    },
    Row {
        label: "A FORMA   misturada da' uma LENTE -- presa a uma lane, o CIRCULO",
        left: Cell::Geom { from: 0.0 },
        right: Cell::Geom { from: 1.0 },
    },
    Row {
        label: "A ORDEM   uma diagonal -- e a MESMA diagonal PARTIDA e deslizada",
        left: Cell::Order { shift: 0.0 },
        right: Cell::Order { shift: SHIFT },
    },
    Row {
        label: "O PONTO   os mesmos dois numeros: uma DIAGONAL -- e uma VOLTA",
        left: Cell::Point { polar: false },
        right: Cell::Point { polar: true },
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, f32, f32) {
    (ROWS_TABLE.len(), COUNT, SHIFT)
}

/// O documento da cena `=81` — uma sink por célula (duas por linha).
pub(crate) fn build_util_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
        for (half, cell) in [(0usize, row.left), (1, row.right)] {
            let lane = 100.0 + (k * 2 + half) as f32 * 260.0;
            let node = build_cell(g, cell, lane)?;
            let place = g.add_node("motion.transform");
            g.set_param(place, "offset_x", if half == 0 { -COL_X } else { COL_X });
            g.set_param(place, "offset_y", y);
            let out = g.add_node("motion.output");
            g.set_pos(place, Pos { x: 1200.0, y: lane });
            g.set_pos(out, Pos { x: 1400.0, y: lane });
            wire(g, node, 0, place, 0)?;
            wire(g, place, 0, out, 0)?;
            sinks.push(out);
        }
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

/// Uma fileira de [`COUNT`] peças com a rampa `0..1` já calculada — o berço das
/// quatro linhas.
fn seed(g: &mut ph2d_nodegraph::graph::Graph, lane: f32) -> Option<(NodeId, NodeId)> {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", COUNT);
    g.set_param(grid, "gap_x", 2.0 * R / (COUNT - 1.0));
    g.set_param(grid, "gap_y", 0.0);
    let dot = g.add_node("motion.scale");
    g.set_param(dot, "amount", DOT);
    wire(g, grid, 0, dot, 0)?;
    let ramp = g.add_node("value.instance_field");
    g.set_param(ramp, "mode", 1.0); // Ramp: i/(N−1)
    wire(g, dot, 0, ramp, 0)?;
    g.set_pos(grid, Pos { x: 80.0, y: lane });
    g.set_pos(dot, Pos { x: 260.0, y: lane });
    Some((dot, ramp))
}

/// A rampa esticada — PLUMBING.
fn stretch(g: &mut ph2d_nodegraph::graph::Graph, ramp: NodeId, lo: f32, hi: f32) -> Option<NodeId> {
    let mr = g.add_node("value.map_range");
    g.set_param(mr, "out_lo", lo);
    g.set_param(mr, "out_hi", hi);
    wire(g, ramp, 0, mr, 0)?;
    Some(mr)
}

/// Uma lane: a fileira com o Y conduzido pelo valor dado.
fn lane_of(g: &mut ph2d_nodegraph::graph::Graph, dot: NodeId, value: NodeId) -> Option<NodeId> {
    let d = g.add_node("motion.drive");
    g.set_param(d, "channel", 1.0); // Y
    g.set_param(d, "mode", 1.0); // Set — a lane É o valor, não um empurrão
    wire(g, dot, 0, d, 0)?;
    wire(g, value, 0, d, 1)?;
    Some(d)
}

fn build_cell(g: &mut ph2d_nodegraph::graph::Graph, cell: Cell, lane: f32) -> Option<NodeId> {
    let (dot, ramp) = seed(g, lane)?;
    Some(match cell {
        Cell::Mix { mode } => {
            // Duas lanes CRUZADAS: uma sobe, a outra desce. É o cruzamento que faz
            // `Avg` e `Min` desenharem formas diferentes — com duas lanes paralelas
            // os dois modos dariam retas, e o par nasceria vazio.
            let up = stretch(g, ramp, -R, R)?;
            let down = stretch(g, ramp, R, -R)?;
            let a = lane_of(g, dot, up)?;
            let b = lane_of(g, dot, down)?;
            let m = g.add_node("motion.mixer");
            g.set_param(m, "mode", mode);
            wire(g, a, 0, m, 0)?;
            wire(g, b, 0, m, 1)?;
            m
        }
        Cell::Geom { from } => {
            // Lane A: um CÍRCULO (o `make_point` polar, que esta mesma cena estreia
            // na linha 4). Lane B: uma fileira reta.
            let turn = stretch(g, ramp, 0.0, 1.0)?;
            let radius = g.add_node("value.lfo");
            g.set_param(radius, "amplitude", 0.0);
            g.set_param(radius, "offset", R);
            wire(g, dot, 0, radius, 0)?;
            let circle = g.add_node("motion.make_point");
            g.set_param(circle, "mode", 1.0); // Polar
            wire(g, dot, 0, circle, 0)?;
            wire(g, radius, 0, circle, 1)?;
            wire(g, turn, 0, circle, 2)?;
            let m = g.add_node("motion.mixer");
            g.set_param(m, "geom_from", from);
            wire(g, circle, 0, m, 0)?;
            wire(g, dot, 0, m, 1)?;
            m
        }
        Cell::Order { shift } => {
            // ⚠️ A chave é **X**, ou seja a ordem que as peças já têm — e isso é o
            // desenho, não preguiça. Com uma chave aleatória as duas metades seriam
            // duas dispersões, e «rodada» não se distingue de «outra aleatória» a
            // olho nenhum. Com a chave já ordenada, a esquerda é uma DIAGONAL limpa
            // e a direita é a MESMA diagonal partida no sítio do shift — a rotação
            // fica desenhada em vez de afirmada.
            let s = g.add_node("motion.sort");
            g.set_param(s, "key", 1.0); // X
            g.set_param(s, "seed", 7.0);
            g.set_param(s, "reindex", 1.0);
            g.set_param(s, "shift", shift);
            wire(g, dot, 0, s, 0)?;
            // O POSTO vira altura.
            let rank = g.add_node("value.instance_field");
            g.set_param(rank, "mode", 1.0); // Ramp sobre a lista JÁ ordenada
            wire(g, s, 0, rank, 0)?;
            let h = stretch(g, rank, -R, R)?;
            let d = g.add_node("motion.drive");
            g.set_param(d, "channel", 1.0);
            g.set_param(d, "mode", 1.0); // Set
            wire(g, s, 0, d, 0)?;
            wire(g, h, 0, d, 1)?;
            d
        }
        Cell::Point { polar } => {
            // Os MESMOS dois números nas duas metades: `a` sobe de `0` a `R`, `b` de
            // `0` a `1`. Em Cartesian isso é uma diagonal; em Polar, uma volta
            // inteira de espiral.
            //
            // ⚠️ **`b` para em `1` e não em `2`, e o motivo é a célula.** Duas voltas
            // fariam a leitura polar mais rica — e a leitura CARTESIANA das mesmas
            // duas voltas é um `y` que sobe até `2`, o dobro da meia-linha. O par tem
            // de receber os MESMOS números (é a sua única afirmação), então o teto é
            // o da metade mais apertada.
            let a = stretch(g, ramp, 0.0, R)?;
            let b = stretch(g, ramp, 0.0, 1.0)?;
            let p = g.add_node("motion.make_point");
            g.set_param(p, "mode", if polar { 1.0 } else { 0.0 });
            wire(g, dot, 0, p, 0)?;
            wire(g, a, 0, p, 1)?;
            wire(g, b, 0, p, 2)?;
            p
        }
    })
}

fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
) -> Option<()> {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

/// O que a cena anuncia — as linhas, na ordem em que estão na tela.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_util_tests.rs"]
mod tests;
