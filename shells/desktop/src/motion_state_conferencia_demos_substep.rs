//! **O SUB-PASSO DO INTEGRADOR** — a cena `=61` (doc 89, folha 17, a linha 76).
//!
//! A simulação anda em **passos**, um por quadro. Quando a força é forte, um passo grande erra por
//! muito — e o erro de Euler explícito não é ruído: ele **acrescenta energia**, então o corpo sobe
//! cada vez mais alto em vez de repetir a mesma ida e volta. Um sub-passo corta o passo em `n`, e
//! como o erro cai pela metade a cada dobra, oito sub-passos são um passo dezasseis vezes menor
//! pelo preço de oito.
//!
//! ⚠️ **Esta cena tem UMA banda de propósito, e a razão é o desenho, não a preguiça.** O ritmo é
//! do **grafo**: `substep_islands` corre todas as ilhas no maior ritmo que qualquer declarante
//! pede, porque um grafo é o contêiner (a DOP Network do Houdini, o System do Niagara) e o
//! sequenciador de device carrega UM playhead por chamada. Dois integradores lado a lado a pedir
//! `1` e `16` correriam **os dois a 16** — a cena diria que o param é inerte. O que separa as duas
//! respostas aqui é o SLIDER, e é por isso que a prosa manda mexer nele.
//!
//! ⚠️ **É a diferença com a cena `=52`**, que mostra `1` contra `8` lado a lado: o `substeps` da
//! `motion.verlet_rope` é um laço dentro do `eval` DELA (chave `solver_substeps`), não o relógio.
//!
//! O ANEL é o oráculo: o corpo parte de cima dele e, sem ganho de energia, teria de voltar
//! exactamente ali a cada meia volta.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O raio de partida, e portanto o raio do anel-alvo.
const START_R: f32 = 4.0;
/// A força do atrator. **MEDIDA** (`measure_integrate_substeps::where_one_substep_breaks_and_
/// eight_hold`, raio máximo em 3 s partindo de 4,0):
///
/// | força | sub=1 | sub=2 | sub=4 | sub=8 | sub=16 | sub=32 |
/// |---|---|---|---|---|---|---|
/// | 800 | 4,28 | 5,37 | 4,77 | 4,61 | 4,03 | 4,00 |
/// | **3.200** | **12,86** | 4,28 | 6,08 | 5,66 | **4,65** | 4,06 |
/// | 12.800 | 320,99 | 12,86 | 12,23 | 8,49 | 4,93 | 4,72 |
///
/// ⚠️ **3.200 e não 12.800, e o motivo é que o extremo mente nos dois sentidos:** a 12.800 o corpo
/// a `1` sai do enquadramento (raio 321 — o artista vê *nada*, que é ambíguo com um bug), e a
/// 16 ele ainda erra 23%. A 800 o par não separa. A 3.200 o `1` vai a **três vezes** o anel e o
/// topo da faixa confortável volta a **16% dele** — as duas leituras cabem na tela ao mesmo tempo.
const STRENGTH: f32 = 3200.0;
/// O raio de influência do atrator — bem maior que a excursão, para o corpo nunca sair do campo
/// (senão a cena mediria a BORDA do campo em vez do passo).
const ATTRACT_R: f32 = 40.0;
/// Quantos pontos desenham o anel-alvo.
const RING_POINTS: f32 = 60.0;
/// Os sub-passos com que a cena NASCE — o topo da faixa confortável do slider.
const SUBSTEPS: f32 = 16.0;
/// O tipo do nó que a cena substepa — **um literal, não dois**: o construtor monta por ele e o
/// gate procura por ele, então uma renomeação parte os dois lados ao mesmo tempo em vez de deixar
/// o teste a medir um grafo que já não é o da cena.
const INTEGRATOR: &str = "motion.integrate";

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

/// O ANEL-ALVO: onde o corpo teria de voltar a cada meia volta, se o passo fosse exacto.
fn ring(g: &mut Graph, x: f32, y: f32) -> Option<NodeId> {
    let r = g.add_node("motion.distribute_radial");
    g.set_pos(r, Pos { x, y });
    g.set_param(r, "count", RING_POINTS);
    g.set_param(r, "rings", 1.0);
    g.set_param(r, "radius", START_R);
    // `inner = 1` colapsa a faixa de raios num anel só — sem isto saem três anéis concêntricos e
    // o alvo deixa de ser um alvo.
    g.set_param(r, "inner", 1.0);

    let t = g.add_node("motion.tint");
    g.set_pos(t, Pos { x: x + 220.0, y });
    g.set_param(t, "r", 0.35);
    g.set_param(t, "g", 0.35);
    g.set_param(t, "b", 0.40);
    wire(g, r, 0, t, 0)?;
    Some(t)
}

/// O CORPO: um ponto solto no anel, puxado para o centro, integrado pelo `motion.integrate`.
///
/// ⚠️ **O laço é `integrate =pre=> attractor =fwd=> integrate.forces`** — a força vive no cone do
/// integrador, e é por isso que cada sub-passada volta a PERGUNTAR quanta força há em vez de
/// reutilizar a do começo do quadro.
fn body(g: &mut Graph, substeps: f32, x: f32, y: f32) -> Option<NodeId> {
    let seed = g.add_node("motion.grid");
    g.set_pos(seed, Pos { x, y });
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);

    let start = g.add_node("motion.move");
    g.set_pos(start, Pos { x: x + 220.0, y });
    g.set_param(start, "dx", START_R);
    wire(g, seed, 0, start, 0)?;

    let integ = g.add_node(INTEGRATOR);
    g.set_pos(integ, Pos { x: x + 440.0, y });
    g.set_param(integ, "substeps", substeps);

    let att = g.add_node("force.attractor");
    g.set_pos(
        att,
        Pos {
            x: x + 440.0,
            y: y + 140.0,
        },
    );
    g.set_param(att, "target_x", 0.0);
    g.set_param(att, "target_y", 0.0);
    g.set_param(att, "strength", STRENGTH);
    g.set_param(att, "radius", ATTRACT_R);

    wire(g, start, 0, integ, 0)?;
    wire_pre(g, integ, 0, att, 0)?;
    wire(g, att, 0, integ, 1)?;

    let t = g.add_node("motion.tint");
    g.set_pos(t, Pos { x: x + 660.0, y });
    g.set_param(t, "r", 1.0);
    g.set_param(t, "g", 0.55);
    g.set_param(t, "b", 0.15);
    wire(g, integ, 0, t, 0)?;
    Some(t)
}

/// Monta a cena: o anel e o corpo no MESMO sítio, porque a leitura é a distância entre eles.
pub(crate) fn build_substep_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let target = ring(g, 0.0, 0.0)?;
    let moving = body(g, SUBSTEPS, 0.0, 320.0)?;

    let merge = g.add_node("motion.combine");
    g.set_pos(merge, Pos { x: 900.0, y: 160.0 });
    wire(g, target, 0, merge, 0)?;
    wire(g, moving, 0, merge, 1)?;

    let out = g.add_node("motion.output");
    g.set_pos(
        out,
        Pos {
            x: 1120.0,
            y: 160.0,
        },
    );
    wire(g, merge, 0, out, 0)?;
    Some(vec![out])
}

/// Os números que a prosa cita — daqui, e não de uma segunda cópia deles.
pub(crate) fn numbers() -> (f32, f32, f32) {
    (START_R, SUBSTEPS, STRENGTH)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_substep_tests.rs"]
mod tests;
