//! **DOIS BERÇOS DE ONDA** (`PH2D_GPU_COOK_DEMO=105`) — os *Producers* do AE Wave World
//! (doc 89, folha 06, célula 35).
//!
//! ⚠️ **Esta cena PRECISA de Play.**
//!
//! ```text
//!   ESQUERDA  o tanque de sempre -- as ondas nascem SO' no centro
//!   DIREITA   o mesmo tanque com DUAS fontes fora do centro
//! ```
//!
//! ⚠️ **A capacidade já existia por composição — o que faltava era o GESTO.** A rota medida
//! pela folha eram quatro nós, três arestas, e saber que a coluna de estado se chama `wave_h`,
//! um nome que nenhum picker oferece. Aqui são **duas peças e uma aresta**, e a coluna lida é
//! o `falloff` — o que a família `field.*` inteira já escreve.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O lado de cada tanque, em células.
pub(super) const SIDE: f32 = 21.0;
const SPACING: f32 = 0.16;
/// Onde os dois tanques ficam.
const POND_X: f32 = 2.1;
/// Onde as duas fontes ficam, em relação ao centro do tanque delas.
pub(super) const SOURCE_X: f32 = 1.1;
/// Quanto cada fonte injecta por tique.
pub(super) const STRENGTH: f32 = 0.5;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// Um tanque; com `sources`, duas caixas fora do centro alimentam a porta `inject`.
fn pond(g: &mut Graph, x: f32, sources: bool, row: f32) -> Option<NodeId> {
    let w = g.add_node("motion.wave");
    g.set_pos(w, Pos { x: 420.0, y: row });
    for (k, v) in [
        ("rows", SIDE),
        ("cols", SIDE),
        ("spacing", SPACING),
        ("speed", 0.35),
        ("damping", 0.01),
        ("center_x", x),
        ("edges", 1.0), // absorve — os anéis das fontes lêem-se melhor sem eco
    ] {
        g.set_param(w, k, v);
    }
    g.set_label(
        w,
        if sources {
            "Com DUAS fontes"
        } else {
            "So' o centro"
        },
    );

    // A fonte do centro que o nó sempre teve, igual nos dois lados.
    let lfo = g.add_node("value.lfo");
    g.set_pos(lfo, Pos { x: 220.0, y: row });
    g.set_param(lfo, "period", 0.7);
    g.set_param(lfo, "amplitude", 1.0);
    wire(g, lfo, 0, w, 0, false)?;
    wire(g, w, 0, w, 1, true)?; // o laço de estado

    if sources {
        // ⭐ **DUAS peças e UMA aresta** — a rota que a célula media custava quatro nós.
        // ⚠️ As duas caixas COMPÕEM por `field.combine`, que é o que a família já faz.
        let a = g.add_node("field.box");
        g.set_pos(
            a,
            Pos {
                x: 220.0,
                y: row + 130.0,
            },
        );
        for (k, v) in [
            ("width", 0.5),
            ("height", 0.5),
            ("soft", 0.3),
            ("center_x", x - SOURCE_X),
            ("center_y", 0.0),
        ] {
            g.set_param(a, k, v);
        }
        wire(g, w, 0, a, 0, true)?;

        let b = g.add_node("field.box");
        g.set_pos(
            b,
            Pos {
                x: 220.0,
                y: row + 260.0,
            },
        );
        for (k, v) in [
            ("width", 0.5),
            ("height", 0.5),
            ("soft", 0.3),
            ("center_x", x + SOURCE_X),
            ("center_y", 0.0),
        ] {
            g.set_param(b, k, v);
        }
        wire(g, w, 0, b, 0, true)?;

        // ⚠️⚠️ **AS DUAS CAIXAS UNEM-SE POR `field.combine`, NUNCA POR ENCADEAMENTO.**
        // Encadear dois `field.box` é uma **intersecção** (o segundo mascara o que o
        // primeiro deixou), e como estas duas não se sobrepõem o produto é **zero em toda
        // parte** — medido: a fonte entregava `falloff` com `max = 0,0000` e os dois tanques
        // saíam byte-idênticos. É a mesma forma que a folha 13 já tinha registado do outro
        // lado: *encadear colisores é conjunção, não união.*
        let both = g.add_node("field.combine");
        g.set_pos(
            both,
            Pos {
                x: 380.0,
                y: row + 190.0,
            },
        );
        g.set_param(both, "mode", 6.0); // Max — a união de duas máscaras
        wire(g, a, 0, both, 0, false)?;
        wire(g, b, 0, both, 1, false)?;

        g.set_param(w, "inject_gain", STRENGTH);
        wire(g, both, 0, w, 2, false)?;
    }

    let place = g.add_node("motion.transform");
    g.set_pos(place, Pos { x: 620.0, y: row });
    wire(g, w, 0, place, 0, false)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 780.0, y: row });
    wire(g, place, 0, out, 0, false)?;
    Some(out)
}

/// **DOIS BERÇOS DE ONDA** (`PH2D_GPU_COOK_DEMO=105`).
pub(super) fn build_gpu_producers_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![
        pond(g, -POND_X, false, -220.0)?,
        pond(g, POND_X, true, 120.0)?,
    ])
}

#[cfg(test)]
#[path = "motion_state_gpu_producers_demo_tests.rs"]
mod tests;
