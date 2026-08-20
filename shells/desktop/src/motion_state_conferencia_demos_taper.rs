//! **A FILA E A MISTURA** — a cena `=64` (doc 89, folha 08: o taper por cópia do
//! `motion.clone` e o peso por entrada do `motion.mixer`).
//!
//! Duas colunas de bandas, cada uma um PAR — o mesmo grafo dos dois lados, um número
//! diferente. Um par que sai igual dos dois lados não prova knob nenhum.
//!
//! | banda | o que muda |
//! |---|---|
//! | 1 | `motion.clone` como sempre foi — seis cópias iguais |
//! | 2 | o mesmo clone com `Scale Taper 0,25` e `Rot Taper 90` — a fila afunila e vira |
//! | 3 | `motion.mixer(Avg)` de uma FILEIRA e de uma COLUNA, pesos iguais — a diagonal a 45° |
//! | 4 | a mesma mistura com `Weight 1 = 3` — a diagonal inclina-se para a coluna |
//!
//! ⚠️ **A peça cabe no passo, e isso é lei desta casa desde a cena `=63`**: uma instância sem
//! coluna `size` é desenhada com `SIZE_IDENTITY` = 1,0 unidade de mundo, e uma peça maior que
//! a distância entre vizinhas tapa o que a banda existe para mostrar.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O passo entre as cópias da fila, e entre os pontos das duas fontes da mistura.
const PITCH: f32 = 1.3;
/// O lado de cada peça — ver o ⚠️ do topo. Cabe no menor passo que **qualquer** banda produz,
/// e o menor não é o da fila.
///
/// ⚠️ **A banda mais apertada é a MISTURA de pesos iguais, e a conta não é óbvia:** a média de
/// uma fileira de passo `1,3` com uma coluna de passo `1,3` põe cada ponto em `(x/2, y/2)`, e
/// o passo do resultado é **metade** — `0,65` em cada eixo. É a distância de Chebyshev que
/// manda (duas peças quadradas de lado `w` sobrepõem-se sse `|dx| < w` **e** `|dy| < w`), e
/// `0,7` reprovou o gate desta cena antes de o Enio a ver. O passo da fila (`1,3`) e o da
/// mistura pesada (`0,975`) sobram; quem decide é o mínimo.
const PIECE: f32 = 0.55;
/// Quantas cópias a fila tem.
const COPIES: f32 = 6.0;
/// Quantos pontos cada fonte da mistura tem.
const POINTS: f32 = 7.0;
/// O taper que a banda 2 autora: a última cópia mede um quarto…
const TAPER_SCALE: f32 = 0.25;
/// …e chega a um quarto de volta.
const TAPER_ROT: f32 = 90.0;
/// O peso que a banda 4 dá à COLUNA.
const HEAVY: f32 = 3.0;
/// O vão entre as duas linhas de bandas, e entre as duas colunas.
const GAP_Y: f32 = 3.6;
const GAP_X: f32 = 5.6;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Encolhe a peça, tinge a banda e leva-a para o seu quadrante. Devolve o sink.
fn finish(g: &mut Graph, head: NodeId, rgb: [f32; 3], x: f32, y: f32) -> Option<NodeId> {
    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 660.0, y });
    g.set_param(fit, "amount", PIECE);
    wire(g, head, 0, fit, 0)?;

    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 800.0, y });
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    wire(g, fit, 0, tint, 0)?;

    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 940.0, y });
    g.set_param(mv, "dx", x);
    g.set_param(mv, "dy", y);
    wire(g, tint, 0, mv, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1080.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// **A FILA**: uma peça só, clonada seis vezes ao longo de uma reta.
///
/// ⚠️ O `center` fica LIGADO nas duas bandas — a fila straddle o original, e o par continua a
/// mostrar só o taper. É também o que torna visível que os dois controles são ortogonais: o
/// afunilamento corre da primeira cópia à última, e não do meio para fora.
fn clone_band(g: &mut Graph, scale_taper: f32, rot_taper: f32, y: f32) -> Option<NodeId> {
    let seed = g.add_node("motion.grid");
    g.set_pos(seed, Pos { x: 0.0, y });
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);

    let clone = g.add_node("motion.clone");
    g.set_pos(clone, Pos { x: 300.0, y });
    g.set_param(clone, "count", COPIES);
    g.set_param(clone, "distance", PITCH);
    g.set_param(clone, "center", 1.0);
    g.set_param(clone, "scale_taper", scale_taper);
    g.set_param(clone, "rot_taper", rot_taper);
    wire(g, seed, 0, clone, 0)?;
    Some(clone)
}

/// **A MISTURA**: uma FILEIRA e uma COLUNA dos mesmos sete pontos, fundidas elemento a
/// elemento. Com pesos iguais o resultado é a diagonal exacta; com a coluna a pesar o triplo
/// ela inclina-se para a vertical.
///
/// ⚠️ As duas fontes são deliberadamente **perpendiculares**: assim o peso não muda um
/// tamanho ou uma cor, muda a INCLINAÇÃO da linha — que é a leitura mais barata que existe.
fn mixer_band(g: &mut Graph, weight_col: f32, y: f32) -> Option<NodeId> {
    let row = g.add_node("motion.grid");
    g.set_pos(row, Pos { x: 0.0, y });
    g.set_param(row, "rows", 1.0);
    g.set_param(row, "cols", POINTS);
    g.set_param(row, "gap_x", PITCH);

    let col = g.add_node("motion.grid");
    g.set_pos(
        col,
        Pos {
            x: 0.0,
            y: y + 90.0,
        },
    );
    g.set_param(col, "rows", POINTS);
    g.set_param(col, "cols", 1.0);
    g.set_param(col, "gap_y", PITCH);

    let mix = g.add_node("motion.mixer");
    g.set_pos(mix, Pos { x: 300.0, y });
    g.set_param(mix, "mode", 0.0); // Avg
    g.set_param(mix, "weight_1", weight_col);
    wire(g, row, 0, mix, 0)?;
    wire(g, col, 0, mix, 1)?;
    Some(mix)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_taper_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let warm = [1.0, 0.72, 0.24];
    let cool = [0.42, 0.66, 1.0];
    let mut sinks = Vec::with_capacity(4);
    for (i, (taper_s, taper_r)) in [(1.0, 0.0), (TAPER_SCALE, TAPER_ROT)]
        .into_iter()
        .enumerate()
    {
        let gy = i as f32 * 260.0;
        let head = clone_band(g, taper_s, taper_r, gy)?;
        let y = GAP_Y - i as f32 * 2.0 * GAP_Y;
        sinks.push(finish(g, head, warm, -GAP_X, y)?);
    }
    for (i, w) in [1.0, HEAVY].into_iter().enumerate() {
        let gy = 520.0 + i as f32 * 260.0;
        let head = mixer_band(g, w, gy)?;
        let y = GAP_Y - i as f32 * 2.0 * GAP_Y;
        sinks.push(finish(g, head, cool, GAP_X, y)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "FILA (esquerda, em cima) -- o clone como sempre foi: seis copias iguais",
        "FILA com taper -- Scale Taper 0,25 e Rot Taper 90: ela afunila E vira",
        "MISTURA (direita, em cima) -- fileira + coluna com pesos iguais: a diagonal a 45",
        "MISTURA com Weight 1 = 3 -- a mesma dupla, e a linha inclina-se para a coluna",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32, f32) {
    (TAPER_SCALE, TAPER_ROT, HEAVY)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_taper_tests.rs"]
mod tests;
