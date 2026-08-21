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
//! ⚠️ **A cortina é um `motion.soft_body`, e não uma grelha de pontos soltos** (Enio,
//! 2026-08-21: *"porque usar grid se temos nós de tecido?"*). Ele tinha razão, e o custo
//! do erro era o exemplo: uma nuvem de pontos que voa não mostra um pano a **rasgar** —
//! mostra pontos a irem embora. Com o tecido vê-se a folha inteira soltar-se.
//!
//! ⚠️ **E o tecido é o idioma FÁCIL do pin.** O `motion.soft_body` lê o `inv_mass` **e**
//! o `accel` da MESMA cadeia — a de estado —, então o pin cabe dentro do laço e o `in`
//! dele já traz a carga. (Com o `motion.integrate` não cabia: ele lê o `accel` do
//! `state` mas o `inv_mass` do `rest`, e foi isso que fez a v1 desta cena sair com
//! *"tudo foi levado pelo vento, nada rasgou"* — mecanismo no doc do `BREAK_ABOVE`.)
//!
//! ```text
//! soft_body ═pre═► force.wind ──► pin_constraint ──► soft_body.state
//!                                            pin ═pre═► pin.state   (a memória)
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

/// A cortina da linha 1 — um **tecido**: `(colunas, linhas, passo, tamanho da peça)`.
pub(crate) const CURTAIN: (f32, f32, f32, f32) = (7.0, 7.0, 0.22, 0.11);
/// A gravidade e a rigidez do tecido — uma folha que PENDE e ondula, não uma
/// placa: rígida demais ela não flutua, mole demais ela desfaz-se.
const GRAVITY: f32 = 8.0;
const STIFFNESS: f32 = 0.30;
/// Quantas peças o bando da linha 2 tem, e o espalhamento inicial.
pub(crate) const FLOCK: f32 = 40.0;

const LIT: [[f32; 3]; 2] = [[0.55, 0.85, 1.0], [1.0, 0.78, 0.40]];
const LABEL_RGB: [f32; 3] = [0.62, 0.64, 0.70];
/// A cor da PEDRA e dos pinos — branco, para se distinguirem de quem os sofre.
const MARK_RGB: [f32; 3] = [1.0, 1.0, 1.0];

/// A carga acima da qual o pin da direita rasga.
///
/// ⚠️ **Ela é MENOR que a força do vento de propósito** — senão o pin nunca rasgaria e a
/// linha ficaria com as duas metades iguais. O gate `the_tear_threshold_is_below_the_wind`
/// deriva a comparação em vez de a repetir.
/// ⚠️ **O VENTO SOBE COM O TEMPO, e é isso que faz o rasgo ser VISÍVEL.**
///
/// O smoke devolveu *"não rasga"* com um vento constante — e ele estava certo: com a
/// carga acima do limiar desde o primeiro quadro, o pano da direita já nasce a voar.
/// Não se **vê** rasgar; vê-se um painel vazio. ⚠️ A rajada também não serve: o ruído do
/// `force.wind` é **por instância**, então alguma das peças pregadas já nasce perto do
/// pico — MEDIDO, todos os limiares até 5,5 eram cruzados a **0,02 s**.
///
/// A cura é uma carga que CRUZA o limiar com o tempo: `value.time → value.map_range`
/// dirige o `strength` do vento por um fio (`Graph::drive_param`, doc 58). O vento vai
/// de `0` a [`WIND_TOP`] em [`RAMP_SECS`], igual nas duas metades — então o pano pendura,
/// começa a levantar, e a certa altura o da direita **solta-se**.
pub(crate) const BREAK_ABOVE: f32 = 4.6;
/// O topo da rampa do vento e quanto tempo ela leva a lá chegar.
pub(crate) const WIND_TOP: f32 = 9.0;
pub(crate) const RAMP_SECS: f32 = 4.0;
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

/// **LINHA 1 · RASGA** — o TECIDO pregado no topo, contra o vento.
///
/// ⚠️ O `pin` do próprio `motion.soft_body` fica em **0**: quem segura a folha é o
/// `motion.pin_constraint`, porque é ele que sabe rasgar. Com o pin intrínseco ligado a
/// folha ficaria presa de qualquer maneira e o par sairia mudo.
fn tear_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = f32::from(u8::from(right)) * 240.0;
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[0]];
    let (cols, rows, spacing, piece) = CURTAIN;
    let cloth = node(
        g,
        "motion.soft_body",
        &[
            ("rows", rows),
            ("cols", cols),
            ("spacing", spacing),
            ("gravity", GRAVITY),
            ("stiffness", STIFFNESS),
            ("damping", 0.03),
            // ⚠️ O pin INTRÍNSECO desligado — ver o doc acima.
            ("pin", 0.0),
        ],
        ey,
        0.0,
    );
    // O LAÇO de estado do tecido: o vento acumula a carga, o pin lê-a e devolve o
    // `inv_mass` pela mesma cadeia.
    let wind = node(
        g,
        "force.wind",
        &[("angle", WIND_ANGLE), ("gust", 0.0)],
        ey + 150.0,
        320.0,
    );
    // **A RAMPA que dirige a força do vento** — ver [`BREAK_ABOVE`]. O relógio nasce de
    // uma grelha 1×1 própria: alimentá-lo do tecido fecharia um ciclo, e o
    // `drive_param` recusa-o (com razão).
    let clock = node(
        g,
        "motion.grid",
        &[("rows", 1.0), ("cols", 1.0)],
        ey + 300.0,
        0.0,
    );
    let now = push(g, clock, "value.time", &[("rate", 1.0)], ey + 300.0, 130.0);
    let ramp = push(
        g,
        now,
        "value.map_range",
        &[
            ("in_lo", 0.0),
            ("in_hi", RAMP_SECS),
            ("out_lo", 0.0),
            ("out_hi", WIND_TOP),
            // Travado no topo: sem isto o vento cresce para sempre e a cena deixa de
            // ter um estado final para se olhar.
            ("clamp", 1.0),
        ],
        ey + 300.0,
        280.0,
    );
    g.drive_param(wind, "strength", (ramp, 0)).ok()?;
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
        ey + 150.0,
        470.0,
    );
    wire_pre(g, cloth, wind, 0)?;
    wire(g, wind, 0, pin, 0)?;
    wire(g, pin, 0, cloth, 2)?;
    // A MEMÓRIA do rasgo: sem este fio ele cede e volta a pinar (elástico, não rasgo).
    wire_pre(g, pin, pin, 1)?;

    let sized = push(g, cloth, "motion.scale", &[("amount", piece)], ey, 620.0);
    let placed = place(g, sized, at, ey, 760.0);
    let lit = tint(g, placed, LIT[0], ey, 900.0);
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
