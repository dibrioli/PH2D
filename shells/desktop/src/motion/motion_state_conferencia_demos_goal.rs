//! **DOIS EXEMPLOS, UM POR LINHA** — a cena `=74` (doc 89, folha 02: as duas células
//! que sobravam, e as duas são do `force.attractor`).
//!
//! | linha | esquerda | direita |
//! |---|---|---|
//! | **ALVO** | um ponto só (os dois params) | **um STREAM** — cada peça vai ao ponto mais PRÓXIMO dela |
//! | **MIRA** | mirar onde o alvo ESTÁ | **antecipar** — cada peça lidera pelo próprio tempo-de-chegada |
//!
//! ⚠️ **SÓ SE JULGA COM O PLAY.** Uma força não move nada sozinha: ela acumula em
//! `accel` e é o `motion.integrate` que aplica. Paradas, as quatro bandas são quatro
//! nuvens iguais — a leitura é o CAMINHO que cada uma faz.
//!
//! ⚠️ **A colocação corre ANTES do campo** (a lei que a `=73` pagou — ver o módulo
//! irmão): todo comportamento desta biblioteca é multiplicado pelo `falloff`, e um
//! `motion.transform` posto depois de um campo desloca cada peça uma distância
//! diferente.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna, e de cada linha.
pub(crate) const COL_X: f32 = 3.1;
pub(crate) const ROW_Y: [f32; 2] = [2.6, -2.9];
/// Onde os dois rótulos de coluna assentam.
const HEADER_Y: f32 = 5.6;
const LABEL_SIZE: f32 = 0.42;
/// O nome de cada linha, pintado no vão entre as duas colunas.
pub(crate) const ROW_LABELS: [&str; 2] = ["ALVO", "MIRA"];

/// A nuvem de cada linha: `(colunas, linhas, passo, tamanho da peça)`.
pub(crate) const BANDS: [(f32, f32, f32, f32); 2] =
    [(6.0, 6.0, 0.30, 0.13), (10.0, 1.0, 0.22, 0.13)];

/// A largura e a altura que a nuvem da linha `k` ocupa em REPOUSO.
///
/// ⚠️ `#[cfg(test)]` porque só o GATE de layout a consulta — a cena posiciona pelo
/// centro do quadrante, e é o gate que precisa de saber quanto a nuvem ocupa à volta
/// dele. Deixá-la viva em produção seria código morto a fingir-se de API.
#[cfg(test)]
#[must_use]
pub(crate) fn footprint(k: usize) -> (f32, f32) {
    let (cols, rows, gap, _) = BANDS[k];
    ((cols - 1.0) * gap, (rows - 1.0) * gap)
}

/// A cor de repouso e a de cada linha.
const REST: [f32; 3] = [0.26, 0.27, 0.32];
const LIT: [[f32; 3]; 2] = [[0.45, 0.85, 1.0], [1.0, 0.72, 0.35]];
const LABEL_RGB: [f32; 3] = [0.62, 0.64, 0.70];
/// A cor dos ALVOS — branco, para que se distingam das peças que os perseguem.
const TARGET_RGB: [f32; 3] = [1.0, 1.0, 1.0];

/// `Target = Stream` (o valor do enum).
pub(crate) const TARGET_STREAM: f32 = 1.0;
/// Quantos alvos a linha 1 põe à direita, e o raio do anel deles.
pub(crate) const GOALS: f32 = 3.0;
pub(crate) const GOAL_RADIUS: f32 = 1.05;
/// A antecipação que a linha 2 autora à direita, em segundos.
pub(crate) const LEAD: f32 = 0.9;
/// O balanço do alvo da linha 2 — amplitude e período.
pub(crate) const SWING: f32 = 1.6;
pub(crate) const SWING_HZ: f32 = 0.35;
/// A força e o alcance do atrator, iguais nas quatro bandas.
const PULL: f32 = 7.0;
const REACH: f32 = 9.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// O fio de RETORNO do integrador para a força — o laço que o artista não desenha.
fn wire_pre(g: &mut Graph, from: NodeId, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, 0),
        to: (to, tp),
        delayed: true,
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

/// **A colocação, e ela corre logo a seguir à fonte.** Ver o aviso no topo do módulo.
fn place(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32, x: f32) -> NodeId {
    push(
        g,
        head,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        x,
    )
}

fn out_of(g: &mut Graph, tail: NodeId, ey: f32) -> Option<NodeId> {
    let out = node(g, "motion.output", &[], ey, 1060.0);
    wire(g, tail, 0, out, 0)?;
    Some(out)
}

/// A nuvem de uma linha, já posicionada e já pintada de repouso.
fn seed(g: &mut Graph, k: usize, right: bool, ey: f32) -> NodeId {
    let (cols, rows, gap, piece) = BANDS[k];
    let n = node(
        g,
        "motion.grid",
        &[
            ("rows", rows),
            ("cols", cols),
            ("gap_x", gap),
            ("gap_y", gap),
        ],
        ey,
        0.0,
    );
    let scaled = push(g, n, "motion.scale", &[("amount", piece)], ey, 110.0);
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[k]];
    let placed = place(g, scaled, at, ey, 210.0);
    push(
        g,
        placed,
        "motion.tint",
        &[("r", REST[0]), ("g", REST[1]), ("b", REST[2])],
        ey,
        310.0,
    )
}

/// Fecha uma banda: o integrador, o laço da força, a cor e a saída.
///
/// `force` devolve **(cabeça, ponta)** da cadeia de forças — a cabeça recebe o `pre` do
/// integrador, a ponta alimenta o `forces` dele.
fn band(
    g: &mut Graph,
    k: usize,
    right: bool,
    force: impl FnOnce(&mut Graph, f32) -> (NodeId, NodeId),
) -> Option<NodeId> {
    let ey = (k * 2 + usize::from(right)) as f32 * 240.0;
    let base = seed(g, k, right, ey);
    let integ = node(g, "motion.integrate", &[], ey, 560.0);
    wire(g, base, 0, integ, 0)?;
    let (head, tail) = force(g, ey);
    wire_pre(g, integ, head, 0)?;
    wire(g, tail, 0, integ, 1)?;
    let tint = push(
        g,
        integ,
        "motion.tint",
        &[("r", LIT[k][0]), ("g", LIT[k][1]), ("b", LIT[k][2])],
        ey,
        860.0,
    );
    out_of(g, tint, ey)
}

/// Uma palavra no canvas.
fn label(g: &mut Graph, word: &str, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let t = g.add_node("source.text");
    g.set_pos(t, Pos { x: 0.0, y: ey });
    g.set_text_param(t, ph2d_node_source_text::TEXT_KEY, word);
    g.set_param(t, ph2d_node_source_text::param::SIZE, LABEL_SIZE);
    g.set_param(t, ph2d_node_source_text::param::ALIGN, 1.0);
    let placed = place(g, t, at, ey, 200.0);
    let tinted = push(
        g,
        placed,
        "motion.tint",
        &[
            ("r", LABEL_RGB[0]),
            ("g", LABEL_RGB[1]),
            ("b", LABEL_RGB[2]),
        ],
        ey,
        320.0,
    );
    out_of(g, tinted, ey)
}

/// **OS ALVOS, VISÍVEIS.** Um alvo que não se vê torna a cena um enigma: as peças
/// convergem para *algum sítio* e o artista tem de adivinhar qual.
///
/// Devolve **(o stream dos alvos, o sink que os desenha)** — o mesmo nó alimenta a
/// força e a tela, para que não haja dois números a dizer onde o alvo está.
fn goals(g: &mut Graph, k: usize, right: bool, ey: f32, ring: bool) -> Option<(NodeId, NodeId)> {
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[k]];
    let src = if ring {
        node(
            g,
            "motion.distribute_radial",
            &[
                ("count", GOALS),
                ("rings", 1.0),
                ("radius", GOAL_RADIUS),
                ("inner", 0.0),
            ],
            ey + 150.0,
            0.0,
        )
    } else {
        node(
            g,
            "motion.grid",
            &[("rows", 1.0), ("cols", 1.0)],
            ey + 150.0,
            0.0,
        )
    };
    let big = push(
        g,
        src,
        "motion.scale",
        &[("amount", 0.22)],
        ey + 150.0,
        110.0,
    );
    let placed = place(g, big, at, ey + 150.0, 210.0);
    let tinted = push(
        g,
        placed,
        "motion.tint",
        &[
            ("r", TARGET_RGB[0]),
            ("g", TARGET_RGB[1]),
            ("b", TARGET_RGB[2]),
        ],
        ey + 150.0,
        310.0,
    );
    Some((tinted, out_of(g, tinted, ey + 150.0)?))
}

/// **LINHA 1 · ALVO** — um ponto contra um STREAM de pontos.
fn goal_band(g: &mut Graph, right: bool) -> Option<(NodeId, NodeId)> {
    let ey = f32::from(u8::from(right)) * 240.0;
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[0]];
    let (targets, shown) = goals(g, 0, right, ey, right)?;
    let sink = band(g, 0, right, |g, ey| {
        let a = node(
            g,
            "force.attractor",
            &[
                ("strength", PULL),
                ("radius", REACH),
                ("target_mode", if right { TARGET_STREAM } else { 0.0 }),
                ("target_x", at[0]),
                ("target_y", at[1]),
            ],
            ey + 300.0,
            560.0,
        );
        if right {
            let _ = wire(g, targets, 0, a, 1);
        }
        (a, a)
    })?;
    Some((sink, shown))
}

/// **LINHA 2 · MIRA** — perseguir onde o alvo está, contra antecipar.
///
/// O alvo BALANÇA (um `motion.oscillator` em Y) e um `motion.velocity` publica a
/// velocidade dele — sem essa coluna não há o que antecipar, e o knob seria mudo.
fn aim_band(g: &mut Graph, right: bool) -> Option<(NodeId, NodeId)> {
    let ey = (2 + usize::from(right)) as f32 * 240.0;
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[1]];
    // O alvo: um ponto que sobe e desce, com a velocidade dele publicada.
    let one = node(
        g,
        "motion.grid",
        &[("rows", 1.0), ("cols", 1.0)],
        ey + 150.0,
        0.0,
    );
    let big = push(
        g,
        one,
        "motion.scale",
        &[("amount", 0.22)],
        ey + 150.0,
        110.0,
    );
    let swung = push(
        g,
        big,
        "motion.oscillator",
        &[
            ("channel", 1.0), // Y
            ("amplitude", SWING),
            ("frequency", SWING_HZ),
        ],
        ey + 150.0,
        210.0,
    );
    let placed = place(g, swung, at, ey + 150.0, 310.0);
    // ⚠️ O `motion.velocity` MEDE (ele não escreve pose), e é a coluna `vel` dele que
    // a antecipação lê. Ele carrega estado: o `pre` self-loop é o que lhe dá o quadro
    // anterior para comparar.
    let vel = push(g, placed, "motion.velocity", &[], ey + 150.0, 410.0);
    g.connect(Edge {
        from: (vel, 0),
        to: (vel, 1),
        delayed: true,
    })
    .ok()?;
    let tinted = push(
        g,
        vel,
        "motion.tint",
        &[
            ("r", TARGET_RGB[0]),
            ("g", TARGET_RGB[1]),
            ("b", TARGET_RGB[2]),
        ],
        ey + 150.0,
        510.0,
    );
    let shown = out_of(g, tinted, ey + 150.0)?;
    let sink = band(g, 1, right, |g, ey| {
        let a = node(
            g,
            "force.attractor",
            &[
                ("strength", PULL),
                ("radius", REACH),
                ("target_mode", TARGET_STREAM),
                ("lead", if right { LEAD } else { 0.0 }),
            ],
            ey + 300.0,
            560.0,
        );
        let _ = wire(g, vel, 0, a, 1);
        (a, a)
    })?;
    Some((sink, shown))
}

/// Monta a cena. Devolve os quatro sinks das NUVENS seguidos dos que desenham os
/// alvos — as legendas têm sinks próprios, fora desta lista.
pub(crate) fn build_goal_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut clouds = Vec::with_capacity(4);
    let mut marks = Vec::with_capacity(4);
    for right in [false, true] {
        let (c, m) = goal_band(g, right)?;
        clouds.push(c);
        marks.push(m);
    }
    for right in [false, true] {
        let (c, m) = aim_band(g, right)?;
        clouds.push(c);
        marks.push(m);
    }
    label(g, "ANTES", [-COL_X, HEADER_Y], 2000.0)?;
    label(g, "DEPOIS", [COL_X, HEADER_Y], 2140.0)?;
    for (k, word) in ROW_LABELS.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "duas linhas")]
        let ey = 2280.0 + k as f32 * 140.0;
        label(g, word, [0.0, ROW_Y[k]], ey)?;
    }
    clouds.extend(marks);
    Some(clouds)
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (GOALS, LEAD)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_goal_tests.rs"]
mod tests;
