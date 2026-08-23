//! **A FORMA QUE O ARTISTA DESENHA, E OS DOIS EIXOS** (`PH2D_GPU_COOK_DEMO=85`) — a cena
//! da folha 06 ([conferência 89](../../../docs/Motion%20Nodes/89_conferencia/06_animadores.md)).
//!
//! ## Três linhas que se julgam PARADAS
//!
//! - **ONDA** — o `motion.oscillator` ganhou a forma `Custom`. ⚠️ A fileira **É** a onda: com
//!   `frequency = 0` a fase de cada peça é só o `phase_stagger` dela, então as `N` peças cobrem
//!   um ciclo inteiro **no espaço** e o desenho fica parado na tela. À esquerda a senoide de
//!   sempre; à direita a curva autorada (sobe depressa, desce devagar).
//! - **ESCADA** — o `motion.stagger` ganhou a ease `Custom`. Ele é por ÍNDICE, logo já era
//!   estático. À esquerda a rampa `Linear`; à direita o V desenhado (sobe até ao meio e volta).
//! - **DOIS EIXOS** — o `motion.noise` ganhou o canal `Position XY`. À esquerda ele escreve só
//!   Y (as peças sobem e descem **nas colunas delas**, o espaçamento horizontal fica perfeito);
//!   à direita escreve os dois, e o espaçamento desarruma-se — que é a diferença inteira.
//!
//! ⚠️ **O quarto controle desta folha não cabe aqui, e é uma propriedade dele:** o
//! `motion.path` anda numa curva **desenhada**, e o documento vetorial só existe no smoke
//! próprio dele (`PH2D_MOTION_NODE_PATH_SMOKE=3`, a cena da NORMAL). A mensagem diz.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Quantas peças cada fileira tem.
pub(crate) const COUNT: f32 = 15.0;
/// O tamanho de uma peça.
const DOT: f32 = 0.34;
/// A distância vertical entre linhas.
pub(crate) const ROW_GAP: f32 = 3.2;
/// A que distância do centro cada coluna vive.
pub(crate) const COL_X: f32 = 2.9;
/// A meia-largura de uma fileira.
const R: f32 = 1.25;

/// A onda `Custom` do `motion.oscillator` (índice 5) e a ease `Custom` do
/// `motion.stagger` (índice 8) — os dois números vêm das crates, não daqui.
const WAVE_CUSTOM: f32 = 5.0;
const EASE_CUSTOM: f32 = 8.0;
/// O canal `Position XY` (índice 4) do `motion.noise`.
const CH_XY: f32 = 4.0;

/// **A onda desenhada da linha 1** — sobe depressa, desce devagar.
///
/// ⚠️ Escolhida para ser **inconfundível com as cinco enumeradas**: nenhuma delas é
/// assimétrica assim. Uma curva parecida com uma senoide deixaria o smoke sem veredito.
const CURVE_WAVE: &str = "c1 0:0:L 0.2:1:L 1:0:L";
/// **A ease desenhada da linha 2** — sobe até ao meio e volta.
///
/// ⚠️ Ela **não é monótona**, e as oito famílias enumeradas são: é isso que impede o
/// olho de a confundir com um `Quad` ou um `Bounce`.
const CURVE_EASE: &str = "c1 0:0:L 0.5:1:L 1:0:L";

/// Qual das três curas esta linha encena.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Case {
    /// `motion.oscillator::wave = Custom` + a curva.
    Wave,
    /// `motion.stagger::ease_curve = Custom` + a curva.
    Ease,
    /// `motion.noise::channel = Position XY`.
    TwoAxis,
}

pub(crate) struct Row {
    pub(crate) label: &'static str,
    pub(crate) left: &'static str,
    pub(crate) right: &'static str,
    pub(crate) case: Case,
}

pub(crate) static ROWS_TABLE: &[Row] = &[
    Row {
        label: "ONDA   — a fileira É a onda; à direita ela é a curva que você desenha",
        left: "1 ONDA · antes: so' as 5 prontas",
        right: "1 ONDA · agora: a desenhada",
        case: Case::Wave,
    },
    Row {
        label: "ESCADA — a rampa por índice; à direita a forma dela é desenhada",
        left: "2 ESCADA · antes: rampa reta",
        right: "2 ESCADA · agora: a desenhada",
        case: Case::Ease,
    },
    Row {
        label: "DOIS EIXOS — o ruído mexia num eixo só; à direita mexe nos dois",
        left: "3 EIXOS · antes: so' vertical",
        right: "3 EIXOS · agora: os dois",
        case: Case::TwoAxis,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, f32) {
    (ROWS_TABLE.len(), COUNT)
}

/// Os rótulos, para a mensagem numerada.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let mut out = Vec::with_capacity(ROWS_TABLE.len() * 2);
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y =
            (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP + ROW_GAP * 0.36;
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

/// O documento da cena `=85` — uma sink por célula (duas por linha).
pub(crate) fn build_drawn_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
        for (half, cured) in [(0usize, false), (1, true)] {
            let lane = 100.0 + (k * 2 + half) as f32 * 320.0;
            let cell = build_cell(g, row.case, cured, lane)?;
            let place = g.add_node("motion.transform");
            g.set_param(place, "offset_x", if half == 0 { -COL_X } else { COL_X });
            g.set_param(place, "offset_y", y);
            let out = g.add_node("motion.output");
            g.set_pos(place, Pos { x: 1500.0, y: lane });
            g.set_pos(out, Pos { x: 1700.0, y: lane });
            wire(g, cell, 0, place, 0)?;
            wire(g, place, 0, out, 0)?;
            sinks.push(out);
        }
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

/// Uma fileira de [`COUNT`] peças.
fn dots(g: &mut ph2d_nodegraph::graph::Graph, lane: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", COUNT);
    g.set_param(grid, "gap_x", 2.0 * R / (COUNT - 1.0));
    g.set_param(grid, "gap_y", 0.0);
    let size = g.add_node("motion.scale");
    g.set_param(size, "amount", DOT);
    wire(g, grid, 0, size, 0)?;
    g.set_pos(grid, Pos { x: 80.0, y: lane });
    g.set_pos(size, Pos { x: 260.0, y: lane });
    Some(size)
}

fn build_cell(
    g: &mut ph2d_nodegraph::graph::Graph,
    case: Case,
    cured: bool,
    lane: f32,
) -> Option<NodeId> {
    let row = dots(g, lane)?;
    let node = match case {
        Case::Wave => {
            let osc = g.add_node("motion.oscillator");
            g.set_param(osc, "channel", 1.0); // Y
            g.set_param(osc, "amplitude", 0.85);
            // ⚠️ **`frequency = 0` é o que torna esta linha JULGÁVEL PARADA:** a fase de
            // cada peça passa a ser só o `phase_stagger` dela, então a fileira desenha um
            // ciclo inteiro no ESPAÇO e nada se move no tempo.
            g.set_param(osc, "frequency", 0.0);
            g.set_param(osc, "phase_stagger", 1.0 / COUNT);
            if cured {
                g.set_param(osc, "wave", WAVE_CUSTOM);
                g.set_text_param(osc, "curve", CURVE_WAVE);
            }
            osc
        }
        Case::Ease => {
            let st = g.add_node("motion.stagger");
            g.set_param(st, "channel", 1.0); // Y
            g.set_param(st, "min", -0.85);
            g.set_param(st, "max", 0.85);
            if cured {
                g.set_param(st, "ease_curve", EASE_CUSTOM);
                g.set_text_param(st, "curve", CURVE_EASE);
            }
            st
        }
        Case::TwoAxis => {
            let ns = g.add_node("motion.noise");
            g.set_param(ns, "amplitude", 0.6);
            g.set_param(ns, "scale", 0.9);
            g.set_param(ns, "speed", 0.0);
            // ⚠️ **A metade da esquerda é o canal Y de sempre**, e é ela que faz a
            // diferença ser legível: as peças sobem e descem MAS ficam nas colunas
            // delas, então o espaçamento horizontal continua perfeito. À direita ele
            // desarruma-se, e é isso que o olho lê como *"agora ela vagueia"*.
            g.set_param(ns, "channel", if cured { CH_XY } else { 1.0 });
            ns
        }
    };
    wire(g, row, 0, node, 0)?;
    g.set_pos(node, Pos { x: 700.0, y: lane });
    Some(node)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_drawn_tests.rs"]
mod tests;
