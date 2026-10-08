//! **N PRODUTORES NUM CAMPO DE ONDA** — a cena `=57`, a folha 06 linha 35.
//!
//! ⚠️ **Esta cena não demonstra uma feature NOVA: ela demonstra que a composição
//! JÁ exprime o item**, e é por isso que o veredito da folha é `P2` (ergonomia) e
//! não `P0/P1` (omissão). O `motion.wave` tem **uma** fonte de Dirichlet — a célula
//! do centro —, e a célula pedia N. A resposta não é um param: é o
//! `motion.drive(Custom…)` do **Grupo P** escrevendo na coluna de ESTADO do campo
//! (`wave_h`) dentro do laço, que é um produtor tão real quanto o do centro.
//!
//! **A cadeia, quatro nós e três arestas:**
//! `wave.out --pre--> field.box --> value.attribute("falloff")
//!  --> motion.drive(Custom "wave_h", Add) --> wave.state`
//!
//! ⚠️ **O `pre` mora na aresta que ENTRA na cadeia, nunca na que volta ao `state`** —
//! é ela que quebra o ciclo, e os três nós do meio são `Effect::Pure`, logo não
//! carimbam `sim_t` e a onda ainda vê o `dt` do próprio relógio no tique seguinte.
//!
//! **Duas bandas, e a de cima é o CONTROLE:**
//! - **1** o campo com a fonte do centro e mais nada — as ondas nascem no MEIO.
//! - **2** o MESMO campo com a cadeia ligada — um segundo berço nasce à ESQUERDA,
//!   e as duas frentes se cruzam no caminho.
//!
//! ⚠️ **A leitura é de ONDE as ondas nascem, não de a de baixo "mexer mais"** —
//! um campo mais agitado também mexeria mais, e não seria um produtor.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Lado da grade (em células). 21×21 = 441 peças por banda.
const SIDE: f32 = 21.0;
/// Distância entre células, em unidades de mundo. A grade mede `(SIDE-1)*SPACING`.
const SPACING: f32 = 0.5;
/// O vão vertical entre as duas bandas — maior que a altura de uma grade (10,0),
/// senão elas se sobrepõem e a leitura de *onde nasce* fica impossível.
const BAND_DY: f32 = 12.5;
/// O centro do produtor INJETADO, em unidades de mundo. A grade vai de −5 a +5 em
/// x, então isto é bem longe do berço nativo (o centro) — a distância é o que faz
/// as duas frentes se encontrarem no meio do caminho em vez de coincidirem.
const BOX_X: f32 = -3.0;
/// O nome da coluna de ESTADO em que o drive escreve. ⚠️ Ele **não** está no
/// `is_bookkeeping_column` da `ph2d-nodegraph`, e é por isso que o canal `Custom…`
/// o aceita — a recusa de escrituração guarda `id`/`sim_t`/`sim_d`/`dl_*`.
const STATE_COLUMN: &str = "wave_h";

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// O campo, com a fonte do centro que ele sempre teve.
fn wave(g: &mut Graph, x: f32, y: f32) -> NodeId {
    let w = g.add_node("motion.wave");
    g.set_pos(w, Pos { x, y });
    for (k, v) in [
        ("rows", SIDE),
        ("cols", SIDE),
        ("spacing", SPACING),
        ("speed", 0.35),
        ("damping", 0.02),
    ] {
        g.set_param(w, k, v);
    }

    let lfo = g.add_node("value.lfo");
    g.set_pos(lfo, Pos { x: x - 220.0, y });
    g.set_param(lfo, "period", 0.5);
    g.set_param(lfo, "amplitude", 1.0);
    wire(g, lfo, 0, w, 0, false);
    w
}

/// A cadeia de INJEÇÃO — o segundo produtor, por composição.
///
/// Devolve o nó cuja saída volta ao `state` do campo.
fn injector(g: &mut Graph, w: NodeId, x: f32, y: f32) -> Option<NodeId> {
    let bx = g.add_node("field.box");
    g.set_pos(bx, Pos { x, y });
    for (k, v) in [
        ("width", 0.8),
        ("height", 0.8),
        ("soft", 0.3),
        ("center_x", BOX_X),
        ("center_y", 0.0),
    ] {
        g.set_param(bx, k, v);
    }
    wire(g, w, 0, bx, 0, true)?;

    let rd = g.add_node("value.attribute");
    g.set_pos(rd, Pos { x: x + 200.0, y });
    g.set_param(rd, "mode", 0.0); // a coluna escalar, crua
    g.set_text_param(rd, "attr", "falloff");
    wire(g, bx, 0, rd, 0, false)?;

    let dr = g.add_node("motion.drive");
    g.set_pos(dr, Pos { x: x + 400.0, y });
    g.set_param(dr, "channel", 9.0); // Custom...
    g.set_param(dr, "mode", 0.0); // Add
    // ⚠️ **0,42 é MEDIDO na ordem do PRODUTO** (cozer, depois avançar — doc 123 §4).
    // O 0,25 antigo foi calibrado com o arnês ao contrário, que re-cozia o `pre` com
    // `dt = 0` e DEPOSITAVA DUAS VEZES por tique: no app o produtor sempre teve metade
    // da força que a cena dizia (0,25 dava −3,00 e 1,24× no arnês, +0,50 e 0,96× no
    // produto). Abaixo de ~0,37 o injetado **não vira o berço** (o pico fica no centro
    // ou salta para a borda). `probe_the_drive_scale_in_product_order`, controle
    // 0,2231 (pico −0,50); peças > passo do controle = 21:
    //
    // | scale | comp/ctrl | pico 240 · 360 · 480 tiques | peças > passo |
    // |-------|-----------|-----------------------------|---------------|
    // | 0,25  | 0,96      | +0,50 · −5,00 · −5,00       | 17 |
    // | 0,35  | 1,00      | −5,00 · −3,50 · −5,00       | 20 |
    // | 0,37  | 1,07      | −3,50 · −3,50 · −3,50       | 20 |
    // | 0,42  | **1,25**  | **−3,50** (a CAIXA, ½ célula) nos três | **26** |
    // | 0,45  | 1,36      | −3,50 nos três              | 30 |
    // | 0,50  | 1,55      | −3,50 nos três              | 34 |
    //
    // ⇒ 0,42 põe a razão no MEIO da barra `1,0..1,5` e o berço estável na caixa nos
    // três horizontes. Igualdade exata é inalcançável **por física**: dois produtores
    // de mesma força deixam o pico com quem estiver mais alto naquele instante.
    g.set_param(dr, "scale", 0.42);
    g.set_text_param(dr, "column", STATE_COLUMN);
    wire(g, bx, 0, dr, 0, false)?;
    wire(g, rd, 0, dr, 1, false)?;
    Some(dr)
}

/// Põe a banda no lugar e a termina num `motion.output`.
///
/// ⚠️ O sink **não** é decoração: o laço de render re-resolve os sinks a cada quadro
/// a partir dos nós de saída do grafo, então uma banda sem `motion.output` cozinha
/// certo, satisfaz os gates e **desenha NADA** (a lição da cena `=48`).
fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0, false)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 200.0, y });
    wire(g, mv, 0, out, 0, false)?;
    Some(out)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub fn build_wave_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(2);
    for (row_i, inject) in [false, true].into_iter().enumerate() {
        let gy = row_i as f32 * 320.0;
        let w = wave(g, 0.0, gy);
        if inject {
            let tail = injector(g, w, 220.0, gy + 140.0)?;
            wire(g, tail, 0, w, 1, false)?;
        } else {
            // Sem cadeia, o campo ainda precisa do laço de estado.
            wire(g, w, 0, w, 1, true)?;
        }
        sinks.push(place(g, w, (0.5 - row_i as f32) * BAND_DY, 700.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das duas bandas, na ordem em que a cena as monta.
pub fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CONTROLE -- uma fonte so', a do CENTRO: as ondas nascem no meio da grade",
        "DOIS PRODUTORES -- drive(Custom `wave_h`) no laco: um segundo berco a' ESQUERDA",
    ]
    .into_iter()
    .enumerate()
}

/// O nome da coluna de estado que a cena escreve — o painel do grafo o mostra no
/// text param do `motion.drive`, e é ele que o artista tem de saber digitar.
pub fn state_column() -> &'static str {
    STATE_COLUMN
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_wave_tests.rs"]
mod tests;
