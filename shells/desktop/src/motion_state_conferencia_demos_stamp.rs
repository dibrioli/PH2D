//! **O VOCABULÁRIO DO CARIMBO** — a cena `=98` (doc 89, folha 08).
//!
//! Três pares. ⚠️ **Esta cena é ESTÁTICA** — não precisa de Play.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | transferência | `Shape Wins` (o de sempre) — **a cor do arranjo SOME** | `Point Wins` — ela chega |
//! | transferência | `Point Wins` — a cor do ponto | **`Multiply`** — ela é tingida pela da forma |
//! | ordenação | `Random` sobre a fila TODA — confete | **`Random` dentro de GRUPOS** — quatro faixas |
//!
//! ## O que cada par prova
//!
//! ⚠️ **As duas primeiras fileiras são um DEFEITO, não um knob.** O `motion.duplicator`
//! replicava as colunas da FORMA e somava `P`/`rot`; toda coluna autorada só nos PONTOS
//! desaparecia — e uma rampa de cor sobre o arranjo é o gesto mais comum que há. À esquerda
//! do 1.º par as dezasseis cópias saem todas com a cor da forma; à direita sai a rampa que
//! foi de facto autorada.
//!
//! ⚠️ **A terceira mostra uma ordem, e ordem não se vê** — por isso a cor é a régua: depois
//! de ordenar, a peça de posto `r` recebe `rampa(r/(N−1))`, e a POSIÇÃO dela é a que ela
//! sempre teve. Um baralhamento global espalha as cores por toda a fila; um baralhamento
//! **dentro de grupos** deixa cada quarto da fila com as cores do quarto dele — quatro
//! faixas legíveis, cada uma embaralhada por dentro.
//!
//! ⚠️ **A régua da 3.ª fileira é a MESMA nos dois lados** (`Random`, a mesma semente): o que
//! muda é só a porta `group` estar ligada. Se as sementes diferissem, o olho atribuiria a
//! diferença ao acaso e não à lei.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna — a da esquerda é sempre *como era*.
const COL_X: f32 = 2.6;
/// O centro de cada fileira, de cima para baixo.
const ROW_Y: [f32; 3] = [2.7, 0.0, -2.7];
/// Quantas peças tem cada fila.
const PIECES: f32 = 16.0;
/// O tamanho de uma peça e o passo entre elas, em unidades de mundo.
const PIECE: f32 = 0.2;
const GAP: f32 = 0.26;
/// Em quantos GRUPOS a 3.ª fileira parte a fila. `PIECES / GROUPS` peças em cada.
const GROUPS: f32 = 4.0;

/// A escada do param `transfer` do `motion.duplicator` (a ordem do `ParamWidget::Enum`).
const SHAPE_WINS: f32 = 0.0;
const POINT_WINS: f32 = 1.0;
const MULTIPLY: f32 = 3.0;
/// O modo **Floor** do `value.quantize`. ⚠️ **O default é `Round`, e com ele a cena mentia:**
/// `round(i / 4) · 4` põe os índices `2` e `3` no grupo `4`, então as fronteiras dos grupos
/// caem a meio das faixas e o gate mede 5 peças certas em 16. *Um `quantize` para agrupar
/// quer FLOOR — a fronteira tem de cair entre grupos, não no meio deles.*
const QUANTIZE_FLOOR: f32 = 1.0;
/// A escada do param `mode` do `value.instance_field`.
const FIELD_INDEX: f32 = 0.0;
const FIELD_RAMP: f32 = 1.0;
/// A escada do param `key` do `motion.sort` (`3` = Random).
const SORT_RANDOM: f32 = 3.0;
/// A semente das duas metades do 3.º par — **a mesma**, de propósito.
const SORT_SEED: f32 = 11.0;

/// A cor da FORMA nas duas primeiras fileiras. ⚠️ Um cinzento a meio caminho: no
/// `Multiply` ele tem de escurecer a rampa **visivelmente** sem a apagar, e o branco
/// (`1,1,1`) é o neutro daquele modo — com ele o par saía igual dos dois lados.
const SHAPE_RGB: [f32; 3] = [0.55, 0.62, 0.45];

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = node(g, kind, ps, ey, x);
    let _ = wire(g, head, 0, n, 0);
    n
}

/// A fila de `PIECES` peças, já posta no quadrante da banda.
///
/// ⚠️ **O `motion.transform` corre imediatamente a seguir à fonte** — é a lei que a cena
/// `=73` pagou: um deslocamento posto depois de um campo vira `dx · falloff` e a banda
/// estica-se por cima das vizinhas.
fn row_at(g: &mut Graph, at: [f32; 2], ey: f32) -> NodeId {
    let grid = node(
        g,
        "motion.grid",
        &[("rows", 1.0), ("cols", PIECES), ("gap_x", GAP)],
        ey,
        80.0,
    );
    let placed = push(
        g,
        grid,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        220.0,
    );
    push(g, placed, "motion.scale", &[("amount", PIECE)], ey, 340.0)
}

/// A rampa de cor sobre um stream, com o `t` a vir do posto de cada peça NAQUELE stream.
///
/// ⚠️ **O `t` é minado depois** do nó que decide a ordem: o `value.instance_field(Ramp)` dá
/// `i/(N−1)` pela POSIÇÃO na lista, que depois de um `sort` é o posto. É isso que faz a cor
/// ser a régua da ordem.
fn ramp_over(g: &mut Graph, head: NodeId, ey: f32, x: f32) -> Option<NodeId> {
    let t = node(g, "value.instance_field", &[("mode", FIELD_RAMP)], ey, x);
    wire(g, head, 0, t, 0)?;
    let ramp = node(g, "motion.color_ramp", &[], ey, x + 140.0);
    wire(g, head, 0, ramp, 0)?;
    wire(g, t, 0, ramp, 1)?;
    Some(ramp)
}

/// Uma banda das DUAS primeiras fileiras: uma forma cinzenta carimbada numa fila de pontos
/// que trazem a rampa, com o `transfer` a decidir se a cor do ponto chega.
fn stamp_band(g: &mut Graph, at: [f32; 2], ey: f32, transfer: f32) -> Option<NodeId> {
    // A FORMA: uma peça só, com a cor dela.
    let one = node(g, "motion.grid", &[("rows", 1.0), ("cols", 1.0)], ey, 80.0);
    let sized = push(g, one, "motion.scale", &[("amount", PIECE)], ey, 220.0);
    let shape = push(
        g,
        sized,
        "motion.tint",
        &[
            ("r", SHAPE_RGB[0]),
            ("g", SHAPE_RGB[1]),
            ("b", SHAPE_RGB[2]),
        ],
        ey,
        340.0,
    );
    // OS PONTOS: a fila, com a rampa autorada em cima deles.
    let points = row_at(g, at, ey + 120.0);
    let coloured = ramp_over(g, points, ey + 120.0, 480.0)?;
    // O CARIMBO.
    let dup = node(g, "motion.duplicator", &[("transfer", transfer)], ey, 820.0);
    wire(g, shape, 0, dup, 0)?;
    wire(g, coloured, 0, dup, 1)?;
    let out = node(g, "motion.output", &[], ey, 980.0);
    wire(g, dup, 0, out, 0)?;
    Some(out)
}

/// Uma banda da TERCEIRA fileira: a fila baralhada, com ou sem a porta `group`.
fn sort_band(g: &mut Graph, at: [f32; 2], ey: f32, grouped: bool) -> Option<NodeId> {
    let row = row_at(g, at, ey);
    let sort = node(
        g,
        "motion.sort",
        &[("key", SORT_RANDOM), ("seed", SORT_SEED)],
        ey,
        480.0,
    );
    wire(g, row, 0, sort, 0)?;
    if grouped {
        // O GRUPO: o índice ORIGINAL de cada peça, arredondado ao tamanho do grupo. Tem de
        // sair do stream de ANTES do sort — é assim que ele chega alinhado com a entrada.
        let idx = node(
            g,
            "value.instance_field",
            &[("mode", FIELD_INDEX)],
            ey + 120.0,
            480.0,
        );
        wire(g, row, 0, idx, 0)?;
        let grp = push(
            g,
            idx,
            "value.quantize",
            &[("step", PIECES / GROUPS), ("mode", QUANTIZE_FLOOR)],
            ey + 120.0,
            620.0,
        );
        wire(g, grp, 0, sort, 2)?;
    }
    let coloured = ramp_over(g, sort, ey, 760.0)?;
    let out = node(g, "motion.output", &[], ey, 1060.0);
    wire(g, coloured, 0, out, 0)?;
    Some(out)
}

/// `(fileira, coluna)` de cada uma das seis bandas, na ordem em que a cena as monta.
fn quadrant(k: usize) -> [f32; 2] {
    let (row, col) = (k / 2, k % 2);
    [if col == 0 { -COL_X } else { COL_X }, ROW_Y[row]]
}

/// Monta a cena. Devolve os seis sinks, em pares.
pub(crate) fn build_stamp_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(6);
    for k in 0..6 {
        let ey = k as f32 * 320.0;
        let at = quadrant(k);
        let sink = match k {
            0 => stamp_band(g, at, ey, SHAPE_WINS)?,
            1 => stamp_band(g, at, ey, POINT_WINS)?,
            2 => stamp_band(g, at, ey, POINT_WINS)?,
            3 => stamp_band(g, at, ey, MULTIPLY)?,
            4 => sort_band(g, at, ey, false)?,
            _ => sort_band(g, at, ey, true)?,
        };
        sinks.push(sink);
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das seis bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "Shape Wins -- a cor autorada no ARRANJO some: 16 copias iguais",
        "Point Wins -- a mesma cena, e a rampa chega",
        "Point Wins -- a cor do ponto, limpa",
        "Multiply -- a mesma rampa, tingida pela cor da forma",
        "Random na fila TODA -- as cores espalham-se de ponta a ponta",
        "Random dentro de GRUPOS -- quatro faixas, cada uma baralhada por dentro",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let at = quadrant(k);
            crate::motion_demo_legend::Caption::new([at[0], at[1] + 0.75], short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro `--`.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_stamp_tests.rs"]
mod tests;
