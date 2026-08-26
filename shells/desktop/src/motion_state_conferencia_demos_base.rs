//! **A BASE DO RUÍDO** — a cena `=97` (doc 89, folha 06 linha 21).
//!
//! Quatro pares. ⚠️ **Esta cena é ESTÁTICA** — não precisa de Play.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | base | `Gradient` (o de sempre) | **`Value`** — os extremos caem NA grelha |
//! | base | `Gradient` | **`Cellular`** — células, não manchas |
//! | métrica | `Euclidean` | **`Manhattan`** — losangos |
//! | métrica | `Euclidean` | **`Chebyshev`** — quadrados |
//!
//! ⚠️ **A BASE não é o `type`.** O `type` escolhe a rectificação por oitava (fBm ·
//! turbulência · ridged), que é o que se faz *com* o ruído; a base é o ruído em si. Nenhum
//! valor de `type` produz uma célula.
//!
//! ## ⚠️ A LEI QUE ESTA CENA HERDA: **posicionar é UPSTREAM da máscara**
//!
//! Paga pela cena `=73`: todo comportamento desta biblioteca é mascarado pelo `falloff`,
//! então um deslocamento de colocação posto DEPOIS do campo vira `dx · falloff_i` e a banda
//! estica-se por cima das vizinhas. ⛔ [`place`] corre imediatamente a seguir à fonte.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna — a da esquerda é sempre *como era*.
const COL_X: f32 = 2.35;
/// O centro de cada linha, de cima para baixo.
const ROW_Y: [f32; 4] = [3.4, 1.1, -1.2, -3.5];
/// A grelha de cada banda.
const SIDE: f32 = 15.0;
const GAP: f32 = 0.135;
/// O tamanho de repouso, e quanto o ruído acrescenta por cima dele.
///
/// ⛔ **O repouso tem de ser MAIOR que a amplitude, e o número que obriga a isso saiu da
/// medição desta mesma wave:** o gradiente a 4 oitavas só alcança `65%` da faixa declarada e
/// as bases novas alcançam `~90%`, logo uma amplitude segura para o Perlin deixa peças de
/// tamanho **negativo** na base `Value`. A 1.ª versão desta cena tinha `0,055` de repouso
/// contra `0,075` de amplitude e a banda 1 nasceu sem tamanho. *A minha própria medição
/// previa isto e eu não a apliquei.*
const PIECE: f32 = 0.095;
const AMPLITUDE: f32 = 0.07;
/// A escala do campo — quantas feições cabem na banda.
const SCALE: f32 = 1.35;

/// O canal `Size` do `motion.noise` (a escada é a do `ParamWidget::Enum` daquele nó).
const NOISE_SIZE: f32 = 3.0;

/// As bases e as métricas, com os nomes que o painel mostra.
const BASE_GRADIENT: f32 = 0.0;
const BASE_VALUE: f32 = 1.0;
const BASE_CELLULAR: f32 = 2.0;
const METRIC_EUCLID: f32 = 0.0;
const METRIC_MANHATTAN: f32 = 1.0;
const METRIC_CHEBYSHEV: f32 = 2.0;

/// `(base, métrica)` de cada uma das oito bandas, na ordem em que a cena as monta.
const BANDS: [(f32, f32); 8] = [
    (BASE_GRADIENT, METRIC_EUCLID),
    (BASE_VALUE, METRIC_EUCLID),
    (BASE_GRADIENT, METRIC_EUCLID),
    (BASE_CELLULAR, METRIC_EUCLID),
    (BASE_CELLULAR, METRIC_EUCLID),
    (BASE_CELLULAR, METRIC_MANHATTAN),
    (BASE_CELLULAR, METRIC_EUCLID),
    (BASE_CELLULAR, METRIC_CHEBYSHEV),
];

/// A cor de cada linha.
const LIT: [[f32; 3]; 4] = [
    [0.52, 0.76, 1.0],
    [1.0, 0.78, 0.4],
    [0.66, 1.0, 0.72],
    [0.95, 0.6, 0.78],
];

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

/// **PÔR A BANDA NO QUADRANTE DELA — e isto corre ANTES de qualquer campo.**
fn place(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32) -> NodeId {
    push(
        g,
        head,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        200.0,
    )
}

/// Uma banda: a grelha posicionada, o ruído por cima, a cor e a saída.
fn band(g: &mut Graph, k: usize) -> Option<NodeId> {
    let (row, col) = (k / 2, k % 2);
    let ey = k as f32 * 240.0;
    let (base, metric) = BANDS[k];
    let n = node(
        g,
        "motion.grid",
        &[
            ("rows", SIDE),
            ("cols", SIDE),
            ("gap_x", GAP),
            ("gap_y", GAP),
        ],
        ey,
        80.0,
    );
    let at = [if col == 0 { -COL_X } else { COL_X }, ROW_Y[row]];
    let placed = place(g, n, at, ey);
    let sized = push(g, placed, "motion.scale", &[("amount", PIECE)], ey, 300.0);
    // ⚠️ O `seed` é o MESMO nos oito: o que muda entre os dois lados de um par tem de ser
    // só a base (ou a métrica), senão o olho não sabe a que atribuir a diferença.
    let noisy = push(
        g,
        sized,
        "motion.noise",
        &[
            ("channel", NOISE_SIZE),
            ("amplitude", AMPLITUDE),
            ("scale", SCALE),
            ("octaves", 1.0),
            ("seed", 7.0),
            ("base", base),
            ("metric", metric),
        ],
        ey,
        420.0,
    );
    let t = node(
        g,
        "motion.tint",
        &[("r", LIT[row][0]), ("g", LIT[row][1]), ("b", LIT[row][2])],
        ey,
        700.0,
    );
    wire(g, noisy, 0, t, 0)?;
    let out = node(g, "motion.output", &[], ey, 840.0);
    wire(g, t, 0, out, 0)?;
    Some(out)
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_base_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(8);
    for k in 0..8 {
        sinks.push(band(g, k)?);
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "Gradient -- o ruido de sempre: manchas suaves, extremos ENTRE a grelha",
        "Value -- os extremos caem NA grelha, e ela ve^-se",
        "Gradient -- outra vez, para comparar",
        "Cellular -- CELULAS: cada uma tem um centro e uma fronteira",
        "Cellular Euclidean -- as celulas sao redondas",
        "Cellular Manhattan -- as celulas viram LOSANGOS",
        "Cellular Euclidean -- outra vez, para comparar",
        "Cellular Chebyshev -- as celulas viram QUADRADOS",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let at = [if col == 0 { -COL_X } else { COL_X }, ROW_Y[row] + 0.95];
            crate::motion_demo_legend::Caption::new(at, short_of(label))
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
#[path = "motion_state_conferencia_demos_base_tests.rs"]
mod tests;
