//! **O ECO QUE VÊ O FUTURO** (`PH2D_GPU_COOK_DEMO=88`) — a cena do
//! `Source: Resampled` do `motion.trail` (doc 89, folha 07, o P1 / `SUPERAR:` S1).
//!
//! Três elementos a percorrerem o MESMO caminho (uma figura de Lissajous, pura
//! função do playhead), cada um com um rastro diferente:
//!
//! - **LEMBRADO** — o ring que sempre existiu. O CONTROLE.
//! - **RE-COZIDO** — a entrada re-cozida em `t − k·spacing`. Tem de desenhar **a
//!   mesma cauda** que o de cima: é a redução que faz o modo novo nascer no
//!   ponto neutro.
//! - **PARA A FRENTE** — a entrada re-cozida em `t + k·spacing`. A cauda vai
//!   ADIANTE do elemento, e isto **um ring não pode fazer por construção**: ele
//!   contém o passado porque passado é o que um ring é.
//!
//! ⚠️ **O caminho tem de ser PURO** (uma função do playhead, sem `pre`), e é a
//! cena inteira: um simulador não é função de `t`, e o cook recusa um leque sobre
//! ele em vez de desenhar uma trajectória plausível e falsa
//! (`CookError::SequentialInTimeScope`). É o mesmo limite que o `motion.delay` já
//! escreve sobre o `time_remap`, e é por isso que `Resampled` é um MODO e nunca
//! uma substituição.
//!
//! ⚠️ **A figura é uma LISSAJOUS e não um círculo**, de propósito: num círculo a
//! cauda para a frente e a cauda para trás pousam no mesmo arco e as duas linhas
//! de baixo ficariam indistinguíveis. Num caminho que se cruza, o que vai à
//! frente e o que fica atrás são visivelmente coisas diferentes.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A distância vertical entre as linhas.
pub(crate) const ROW_GAP: f32 = 4.6;
/// Quantos ecos cada rastro tem (a cabeça viva incluída).
pub(crate) const LENGTH: f32 = 10.0;
/// Quantos tiques separam um eco do seguinte.
pub(crate) const SPACING: f32 = 4.0;
/// O tamanho de cada peça.
const DOT: f32 = 0.34;

/// A meia-largura e a meia-altura da figura.
const AX: f32 = 3.2;
const AY: f32 = 1.3;
/// Os dois períodos, em segundos. A razão **3:2** é o que faz o caminho CRUZAR-SE
/// — com uma razão de 1:1 sairia uma elipse, e as duas linhas de baixo pousariam
/// no mesmo arco.
const PERIOD_X: f32 = 3.0;
const PERIOD_Y: f32 = 2.0;

/// O que a cauda deste rastro faz.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Echo {
    /// O ring de sempre.
    Remembered,
    /// A entrada re-cozida no passado — tem de igualar o de cima.
    Resampled,
    /// A entrada re-cozida no FUTURO.
    Forward,
}

pub(crate) struct Row {
    pub(crate) label: &'static str,
    pub(crate) caption: &'static str,
    pub(crate) echo: Echo,
}

pub(crate) static ROWS_TABLE: &[Row] = &[
    Row {
        label: "LEMBRADO — o rastro de sempre. O CONTROLE",
        caption: "1 LEMBRADO · a cauda de sempre",
        echo: Echo::Remembered,
    },
    Row {
        label: "RE-COZIDO — a MESMA cauda, sem lembrar de nada",
        caption: "2 RE-COZIDO · tem de ser igual a 1",
        echo: Echo::Resampled,
    },
    Row {
        label: "PARA A FRENTE — a cauda ADIANTE do elemento",
        caption: "3 PARA A FRENTE · o eco vai na frente",
        echo: Echo::Forward,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, u32, u32) {
    (ROWS_TABLE.len(), LENGTH as u32, SPACING as u32)
}

/// Os rótulos, para a mensagem numerada.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

/// A altura da linha `k`, em mundo.
pub(crate) fn row_y(k: usize) -> f32 {
    (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    ROWS_TABLE
        .iter()
        .enumerate()
        .map(|(k, r)| {
            crate::motion_demo_legend::Caption::new([0.0, row_y(k) + ROW_GAP * 0.34], r.caption)
        })
        .collect()
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn wire_pre(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: true,
    })
    .ok()
}

/// Uma `value.lfo` senoidal — o relógio PURO de que o leque depende.
fn lfo(g: &mut Graph, period: f32, amplitude: f32, phase: f32, x: f32, y: f32) -> NodeId {
    let n = g.add_node("value.lfo");
    g.set_param(n, "period", period);
    g.set_param(n, "amplitude", amplitude);
    g.set_param(n, "phase", phase);
    g.set_pos(n, Pos { x, y });
    n
}

/// **O elemento no caminho** — uma peça só, deslocada por dois params DIRIGIDOS.
///
/// ⚠️ **Params dirigidos, e não um `motion.move` animado por estado:** um param
/// dirigido é cozido na MESMA recursão que a porta (doc 58), então ele viaja com
/// o leque e responde em `t ± k·s`. Um caminho guardado em estado não viajaria, e
/// as três linhas desenhariam a mesma coisa.
fn mover(g: &mut Graph, lane: f32) -> Option<NodeId> {
    let dot = g.add_node("motion.grid");
    g.set_param(dot, "rows", 1.0);
    g.set_param(dot, "cols", 1.0);
    g.set_pos(dot, Pos { x: 60.0, y: lane });

    let size = g.add_node("motion.scale");
    g.set_param(size, "amount", DOT);
    g.set_pos(size, Pos { x: 220.0, y: lane });
    wire(g, dot, 0, size, 0)?;

    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 460.0, y: lane });
    wire(g, size, 0, mv, 0)?;
    let x = lfo(g, PERIOD_X, AX, 0.0, 220.0, lane + 80.0);
    // Um quarto de volta no eixo curto — sem a defasagem a figura degenera num
    // segmento, e uma linha reta não distingue frente de trás.
    let y = lfo(g, PERIOD_Y, AY, 0.25, 220.0, lane + 160.0);
    g.drive_param(mv, "dx", (x, 0)).ok()?;
    g.drive_param(mv, "dy", (y, 0)).ok()?;
    Some(mv)
}

/// O documento da cena `=88` — uma sink por linha.
pub(crate) fn build_echo_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 120.0 + k as f32 * 420.0;
        let src = mover(g, lane)?;

        let trail = g.add_node("motion.trail");
        g.set_pos(trail, Pos { x: 700.0, y: lane });
        g.set_param(trail, "length", LENGTH);
        g.set_param(trail, "spacing", SPACING);
        // Uma cauda que desbota e encolhe — é o que torna a DIREÇÃO dela legível.
        g.set_param(trail, "fade", 0.12);
        g.set_param(trail, "shrink", 0.35);
        match row.echo {
            Echo::Remembered => {}
            Echo::Resampled => {
                g.set_param(trail, ph2d_node_motion_trail::SOURCE, 1.0);
            }
            Echo::Forward => {
                g.set_param(trail, ph2d_node_motion_trail::SOURCE, 1.0);
                g.set_param(trail, ph2d_node_motion_trail::FORWARD, LENGTH - 1.0);
            }
        }
        wire(g, src, 0, trail, 0)?;
        // ⚠️ **A cadeia de estado fica ligada nos TRÊS**, e de propósito: ela é o
        // que o modo `Remembered` usa, e desligá-la nas outras duas faria a cena
        // comparar dois grafos em vez de dois modos. Em `Resampled` o nó
        // simplesmente não a lê.
        wire_pre(g, trail, 0, trail, 1)?;

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", row_y(k));
        g.set_pos(place, Pos { x: 900.0, y: lane });
        let out = g.add_node("motion.output");
        g.set_pos(out, Pos { x: 1100.0, y: lane });
        wire(g, trail, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_echo_tests.rs"]
mod tests;
