//! **OS DOIS EIXOS E O RELÓGIO CURVADO** — a cena `=58`, a folha 06 linhas 39 e 45.
//!
//! Duas metades, e elas **julgam-se de formas diferentes** (o precedente da cena `=55`):
//! o par de cima é de **FORMA** e lê-se PARADO; o par de baixo é de **TEMPO** e só
//! existe com o PLAY.
//!
//! **1-2 — `Size X` ≠ `Size Y`** (linha 39). O `motion.drive` escrevia os dois eixos
//! com o MESMO número; agora há um canal por eixo. ⚠️ **O CONTROLE não é uma fileira
//! parada** — é a mesma fileira com o canal `Size` de sempre, cujas peças são
//! **QUADRADAS por construção**; sem ele, *"as de baixo têm formas diferentes"* seria
//! satisfeito por qualquer coisa que mexesse no tamanho.
//!
//! ⚠️ E os dois campos da banda 2 têm **sementes diferentes**, de propósito: com o
//! mesmo campo nos dois eixos as peças voltariam a ser quadradas, e a cena provaria o
//! contrário do que diz. É a **razão x/y variar de peça para peça** que nenhuma
//! anisotropia fixa reproduz — e anisotropia fixa é exactamente o que a composição
//! `drive(Size) → motion.scale(não-uniforme)` já dava (medido, `measure_size_axes`).
//!
//! **3-4 — o relógio CURVADO** (linha 45). A mesma sub-árvore oscilando, e só a de
//! baixo passa por um `motion.time_remap` em modo **Curve** com uma **PAUSA** desenhada
//! no meio. ⚠️ A leitura é *quando ela para*, não *quão longe ela vai*: as duas têm a
//! mesma amplitude porque o remap reescreve o **relógio**, nunca a amplitude.
//!
//! ⚠️ **Fora da janela o relógio SEGURA**, e isso é a semântica, não um defeito: a
//! curva é lida em `t·scale / duration` clampado a `[0,1]`, como o `ph2d_curve::eval`
//! já segura fora do próprio vão autorado. Depois de `WINDOW_S` a banda 4 congela.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira. Ímpar de propósito: com um número par nenhuma peça cai
/// no meio da fileira, que é onde a pausa do relógio é mais fácil de ver.
const PIECES: f32 = 25.0;
/// O vão horizontal entre peças, em unidades de mundo.
const GAP_X: f32 = 0.55;
/// O vão vertical entre bandas — maior que a maior peça que a banda 2 produz (2,2),
/// senão duas fileiras vizinhas se tocam e o olho perde a fronteira.
const BAND_DY: f32 = 3.2;
/// Quanto o campo acrescenta ao tamanho unitário. Com `Add` sobre a identidade `1`,
/// as peças medem `1,0 .. 2,2` — grandes o bastante para a razão x/y se ver.
const SIZE_GAIN: f32 = 1.2;
/// A janela que a curva do relógio mapeia, em segundos. Ver a nota do cabeçalho: fora
/// dela o relógio segura.
const WINDOW_S: f32 = 6.0;

/// A forma que a banda 4 desenha no relógio: sobe, **PARA** no meio, e volta a subir.
/// ⚠️ O `Interp::Hold` segura o valor DESTE ponto até o próximo — é o par
/// `(0,40 → 0,60)` que é a pausa, e ela dura `0,2 · WINDOW_S = 1,2 s`.
fn paused_clock() -> String {
    ph2d_curve::serialize(&ph2d_curve::Curve {
        points: vec![
            ph2d_curve::Point {
                x: 0.0,
                y: 0.0,
                interp: ph2d_curve::Interp::Smooth,
            },
            ph2d_curve::Point {
                x: 0.40,
                y: 0.42,
                interp: ph2d_curve::Interp::Hold,
            },
            ph2d_curve::Point {
                x: 0.60,
                y: 0.42,
                interp: ph2d_curve::Interp::Smooth,
            },
            ph2d_curve::Point {
                x: 1.0,
                y: 1.0,
                interp: ph2d_curve::Interp::Linear,
            },
        ],
    })
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Uma fileira de peças.
fn row(g: &mut Graph, x: f32, y: f32) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x, y });
    for (k, v) in [("rows", 1.0), ("cols", PIECES), ("gap_x", GAP_X)] {
        g.set_param(grid, k, v);
    }
    grid
}

/// Um campo por-peça em `[0,1]` — `Random`, para os dois eixos da banda 2 não serem
/// múltiplos um do outro.
fn field(g: &mut Graph, src: NodeId, seed: f32, x: f32, y: f32) -> Option<NodeId> {
    let f = g.add_node("value.instance_field");
    g.set_pos(f, Pos { x, y });
    g.set_param(f, "mode", 2.0); // Random
    g.set_param(f, "seed", seed);
    wire(g, src, 0, f, 0)?;
    Some(f)
}

/// Um `motion.drive` que soma o campo ao canal pedido.
fn drive(
    g: &mut Graph,
    src: NodeId,
    value: NodeId,
    channel: f32,
    x: f32,
    y: f32,
) -> Option<NodeId> {
    let d = g.add_node("motion.drive");
    g.set_pos(d, Pos { x, y });
    g.set_param(d, "channel", channel);
    g.set_param(d, "mode", 0.0); // Add, sobre a identidade unitaria do tamanho
    g.set_param(d, "scale", SIZE_GAIN);
    wire(g, src, 0, d, 0)?;
    wire(g, value, 0, d, 1)?;
    Some(d)
}

/// Põe a banda no lugar e a termina num `motion.output`.
///
/// ⚠️ O sink **não** é decoração: o laço de render re-resolve os sinks a cada quadro a
/// partir dos nós de saída do grafo, então uma banda sem `motion.output` cozinha certo,
/// satisfaz os gates e **desenha NADA** (a lição da cena `=48`).
fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 200.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// Monta a cena. Devolve os sinks, um por banda, na ordem dos rótulos.
pub(crate) fn build_axes_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);

    // 1 — CONTROLE: o canal `Size`, que escreve os DOIS eixos com o mesmo numero.
    let src = row(g, 0.0, 0.0);
    let f = field(g, src, 0.0, 220.0, 0.0)?;
    let d = drive(g, src, f, 3.0, 440.0, 0.0)?;
    sinks.push(place(g, d, 1.5 * BAND_DY, 660.0, 0.0)?);

    // 2 — os dois eixos, cada um com o SEU campo.
    let src = row(g, 0.0, 200.0);
    let fx = field(g, src, 0.0, 220.0, 200.0)?;
    let dx = drive(g, src, fx, 10.0, 440.0, 200.0)?; // Size X
    let fy = field(g, src, 7.0, 220.0, 300.0)?;
    let dy = drive(g, dx, fy, 11.0, 660.0, 200.0)?; // Size Y
    sinks.push(place(g, dy, 0.5 * BAND_DY, 880.0, 200.0)?);

    // 3 — CONTROLE: a mesma oscilacao, com o relogio de sempre.
    // 4 — a mesma oscilacao sob um relogio CURVADO com uma pausa no meio.
    for (i, curved) in [false, true].into_iter().enumerate() {
        let gy = 460.0 + i as f32 * 200.0;
        let src = row(g, 0.0, gy);
        let osc = g.add_node("motion.oscillator");
        g.set_pos(osc, Pos { x: 220.0, y: gy });
        g.set_param(osc, "channel", 1.0); // Y — a fileira sobe e desce
        g.set_param(osc, "amplitude", 1.1);
        g.set_param(osc, "frequency", 0.35);
        // ⚠️ **SEM defasagem, de propósito.** Com um `phase_stagger` a fileira vira uma
        // onda a viajar, e a MÉDIA dela sobre um ciclo é **constante** — o oráculo
        // cancelaria exactamente o movimento que ele existe para medir (a sonda
        // mediu −1,6000 em todo instante da janela). Rígida, a fileira sobe e desce
        // como uma barra, e *quando ela para* é a coisa mais fácil de ver na cena.
        g.set_param(osc, "phase_stagger", 0.0);
        wire(g, src, 0, osc, 0)?;

        let head = if curved {
            let tr = g.add_node("motion.time_remap");
            g.set_pos(tr, Pos { x: 440.0, y: gy });
            g.set_param(tr, "mode", 5.0); // Curve
            g.set_param(tr, "duration", WINDOW_S);
            g.set_text_param(tr, "curve", paused_clock());
            wire(g, osc, 0, tr, 0)?;
            tr
        } else {
            osc
        };
        sinks.push(place(g, head, (-0.5 - i as f32) * BAND_DY, 660.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CONTROLE -- drive(Size): um campo, os DOIS eixos -- toda peca e' QUADRADA",
        "DOIS EIXOS -- drive(Size X) + drive(Size Y), campos independentes: RETANGULOS",
        "CONTROLE -- a oscilacao com o relogio de sempre",
        "RELOGIO CURVADO -- time_remap(Curve) com uma PAUSA desenhada no meio",
    ]
    .into_iter()
    .enumerate()
}

/// A janela que a curva mapeia, em segundos — o número que a mensagem cita.
pub(crate) fn window_seconds() -> f32 {
    WINDOW_S
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_axes_tests.rs"]
mod tests;
