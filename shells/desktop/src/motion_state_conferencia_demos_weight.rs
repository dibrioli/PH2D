//! **O PESO POR PARTÍCULA E OS SUB-PASSOS** — a cena `=52`, o grupo K da
//! conferência (folha 03, as linhas do `soft_body` e da `verlet_rope`).
//!
//! ⚠️ **São duas waves independentes, e a cena as separa em dois pares.** O que
//! elas partilham não é o mecanismo, é a pergunta: *o simulador está a honrar um
//! número que o artista autorou, ou está a fingir que ele não existe?*
//!
//! **(1) O PESO.** O `motion.soft_body` é shape matching de Müller et al. 2005,
//! e o paper carrega um peso `wᵢ` por partícula nos centroides e nas duas
//! matrizes — o nosso doc-comment **declarava** que os tinha fixado em 1. O
//! `falloff` é esse peso, e ele é UMA ideia com DUAS consequências: pertencer
//! significa **definir** a forma e **ser puxado** de volta a ela. Escrever só a
//! segunda deixaria a cauda solta a arrastar o referencial do corpo inteiro.
//!
//! **(2) OS SUB-PASSOS.** Um solver de posição relaxa em direção às restrições
//! do estado em que o passo começou, então iterar mais converge para *a resposta
//! daquele passo* — e é a resposta daquele passo que está errada quando o passo
//! é grande. Um sub-passo **volta a perguntar onde a corda está** (Macklin et
//! al., *Small Steps in Physics Simulation*, SCA 2019).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 9.0;
/// A malha do corpo mole (6×6 = 36 partículas, seis LINHAS).
const BODY_SIDE: f32 = 6.0;
/// A fracção da lista que fica com peso CHEIO na banda do peso. `4/6` deixa as
/// **duas últimas linhas** a zero — a cauda que se solta.
const KEPT: f32 = 4.0 / BODY_SIDE;
/// Nós da corda.
const ROPE_COUNT: f32 = 24.0;
/// Sub-passos da banda 4. ⚠️ **8 e não 24:** o teto é 16 e o joelho da tabela
/// medida está aqui (`1 → 8,30%`, `8 → 1,17%`), então esta é a escolha que um
/// artista de facto faria, não o extremo que lisonjeia a feature.
const SUBSTEPS: f32 = 8.0;

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
/// ⚠️ **O laço de render RE-RESOLVE os sinks a cada quadro** a partir dos nós de
/// saída do grafo, então uma cadeia sem Output cozinha certo, satisfaz os gates e
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

/// O CORPO MOLE pendurado pela linha de topo. `banded` põe um `field.index_range`
/// no laço de estado, dando peso **zero** às duas últimas linhas.
///
/// ⚠️ **O campo entra pela cadeia de ESTADO, e o `pre` mora na aresta que ENTRA
/// nele** — é ela que quebra o ciclo, e o `field.index_range` é `Effect::Pure`,
/// logo não carimba `sim_t` e o solver ainda vê o `dt` do próprio relógio.
fn body(g: &mut Graph, banded: bool, x: f32, y: f32) -> Option<NodeId> {
    let b = g.add_node("motion.soft_body");
    g.set_pos(b, Pos { x, y });
    g.set_param(b, "rows", BODY_SIDE);
    g.set_param(b, "cols", BODY_SIDE);
    g.set_param(b, "spacing", 0.7);
    g.set_param(b, "gravity", 9.8);
    g.set_param(b, "pin", 1.0);
    g.set_param(b, "stiffness", 0.55);

    if banded {
        let f = g.add_node("field.index_range");
        g.set_pos(
            f,
            Pos {
                x: x + 220.0,
                y: y + 90.0,
            },
        );
        g.set_param(f, "start", 0.0);
        g.set_param(f, "end", KEPT);
        g.set_param(f, "soft", 0.0);
        wire_pre(g, b, 0, f, 0)?;
        wire(g, f, 0, b, 2)?;
    } else {
        wire_pre(g, b, 0, b, 2)?;
    }
    Some(b)
}

/// A CORDA chicoteada por uma `value.lfo` na âncora — uma corda PENDURADA nunca
/// estica (ela assenta no comprimento de repouso e fica lá), então sem o chicote
/// as duas bandas sairiam idênticas e a cena diria *"funciona"* sobre um param
/// inerte.
fn rope(g: &mut Graph, substeps: bool, x: f32, y: f32) -> Option<NodeId> {
    let r = g.add_node("motion.verlet_rope");
    g.set_pos(r, Pos { x, y });
    g.set_param(r, "count", ROPE_COUNT);
    g.set_param(r, "length", 6.0);
    g.set_param(r, "gravity", 60.0);
    // ⚠️ **Poucas iterações de propósito.** Com o solver já bem convergido as
    // duas bandas empatam e a cena não separa nada; o regime em que a diferença
    // VIVE é o do passo grande, que é onde o artista a encontra.
    g.set_param(r, "iterations", 3.0);
    g.set_param(r, "substeps", if substeps { SUBSTEPS } else { 1.0 });

    let whip = g.add_node("value.lfo");
    g.set_pos(whip, Pos { x: x - 220.0, y });
    g.set_param(whip, "period", 0.7);
    g.set_param(whip, "amplitude", 3.5);
    wire(g, whip, 0, r, 0)?;

    wire_pre(g, r, 0, r, 2)?;
    Some(r)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_weight_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);

    for (row, banded) in [false, true].into_iter().enumerate() {
        let gy = row as f32 * 300.0;
        let head = body(g, banded, 0.0, gy)?;
        sinks.push(place(
            g,
            head,
            -8.0,
            BAND_DY - row as f32 * BAND_DY,
            440.0,
            gy,
        )?);
    }
    for (row, subs) in [false, true].into_iter().enumerate() {
        let gy = (row + 2) as f32 * 300.0;
        let head = rope(g, subs, 0.0, gy)?;
        sinks.push(place(
            g,
            head,
            8.0,
            -BAND_DY - row as f32 * BAND_DY,
            440.0,
            gy,
        )?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CORPO, controle -- as seis linhas pertencem ao corpo",
        "CORPO, as duas ultimas linhas com peso ZERO -- a cauda desprende",
        "CORDA, substeps 1 -- ela ESTICA no chicote",
        "CORDA, substeps 8 -- ela mantem o comprimento",
    ]
    .into_iter()
    .enumerate()
}

/// Quantos sub-passos a banda 4 pede — para a mensagem do smoke citar o número
/// que a cena de facto autora, em vez de uma segunda cópia dele.
pub(crate) fn substeps() -> f32 {
    SUBSTEPS
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_weight_tests.rs"]
mod tests;
