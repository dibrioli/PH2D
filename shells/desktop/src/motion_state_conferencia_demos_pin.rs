//! **O PINO ALCANÇA AS SIMULAÇÕES** — a cena `=50` (doc 89, o grupo J).
//!
//! Três pares, um por gerador, e em cada par só uma coisa difere: um
//! `motion.pin_constraint` na **cadeia de estado**.
//!
//! ⚠️ **A cerca que esta cena derruba estava escrita em DOIS docs.** O do
//! `motion.pin_constraint` dizia que os três geradores *"mintam os próprios pontos
//! e nenhum stream de instâncias entra, então um pino a montante não tem fio por
//! onde os alcançar"*, e o doc 34 §7 repetia-a como *"hoje não há como um stream
//! entrar neles"*. As duas eram verdade sobre a porta `in` e **falsas sobre a
//! cadeia de estado**, que é um fio — e que já era o fio pelo qual o `accel` entra.
//!
//! ⚠️ **A ÂNCORA CHICOTEADA não é decoração: é a fixture.** Uma corda pendurada
//! nunca se dobra (o nó tem uma âncora só) e um pino no meio dela seria
//! indistinguível de um nó de passagem; com a cabeça a varrer, a corda que carrega
//! o pino **DOBRA sobre ele** e a que não carrega passa direito. É a mesma lição
//! que o `rope_thickness` pagou duas vezes.
//!
//! ⚠️ **A cena julga-se com PLAY** — as três famílias são `Effect::Temporal` e uma
//! foto de um instante não distingue *segurou* de *ainda não caiu*.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantos pares a cena monta (um por gerador).
pub(crate) const PAIRS: usize = 3;
/// O vão horizontal entre o CONTROLE e o pinado do mesmo par.
const PAIR_DX: f32 = 9.0;
/// O vão vertical entre pares.
const PAIR_DY: f32 = 9.0;

/// A corda: 24 pontos, 6 unidades, com a cabeça varrida por uma `value.lfo`.
const ROPE_COUNT: f32 = 24.0;
/// O ponto que o pino prega — no MEIO, onde nem a cabeça nem a cauda o alcançam:
/// é o *índice arbitrário* que a folha 03 (linha 51) pedia e que nenhum param
/// deste nó exprime.
const ROPE_PIN: f32 = 12.0;
/// A malha do corpo mole (6×6 = 36 partículas).
const BODY_SIDE: f32 = 6.0;
/// Quantas partículas o pino segura — a primeira LINHA da malha, para o corpo
/// pender dela em vez de ficar preso por um canto.
const BODY_PIN: f32 = 6.0;
/// Agentes do bando.
const FLOCK_COUNT: f32 = 40.0;
/// Quantos agentes viram obstáculo parado.
const FLOCK_PIN: f32 = 3.0;

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

/// Fecha o laço de estado do gerador — com o pino no meio, ou nu.
///
/// ⚠️ **O `pre` mora na aresta que ENTRA no pino**, não na que sai dele: é ela que
/// quebra o ciclo. O `motion.pin_constraint` é `Effect::Pure` ⇒ não carimba
/// `sim_t`, então o solver ainda vê o `dt` do próprio relógio no tique seguinte.
///
/// ⚠️ E o `pre` sai do GERADOR, não do `motion.move` que posiciona a banda: o laço
/// tem de carregar o estado do solver, e não a cópia deslocada dele.
fn close_loop(g: &mut Graph, sim: NodeId, pin: Option<(f32, f32)>, x: f32, y: f32) -> Option<()> {
    match pin {
        None => wire_pre(g, sim, 0, sim, 2),
        Some((first, count)) => {
            let p = g.add_node("motion.pin_constraint");
            g.set_pos(p, Pos { x, y });
            g.set_param(p, "first", first);
            g.set_param(p, "count", count);
            g.set_param(p, "strength", 1.0);
            wire_pre(g, sim, 0, p, 0)?;
            wire(g, p, 0, sim, 2)
        }
    }
}

/// Termina a banda num `motion.output`, deslocada para o seu lugar na grade.
///
/// ⚠️ **O laço de render RE-RESOLVE os sinks a cada quadro** a partir dos nós de
/// saída do grafo, então uma cadeia sem Output cozinha certo, passa na suíte e
/// **desenha NADA na tela**.
fn place(g: &mut Graph, head: NodeId, dx: f32, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 220.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// Uma corda chicoteada — a fixture que faz um pino do MEIO ser visível.
fn rope(g: &mut Graph, pinned: bool, x: f32, y: f32) -> Option<NodeId> {
    let r = g.add_node("motion.verlet_rope");
    g.set_pos(r, Pos { x, y });
    g.set_param(r, "count", ROPE_COUNT);
    g.set_param(r, "length", 6.0);
    g.set_param(r, "gravity", 9.8);
    g.set_param(r, "iterations", 16.0);

    let lfo = g.add_node("value.lfo");
    g.set_pos(lfo, Pos { x: x - 220.0, y });
    g.set_param(lfo, "period", 1.6);
    g.set_param(lfo, "amplitude", 3.0);
    wire(g, lfo, 0, r, 0)?;

    close_loop(g, r, pinned.then_some((ROPE_PIN, 1.0)), x + 220.0, y + 90.0)?;
    Some(r)
}

fn body(g: &mut Graph, pinned: bool, x: f32, y: f32) -> Option<NodeId> {
    let b = g.add_node("motion.soft_body");
    g.set_pos(b, Pos { x, y });
    g.set_param(b, "rows", BODY_SIDE);
    g.set_param(b, "cols", BODY_SIDE);
    g.set_param(b, "spacing", 0.7);
    g.set_param(b, "gravity", 9.8);
    // ⚠️ O pino INTRÍNSECO fica DESLIGADO nos dois lados do par: com ele ligado a
    // linha de topo já segura, e o par mediria a diferença entre dois corpos
    // pendurados — o pino genérico ficaria invisível sobre um corpo que nunca
    // precisou dele.
    g.set_param(b, "pin", 0.0);
    close_loop(g, b, pinned.then_some((0.0, BODY_PIN)), x + 220.0, y + 90.0)?;
    Some(b)
}

fn flock(g: &mut Graph, pinned: bool, x: f32, y: f32) -> Option<NodeId> {
    let f = g.add_node("motion.boids");
    g.set_pos(f, Pos { x, y });
    g.set_param(f, "count", FLOCK_COUNT);
    g.set_param(f, "seed", 7.0);
    g.set_param(f, "separation", 2.4);
    close_loop(
        g,
        f,
        pinned.then_some((0.0, FLOCK_PIN)),
        x + 220.0,
        y + 90.0,
    )?;
    Some(f)
}

/// Monta a cena. Devolve os sinks, um por banda (dois por par).
pub(crate) fn build_pin_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(PAIRS * 2);

    for (row, kind) in [0usize, 1, 2].into_iter().zip([0u8, 1, 2]) {
        let gy = row as f32 * 300.0;
        let dy = -(row as f32) * PAIR_DY + PAIR_DY;
        for (col, pinned) in [false, true].into_iter().enumerate() {
            let gx = col as f32 * 700.0;
            let dx = col as f32 * PAIR_DX - PAIR_DX * 0.5;
            let head = match kind {
                0 => rope(g, pinned, gx, gy)?,
                1 => body(g, pinned, gx, gy)?,
                _ => flock(g, pinned, gx, gy)?,
            };
            sinks.push(place(g, head, dx, dy, gx + 440.0, gy)?);
        }
    }
    Some(sinks)
}

/// Os rótulos das seis bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CORDA, controle -- a cabeca varre e a corda segue inteira",
        "CORDA, pino no ponto 12 -- ela DOBRA sobre um ponto que nao e' ponta",
        "CORPO MOLE, controle -- cai livre, sem pino intrinseco",
        "CORPO MOLE, primeira linha pinada -- o corpo PENDE dela",
        "BANDO, controle -- 40 agentes livres",
        "BANDO, 3 agentes pinados -- obstaculos parados que o bando contorna",
    ]
    .into_iter()
    .enumerate()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_pin_tests.rs"]
mod tests;
