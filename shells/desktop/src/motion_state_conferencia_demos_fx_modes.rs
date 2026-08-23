//! **O QUE O EFEITO NÃO SABIA FAZER** (`PH2D_GPU_COOK_DEMO=84`) — a cena da folha 11
//! ([conferência 89](../../../docs/Motion%20Nodes/89_conferencia/11_fx_raster.md)).
//!
//! ## Três linhas que se julgam PARADAS, e um halo que se experimenta
//!
//! - **SOMBRA** — o `fx.drop_shadow` ganhou o MODO com que a sombra se mistura. A diferença só
//!   existe **sobre alguma coisa**, então cada célula desenha uma barra clara e a fileira por
//!   cima dela: à esquerda a sombra COBRE (o de sempre), à direita ela MULTIPLICA — que é o
//!   default do Photoshop e o que uma sombra faz no mundo.
//! - **LENTE** — o `fx.rgb_split` ganhou o EIXO e o RAIO LIMPO. À esquerda a franja nasce no
//!   centro da fileira (o de sempre); à direita o eixo está deslocado e há um miolo limpo, então
//!   o ponto sem franja mudou de sítio **e** ficou maior.
//! - **ALFA** — a linha que o Enio pediu em 23/08 (*"o multiply não obedece o alpha"*). As duas
//!   metades MULTIPLICAM e só a alfa da sombra muda (15% × 85%): a fraca tem de sair mais
//!   CLARA. ⚠️ **A resposta estava INVERTIDA no renderer**, e o defeito não era do nó: uma
//!   fonte pré-multiplicada codifica *"não contribuo"* como **zero**, que é o neutro de todo
//!   modo menos o `Multiply` — cujo neutro é `1`. Cura e tabela medida em
//!   [`ph2d_render::pipeline::blend_state_for`].
//!
//! ⚠️ **O HALO não cabe nesta comparação, e é uma propriedade dele, não um esquecimento:** o
//! `fx.glow` é um passe de TELA e o documento só honra **um** — as duas metades partilhariam o
//! mesmo halo. Os três controles novos dele (operação, fonte e rampa) smokam-se **mexendo no
//! nó**, e a mensagem diz como.

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
/// O modo `Multiply` na escada do `shadow_blend` (`0 = Sink`, e daí os seis do sink).
const MULTIPLY: f32 = 4.0;
/// As duas alfas da linha 3 — o par que denuncia uma resposta INVERTIDA.
const ALPHA_FAINT: f32 = 0.15;
const ALPHA_STRONG: f32 = 0.85;

/// Qual das duas curas esta linha encena.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Case {
    /// `fx.drop_shadow::shadow_blend`.
    Shadow,
    /// `fx.rgb_split::center_x` + `::start`.
    Lens,
    /// **A ALFA DA SOMBRA que MULTIPLICA** — as duas metades no mesmo modo, e só `a` muda.
    ///
    /// ⚠️ Esta linha não é *antes × agora*: é **fraca × forte**, e existe porque o defeito que
    /// o Enio viu em 23/08 (*"o multiply não obedece o alpha"*) só é visível num PAR. A
    /// resposta era invertida — `a = 0,15` pintava mais escuro que `a = 0,85`, e `a = 0`
    /// pintava PRETO em vez de nada. Ver `ph2d_render::pipeline::blend_state_for`.
    ShadowAlpha,
}

pub(crate) struct Row {
    pub(crate) label: &'static str,
    pub(crate) left: &'static str,
    pub(crate) right: &'static str,
    pub(crate) case: Case,
}

pub(crate) static ROWS_TABLE: &[Row] = &[
    Row {
        label: "SOMBRA — ela só sabia COBRIR; agora escolhe o modo, e a de Photoshop MULTIPLICA",
        left: "1 SOMBRA · antes: cobre",
        right: "1 SOMBRA · agora: escurece",
        case: Case::Shadow,
    },
    Row {
        label: "LENTE  — a franja nascia no centro e ponto; agora o eixo move-se e há miolo limpo",
        left: "2 LENTE · antes: centro fixo",
        right: "2 LENTE · agora: eixo movido",
        case: Case::Lens,
    },
    Row {
        label: "ALFA   — as duas MULTIPLICAM e só a alfa muda: a fraca tem de ser mais CLARA",
        left: "3 ALFA · sombra fraca (15%)",
        right: "3 ALFA · sombra forte (85%)",
        case: Case::ShadowAlpha,
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

/// O documento da cena `=84` — uma sink por célula (duas por linha).
pub(crate) fn build_fx_demo_document(
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

/// Uma fileira de [`COUNT`] peças coloridas.
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

/// **A BARRA CLARA por baixo** — sem ela a linha da sombra não tem o que julgar.
///
/// ⚠️ **Uma sombra que MULTIPLICA e uma que COBRE são idênticas sobre o vazio**: o `Multiply` de
/// uma cor sobre nada é a mesma coisa que a pousar ali. A diferença é sobre um FUNDO, e é por
/// isso que esta cena desenha um, em vez de comparar duas sombras no ar.
fn backdrop(g: &mut ph2d_nodegraph::graph::Graph, lane: f32) -> Option<NodeId> {
    let one = g.add_node("motion.grid");
    g.set_param(one, "rows", 1.0);
    g.set_param(one, "cols", 1.0);
    // Larga e baixa: uma barra, não um quadrado. O `uniform = 0` é o que separa os dois eixos.
    let bar = g.add_node("motion.scale");
    g.set_param(bar, "uniform", 0.0);
    g.set_param(bar, "amount", 2.0 * R + DOT);
    g.set_param(bar, "amount_y", DOT * 2.2);
    wire(g, one, 0, bar, 0)?;
    let tint = g.add_node("motion.tint");
    for (ch, v) in [("r", 1.0), ("g", 0.86), ("b", 0.35), ("a", 1.0)] {
        g.set_param(tint, ch, v);
    }
    wire(g, bar, 0, tint, 0)?;
    g.set_pos(
        one,
        Pos {
            x: 80.0,
            y: lane + 120.0,
        },
    );
    g.set_pos(
        tint,
        Pos {
            x: 420.0,
            y: lane + 120.0,
        },
    );
    Some(tint)
}

/// Uma fileira com a barra por baixo e uma sombra por cima, com o modo e a alfa pedidos.
///
/// ⚠️ **A BARRA PRIMEIRO**: a ordem do stream é a ordem de desenho, e o fundo tem de ficar por
/// baixo do bloco de sombras que o `fx.drop_shadow` emite à frente.
fn shadow_cell(
    g: &mut ph2d_nodegraph::graph::Graph,
    lane: f32,
    blend: f32,
    alpha: f32,
) -> Option<NodeId> {
    let back = backdrop(g, lane)?;
    let row = dots(g, lane)?;
    let sh = g.add_node("fx.drop_shadow");
    g.set_param(sh, "direction", 300.0);
    g.set_param(sh, "distance", 0.34);
    g.set_param(sh, "a", alpha);
    g.set_param(sh, "shadow_blend", blend);
    wire(g, row, 0, sh, 0)?;
    let join = g.add_node("motion.combine");
    wire(g, back, 0, join, 0)?;
    wire(g, sh, 0, join, 1)?;
    g.set_pos(sh, Pos { x: 700.0, y: lane });
    g.set_pos(join, Pos { x: 1000.0, y: lane });
    Some(join)
}

fn build_cell(
    g: &mut ph2d_nodegraph::graph::Graph,
    case: Case,
    cured: bool,
    lane: f32,
) -> Option<NodeId> {
    match case {
        Case::Shadow => shadow_cell(g, lane, if cured { MULTIPLY } else { 0.0 }, 0.55),
        // As duas MULTIPLICAM: o que varia é só a alfa. É esse par que torna a resposta da
        // alfa julgável a olho — uma metade só diria "está escuro".
        Case::ShadowAlpha => shadow_cell(
            g,
            lane,
            MULTIPLY,
            if cured { ALPHA_STRONG } else { ALPHA_FAINT },
        ),
        Case::Lens => {
            let row = dots(g, lane)?;
            let split = g.add_node("fx.rgb_split");
            // Aberração: a franja cresce com a distância ao eixo.
            g.set_param(split, "mode", 1.0);
            g.set_param(split, "strength", 0.30);
            if cured {
                // O eixo sai do centroide para a ponta esquerda, e o miolo fica limpo.
                g.set_param(split, "center_x", -0.9);
                g.set_param(split, "start", 0.55);
            }
            wire(g, row, 0, split, 0)?;
            g.set_pos(split, Pos { x: 700.0, y: lane });
            Some(split)
        }
    }
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_fx_modes_tests.rs"]
mod tests;
