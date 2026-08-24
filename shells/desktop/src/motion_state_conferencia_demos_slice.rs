//! **QUAL FATIA, QUE EIXO, QUE LEQUE** — a cena `=90` (doc 89, folha 04: as sete células que
//! restavam, três delas construídas e três recusadas com mecanismo).
//!
//! ⚠️ **Irmã da `=68`, e não a mesma cena.** Aquela mostra os cinco knobs que a folha 04 fechou
//! em 2026-08-19 (direção, aro, perfil, lente elíptica, parametrização); esta mostra o que
//! restava — *o deformador não sabia QUE FATIA dele dobra, nem QUE EIXO corre na curva, nem
//! afunilar ao longo dela; e a casa não sabia dispor cópias num ARCO.* O corte é por ASSUNTO,
//! como nas irmãs.
//!
//! Cinco pares. O mesmo grafo dos dois lados; só o número novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.bend` | `Unlimited` — a fileira inteira dobra | **`Limited`** — só metade dobra, e o resto ACOMPANHA |
//! | `motion.bend` | `Limited` — o resto acompanha | **`Within Box`** — o resto FICA onde estava |
//! | `motion.spline_wrap` | uma COLUNA, no eixo de sempre: sai reta | **`Axis 90°`** — a mesma coluna segue a curva |
//! | `motion.spline_wrap` | sem afunilamento | **`Size Start/End`** — a fileira afina ao longo do arco |
//! | `motion.clone` | `Linear` — a fila em recta | **`Radial`** — o leque em torno de um pivô |
//!
//! ⚠️ **Sem gate de oclusão, como a `=68`, e pela mesma razão:** um deformador existe para
//! mudar o espaçamento. Exigir que nenhuma peça toque a vizinha proibiria o que os knobs fazem.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O passo da grelha de cada banda.
const PITCH: f32 = 0.52;
/// O lado da peça.
const PIECE: f32 = 0.26;
/// O vão entre as duas colunas e entre as cinco linhas.
const GAP_X: f32 = 5.6;
const GAP_Y: f32 = 4.4;
/// A volta que os dois pares do `motion.bend` autoram.
const BEND_ANGLE: f32 = 180.0;
/// A fatia que eles dobram: da ponta de trás até ao PIVÔ (a metade de trás).
const LIMIT_LO: f32 = -1.0;
const LIMIT_HI: f32 = 0.0;
/// Quantas cópias o par do `motion.clone` faz, e a que raio.
const FAN_COUNT: f32 = 9.0;
const FAN_RADIUS: f32 = 1.6;

/// A FORMA do layout que cada par precisa — a pergunta que cada célula faz é sobre uma
/// disposição diferente, e usar a mesma para as cinco esconderia metade dos achados.
#[derive(Clone, Copy)]
enum Layout {
    /// Uma fileira larga e baixa — o que uma dobra e um embrulho deformam.
    Row,
    /// Uma COLUNA alta e estreita: sem extensão no eixo que o embrulho lia.
    Column,
    /// Uma peça pequena, para o leque a repetir.
    Piece,
}

impl Layout {
    fn grid(self) -> (f32, f32) {
        match self {
            Self::Row => (3.0, 13.0),
            Self::Column => (13.0, 1.0),
            Self::Piece => (2.0, 2.0),
        }
    }
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// A grelha da banda, encolhida. ⚠️ Centrada na ORIGEM: os deformadores trabalham em torno de
/// um pivô/centroide ali, e a banda só vai para o seu quadrante no fim.
fn source(g: &mut Graph, ey: f32, layout: Layout) -> NodeId {
    let (rows, cols) = layout.grid();
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y: ey });
    g.set_param(grid, "rows", rows);
    g.set_param(grid, "cols", cols);
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

/// Uma banda: a grelha por UM nó, com os params pedidos.
fn band(
    g: &mut Graph,
    kind: &'static str,
    ps: &[(&str, f32)],
    ey: f32,
    layout: Layout,
) -> Option<NodeId> {
    let src = source(g, ey, layout);
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x: 400.0, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    wire(g, src, 0, n, 0)?;
    Some(n)
}

/// Uma LINHA da cena. Nomeada porque a tupla crua dispara o `type_complexity` do clippy.
type Row<'a> = (
    &'static str,
    Vec<(&'a str, f32)>,
    Vec<(&'a str, f32)>,
    [f32; 3],
    Layout,
);

/// O S que os dois pares do `motion.spline_wrap` herdam — o mesmo dos dois lados.
fn s_curve() -> Vec<(&'static str, f32)> {
    vec![
        ("p0x", -2.6),
        ("p0y", -1.0),
        ("p1x", -0.9), // NÃO um S simétrico: uma curva que só dobra num sentido
        ("p1y", 1.5),  // esconderia metade do que o eixo e o afunilamento fazem.
        ("p2x", 0.9),
        ("p2y", -1.5),
        ("p3x", 2.6),
        ("p3y", 1.0),
    ]
}

/// As cinco linhas, na ordem em que a cena as monta.
fn rows() -> Vec<Row<'static>> {
    let bend_slice: Vec<(&str, f32)> = vec![
        ("angle", BEND_ANGLE),
        ("limit_lo", LIMIT_LO),
        ("limit_hi", LIMIT_HI),
    ];
    let mut limited = bend_slice.clone();
    limited.push(("mode", 1.0));
    let mut within = bend_slice.clone();
    within.push(("mode", 2.0));

    let (axis_off, mut axis_on) = (s_curve(), s_curve());
    axis_on.push(("direction", 90.0));

    let (flat, mut tapered) = (s_curve(), s_curve());
    tapered.push(("size_start", 1.7));
    tapered.push(("size_end", 0.15));
    tapered.push(("size_profile", 2.0));

    // ⚠️ O par do leque autora a MESMA contagem dos dois lados: o que muda é por onde as
    // cópias andam, e um `count` diferente faria a comparação medir duas coisas. O passo da
    // FILA é menor que o raio do leque, de propósito — uma fila de nove com passo `1,6`
    // atravessaria o quadrante vizinho, e a comparação passaria a ser sobre o enquadramento.
    let linear: Vec<(&str, f32)> = vec![("count", FAN_COUNT), ("distance", 0.62)];
    let radial: Vec<(&str, f32)> = vec![
        ("count", FAN_COUNT),
        ("distance", FAN_RADIUS),
        ("mode", 1.0),
    ];

    // ⚠️ O `axis_off` e o `flat` ficam sem knob novo NENHUM — eles são os CONTROLES, e é
    // deles que sai o defeito (a coluna reta, as peças todas iguais). Sem esse lado as
    // linhas 3 e 4 não provariam nada.

    vec![
        (
            "motion.bend",
            vec![("angle", BEND_ANGLE)],
            limited,
            [0.46, 0.72, 1.0],
            Layout::Row,
        ),
        (
            "motion.bend",
            bend_slice,
            within,
            [1.0, 0.74, 0.3],
            Layout::Row,
        ),
        (
            "motion.spline_wrap",
            axis_off,
            axis_on,
            [0.62, 1.0, 0.66],
            Layout::Column,
        ),
        (
            "motion.spline_wrap",
            flat,
            tapered,
            [1.0, 0.6, 0.72],
            Layout::Row,
        ),
        (
            "motion.clone",
            linear,
            radial,
            [0.85, 0.78, 1.0],
            Layout::Piece,
        ),
    ]
}

/// ⚠️ **A linha 2 tem de trazer o `mode = Limited` do lado ESQUERDO**, senão ela repete a
/// linha 1: o par dela é *o que acontece FORA da fatia*, e os dois lados partilham a fatia.
fn left_of(row: usize, mut ps: Vec<(&'static str, f32)>) -> Vec<(&'static str, f32)> {
    if row == 1 {
        ps.push(("mode", 1.0));
    }
    ps
}

/// Monta a cena. Devolve os dez sinks, em pares.
pub(crate) fn build_slice_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(10);
    for (row, (kind, left, right, rgb, layout)) in rows().into_iter().enumerate() {
        for (col, ps) in [left_of(row, left), right].into_iter().enumerate() {
            let ey = (row * 2 + col) as f32 * 240.0;
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 2.0 - row as f32 * GAP_Y,
            ];
            let head = band(g, kind, &ps, ey, layout)?;
            sinks.push(finish(g, head, rgb, at, ey)?);
        }
    }
    Some(sinks)
}

/// Os rótulos das dez bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "DOBRA Unlimited -- a fileira INTEIRA curva, como sempre foi",
        "DOBRA Limited -- so' metade curva, e a outra metade ACOMPANHA a ponta, direita",
        "DOBRA Limited (a mesma de cima, para comparar)",
        "DOBRA Within Box -- so' metade curva, e a outra metade FICA onde estava",
        "CURVA eixo de sempre -- a COLUNA sai numa reta: a curva nao existe para ela",
        "CURVA com Axis 90 -- a mesma coluna a percorrer o S inteiro",
        "CURVA sem afunilamento -- as pecas todas do mesmo tamanho",
        "CURVA com Size Start/End -- ela ENGROSSA no comeco e afina ate' sumir no fim",
        "COPIAS Linear -- a fila em recta que o no' sempre fez",
        "COPIAS Radial -- as mesmas copias num LEQUE, em torno de um pivo",
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
                GAP_Y * 2.0 - row as f32 * GAP_Y + GAP_Y * 0.40,
            ];
            // A ficha traz só a primeira palavra + o modo — o rótulo inteiro vai no terminal.
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
pub(crate) fn authored() -> (f32, f32, u32) {
    (BEND_ANGLE, FAN_RADIUS, FAN_COUNT as u32)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_slice_tests.rs"]
mod tests;
