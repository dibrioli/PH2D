//! **O TREMOR GANHA TEXTURA (e uma volta)** — a cena `=54`, o grupo N da
//! conferência (folha 06, as linhas do `motion.wiggle`).
//!
//! ⚠️ **A pergunta desta cena é a FORMA do tremor, não o tamanho dele.** As
//! quatro fileiras têm a MESMA amplitude e a MESMA frequência de propósito — se
//! diferissem em amplitude, *"a de baixo mexe mais"* responderia por qualquer
//! coisa que a wave tivesse feito.
//!
//! **Dois pares, cada um com o seu CONTROLE ao lado:**
//! - **1-2** as OITAVAS: a de cima ondula, a de baixo ondula **e treme** — a
//!   mesma onda com detalhe empilhado em cima (`wiggle(freq, amp, octaves,
//!   amp_mult)`, a assinatura do After Effects).
//! - **3-4** o LAÇO: a de baixo **repete** a cada 2 s; a de cima não repete
//!   nunca. ⚠️ Este par **só se julga com PLAY, e olhando por uns segundos** —
//!   uma foto não distingue um campo que fecha de um que não fecha.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 7.0;
/// Peças por fileira.
const COUNT: f32 = 24.0;
/// A amplitude do tremor — a MESMA nas quatro.
const AMPLITUDE: f32 = 2.2;
/// A frequência do tremor — a MESMA nas quatro.
const FREQUENCY: f32 = 0.9;
/// Quantas oitavas a banda 2 empilha.
const OCTAVES: f32 = 5.0;
/// O peso de cada oitava seguinte (o `amp_mult` do AE).
const AMP_MULT: f32 = 0.65;
/// O período do laço da banda 4, em segundos.
const LOOP_LEN: f32 = 2.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Termina a banda num `motion.output`, deslocada para o seu lugar.
///
/// ⚠️ O laço de render **RE-RESOLVE** os sinks a cada quadro a partir dos nós de
/// saída do grafo, então uma cadeia sem Output cozinha certo, satisfaz os gates
/// e **desenha NADA na tela**.
fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 220.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// Uma fileira cujo Y é tremido por um `motion.wiggle`, com as oitavas e o laço
/// que a banda pedir.
fn row(g: &mut Graph, octaves: f32, loop_len: f32, x: f32, y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x, y });
    g.set_param(grid, "cols", COUNT);
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "spacing", 0.5);

    let wig = g.add_node("motion.wiggle");
    g.set_pos(wig, Pos { x: x + 220.0, y });
    g.set_param(wig, "channel", 1.0); // Y
    g.set_param(wig, "amplitude", AMPLITUDE);
    g.set_param(wig, "frequency", FREQUENCY);
    g.set_param(wig, "octaves", octaves);
    g.set_param(wig, "amp_mult", AMP_MULT);
    g.set_param(wig, "loop_len", loop_len);
    wire(g, grid, 0, wig, 0)?;
    Some(wig)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_octave_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    // ⚠️ Os dois CONTROLES são idênticos de propósito: cada par responde a uma
    // pergunta diferente, e comparar uma banda com o controle do outro par
    // mediria as duas mudanças de uma vez.
    let bands = [(1.0, 0.0), (OCTAVES, 0.0), (1.0, 0.0), (1.0, LOOP_LEN)];
    for (row_i, (oct, loop_len)) in bands.into_iter().enumerate() {
        let gy = row_i as f32 * 300.0;
        let head = row(g, oct, loop_len, 0.0, gy)?;
        sinks.push(place(g, head, (1.5 - row_i as f32) * BAND_DY, 440.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "OITAVAS, controle -- uma onda lenta e lisa",
        "OITAVAS = 5 -- a MESMA onda, agora com tremor em cima",
        "LACO, controle -- o tremor nunca se repete",
        "LACO = 2s -- o MESMO tremor volta ao comeco a cada 2 segundos",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a cena autora — para a mensagem do smoke citar o que ela de
/// facto escreve, em vez de uma segunda cópia deles.
pub(crate) fn knobs() -> (f32, f32, f32) {
    (OCTAVES, AMP_MULT, LOOP_LEN)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_octave_tests.rs"]
mod tests;
