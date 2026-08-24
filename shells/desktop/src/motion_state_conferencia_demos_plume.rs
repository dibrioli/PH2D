//! **O EMISSOR QUE DEIXA RASTO** (`PH2D_GPU_COOK_DEMO=89`) — a cena do
//! `Emitter Motion` (doc 89, folha 01, o P1).
//!
//! Três fontes idênticas, varridas de um lado ao outro pelo MESMO relógio, com
//! uma diferença só:
//!
//! - **CARREGA** — o penacho anda com o emissor. É o que sempre houve, e não é um
//!   bug com nome bonito: um efeito ANEXADO (a chama que anda com a tocha) quer
//!   exactamente isto.
//! - **DEIXA** — a partícula fica onde nasceu. É a base de toda referência, e o
//!   que o artista espera ao arrastar uma fonte.
//! - **HERDA** — fica onde nasceu **e** parte com a velocidade que o emissor
//!   tinha (Cavalry *Use Emitter Velocity*, Niagara *Inherit Velocity*): o
//!   penacho INCLINA-SE para o lado da marcha.
//!
//! ⚠️ **A origem tem de ser DIRIGIDA POR FIO**, e é a cena inteira: a história da
//! origem só existe quando ela se mexe, e um `x` estático faz os três modos
//! coincidirem *por aritmética* (não há história a ler). Um smoke com o emissor
//! parado ficaria verde sobre três desenhos idênticos.
//!
//! ⚠️ **E o `speed` é BAIXO de propósito.** A diferença entre os três está no que
//! o EMISSOR faz; uma boca rápida afoga isso e as três linhas voltam a parecer o
//! mesmo jacto.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A distância vertical entre as linhas.
pub(crate) const ROW_GAP: f32 = 4.2;
/// A vida de uma partícula, em segundos — e a janela da história.
pub(crate) const LIFE: f32 = 2.2;
/// Quantas nascem por segundo.
pub(crate) const RATE: f32 = 60.0;
/// O curso do varrimento, em unidades de mundo.
pub(crate) const SWEEP: f32 = 3.4;
/// O período do varrimento, em segundos.
const PERIOD: f32 = 2.8;
/// A velocidade de boca — baixa de propósito (ver o cabeçalho).
const MUZZLE: f32 = 0.8;

/// O que a partícula guarda do movimento da fonte.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Motion {
    /// O penacho anda com o emissor (o default).
    Carry,
    /// A partícula fica onde nasceu.
    Leave,
    /// Fica onde nasceu e leva a velocidade da fonte.
    Inherit,
}

impl Motion {
    /// O número que o param carrega.
    pub(crate) fn param(self) -> f32 {
        match self {
            Self::Carry => 0.0,
            Self::Leave => 1.0,
            Self::Inherit => 2.0,
        }
    }
}

pub(crate) struct Row {
    pub(crate) label: &'static str,
    pub(crate) caption: &'static str,
    pub(crate) motion: Motion,
}

pub(crate) static ROWS_TABLE: &[Row] = &[
    Row {
        label: "CARREGA — o penacho anda junto. O CONTROLE (e o que sempre houve)",
        caption: "1 CARREGA · o penacho anda junto",
        motion: Motion::Carry,
    },
    Row {
        label: "DEIXA — a particula fica onde nasceu: um rasto no ar",
        caption: "2 DEIXA · fica onde nasceu",
        motion: Motion::Leave,
    },
    Row {
        label: "HERDA — e ainda leva a velocidade da fonte: o jacto INCLINA",
        caption: "3 HERDA · leva a velocidade junto",
        motion: Motion::Inherit,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, u32, f32) {
    (ROWS_TABLE.len(), RATE as u32, LIFE)
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
            crate::motion_demo_legend::Caption::new([0.0, row_y(k) + ROW_GAP * 0.36], r.caption)
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

/// Uma fonte varrida, com o modo que a linha pede.
fn plume(g: &mut Graph, motion: Motion, row: usize, lane: f32) -> Option<NodeId> {
    let em = g.add_node("motion.emitter");
    g.set_pos(em, Pos { x: 320.0, y: lane });
    g.set_param(em, "rate", RATE);
    g.set_param(em, "life", LIFE);
    g.set_param(em, "speed", MUZZLE);
    g.set_param(em, "angle", 90.0); // para cima
    g.set_param(em, "spread", 18.0);
    g.set_param(em, "size", 0.06);
    g.set_param(em, "seed", 5.0);
    g.set_param(em, "y", row_y(row));
    g.set_param(em, ph2d_node_motion_emitter::MOTION, motion.param());

    // ⚠️ **A origem é DIRIGIDA**, e é o que dá história ao emissor. Um `x` de
    // param seria o mesmo número em todo instante, e os três modos coincidiriam.
    let sweep = g.add_node("value.lfo");
    g.set_pos(sweep, Pos { x: 80.0, y: lane });
    g.set_param(sweep, "period", PERIOD);
    g.set_param(sweep, "amplitude", SWEEP);
    g.drive_param(em, "x", (sweep, 0)).ok()?;

    // O integrador move o que nasceu — a cadeia canónica do módulo.
    let ig = g.add_node("motion.integrate");
    g.set_pos(ig, Pos { x: 620.0, y: lane });
    wire(g, em, 0, ig, 0)?;
    wire_pre(g, ig, 0, ig, 1)?;
    Some(ig)
}

/// O documento da cena `=89` — uma sink por linha.
pub(crate) fn build_plume_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 120.0 + k as f32 * 380.0;
        let head = plume(g, row.motion, k, lane)?;
        let out = g.add_node("motion.output");
        g.set_pos(out, Pos { x: 880.0, y: lane });
        wire(g, head, 0, out, 0)?;
        sinks.push(out);
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_plume_tests.rs"]
mod tests;
