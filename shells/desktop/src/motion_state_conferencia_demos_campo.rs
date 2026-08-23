//! **O CAMPO QUE ERA UM NÚMERO** (`PH2D_GPU_COOK_DEMO=83`) — a cena do Grupo Y
//! ([doc 90 §5](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md)).
//!
//! ## Três linhas, e o oráculo de cada uma é ver a FIGURA VARIAR
//!
//! Os dois primeiros defeitos eram a mesma coisa: uma porta do domínio `Instances` — um
//! **campo**, por tipo — lida por `.first()`. O gesto óbvio (ligar-lhe um degradê) entregava ao
//! nó inteiro o elemento zero, que num degradê é `0.0`:
//!
//! - **ESQUERDA** — o valor uniforme que o `.first()` de facto entregava. Uma figura só, chapada.
//! - **DIREITA** — o campo a valer por elemento. A figura **varia ao longo de si mesma**.
//!
//! ⚠️ **A esquerda não é «o defeito»: é o que o nó fazia com a porta LIGADA.** É essa a metade
//! que torna a cena honesta — o artista via o degradê ligado e o nó parado, sem erro nenhum.
//!
//! A terceira linha é outra espécie: a onda escrevia a altura **sempre no tamanho**, com
//! `abs()`. Crista e vale desenhavam a mesma bolha.
//!
//! ⚠️ **Esta cena julga-se PARADA** — nenhuma das três linhas depende do relógio.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Quantas peças as duas primeiras linhas têm.
pub(crate) const COUNT: f32 = 28.0;
/// O tamanho das peças.
const DOT: f32 = 0.16;
/// A distância vertical entre linhas.
pub(crate) const ROW_GAP: f32 = 3.0;
/// A que distância do centro cada coluna vive.
pub(crate) const COL_X: f32 = 2.7;
/// A meia-largura de uma figura.
const R: f32 = 1.15;

/// O lado da grelha da onda — ímpar, para haver uma célula EXACTAMENTE no centro.
const WAVE_SIDE: f32 = 9.0;

/// Qual dos três defeitos esta linha encena.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Case {
    /// `motion.spline_wrap::amount` — o embrulho ao longo da curva.
    Wrap,
    /// `motion.lattice::jitter` — o derretimento da treliça.
    Jitter,
    /// `motion.wave::channel` — para onde a altura vai.
    Wave,
}

pub(crate) struct Row {
    pub(crate) label: &'static str,
    /// A ficha que pousa sobre a metade ESQUERDA, no canvas.
    pub(crate) left: &'static str,
    /// E a que pousa sobre a DIREITA.
    pub(crate) right: &'static str,
    pub(crate) case: Case,
}

pub(crate) static ROWS_TABLE: &[Row] = &[
    Row {
        label: "EMBRULHO — o degradê ligado dava ZERO a todos; agora cada peça embrulha o seu",
        left: "1 EMBRULHO · antes: reta",
        right: "1 EMBRULHO · agora: curva de um lado",
        case: Case::Wrap,
    },
    Row {
        label: "TRELIÇA  — idem: sem tremor em lado nenhum; agora ela derrete DE UM LADO SÓ",
        left: "2 TRELIÇA · antes: favo perfeito",
        right: "2 TRELIÇA · agora: derrete à direita",
        case: Case::Jitter,
    },
    Row {
        label: "ONDA     — a altura ia sempre para o TAMANHO (crista = vale); agora escolhe o canal",
        left: "3 ONDA · antes: só engorda",
        right: "3 ONDA · agora: sobe e desce",
        case: Case::Wave,
    },
];

/// **As fichas desta cena, no canvas** — função PURA, e é ela que o gate mede
/// ([`crate::motion_demo_legend`]).
///
/// Uma por metade, pousada **acima** da figura que ela explica: é ali que o olho já está quando
/// compara as duas. Nada de rótulo por baixo — a linha de baixo é onde a onda desce, e uma ficha
/// ali taparia metade do que a cena existe para mostrar.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let mut out = Vec::with_capacity(ROWS_TABLE.len() * 2);
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y =
            (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP + ROW_GAP * 0.34;
        out.push(crate::motion_demo_legend::Caption::new(
            [-COL_X, y],
            row.left,
        ));
        out.push(crate::motion_demo_legend::Caption::new(
            [COL_X, y],
            row.right,
        ));
    }
    out
}

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, f32) {
    (ROWS_TABLE.len(), COUNT)
}

/// Os rótulos, para a mensagem numerada.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    from: NodeId,
    fp: u16,
    to: NodeId,
    tp: u16,
) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// O documento da cena `=83` — uma sink por célula (duas por linha).
pub(crate) fn build_campo_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
        for (half, field) in [(0usize, false), (1, true)] {
            let lane = 100.0 + (k * 2 + half) as f32 * 320.0;
            let cell = build_cell(g, row.case, field, lane)?;
            let place = g.add_node("motion.transform");
            g.set_param(place, "offset_x", if half == 0 { -COL_X } else { COL_X });
            g.set_param(place, "offset_y", y);
            let out = g.add_node("motion.output");
            g.set_pos(place, Pos { x: 1400.0, y: lane });
            g.set_pos(out, Pos { x: 1600.0, y: lane });
            wire(g, cell, 0, place, 0)?;
            wire(g, place, 0, out, 0)?;
            sinks.push(out);
        }
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

/// Uma fileira de [`COUNT`] peças e a rampa `0..1` sobre ela.
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

/// **O valor que a porta recebe.** `field = false` é o número único que o `.first()` entregava
/// — e `0.0` é literalmente o elemento zero de um degradê, que é o que o artista ligava.
fn feed(
    g: &mut ph2d_nodegraph::graph::Graph,
    ramp: NodeId,
    field: bool,
    lo: f32,
    hi: f32,
) -> Option<NodeId> {
    let mr = g.add_node("value.map_range");
    if field {
        g.set_param(mr, "out_lo", lo);
        g.set_param(mr, "out_hi", hi);
    } else {
        // O que o `.first()` de facto lia: o elemento 0 do degradê, para TODOS.
        g.set_param(mr, "out_lo", lo);
        g.set_param(mr, "out_hi", lo);
    }
    wire(g, ramp, 0, mr, 0)?;
    Some(mr)
}

fn build_cell(
    g: &mut ph2d_nodegraph::graph::Graph,
    case: Case,
    field: bool,
    lane: f32,
) -> Option<NodeId> {
    match case {
        Case::Wrap => {
            let (dot, ramp) = seed(g, lane)?;
            let amount = feed(g, ramp, field, 0.0, 1.0)?;
            let sw = g.add_node("motion.spline_wrap");
            g.set_param(sw, "height_scale", 0.0);
            // Um arco simétrico: o meio sai claramente da corda das pontas.
            for (k, v) in [
                ("p0x", -R),
                ("p0y", 0.0),
                ("p1x", -R * 0.34),
                ("p1y", 1.5),
                ("p2x", R * 0.34),
                ("p2y", 1.5),
                ("p3x", R),
                ("p3y", 0.0),
            ] {
                g.set_param(sw, k, v);
            }
            wire(g, dot, 0, sw, 0)?;
            wire(g, amount, 0, sw, 1)?;
            Some(sw)
        }
        Case::Jitter => {
            let (dot, ramp) = seed(g, lane)?;
            let jitter = feed(g, ramp, field, 0.0, 0.5)?;
            let lat = g.add_node("motion.lattice");
            g.set_param(lat, "rows", 3.0);
            g.set_param(lat, "cols", 10.0);
            g.set_param(lat, "spacing", 0.24);
            g.set_param(lat, "seed", 7.0);
            wire(g, jitter, 0, lat, 0)?;
            let dots = g.add_node("motion.scale");
            g.set_param(dots, "amount", DOT);
            wire(g, lat, 0, dots, 0)?;
            let _ = dot;
            Some(dots)
        }
        Case::Wave => {
            // ⚠️ A onda precisa do laço `pre` — sem ele o campo nunca acumula e as duas
            // metades saem planas e IGUAIS, que é o modo de falha desta linha.
            let w = g.add_node("motion.wave");
            g.set_param(w, "rows", WAVE_SIDE);
            g.set_param(w, "cols", WAVE_SIDE);
            g.set_param(w, "spacing", 0.15);
            g.set_param(w, "speed", 0.4);
            g.set_param(w, "damping", 0.0);
            // ESQUERDA = `Size` (o de sempre) · DIREITA = `Y` (o sinal sobrevive).
            g.set_param(w, "height_channel", if field { 1.0 } else { 0.0 });
            // ⚠️ **A porta `drive` DESLIGADA significa fonte NENHUMA**, não uma fonte de valor
            // zero — está escrito no manifesto do nó. Sem ela o campo fica plano nas DUAS
            // metades e a linha sai igual dos dois lados: o modo de falha desta cena, e a
            // primeira versão dela caiu nele (as duas metades mediram `2,08`, que é só a
            // altura da grelha).
            let lfo = g.add_node("value.lfo");
            g.set_param(lfo, "period", 1.2);
            g.set_param(lfo, "amplitude", 0.3);
            g.set_pos(lfo, Pos { x: 120.0, y: lane });
            wire(g, lfo, 0, w, 0)?;
            g.connect(Edge {
                from: (w, 0),
                to: (w, 1),
                delayed: true,
            })
            .ok()?;
            g.set_pos(w, Pos { x: 300.0, y: lane });
            Some(w)
        }
    }
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_campo_tests.rs"]
mod tests;
