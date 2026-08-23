//! **O CORPO NÃO É UM RETÂNGULO** (`PH2D_GPU_COOK_DEMO=87`) — a cena da porta
//! `shape` do `motion.soft_body` (doc 89, folha 03, o P1).
//!
//! Três gelatinas penduradas do MESMO mastro (uma `value.lfo` a varrer a âncora
//! das três), cada uma com uma forma de repouso diferente:
//!
//! - **RETÂNGULO** — a malha `rows × cols` de sempre, com a porta VAZIA. É o
//!   CONTROLE: se ela mudou de comportamento, a wave partiu o que já existia.
//! - **ANEL** — um `motion.distribute_radial`. Um buraco no meio, que nenhum
//!   `rows × cols` exprime.
//! - **CRUZ** — dois `motion.grid` num `motion.combine`. ⚠️ *É a linha que mostra
//!   que a forma é AUTORADA NO GRAFO*: o artista compõe a nuvem com os nós que já
//!   usa, e o corpo passa a ser aquilo.
//!
//! ⚠️ **As três têm de BALANÇAR e VOLTAR.** Um corpo mole que apenas cai é uma
//! nuvem de pontos soltos; o que este nó promete é a recuperação da forma, e é
//! isso que a cena põe lado a lado — a cruz tem de continuar uma cruz depois de
//! chacoalhada.
//!
//! ⚠️ **O `pin` está LIGADO nas três**, e a linha de topo de cada uma é *o `y`
//! máximo do repouso* — a lei nova. Numa malha isso é a primeira fileira (a de
//! sempre); no anel é o arco de cima; na cruz é o braço de cima. Se alguma delas
//! ficar pendurada por um ponto só, ou por uma fileira que não é a de cima, é
//! esse o defeito.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A que distância uma coluna vive da seguinte.
pub(crate) const COL_X: f32 = 5.4;
/// A altura a que os corpos são pendurados.
pub(crate) const ROW_Y: f32 = 2.2;
/// O lado da malha autorada (a coluna de controle).
pub(crate) const MESH_SIDE: f32 = 7.0;
/// O espaçamento da malha — e a régua de tamanho das outras duas formas, para as
/// três chegarem à tela com a mesma envergadura.
const SPACING: f32 = 0.42;
/// O raio exterior do anel e o alcance dos braços da cruz, DERIVADOS da malha:
/// `(lado − 1) · espaçamento / 2` é a meia-largura do rectângulo de controle.
const REACH: f32 = (MESH_SIDE - 1.0) * SPACING * 0.5;

/// O período do mastro, em segundos. Lento de propósito: o que a cena mostra é a
/// forma a VOLTAR, e um balanço rápido lê-se como tremor.
const MAST_PERIOD: f32 = 2.6;
/// O curso do mastro.
const MAST_AMPLITUDE: f32 = 2.0;

/// A física, a MESMA nas três — a cena compara FORMAS, e um corpo mais mole que o
/// vizinho tornaria a comparação sobre o `stiffness`.
const GRAVITY: f32 = 9.0;
const STIFFNESS: f32 = 0.35;
const DAMPING: f32 = 0.04;
/// Regiões sobrepostas: sem elas um corpo só translada e roda, e uma cruz
/// pendurada ficaria rígida como uma chapa recortada (Müller 2005 §4.3).
const CLUSTERS: f32 = 4.0;

/// Qual forma esta coluna encena.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Form {
    /// A malha autorada, com a porta `shape` VAZIA.
    Mesh,
    /// Um anel — um buraco no meio.
    Ring,
    /// Uma cruz, montada de dois `motion.grid`.
    Cross,
}

pub(crate) struct Col {
    pub(crate) label: &'static str,
    pub(crate) caption: &'static str,
    pub(crate) form: Form,
}

pub(crate) static COLS_TABLE: &[Col] = &[
    Col {
        label: "RETANGULO — a malha de sempre (a porta vazia). O CONTROLE",
        caption: "1 RETANGULO · como sempre foi",
        form: Form::Mesh,
    },
    Col {
        label: "ANEL — um buraco no meio: nenhum rows×cols faz isto",
        caption: "2 ANEL · a forma vem da porta",
        form: Form::Ring,
    },
    Col {
        label: "CRUZ — dois grids somados: a forma e' AUTORADA no grafo",
        caption: "3 CRUZ · dois grids somados",
        form: Form::Cross,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, u32) {
    (COLS_TABLE.len(), MESH_SIDE as u32)
}

/// Os rótulos, para a mensagem numerada.
pub(crate) fn col_labels() -> impl Iterator<Item = (usize, &'static str)> {
    COLS_TABLE.iter().enumerate().map(|(i, c)| (i, c.label))
}

/// Onde a coluna `k` fica, em mundo.
pub(crate) fn col_x(k: usize) -> f32 {
    (COLS_TABLE.len() as f32 - 1.0) * -0.5 * COL_X + k as f32 * COL_X
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    COLS_TABLE
        .iter()
        .enumerate()
        .map(|(k, c)| crate::motion_demo_legend::Caption::new([col_x(k), ROW_Y + 1.5], c.caption))
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

/// Um `motion.grid` de `rows × cols`, espaçado como a malha de controle.
fn grid(g: &mut Graph, rows: f32, cols: f32, lane: f32, x: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", rows);
    g.set_param(n, "cols", cols);
    g.set_param(n, "gap_x", SPACING);
    g.set_param(n, "gap_y", SPACING);
    g.set_pos(n, Pos { x, y: lane });
    n
}

/// A nuvem que vai à porta `shape` — `None` para a coluna de controle, que é
/// exactamente o que "a porta vazia" quer dizer.
fn shape_source(g: &mut Graph, form: Form, lane: f32) -> Option<Option<NodeId>> {
    Some(match form {
        Form::Mesh => None,
        Form::Ring => {
            let r = g.add_node("motion.distribute_radial");
            g.set_param(r, "count", 84.0);
            g.set_param(r, "rings", 3.0);
            g.set_param(r, "radius", REACH);
            // O buraco: o anel começa a 55% do raio, então o meio fica VAZIO.
            g.set_param(r, "inner", 0.55);
            g.set_pos(r, Pos { x: 80.0, y: lane });
            Some(r)
        }
        Form::Cross => {
            // ⚠️ Os dois braços partilham o espaçamento da malha, então a cruz tem
            // a densidade das outras duas — uma nuvem mais rala leria como um
            // corpo mais mole, e a cena estaria a comparar densidade.
            let across = grid(g, 3.0, MESH_SIDE, lane, 80.0);
            let down = grid(g, MESH_SIDE, 3.0, lane, 80.0);
            g.set_pos(
                down,
                Pos {
                    x: 80.0,
                    y: lane + 90.0,
                },
            );
            let sum = g.add_node("motion.combine");
            g.set_pos(sum, Pos { x: 300.0, y: lane });
            wire(g, across, 0, sum, 0)?;
            wire(g, down, 0, sum, 1)?;
            Some(sum)
        }
    })
}

/// Uma gelatina pendurada, com a forma que `form` pede.
fn body(g: &mut Graph, form: Form, lane: f32) -> Option<NodeId> {
    let b = g.add_node("motion.soft_body");
    g.set_pos(b, Pos { x: 620.0, y: lane });
    g.set_param(b, "rows", MESH_SIDE);
    g.set_param(b, "cols", MESH_SIDE);
    g.set_param(b, "spacing", SPACING);
    g.set_param(b, "gravity", GRAVITY);
    g.set_param(b, "stiffness", STIFFNESS);
    g.set_param(b, "damping", DAMPING);
    g.set_param(b, "clusters", CLUSTERS);
    g.set_param(b, "pin", 1.0);

    // O MASTRO — a âncora é uma PORTA, nunca um param, e é a mesma lei nas três.
    let mast = g.add_node("value.lfo");
    g.set_pos(mast, Pos { x: 380.0, y: lane });
    g.set_param(mast, "period", MAST_PERIOD);
    g.set_param(mast, "amplitude", MAST_AMPLITUDE);
    wire(g, mast, 0, b, 0)?;

    if let Some(src) = shape_source(g, form, lane)? {
        wire(g, src, 0, b, 3)?;
    }
    // A cadeia de estado: a saída deste tique é o estado do próximo.
    wire_pre(g, b, 0, b, 2)?;
    Some(b)
}

/// O documento da cena `=87` — uma sink por coluna.
pub(crate) fn build_body_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, col) in COLS_TABLE.iter().enumerate() {
        let lane = 120.0 + k as f32 * 360.0;
        let b = body(g, col.form, lane)?;
        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_x", col_x(k));
        g.set_param(place, "offset_y", ROW_Y);
        g.set_pos(place, Pos { x: 900.0, y: lane });
        let out = g.add_node("motion.output");
        g.set_pos(out, Pos { x: 1100.0, y: lane });
        wire(g, b, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_body_tests.rs"]
mod tests;
