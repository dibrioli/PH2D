//! **OS DEFORMADORES** — a cena `=68` (doc 89, folha 04: cinco células, quatro nós).
//!
//! Cinco pares. O mesmo grafo dos dois lados de cada par; só o número novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.bend` | dobra no eixo X, como sempre | **`Direction`** — a mesma dobra, noutro eixo |
//! | `motion.twist` | o aro MEDIDO (auto) | **`Radius`** — um aro autorado, e o resto satura |
//! | `motion.twist` | perfil `Linear` | **`Profile Smoother`** — a mesma volta, outra rampa |
//! | `motion.spherize` | a lente redonda | **`Radius Y`** — uma lente ELÍPTICA |
//! | `motion.spline_wrap` | `Fit Spline` — estica até as pontas | **`Keep Length`** — mantém a escala |
//!
//! ⚠️ **Esta cena NÃO tem o gate de oclusão das irmãs, e a ausência é deliberada.** Um
//! deformador existe para mudar o espaçamento — um twist aperta o miolo, uma lente afasta o
//! centro. Exigir que nenhuma peça toque a vizinha seria proibir o que os cinco knobs fazem.
//! A peça é pequena o bastante para a figura se ler, e é isso que a cena promete.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O passo da grelha de cada banda.
const PITCH: f32 = 0.62;
/// O lado da peça.
const PIECE: f32 = 0.3;
/// O lado da grelha (⚠️ ímpar, para haver uma peça no CENTRO — é ela que o twist e a lente
/// deixam quieta, e é a âncora do olho).
const SIDE: f32 = 7.0;
/// O que a banda 1 da direita autora.
const BEND_DIR: f32 = 55.0;
/// O aro que a banda 2 da direita autora.
const RIM: f32 = 1.1;
/// O vão entre as duas colunas e entre as cinco linhas.
const GAP_X: f32 = 5.4;
const GAP_Y: f32 = 4.6;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// A grelha da banda, encolhida. ⚠️ Centrada na ORIGEM: os deformadores todos trabalham em
/// torno de um pivô/centroide ali, e a banda só vai para o seu quadrante no fim (o `falloff`
/// está ausente nesta cena, então o `motion.move` do fim não é mascarado por ninguém).
fn source(g: &mut Graph, ey: f32, wide: bool) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y: ey });
    g.set_param(grid, "rows", if wide { 3.0 } else { SIDE });
    g.set_param(grid, "cols", if wide { 11.0 } else { SIDE });
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

/// Uma banda: a grelha por UM deformador, com os params pedidos.
fn band(
    g: &mut Graph,
    kind: &'static str,
    ps: &[(&str, f32)],
    ey: f32,
    wide: bool,
) -> Option<NodeId> {
    let src = source(g, ey, wide);
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x: 400.0, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    wire(g, src, 0, n, 0)?;
    Some(n)
}

/// Uma LINHA da cena: o tipo do nó, os params do lado esquerdo, os do direito, a cor e se a
/// grelha é larga. Nomeada porque a tupla crua dispara o `type_complexity` do clippy.
type Row<'a> = (
    &'static str,
    Vec<(&'a str, f32)>,
    Vec<(&'a str, f32)>,
    [f32; 3],
    bool,
);

/// Monta a cena. Devolve os dez sinks, em pares.
pub(crate) fn build_deform_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    // A curva do `spline_wrap`: um S largo, o mesmo nos dois lados do par.
    let s_curve: &[(&str, f32)] = &[
        ("p0x", -3.0),
        ("p0y", -0.9),
        ("p1x", -1.0),
        ("p1y", 1.4),
        ("p2x", 1.0),
        ("p2y", -1.4),
        ("p3x", 3.0),
        ("p3y", 0.9),
    ];
    let mut rows: Vec<Row<'_>> = vec![
        (
            "motion.bend",
            vec![("angle", 80.0)],
            vec![("angle", 80.0), ("direction", BEND_DIR)],
            [0.46, 0.72, 1.0],
            false,
        ),
        (
            "motion.twist",
            vec![("angle", 150.0)],
            vec![("angle", 150.0), ("radius", RIM)],
            [1.0, 0.74, 0.3],
            false,
        ),
        (
            "motion.twist",
            vec![("angle", 150.0), ("profile", 0.0)],
            vec![("angle", 150.0), ("profile", 3.0)],
            [0.62, 1.0, 0.66],
            false,
        ),
        (
            "motion.spherize",
            vec![("radius", 2.2)],
            vec![("radius", 2.2), ("radius_y", 0.9)],
            [1.0, 0.6, 0.72],
            false,
        ),
    ];
    // O par do `spline_wrap` herda a curva nos dois lados; só o `mode` muda.
    let (mut fit, mut keep) = (s_curve.to_vec(), s_curve.to_vec());
    fit.push(("mode", 0.0));
    keep.push(("mode", 1.0));
    rows.push(("motion.spline_wrap", fit, keep, [0.85, 0.78, 1.0], true));

    let mut sinks = Vec::with_capacity(10);
    for (row, (kind, left, right, rgb, wide)) in rows.into_iter().enumerate() {
        for (col, ps) in [left, right].into_iter().enumerate() {
            let ey = (row * 2 + col) as f32 * 240.0;
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 2.0 - row as f32 * GAP_Y,
            ];
            let head = band(g, kind, &ps, ey, wide)?;
            sinks.push(finish(g, head, rgb, at, ey)?);
        }
    }
    Some(sinks)
}

/// Os rótulos das dez bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "DOBRA -- como sempre: a grelha curva no eixo horizontal",
        "DOBRA com Direction -- a mesma dobra, correndo noutro eixo",
        "TORCAO -- aro AUTO: a peca mais externa leva a volta inteira",
        "TORCAO com Radius -- o aro fica no meio, e tudo la' fora satura",
        "TORCAO perfil Linear -- a volta cresce em rampa recta do centro para fora",
        "TORCAO perfil Smoother -- a mesma volta no aro, outro caminho ate' la'",
        "LENTE redonda -- o centro incha por igual nos dois eixos",
        "LENTE com Radius Y -- a mesma lente ACHATADA: incha na largura, nao na altura",
        "CURVA Fit -- a fileira estica-se ate' as duas pontas do S",
        "CURVA Keep Length -- a mesma fileira, na sua propria escala, e sobra curva",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (BEND_DIR, RIM)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_deform_tests.rs"]
mod tests;
