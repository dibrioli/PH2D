//! **O IRMÃO SABE E ELE NÃO** — a cena `=91` (doc 89, folhas 05 e 14: as seis células que
//! restavam nas duas).
//!
//! ⚠️ **O tema é UMA frase, e a medição partiu-a em duas.** Cada célula dizia *"o nó ao lado
//! tem esta capacidade e este não"*, e a resposta certa dependeu do PORQUÊ de o irmão a ter:
//! em quatro delas era descuido e virou param; numa era o **preço** de uma classe de efeito, e
//! virou recusa (o `speed` do `motion.rotate` — ele não aparece aqui, e a ausência é o ponto).
//!
//! Quatro pares. O mesmo grafo dos dois lados; só o número novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.transform` | um `Scale` para os dois eixos | **`Uniform` desligado** — espalha só num |
//! | `motion.transform` | o layout como está | **`Scale Y = −1`** — o FLIP, uma escala negativa |
//! | `motion.mirror` | `Both` — o original e o gêmeo | **`Reflection Only`** — só o gêmeo |
//! | `source.shape` | herda o tint, sem rotação | **`Own Fill` + `Rotation`** — a forma decide |
//!
//! ⚠️ **A quarta linha só desenha DENTRO do app**: uma `source.shape` lê a geometria por canal
//! externo, que só o shell publica. Num cook virgem ela emite zero — e é por isso que o gate
//! dela mede o DOCUMENTO (os params que a cena autora) e não as posições.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O passo da grelha de cada banda.
const PITCH: f32 = 0.5;
/// O lado da peça.
const PIECE: f32 = 0.3;
/// O vão entre as duas colunas e entre as quatro linhas.
const GAP_X: f32 = 5.2;
const GAP_Y: f32 = 4.2;
/// O fator que os dois pares do `motion.transform` autoram.
const SPREAD: f32 = 2.4;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// ⚠️ **A grelha é ASSIMÉTRICA nos dois eixos** (mais larga que alta): num quadrado, escalar
/// só o X e escalar os dois dá figuras que se confundem, e o espelho de uma coisa simétrica é
/// ela própria. *A fixtura tem de conter o fenómeno.*
fn source(g: &mut Graph, ey: f32) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y: ey });
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 7.0);
    g.set_param(grid, "gap_x", PITCH);
    g.set_param(grid, "gap_y", PITCH);
    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 140.0, y: ey });
    g.set_param(fit, "amount", PIECE);
    let _ = wire(g, grid, 0, fit, 0);
    fit
}

/// Leva a banda ao quadrante, pinta-a e fecha.
fn finish(g: &mut Graph, head: NodeId, rgb: [f32; 3], at: [f32; 2], ey: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 700.0, y: ey });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    wire(g, head, 0, mv, 0)?;
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 840.0, y: ey });
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    wire(g, mv, 0, tint, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: ey });
    wire(g, tint, 0, out, 0)?;
    Some(out)
}

/// Uma LINHA da cena. Nomeada porque a tupla crua dispara o `type_complexity` do clippy.
type Row<'a> = (
    &'static str,
    Vec<(&'a str, f32)>,
    Vec<(&'a str, f32)>,
    [f32; 3],
);

/// As quatro linhas, na ordem em que a cena as monta.
fn rows() -> Vec<Row<'static>> {
    vec![
        (
            "motion.transform",
            vec![("scale", SPREAD)],
            vec![("scale", SPREAD), ("uniform", 0.0), ("scale_y", 1.0)],
            [0.46, 0.72, 1.0],
        ),
        (
            "motion.transform",
            vec![("scale", 1.0)],
            vec![("scale", 1.0), ("uniform", 0.0), ("scale_y", -1.0)],
            [1.0, 0.74, 0.3],
        ),
        (
            "motion.mirror",
            vec![("offset", 1.4)],
            vec![("offset", 1.4), ("keep", 1.0)],
            [0.62, 1.0, 0.66],
        ),
        (
            // ⚠️ A quarta é uma FONTE: ela não recebe a grelha, ela É o que desenha.
            "source.shape",
            vec![("kind", 4.0), ("size", 1.1)],
            vec![
                ("kind", 4.0),
                ("size", 1.1),
                ("fill", 1.0),
                ("fill_r", 1.0),
                ("fill_g", 0.35),
                ("fill_b", 0.1),
                ("fill_a", 1.0),
                ("rotation", 36.0),
            ],
            [0.85, 0.78, 1.0],
        ),
    ]
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_sibling_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(8);
    for (row, (kind, left, right, rgb)) in rows().into_iter().enumerate() {
        for (col, ps) in [left, right].into_iter().enumerate() {
            let ey = (row * 2 + col) as f32 * 240.0;
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 1.5 - row as f32 * GAP_Y,
            ];
            let n = g.add_node(kind);
            g.set_pos(n, Pos { x: 400.0, y: ey });
            for (k, v) in &ps {
                g.set_param(n, *k, *v);
            }
            // A fonte não recebe nada; os modificadores recebem a grelha.
            if kind != "source.shape" {
                let src = source(g, ey);
                wire(g, src, 0, n, 0)?;
            }
            sinks.push(finish(g, n, rgb, at, ey)?);
        }
    }
    g.validate(_registry).ok()?;
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ESPALHA -- um Scale so', os dois eixos juntos, como sempre foi",
        "ESPALHA com Uniform desligado -- ela estica na LARGURA e nao na altura",
        "NORMAL -- a grelha como ela e'",
        "FLIP -- Scale Y negativo: a mesma grelha, espelhada de cabeca para baixo",
        "ESPELHO Both -- o original E o gemeo, os dois na tela",
        "ESPELHO Reflection Only -- so' o gemeo: espelhar SEM duplicar",
        "FORMA -- herda a cor de quem a desenhou, e aponta para onde nasceu",
        "FORMA com Own Fill + Rotation -- ela decide a propria cor e o proprio angulo",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 1.5 - row as f32 * GAP_Y + GAP_Y * 0.38,
            ];
            crate::motion_demo_legend::Caption::new(at, short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro `--`, que é o nome da figura.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> f32 {
    SPREAD
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_sibling_tests.rs"]
mod tests;
