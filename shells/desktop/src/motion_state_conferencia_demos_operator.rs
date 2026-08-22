//! **DOIS EXEMPLOS, UM POR LINHA** — a cena `=77` (doc 89, folha 07: o *Echo Operator* do
//! rastro e o *Strobe Operator* do flash, que a folha dizia serem **um conserto só**).
//!
//! | linha | esquerda | direita |
//! |---|---|---|
//! | **RASTRO** | a cauda TAPA o que está atrás | a cauda **SOMA** — onde os ecos se cruzam, acende |
//! | **FLASH** | o flash tapa | o flash **SOMA** — o pico estoura de branco |
//!
//! ⚠️ **SÓ SE JULGA COM O PLAY.** As duas linhas são temporais: um rastro é o passado dos
//! ticks anteriores, e um flash é um envelope no tempo. Paradas, as duas metades são iguais.
//!
//! ⚠️ **As duas metades TÊM de se cruzar consigo mesmas**, senão a soma não tem o que somar:
//! por isso o caminho é um LAÇO (uma lemniscata), e não uma linha reta. Um rastro que nunca
//! se atravessa compõe igual em qualquer operador — a cena provaria nada.
//!
//! ⚠️ **A colocação corre logo a seguir à fonte** (a lei que a `=73` pagou).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna, e de cada linha.
const COL_X: f32 = 3.6;
const ROW_Y: [f32; 2] = [2.6, -2.9];
const HEADER_Y: f32 = 5.6;
const LABEL_SIZE: f32 = 0.42;
const ROW_LABELS: [&str; 2] = ["RASTRO", "FLASH"];
const LABEL_RGB: [f32; 3] = [0.62, 0.64, 0.70];

/// **`Add` no dropdown dos dois nós** — `Sink`(0) · `Normal`(1) · **`Add`(2)**.
pub(crate) const ADD: f32 = 2.0;

/// O tamanho da peça e a cor dela: um azul MÉDIO, de propósito — com `Add`, dois médios
/// somam para um claro, e é isso que se tem de ver. Um branco já saturado não mostraria nada.
const PIECE: f32 = 0.34;
const INK: [f32; 3] = [0.22, 0.42, 0.85];

/// O laço que a peça percorre: amplitude e período. ⚠️ **Uma LEMNISCATA** (o `y` no dobro da
/// frequência do `x`), que é o caminho mais barato que se cruza a si próprio.
const LOOP_R: f32 = 1.5;
const LOOP_HZ: f32 = 0.35;

/// A frequência do laço, para o gate que mede se ele se cruza. ⚠️ **Derivada, nunca
/// re-escrita**: um gate com o número próprio ficaria verde sobre a cena errada.
#[cfg(test)]
pub(crate) const LOOP_HZ_FOR_TEST: f32 = LOOP_HZ;

/// Quantos ecos, e de quanto em quanto tique. Longo o bastante para a cauda dar meia volta
/// e encontrar-se.
pub(crate) const TRAIL_LEN: f32 = 26.0;
const TRAIL_SPACING: f32 = 2.0;

/// O flash: de quanto em quanto tique ele dispara, e quanto dura a queda.
const BEAT_PERIOD: f32 = 0.5;
const FLASH_DECAY: f32 = 26.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// O fio de RETORNO (`out --pre--> state`) — o laço que o artista não desenha.
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

fn out_of(g: &mut Graph, tail: NodeId, ey: f32, x: f32) -> Option<NodeId> {
    let out = node(g, "motion.output", &[], ey, x);
    wire(g, tail, 0, out, 0)?;
    Some(out)
}

/// **A PEÇA QUE PERCORRE O LAÇO** — um ponto só, já colocado e já pintado.
///
/// ⚠️ **A lemniscata é construída com DOIS `motion.oscillator`** (x a `f` e y a `2f`), e não com um
/// `motion.orbit`: uma órbita é um círculo e **não se cruza**, então a cauda dela nunca
/// encontraria a si própria e o operador não teria o que somar.
fn piece(g: &mut Graph, k: usize, right: bool, ey: f32) -> NodeId {
    let one = node(g, "motion.grid", &[("rows", 1.0), ("cols", 1.0)], ey, 0.0);
    let scaled = push(g, one, "motion.scale", &[("amount", PIECE)], ey, 110.0);
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[k]];
    let placed = place(g, scaled, at, ey, 220.0);
    let wx = push(
        g,
        placed,
        "motion.oscillator",
        &[
            ("channel", 0.0), // X
            ("amplitude", LOOP_R),
            ("frequency", LOOP_HZ),
        ],
        ey,
        340.0,
    );
    let wy = push(
        g,
        wx,
        "motion.oscillator",
        &[
            ("channel", 1.0), // Y — o DOBRO da frequência: é isto que fecha o oito.
            ("amplitude", LOOP_R * 0.62),
            ("frequency", LOOP_HZ * 2.0),
        ],
        ey,
        460.0,
    );
    push(
        g,
        wy,
        "motion.tint",
        &[("r", INK[0]), ("g", INK[1]), ("b", INK[2])],
        ey,
        580.0,
    )
}

/// A linha do RASTRO: a mesma cauda, e à direita ela SOMA.
fn trail_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = usize::from(right) as f32 * 240.0;
    let head = piece(g, 0, right, ey);
    let trail = node(
        g,
        "motion.trail",
        &[
            ("length", TRAIL_LEN),
            ("spacing", TRAIL_SPACING),
            // A cauda quase não desbota: com `Add`, o que se quer ver é a SOMA, e uma
            // cauda apagada não soma nada.
            ("fade", 0.55),
            ("shrink", 0.9),
            (
                ph2d_node_motion_trail::ECHO_BLEND,
                if right { ADD } else { 0.0 },
            ),
        ],
        ey,
        700.0,
    );
    wire(g, head, 0, trail, 0)?;
    wire_pre(g, trail, trail, 1)?;
    out_of(g, trail, ey, 860.0)
}

/// A linha do FLASH: o mesmo strobe, e à direita ele SOMA.
fn flash_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = 520.0 + usize::from(right) as f32 * 240.0;
    let head = piece(g, 1, right, ey);
    let beat = node(
        g,
        "pulse.beat",
        &[("period", BEAT_PERIOD)],
        ey + 110.0,
        560.0,
    );
    let strobe = node(
        g,
        "motion.strobe",
        &[
            ("decay", FLASH_DECAY),
            ("size_boost", 1.6),
            // Um flash BRANCO — com `Add` ele estoura, com `Normal` ele só tapa.
            ("flash_r", 1.0),
            ("flash_g", 1.0),
            ("flash_b", 1.0),
            ("flash_amount", 1.0),
            (
                ph2d_node_motion_strobe::FLASH_BLEND,
                if right { ADD } else { 0.0 },
            ),
        ],
        ey,
        700.0,
    );
    wire(g, head, 0, strobe, 0)?;
    wire(g, beat, 0, strobe, 1)?;
    wire_pre(g, strobe, strobe, 2)?;
    out_of(g, strobe, ey, 860.0)
}

/// Uma palavra no canvas.
fn label(g: &mut Graph, word: &str, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let t = g.add_node("source.text");
    g.set_pos(t, Pos { x: 0.0, y: ey });
    g.set_text_param(t, ph2d_node_source_text::TEXT_KEY, word);
    g.set_param(t, ph2d_node_source_text::param::SIZE, LABEL_SIZE);
    g.set_param(t, ph2d_node_source_text::param::ALIGN, 1.0);
    let placed = place(g, t, at, ey, 200.0);
    let painted = push(
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
    out_of(g, painted, ey, 460.0)
}

/// A cena `=77`, montada de uma vez.
pub(crate) fn build_operator_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    for right in [false, true] {
        sinks.push(trail_band(g, right)?);
    }
    for right in [false, true] {
        sinks.push(flash_band(g, right)?);
    }
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
    (TRAIL_LEN, BEAT_PERIOD)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_operator_tests.rs"]
mod tests;
