//! **A PREGUIÇA DO ROTEADOR** (`PH2D_GPU_COOK_DEMO=107`) — a última célula P2 da conferência
//! (doc 89, folha 15): o *"only the input that is passed through the node is computed"* que o
//! Blender documenta duas vezes.
//!
//! ⚠️ **Esta cena não se julga pela IMAGEM, julga-se pelo MOVIMENTO.** A saída é a mesma com o
//! modo ligado e desligado — é essa a promessa —, e o que muda é o custo: quatro ramos de ruído
//! fractal de oito oitavas sobre 4096 peças, dos quais o roteador usa **um**.
//!
//! ```text
//!   modo DESLIGADO  o cook puxa os quatro ramos     ~10,8 ms por cozimento
//!   modo LIGADO     ele puxa so' o escolhido        ~2,8 ms   (o piso de um ramo so')
//! ```
//!
//! ⚠️ **O `select` fica DESLIGADO de propósito.** Uma porta sem aresta lê o campo vazio, que é
//! `0` em todo índice — uniforme por construção, que é a primeira das três condições da
//! preguiça. Ligar-lhe um `value.instance_field` faria dele um campo POR ELEMENTO, e aí cada
//! elemento escolhe o seu ramo, nenhum é dispensável e o modo recua para o caminho de sempre.
//! *A sonda que precificou esta feature usava exactamente esse select, e por isso media um ganho
//! que o mecanismo nunca poderia entregar naquele grafo.*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por lado — `64 × 64 = 4096`, o tamanho em que o relógio diz alguma coisa.
pub(super) const SIDE: f32 = 64.0;
/// Quantas oitavas tornam um ramo CARO.
const OCTAVES: f32 = 8.0;
/// Quantos ramos o roteador tem (o manifesto do nó).
const BRANCHES: usize = 4;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

pub(super) fn build_lazy_switch_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y: 0.0 });
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.06);
    g.set_param(grid, "gap_y", 0.06);

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 150.0, y: 0.0 });
    g.set_param(size, "amount", 0.045);
    wire(g, grid, 0, size, 0)?;

    let sw = g.add_node("value.switch");
    g.set_pos(sw, Pos { x: 620.0, y: 120.0 });
    // ⭐ **Nasce LIGADO nesta cena** — o smoke abre no caminho bom e o artista DESLIGA para
    // sentir o que ele custa. Abrir a arrastar-se pareceria uma cena partida.
    g.set_param(sw, ph2d_node_value_switch::LAZY, 1.0);
    g.set_label(sw, "Switch (Skip Unused Inputs)");

    // Os quatro ramos CAROS. Cada um é ruído fractal de oito oitavas sobre as 4096 peças,
    // lido como valor — o mesmo par que a sonda mede.
    for k in 0..BRANCHES {
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        let row = 260.0 + k as f32 * 110.0;
        let ns = g.add_node("motion.noise");
        g.set_pos(ns, Pos { x: 300.0, y: row });
        g.set_param(ns, "channel", 1.0);
        g.set_param(ns, "amplitude", 1.0);
        g.set_param(ns, "octaves", OCTAVES);
        // ⚠️ `scale` e `seed` — os nomes que o manifesto DECLARA. A 1.ª versão escreveu
        // `frequency`, que este nó não tem: um `set_param` com nome errado é inerte e a cena
        // montava com o ruído de fábrica. Quem o apanhou foi o `graph.validate(&reg)` do gate,
        // que devolve `UnknownParam` — *o construtor de cena que não valida é o que deixa um
        // param inerte passar*.
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        g.set_param(ns, "scale", 0.6 + k as f32 * 0.4);
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        g.set_param(ns, "seed", k as f32);
        wire(g, size, 0, ns, 0)?;
        let rd = g.add_node("value.attribute");
        g.set_pos(rd, Pos { x: 460.0, y: row });
        g.set_text_param(rd, "attr", "P");
        wire(g, ns, 0, rd, 0)?;
        #[expect(clippy::cast_possible_truncation, reason = "0..4")]
        wire(g, rd, 0, sw, k as u16 + 1)?;
    }

    // A saída do roteador CONDUZ o deslocamento vertical das peças — sem um consumidor
    // visível, um smoke de custo não teria o que se julgar a olho.
    let drive = g.add_node("motion.drive");
    g.set_pos(drive, Pos { x: 800.0, y: 0.0 });
    g.set_param(drive, "channel", 1.0);
    g.set_param(drive, "scale", 1.0);
    wire(g, size, 0, drive, 0)?;
    wire(g, sw, 0, drive, 1)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: 0.0 });
    wire(g, drive, 0, out, 0)?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_lazy_switch_demo_tests.rs"]
mod tests;
