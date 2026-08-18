//! **O RELÓGIO É UM CAMPO** — a cena `=59`, o `SUPERAR 1` da folha 06.
//!
//! Quatro bandas, e **entre uma e a seguinte muda UM FIO**. O nó é o mesmo
//! (`motion.oscillator`), os knobs são os mesmos; o que muda é o que está ligado à
//! porta `time`, que é nova. É essa invariância que a cena existe para mostrar: não
//! há knob novo nenhum.
//!
//! 1. **CONTROLE — sem porta.** `phase_stagger = 0`, então a fileira inteira sobe e
//!    desce como **uma barra**. ⚠️ Esta banda é o que dá sentido às outras: sem ela,
//!    *"as peças se movem em tempos diferentes"* seria satisfeito pelo `phase_stagger`
//!    que o nó já tinha desde sempre.
//! 2. **UM RELÓGIO POR PEÇA.** Um `value.time` com `stagger` entra na porta e a barra
//!    vira uma **onda que viaja**. É a receita canônica do Cavalry (*Stagger → Shape
//!    Time Offset*), hoje inexprimível — e a nossa é **contínua**, não em quadros.
//! 3. **O RELÓGIO VEM DO ESPAÇO.** Um bloco cujo relógio é `t + |P|` — a distância ao
//!    centro, somada ao tempo. As peças **à mesma distância partilham o instante**, e
//!    a animação sai do meio como uma **ondulação**. ⚠️ É o item que nenhuma
//!    referência tem: TD/AE/Cavalry/C4D dão um relógio por *objeto* ou por *cópia*;
//!    aqui ele é um **campo**, e podia vir de áudio ou de qualquer `field.*`.
//! 4. **O CICLO FECHA POR CONSTRUÇÃO.** O mesmo relógio escalonado passa por um
//!    `value.wrap(Mirror)`: cada peça vai e volta dentro de uma janela de `WRAP_S`
//!    segundos, **para sempre e sem deriva** — `t` e `t + 2·WRAP_S` são o MESMO número
//!    a entrar no nó. Um `loop_len` por-nó é um cross-fade, que **aproxima**; isto é a
//!    identidade.
//!
//!    ⚠️ **«Sem deriva» é medido como um resíduo que NÃO CRESCE, não como um zero.**
//!    O relógio é um `f32` e `t` cresce sem parar, então a parte fraccionária de
//!    `t/período` perde bits com a magnitude. Medido: uma volta depois o quadro
//!    repete-se a **1,9e-6** de unidade de mundo, e **dez** voltas depois a
//!    **7,6e-6** — a mesma ordem, contra **1,80** de deriva da banda 2 no mesmo
//!    tempo. O que a identidade compra sobre um cross-fade não é o último bit; é
//!    seis ordens de grandeza (a resolução do float contra um erro de MODELO).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Peças por fileira nas bandas 1, 2 e 4.
const PIECES: f32 = 21.0;
/// Lado do bloco da banda 3 — ímpar, para uma peça cair exactamente no centro.
const BLOCK: f32 = 9.0;
/// O vão entre peças, em unidades de mundo.
const GAP: f32 = 0.55;
/// O vão vertical entre bandas.
const BAND_DY: f32 = 3.4;
/// Ciclos por segundo do oscilador — o mesmo nas quatro bandas.
const FREQ: f32 = 0.5;
/// A amplitude, idem. Grande o bastante para a onda se ler de longe.
const AMPLITUDE: f32 = 0.9;
/// Quanto o relógio de uma peça se atrasa em relação à vizinha, em segundos
/// (bandas 2 e 4). ⚠️ `PIECES · STAGGER · FREQ = 2,52` ciclos ao longo da fileira:
/// mais do que um (a onda tem de se ver a viajar) e poucos o bastante para ela ler
/// como onda e não como ruído.
///
/// ⚠️ **`0,24` e não `0,25`, e a diferença é MEDIDA.** Com `0,25` o passo de fase é
/// `1/8` de ciclo exacto, a fileira **repete-se a cada 8 peças** e as 21 peças exibem
/// só **8** alturas distintas — a onda fica com o desenho de um carimbo. Com `0,24` o
/// passo é `0,12`, cujo período é 25 > 21, e as 21 peças são todas distintas.
const STAGGER: f32 = 0.24;
/// A janela do espelho da banda 4, em segundos.
///
/// ⚠️ **`2·WRAP_S·FREQ` NÃO pode ser inteiro.** Com `3,0` o período do espelho eram
/// 6 s = **3 ciclos exactos** do oscilador, e aí a banda 2 (sem espelho nenhum)
/// também se repetia — o controle do gate media 1,9e-6 contra 7,6e-6 do espelhado, e
/// a cena teria "provado" um mecanismo que a aritmética já dava de graça. Com `2,5`
/// são 2,5 ciclos, e o controle deriva por uma ordem de grandeza que não se confunde
/// com o resíduo do `f32`.
const WRAP_S: f32 = 2.5;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Uma grade de `rows × cols`.
fn grid(g: &mut Graph, rows: f32, cols: f32, x: f32, y: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x, y });
    for (k, v) in [
        ("rows", rows),
        ("cols", cols),
        ("gap_x", GAP),
        ("gap_y", GAP),
    ] {
        g.set_param(n, k, v);
    }
    n
}

/// O oscilador — **idêntico nas quatro bandas**, e é esse o ponto.
fn osc(g: &mut Graph, src: NodeId, x: f32, y: f32) -> Option<NodeId> {
    let o = g.add_node("motion.oscillator");
    g.set_pos(o, Pos { x, y });
    g.set_param(o, "channel", 1.0); // Y
    g.set_param(o, "amplitude", AMPLITUDE);
    g.set_param(o, "frequency", FREQ);
    // ⚠️ ZERO. O knob que o nó já tinha fica DESLIGADO nas quatro bandas — o que a
    // cena mostra tem de vir da porta, ou não mostra nada.
    g.set_param(o, "phase_stagger", 0.0);
    wire(g, src, 0, o, 0)?;
    Some(o)
}

/// Um `value.time` sobre a fonte: ligado ⇒ **N** relógios, `t + i·stagger`.
fn clock(g: &mut Graph, src: NodeId, stagger: f32, x: f32, y: f32) -> Option<NodeId> {
    let c = g.add_node("value.time");
    g.set_pos(c, Pos { x, y });
    g.set_param(c, "stagger", stagger);
    wire(g, src, 0, c, 0)?;
    Some(c)
}

/// Põe a banda no lugar e a termina num `motion.output`.
///
/// ⚠️ O sink **não** é decoração: o laço de render re-resolve os sinks a cada quadro a
/// partir dos nós de saída do grafo, então uma banda sem `motion.output` cozinha
/// certo, satisfaz os gates e **desenha NADA** (a lição da cena `=48`).
fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 200.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// Monta a cena. Devolve os sinks, um por banda, na ordem dos rótulos.
pub(crate) fn build_clock_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);

    // 1 — CONTROLE: a porta `time` DESLIGADA. A fileira e' uma barra.
    let src = grid(g, 1.0, PIECES, 0.0, 0.0);
    let o = osc(g, src, 220.0, 0.0)?;
    sinks.push(place(g, o, 1.6 * BAND_DY, 440.0, 0.0)?);

    // 2 — um `value.time(stagger)` na porta: a barra vira uma ONDA QUE VIAJA.
    let src = grid(g, 1.0, PIECES, 0.0, 180.0);
    let o = osc(g, src, 220.0, 180.0)?;
    let c = clock(g, src, STAGGER, 220.0, 260.0)?;
    wire(g, c, 0, o, 1)?;
    sinks.push(place(g, o, 0.5 * BAND_DY, 440.0, 180.0)?);

    // 3 — o relogio vem do ESPACO: `t + |P|`, a distancia ao centro do bloco.
    let src = grid(g, BLOCK, BLOCK, 0.0, 380.0);
    let o = osc(g, src, 220.0, 380.0)?;
    let c = clock(g, src, 0.0, 220.0, 460.0)?;
    let dist = g.add_node("value.attribute");
    g.set_pos(dist, Pos { x: 220.0, y: 540.0 });
    g.set_param(dist, "mode", 1.0); // Length — |P|
    g.set_text_param(dist, ph2d_node_value_attribute::ATTR_KEY, "P".to_string());
    wire(g, src, 0, dist, 0)?;
    let sum = g.add_node("value.math");
    g.set_pos(sum, Pos { x: 440.0, y: 500.0 });
    g.set_param(sum, "op", 0.0); // Add
    wire(g, c, 0, sum, 0)?;
    wire(g, dist, 0, sum, 1)?;
    wire(g, sum, 0, o, 1)?;
    sinks.push(place(g, o, -1.4 * BAND_DY, 660.0, 380.0)?);

    // 4 — o mesmo relogio escalonado, ESPELHADO numa janela: o ciclo fecha.
    let src = grid(g, 1.0, PIECES, 0.0, 700.0);
    let o = osc(g, src, 220.0, 700.0)?;
    let c = clock(g, src, STAGGER, 220.0, 780.0)?;
    let wrap = g.add_node("value.wrap");
    g.set_pos(wrap, Pos { x: 440.0, y: 780.0 });
    g.set_param(wrap, "lo", 0.0);
    g.set_param(wrap, "hi", WRAP_S);
    // ⚠️ `Mirror` é **2** e `Repeat` é **1** — o `0` é `Clamp`, que congelaria a banda
    // depois de `WRAP_S` (o defeito que a cena `=58` pagou noutro nó).
    g.set_param(wrap, "mode", 2.0);
    wire(g, c, 0, wrap, 0)?;
    wire(g, wrap, 0, o, 1)?;
    sinks.push(place(g, o, -3.2 * BAND_DY, 660.0, 700.0)?);

    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CONTROLE -- porta `time` DESLIGADA: a fileira e' UMA BARRA",
        "UM RELOGIO POR PECA -- value.time(stagger) na porta: ONDA QUE VIAJA",
        "O RELOGIO VEM DO ESPACO -- t + |P|: as pecas 'a mesma distancia andam JUNTAS",
        "O CICLO FECHA -- value.wrap(Mirror): vai e volta, para sempre, sem deriva",
    ]
    .into_iter()
    .enumerate()
}

/// A janela do espelho, em segundos — o número que a mensagem cita.
pub(crate) fn wrap_seconds() -> f32 {
    WRAP_S
}

/// O lado do bloco da banda 3 — idem.
pub(crate) fn block_side() -> usize {
    BLOCK as usize
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_clock_tests.rs"]
mod tests;
