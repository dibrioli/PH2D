//! **O ESPAÇO DO CAMPO** — a cena `=60`, a folha 06 linha 20 (o **último P1** dela).
//!
//! Quatro blocos com o **mesmo** `motion.noise`: mesma semente, mesma amplitude, mesma
//! oitava, mesma escala. O que muda é **onde o campo é amostrado**.
//!
//! 1. **CONTROLE** — o campo de sempre.
//! 2. **RODADO 45°** — o mesmo campo, o espaço girado. ⚠️ A leitura é *as manchas viraram*,
//!    nunca *ficou mais agitado*: a amplitude é a mesma nas quatro, de propósito.
//! 3. **COMPRIMIDO no Y** — `uniform` desligado e o eixo Y com escala PRÓPRIA e maior: o
//!    mesmo passo de mundo cobre mais campo, então as manchas ficam **baixas e largas** —
//!    listras deitadas. ⚠️ *Escala maior = feição menor*, que é o contrário do que o nome
//!    sugere, e foi a medição que corrigiu a leitura.
//! 4. **OS DOIS** — comprimido e **depois** rodado, que é a ordem em que o nó os aplica.
//!    ⚠️ É a banda que prova a ORDEM: se ele rodasse primeiro, as listras sairiam
//!    alinhadas com o eixo do mundo e esta banda seria a banda 3 outra vez.
//!
//! ⚠️ **Isto julga-se PARADO.** O `speed` é zero nas quatro — um campo a rolar mostraria
//! movimento e esconderia exactamente o que a cena existe para mostrar, que é a FORMA.
//!
//! ⚠️ **O que a cena NÃO tem, e é decisão MEDIDA:** o *offset* do campo. Ele já sai da
//! composição — `motion.move(+d) → noise → motion.move(−d)` devolve a pose (`|Δx| = 0`) e
//! desloca o campo (`|Δy| = 0,63`). E o *scale uniforme* **já era o param `scale`**: o
//! sanduíche `motion.transform(s) … (1/s)` é bit-a-bit `scale·s` com a amplitude dividida
//! por `s` (pior `|Δy|` entre as duas rotas: **0,000000**). Uma banda a mostrar qualquer um
//! dos dois estaria a ensinar um knob que não existe (`measure_noise_space`).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Lado de cada bloco, em peças. Grande o bastante para o campo mostrar mais de uma mancha.
const SIDE: f32 = 15.0;
/// O vão entre peças, em unidades de mundo.
const GAP: f32 = 0.32;
/// O vão horizontal entre blocos — maior que o lado de um bloco (`15 · 0,32 = 4,48`).
const BAND_DX: f32 = 6.4;
/// A escala espacial do campo. Ajustada ao tamanho do bloco: as manchas medem cerca de um
/// terço dele.
const SCALE: f32 = 0.55;
/// A escala do eixo Y quando `uniform` está desligado. ⚠️ **4×** o `SCALE`, e não 2×: a
/// razão precisa de ser grande para «listra» se ler como listra, e não como «mancha um
/// bocadinho oval». ⚠️ E maior = feição MENOR nesse eixo (medido: a razão de variação
/// `dx/dy` cai de **0,976** para **0,341**).
const SCALE_Y: f32 = SCALE * 4.0;
/// O ângulo das bandas 2 e 4, em graus. ⚠️ **45 e não 90:** com 90° um campo isotrópico
/// parece o mesmo (as manchas trocam de eixo e o olho não tem referência), e a banda 2
/// leria como *"não mudou nada"*.
const TURN: f32 = 45.0;
/// Quanto o ruído desloca cada peça — o MESMO nas quatro bandas.
const AMPLITUDE: f32 = 0.42;

fn wire(g: &mut Graph, from: NodeId, to: NodeId) -> Option<()> {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .ok()
}

/// Um bloco de `SIDE × SIDE` peças.
fn block(g: &mut Graph, x: f32, y: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x, y });
    for (k, v) in [
        ("rows", SIDE),
        ("cols", SIDE),
        ("gap_x", GAP),
        ("gap_y", GAP),
    ] {
        g.set_param(n, k, v);
    }
    n
}

/// O ruído — **idêntico nas quatro bandas** menos pelo espaço, e é esse o ponto.
fn noise(
    g: &mut Graph,
    src: NodeId,
    rotation: f32,
    stretched: bool,
    x: f32,
    y: f32,
) -> Option<NodeId> {
    let n = g.add_node("motion.noise");
    g.set_pos(n, Pos { x, y });
    g.set_param(n, "channel", 1.0); // Y — o campo empurra as peças para cima e para baixo
    g.set_param(n, "amplitude", AMPLITUDE);
    g.set_param(n, "scale", SCALE);
    g.set_param(n, "octaves", 1.0); // uma oitava: a FORMA, sem detalhe a distrair
    g.set_param(n, "seed", 3.0);
    g.set_param(n, "speed", 0.0); // ⚠️ PARADO — ver o cabeçalho
    g.set_param(n, "rotation", rotation);
    g.set_param(n, "uniform", if stretched { 0.0 } else { 1.0 });
    g.set_param(n, "scale_y", SCALE_Y);
    wire(g, src, n)?;
    Some(n)
}

/// Põe a banda no lugar e a termina num `motion.output`.
///
/// ⚠️ O sink **não** é decoração: o laço de render re-resolve os sinks a cada quadro a
/// partir dos nós de saída do grafo, então uma banda sem `motion.output` cozinha certo,
/// satisfaz os gates e **desenha NADA** (a lição da cena `=48`).
fn place(g: &mut Graph, head: NodeId, dx: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", dx);
    wire(g, head, mv)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 200.0, y });
    wire(g, mv, out)?;
    Some(out)
}

/// Monta a cena. Devolve os sinks, um por banda, na ordem dos rótulos.
pub(crate) fn build_field_space_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    // ⚠️ Os blocos ficam LADO A LADO (`dx`), não empilhados: o eixo que a banda 3 estica é
    // o Y, e empilhá-los poria a diferença ENTRE bandas no mesmo eixo da diferença que a
    // cena mede.
    for (i, (rotation, stretched)) in [(0.0, false), (TURN, false), (0.0, true), (TURN, true)]
        .into_iter()
        .enumerate()
    {
        let gy = i as f32 * 200.0;
        let src = block(g, 0.0, gy);
        let ns = noise(g, src, rotation, stretched, 220.0, gy)?;
        sinks.push(place(g, ns, (i as f32 - 1.5) * BAND_DX, 440.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CONTROLE -- o campo de sempre",
        "RODADO 45 graus -- o MESMO campo, o espaco girado",
        "COMPRIMIDO no Y -- uniform OFF: as manchas viram LISTRAS DEITADAS",
        "OS DOIS -- comprimido e DEPOIS rodado (a ordem que o no' aplica)",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem da cena cita: `(ângulo, escala, escala do Y)`.
pub(crate) fn knobs() -> (f32, f32, f32) {
    (TURN, SCALE, SCALE_Y)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_field_space_tests.rs"]
mod tests;
