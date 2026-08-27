//! **O TETO DA TAXA** — a cena `=53`, o grupo L da conferência (folha 07, a
//! linha dos modos + clamp do `motion.spring`, respondida no `motion.delay`).
//!
//! ⚠️ **A pergunta desta cena é *quão DEPRESSA*, não *para ONDE*.** As quatro
//! fileiras seguem o MESMO alvo oscilante e diferem só no que a saída tem
//! licença para fazer entre dois tiques — e por isso ela **só se julga em
//! movimento**: uma foto de um instante mostra quatro fileiras a alturas
//! diferentes e não diz porquê.
//!
//! **Dois pares, cada um com o seu CONTROLE ao lado:**
//! - **1-2** o teto de PASSO: a de baixo não alcança os picos, e a trajetória
//!   dela deixa de ser uma senoide para virar um vaivém de lados retos.
//! - **3-4** o teto de ACELERAÇÃO: ela ARRANCA devagar e depois acompanha —
//!   o oposto de ir devagar o tempo todo.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 7.0;
/// Peças por fileira.
const COUNT: f32 = 24.0;
/// O período do vaivém, em segundos. Curto de propósito: um alvo lento não
/// pede taxa nenhuma, e as quatro fileiras sairiam iguais.
const PERIOD: f32 = 1.2;
/// A amplitude do alvo, em unidades de mundo.
const AMPLITUDE: f32 = 5.0;
/// O teto de passo da banda 2, em unidades por tique.
const MAX_STEP: f32 = 0.08;
/// O teto de aceleração da banda 4, em unidades por tique².
const MAX_ACCEL: f32 = 0.01;
/// A régua do lag, a mesma nas quatro.
const TICKS: f32 = 4.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn wire_pre(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: true,
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

/// Uma fileira cujo Y inteiro é dirigido por um vaivém, atrasada pelo
/// `motion.delay` — com os dois tetos que a banda pedir.
fn row(g: &mut Graph, max_step: f32, max_accel: f32, x: f32, y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x, y });
    g.set_param(grid, "cols", COUNT);
    g.set_param(grid, "rows", 1.0);
    // ⚠️ Era `spacing`, que o `motion.grid` NAO declara — um `set_param` NO-OP
    // silencioso, e a grade shipava com o `gap` de omissao (1,0), DOBRO do autorado.
    // Achado pela `validate` no varredor das 107 (auditoria de 2026-08-27).
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);

    let lfo = g.add_node("value.lfo");
    g.set_pos(lfo, Pos { x, y: y + 110.0 });
    g.set_param(lfo, "period", PERIOD);
    g.set_param(lfo, "amplitude", AMPLITUDE);

    let drive = g.add_node("motion.drive");
    g.set_pos(drive, Pos { x: x + 220.0, y });
    g.set_param(drive, "channel", 1.0); // Y
    wire(g, grid, 0, drive, 0)?;
    wire(g, lfo, 0, drive, 1)?;

    let dly = g.add_node("motion.delay");
    g.set_pos(dly, Pos { x: x + 440.0, y });
    g.set_param(dly, "mode", 2.0); // Blend
    g.set_param(dly, "channel", 1.0); // Y
    g.set_param(dly, "ticks", TICKS);
    g.set_param(dly, "max_step", max_step);
    g.set_param(dly, "max_accel", max_accel);
    wire(g, drive, 0, dly, 0)?;
    wire_pre(g, dly, 0, dly, 1)?;
    Some(dly)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_rate_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    // ⚠️ Os dois CONTROLES são idênticos de propósito: cada par responde a uma
    // pergunta diferente, e comparar uma banda com o controle do outro par
    // mediria as duas mudanças de uma vez.
    let bands = [(0.0, 0.0), (MAX_STEP, 0.0), (0.0, 0.0), (0.0, MAX_ACCEL)];
    for (row_i, (step, accel)) in bands.into_iter().enumerate() {
        let gy = row_i as f32 * 300.0;
        let head = row(g, step, accel, 0.0, gy)?;
        sinks.push(place(g, head, (1.5 - row_i as f32) * BAND_DY, 660.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "TAXA, controle -- a fileira acompanha o vaivem",
        "TAXA, Max Step -- ela nao alcanca os picos, e o caminho fica RETO",
        "ARRANQUE, controle -- ela parte de uma vez",
        "ARRANQUE, Max Accel -- ela parte DEVAGAR e depois acompanha",
    ]
    .into_iter()
    .enumerate()
}

/// Os dois tetos que a cena autora — para a mensagem do smoke citar o número
/// que ela de facto escreve, em vez de uma segunda cópia dele.
pub(crate) fn ceilings() -> (f32, f32) {
    (MAX_STEP, MAX_ACCEL)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_rate_tests.rs"]
mod tests;
