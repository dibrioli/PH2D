//! **DOIS EXEMPLOS, UM POR LINHA** — a cena `=75` (doc 89, folha 03: o pin que rasga e
//! o bando que desvia).
//!
//! | linha | esquerda | direita |
//! |---|---|---|
//! | **RASGA** | o pin segura para sempre | **`Break Above`** — a carga do vento arranca-o |
//! | **DESVIA** | o bando atravessa a pedra | **`Avoid`** — ele contorna-a |
//!
//! ⚠️ **SÓ SE JULGA COM O PLAY.** As duas linhas são simulação: paradas, as quatro
//! bandas são quatro nuvens iguais.
//!
//! ⚠️ **O PIN VIVE NO CAMINHO DA ARTE, e a CARGA chega-lhe por uma porta.** A v1 desta
//! cena pôs o pin dentro do laço da força, e o smoke voltou com *"tudo foi levado pelo
//! vento, nada rasgou"*. MEDIDO: o `motion.integrate` lê o `accel` do `state`
//! (`ctx.input(1)`) mas o **`inv_mass` do `rest`** (`ctx.input(0)`) — um pin no laço
//! escreve um `inv_mass` que **ninguém lê**.
//!
//! ```text
//! grid ──────────────► pin_constraint ──► integrate.rest      (o inv_mass chega)
//! integrate ═pre═► force.wind ─────────► integrate.forces     (o vento move)
//!                   force.wind ═pre═══► pin.load              (a carga chega ao pin)
//!                                  pin ═pre═► pin.state       (a memória do rasgo)
//! ```
//!
//! ⚠️ **A colocação corre ANTES do campo** (a lei que a `=73` pagou).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

pub(crate) const COL_X: f32 = 3.1;
pub(crate) const ROW_Y: [f32; 2] = [2.6, -2.9];
const HEADER_Y: f32 = 5.6;
const LABEL_SIZE: f32 = 0.42;
pub(crate) const ROW_LABELS: [&str; 2] = ["RASGA", "DESVIA"];

/// A cortina da linha 1: `(colunas, linhas, passo, tamanho da peça)`.
pub(crate) const CURTAIN: (f32, f32, f32, f32) = (6.0, 6.0, 0.28, 0.13);
/// Quantas peças o bando da linha 2 tem, e o espalhamento inicial.
pub(crate) const FLOCK: f32 = 40.0;

const REST: [f32; 3] = [0.26, 0.27, 0.32];
const LIT: [[f32; 3]; 2] = [[0.55, 0.85, 1.0], [1.0, 0.78, 0.40]];
const LABEL_RGB: [f32; 3] = [0.62, 0.64, 0.70];
/// A cor da PEDRA e dos pinos — branco, para se distinguirem de quem os sofre.
const MARK_RGB: [f32; 3] = [1.0, 1.0, 1.0];

/// A carga acima da qual o pin da direita rasga.
///
/// ⚠️ **Ela é MENOR que a força do vento de propósito** — senão o pin nunca rasgaria e a
/// linha ficaria com as duas metades iguais. O gate `the_tear_threshold_is_below_the_wind`
/// deriva a comparação em vez de a repetir.
pub(crate) const BREAK_ABOVE: f32 = 4.0;
pub(crate) const WIND: f32 = 9.0;
/// O ângulo do vento — para cima e para o lado, para que o rasgo se veja subir.
const WIND_ANGLE: f32 = 60.0;
/// A fileira de cima da cortina 6×6: a grelha é row-major de BAIXO para cima, então as
/// seis ÚLTIMAS peças são o topo. ⚠️ Derivado, não escrito — mudar a grelha move o pin.
#[must_use]
pub(crate) fn pinned_run() -> (f32, f32) {
    let (cols, rows, ..) = CURTAIN;
    ((rows - 1.0) * cols, cols)
}

/// O peso e o raio do desvio da linha 2, e o anel de pedras.
pub(crate) const AVOID: f32 = 14.0;
pub(crate) const AVOID_RADIUS: f32 = 1.0;
pub(crate) const ROCKS: f32 = 6.0;
pub(crate) const ROCK_RING: f32 = 0.9;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

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

/// A colocação — ver o aviso no topo do módulo.
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
    let out = node(g, "motion.output", &[], ey, 1120.0);
    wire(g, tail, 0, out, 0)?;
    Some(out)
}

fn tint(g: &mut Graph, head: NodeId, rgb: [f32; 3], ey: f32, x: f32) -> NodeId {
    push(
        g,
        head,
        "motion.tint",
        &[("r", rgb[0]), ("g", rgb[1]), ("b", rgb[2])],
        ey,
        x,
    )
}

fn label(g: &mut Graph, word: &str, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let t = g.add_node("source.text");
    g.set_pos(t, Pos { x: 0.0, y: ey });
    g.set_text_param(t, ph2d_node_source_text::TEXT_KEY, word);
    g.set_param(t, ph2d_node_source_text::param::SIZE, LABEL_SIZE);
    g.set_param(t, ph2d_node_source_text::param::ALIGN, 1.0);
    let placed = place(g, t, at, ey, 200.0);
    let tinted = tint(g, placed, LABEL_RGB, ey, 320.0);
    out_of(g, tinted, ey)
}

/// **LINHA 1 · RASGA** — a cortina pinada no topo, contra o vento.
fn tear_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = f32::from(u8::from(right)) * 240.0;
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[0]];
    let (cols, rows, gap, piece) = CURTAIN;
    let grid = node(
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
    let scaled = push(g, grid, "motion.scale", &[("amount", piece)], ey, 110.0);
    let placed = place(g, scaled, at, ey, 210.0);
    let base = tint(g, placed, REST, ey, 310.0);

    // O PIN, no caminho da ARTE — é de lá que o integrador lê o `inv_mass`.
    let (first, count) = pinned_run();
    let pin = node(
        g,
        "motion.pin_constraint",
        &[
            ("first", first),
            ("count", count),
            ("strength", 1.0),
            ("break_above", if right { BREAK_ABOVE } else { 0.0 }),
        ],
        ey,
        420.0,
    );
    wire(g, base, 0, pin, 0)?;
    let integ = node(g, "motion.integrate", &[], ey, 700.0);
    wire(g, pin, 0, integ, 0)?;
    // O LAÇO da força: é ele que MOVE.
    let wind = node(
        g,
        "force.wind",
        &[("angle", WIND_ANGLE), ("strength", WIND), ("gust", 0.0)],
        ey + 150.0,
        560.0,
    );
    wire_pre(g, integ, wind, 0)?;
    wire(g, wind, 0, integ, 1)?;
    // A CARGA: o mesmo vento, pelo `pre` que quebra o ciclo. ⛔ Não duplique a força
    // para dar carga ao pin — seriam dois números a dizer a mesma coisa.
    wire_pre(g, wind, pin, 2)?;
    // A MEMÓRIA do rasgo: sem este fio ele cede e volta a pinar (elástico, não rasgo).
    wire_pre(g, pin, pin, 1)?;

    let lit = tint(g, integ, LIT[0], ey, 900.0);
    out_of(g, lit, ey)
}

/// **LINHA 2 · DESVIA** — o bando contra um anel de pedras.
///
/// Devolve `(o sink do bando, o sink das pedras)` — o mesmo nó alimenta a força e a
/// tela, para que não haja dois números a dizer onde a pedra está.
fn avoid_band(g: &mut Graph, right: bool) -> Option<(NodeId, NodeId)> {
    let ey = (2 + usize::from(right)) as f32 * 240.0;
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[1]];
    // As PEDRAS, na origem — o bando simula ali, e uma colocação só move os dois.
    let ring = node(
        g,
        "motion.distribute_radial",
        &[
            ("count", ROCKS),
            ("rings", 1.0),
            ("radius", ROCK_RING),
            ("inner", 0.0),
        ],
        ey + 150.0,
        0.0,
    );
    let rock_big = push(
        g,
        ring,
        "motion.scale",
        &[("amount", 0.2)],
        ey + 150.0,
        110.0,
    );
    let rock_seen = place(g, rock_big, at, ey + 150.0, 240.0);
    let rock_tint = tint(g, rock_seen, MARK_RGB, ey + 150.0, 380.0);
    let rocks_out = out_of(g, rock_tint, ey + 150.0)?;

    let flock = node(
        g,
        "motion.boids",
        &[
            ("count", FLOCK),
            ("seed", 7.0),
            ("radius", 1.2),
            ("separation", 1.4),
            ("alignment", 1.0),
            ("cohesion", 0.6),
            // O `seek` traz o bando ao centro — é ele que o faz passar pelas pedras.
            ("seek", 2.2),
            ("max_speed", 2.4),
            ("spread", 3.0),
            ("avoid", if right { AVOID } else { 0.0 }),
            ("avoid_radius", AVOID_RADIUS),
            // ⚠️ A antecipação é a MESMA dos dois lados: o que a linha compara é o
            // desvio, não quanto ele olha à frente.
            ("lookahead", 0.25),
        ],
        ey,
        420.0,
    );
    wire_pre(g, flock, flock, 2)?;
    if right {
        wire(g, ring, 0, flock, 3)?;
    }
    let sized = push(g, flock, "motion.scale", &[("amount", 0.13)], ey, 620.0);
    let placed = place(g, sized, at, ey, 760.0);
    let lit = tint(g, placed, LIT[1], ey, 900.0);
    Some((out_of(g, lit, ey)?, rocks_out))
}

/// Monta a cena. Devolve os quatro sinks das nuvens seguidos dos das pedras.
pub(crate) fn build_sim_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(6);
    for right in [false, true] {
        sinks.push(tear_band(g, right)?);
    }
    let mut rocks = Vec::with_capacity(2);
    for right in [false, true] {
        let (band, rock) = avoid_band(g, right)?;
        sinks.push(band);
        rocks.push(rock);
    }
    sinks.extend(rocks);
    label(g, "ANTES", [-COL_X, HEADER_Y], 2000.0)?;
    label(g, "DEPOIS", [COL_X, HEADER_Y], 2140.0)?;
    for (k, word) in ROW_LABELS.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "duas linhas")]
        let ey = 2280.0 + k as f32 * 140.0;
        label(g, word, [0.0, ROW_Y[k]], ey)?;
    }
    Some(sinks)
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (BREAK_ABOVE, ROCKS)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_sim_tests.rs"]
mod tests;
