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

/// Quantas peças por lado — **MEDIDO no quadro real, não escolhido**.
///
/// ```text
///   lado   pecas    modo ON    modo OFF
///    192   36 864    7,75 ms    15,31 ms
///    224   50 176    9,59 ms    33,63 ms   <- esta cena
///    256   65 536   13,92 ms   146,48 ms
/// ```
///
/// `224` é onde os dois lados são inequívocos sobre um orçamento de `16,7 ms`: ligado o quadro
/// sobra, desligado ele estoura por 2×, e a diferença lê-se como solavanco sem que o app deixe
/// de responder. ⛔ `256` seria dramático e enganador — a `146 ms` (7 fps) o artista concluiria
/// que a cena travou, não que o modo custa.
pub(super) const SIDE: f32 = 224.0;
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
    g.set_param(grid, "gap_x", 3.8 / SIDE);
    g.set_param(grid, "gap_y", 3.8 / SIDE);

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

    // Os quatro ramos CAROS — ruído fractal de oito oitavas sobre as 4096 peças, cada um com a
    // sua semente e o seu ritmo.
    //
    // ⚠️ **`value.noise` e não `motion.noise` + `value.attribute`.** A 1.ª versão desta cena
    // usava o par, copiado da sonda — e o `value.attribute` no modo de omissão procura uma
    // coluna ESCALAR chamada `P`, que não existe (o `P` é `Vec2`). Os quatro ramos emitiam
    // VAZIO, o roteador não conduzia nada e o campo ficava parado: *a cena montava, cozinhava
    // 4096 peças, media o custo certo — e não se mexia.* ⚠️ A sonda tem o mesmo par e continua
    // válida, porque o que ela mede é o CUSTO do cozimento e o nó de ruído é cozido na mesma;
    // o que ela nunca precisou foi que o valor chegasse ao fim.
    //
    // ⚠️ **E o `value.noise` é `Effect::Temporal`** — o que faz desta cena a prova, em produto,
    // de que a cerca do estado nomeia o mecanismo certo: ler o relógio não impede o salto, a
    // realimentação é que impede.
    for k in 0..BRANCHES {
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        let row = 260.0 + k as f32 * 110.0;
        let ns = g.add_node("value.noise");
        g.set_pos(ns, Pos { x: 380.0, y: row });
        g.set_param(ns, "octaves", OCTAVES);
        g.set_param(ns, "amplitude", 0.55);
        g.set_param(ns, "speed", 0.6);
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        g.set_param(ns, "frequency", 0.5 + k as f32 * 0.35);
        #[expect(clippy::cast_precision_loss, reason = "0..4")]
        g.set_param(ns, "seed", k as f32 + 1.0);
        wire(g, size, 0, ns, 0)?;
        #[expect(clippy::cast_possible_truncation, reason = "0..4")]
        wire(g, ns, 0, sw, k as u16 + 1)?;
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

    // ⚠️ **UMA SEGUNDA SAÍDA, e ela é a razão de a cena existir como cena.** O cozimento é
    // **GPU-residente por omissão**, e no device o grafo inteiro vira UM dispatch — não há
    // ramo para saltar. Este modo é uma propriedade do cozimento de **CPU**, e um documento vai
    // para a CPU quando o plano de GPU não o cobre: vector vivo, escopos de tempo, nós de
    // CPU-only… ou **mais de um sink** (`motion_bridge_gpu`: `motion.sinks.len() != 1`).
    //
    // ⚠️ **MEDIDO, e é o número que decide o desenho:** neste mesmo grafo com um sink só, a rota
    // de GPU faz o quadro em **3,75 ms** com os quatro ramos, contra **13,10 ms** da CPU com a
    // preguiça ligada. ⇒ *forçar a CPU quando o artista liga o modo tornaria o botão uma
    // armadilha*, e é por isso que a recusa NÃO existe: o modo vale onde a CPU já é o caminho.
    // Uma segunda saída é a forma mais honesta de pôr esta cena lá — é autoria legítima, não um
    // truque, e o texto do smoke di-lo.
    let peek = g.add_node("motion.output");
    g.set_pos(peek, Pos { x: 980.0, y: 260.0 });
    g.set_label(peek, "(segunda saida: poe a cena no cozimento de CPU)");
    wire(g, size, 0, peek, 0)?;
    Some(vec![out, peek])
}

#[cfg(test)]
#[path = "motion_state_lazy_switch_demo_tests.rs"]
mod tests;
