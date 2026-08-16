//! **A VIZINHANÇA VIRA UM NÚMERO** (`PH2D_GPU_COOK_DEMO=49`) — a cena do **grupo I**
//! da conferência (doc 89, folha 03, linhas 42 e 61).
//!
//! ## O que ela prova: os três modos do C4D são COMPOSIÇÃO
//!
//! O *Push Apart Effector* tem **Push · Scale · Hide**, e a folha 03 prescrevia um
//! param `mode` dentro do `motion.collide` (o doc 63 propunha um nó novo). Com o
//! `motion.proximity` a publicar a vizinhança, o **Push** continua sendo o
//! `motion.collide` e os outros dois saem de nós que já existem — é isso que cada
//! par de fileiras mostra, sempre contra o MESMO controle.
//!
//! 1. **SCALE** — em cima a fileira crua: o tamanho cresce da esquerda para a
//!    direita e as peças da direita **SOBREPÕEM**. Em baixo a MESMA fileira passa
//!    por `proximity → value.attribute(Overlap) → value.map_range(0..1 → 1..0) →
//!    motion.drive(Size, Multiply)`: as grandes **encolhem até apenas se tocar** e
//!    as pequenas, que já estavam livres, **não são tocadas**.
//! 2. **HIDE** — o mesmo controle. Em baixo `→ motion.drive(Falloff, Set) →
//!    motion.cull(Falloff, invert)`: quem se sobrepõe **SAI da cena**, e sobra só
//!    o punhado da esquerda que já estava livre.
//! 3. **A CONTAGEM** — a segunda coluna. Em cima o controle; em baixo o tamanho é
//!    **DEFINIDO** por `neighbours`, então cada peça vira um **medidor de quantos
//!    vizinhos ela toca**: as da ponta livre ficam pequenas e as do amontoado
//!    incham. É a fileira que prova que a wave publica DUAS colunas, não uma.
//!
//! ⚠️ **Esta cena julga-se PARADA.** O `motion.proximity` é `Effect::Pure` — ele
//! mede a entrada a cada cook, e nada aqui depende do relógio.
//!
//! ⚠️ **A fileira tem de conter as DUAS espécies**, e é por isso que ela nasce com
//! um gradiente de tamanho em vez de um passo variável: sobre um amontoado uniforme
//! o Hide apagaria a cena inteira e *"apaga quem se sobrepõe"* seria indistinguível
//! de *"apaga tudo"* — foi exactamente o que o gate de composição mediu antes de a
//! fixture dele ganhar as duas espécies.
//!
//! ⚠️ **O gradiente NÃO usa `motion.falloff` + `motion.scale`, e a razão é um
//! CONFUNDIDOR medido:** aquele par deixaria a coluna `falloff` escrita como uma
//! rampa `0..1`, e o `motion.drive` **mascara todo efeito pelo falloff** — na banda
//! da CONTAGEM a metade esquerda ficaria com `falloff ≈ 0` e o medidor **não
//! escreveria nada ali**, exactamente onde a leitura *"livre ⇒ pequeno"* mora. O
//! gradiente vem de `value.instance_field(Ramp)`, que não toca coluna nenhuma além
//! da própria `v` ⇒ `falloff` fica **AUSENTE** (lido como `1`) em toda parte.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira.
pub(crate) const COLS: usize = 13;
/// Quantas fileiras a cena empilha (três PARES).
pub(crate) const BANDS: usize = 6;
/// A distância vertical entre fileiras.
const BAND_GAP: f32 = 1.15;
/// O passo da fileira — FIXO. É o TAMANHO que varia, então o mesmo passo dá peças
/// livres de um lado e amontoadas do outro.
const PITCH: f32 = 0.34;
/// O raio de disco que o `motion.proximity` usa. `2·RADIUS = 0,34` é exactamente o
/// passo ⇒ com tamanho de fábrica (`1`) duas vizinhas **apenas se tocam** e a
/// contagem é zero (o toque é `<` estrito); é o gradiente que as põe em contacto.
const RADIUS: f32 = 0.17;
/// Os dois extremos do gradiente de tamanho. Com `PITCH = 0,34` um par `(k, k+1)`
/// se toca quando `s_k + s_{k+1} > 2` ⇒ com `0,4 → 2,4` os quatro primeiros ficam
/// **livres** e os nove seguintes se amontoam: as DUAS espécies numa fileira só.
const MIN_SCALE: f32 = 0.4;
const MAX_SCALE: f32 = 2.4;

/// Os canais do `motion.drive` (`channel_column` na crate dele) — literais porque
/// são `pub(crate)` lá, e o que impede um número errado de passar em silêncio é a
/// própria cena: um canal trocado desenha outra coisa, e a mensagem do smoke diz o
/// que se tem de ver.
const CH_SIZE: f32 = 3.0;
/// Idem — `5 => "falloff"`.
const CH_FALLOFF: f32 = 5.0;
/// `Combine::Set`.
const COMBINE_SET: f32 = 1.0;
/// `Combine::Multiply`.
const COMBINE_MULTIPLY: f32 = 2.0;
/// `motion.cull`: `MODE_FALLOFF` — mantém quem tem `falloff >= amount`.
const CULL_MODE_FALLOFF: f32 = 1.0;
/// O limiar do Hide: qualquer sobreposição mensurável condena a peça. Com
/// `invert = 1` o `motion.cull` mantém o COMPLEMENTO, isto é `overlap < 1e-4`.
const HIDE_EPS: f32 = 1e-4;
/// A faixa do medidor de vizinhos. `0..4` porque um disco do lado direito alcança
/// **dois** vizinhos de cada lado, e `0,6..2,2` porque um tamanho cru de `3` sairia
/// do quadro.
const COUNT_HI: f32 = 4.0;
const COUNT_SIZE_LO: f32 = 0.6;
const COUNT_SIZE_HI: f32 = 2.2;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// A fileira de partida de uma banda: peças igualmente espaçadas cujo TAMANHO
/// cresce da esquerda para a direita, já deslocada para o `y` da banda.
fn row(g: &mut Graph, row_index: usize) -> Option<NodeId> {
    let offset_y = (row_index as f32) * -BAND_GAP;
    let y = offset_y;

    let src = g.add_node("motion.grid");
    g.set_pos(src, Pos { x: 0.0, y });
    g.set_param(src, "rows", 1.0);
    #[allow(clippy::cast_precision_loss)]
    g.set_param(src, "cols", COLS as f32);
    g.set_param(src, "gap_x", PITCH);
    g.set_param(src, "gap_y", PITCH);

    // `i/(N-1)` em `[0, 1]` — o gradiente ordinal, sem tocar `falloff`.
    let ramp = g.add_node("value.instance_field");
    g.set_pos(ramp, Pos { x: 150.0, y });
    g.set_param(ramp, "mode", 1.0);
    wire(g, src, 0, ramp, 0)?;

    let span = g.add_node("value.map_range");
    g.set_pos(span, Pos { x: 300.0, y });
    g.set_param(span, "in_lo", 0.0);
    g.set_param(span, "in_hi", 1.0);
    g.set_param(span, "out_lo", MIN_SCALE);
    g.set_param(span, "out_hi", MAX_SCALE);
    wire(g, ramp, 0, span, 0)?;

    // `size` ausente lê a identidade `[1, 1]`, então Multiply POUSA o gradiente.
    let grow = g.add_node("motion.drive");
    g.set_pos(grow, Pos { x: 450.0, y });
    g.set_param(grow, "channel", CH_SIZE);
    g.set_param(grow, "mode", COMBINE_MULTIPLY);
    wire(g, src, 0, grow, 0)?;
    wire(g, span, 0, grow, 1)?;

    let place = g.add_node("motion.move");
    g.set_pos(place, Pos { x: 600.0, y });
    g.set_param(place, "dy", offset_y);
    wire(g, grow, 0, place, 0)?;
    Some(place)
}

/// `<fileira> → motion.proximity(radius) → value.attribute(<coluna>)` — o par que
/// toda banda de baixo usa, devolvendo `(o stream medido, o campo de valor)`.
fn measure(g: &mut Graph, head: NodeId, x: f32, y: f32, column: &str) -> Option<(NodeId, NodeId)> {
    let prox = g.add_node("motion.proximity");
    g.set_pos(prox, Pos { x, y });
    g.set_param(prox, "radius", RADIUS);
    wire(g, head, 0, prox, 0)?;

    let attr = g.add_node("value.attribute");
    g.set_pos(attr, Pos { x: x + 150.0, y });
    g.set_param(attr, "mode", 0.0);
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, column);
    wire(g, prox, 0, attr, 0)?;
    Some((prox, attr))
}

/// Termina a banda num `motion.output`.
///
/// ⚠️ **O laço de render RE-RESOLVE os sinks a cada quadro** a partir dos nós de
/// saída do grafo (`motion_bridge.rs`), então o que este construtor devolve serve
/// aos GATES e é descartado pelo app: uma cadeia sem Output cozinha certo, passa em
/// toda a suíte e **desenha NADA na tela**.
fn sink(g: &mut Graph, head: NodeId, x: f32, y: f32) -> Option<NodeId> {
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x, y });
    wire(g, head, 0, out, 0)?;
    Some(out)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_proximity_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(BANDS);

    for band in 0..BANDS {
        let y = (band as f32) * -BAND_GAP;
        let head = row(g, band)?;
        let x = 750.0;

        // As bandas PARES (0, 2, 4) são o CONTROLE: a fileira crua.
        let tail = match band {
            0 | 2 | 4 => head,

            // SCALE: `size *= 1 - overlap`.
            1 => {
                let (prox, attr) = measure(g, head, x, y, "overlap")?;
                let inv = g.add_node("value.map_range");
                g.set_pos(inv, Pos { x: x + 300.0, y });
                g.set_param(inv, "in_lo", 0.0);
                g.set_param(inv, "in_hi", 1.0);
                g.set_param(inv, "out_lo", 1.0);
                g.set_param(inv, "out_hi", 0.0);
                wire(g, attr, 0, inv, 0)?;

                let drive = g.add_node("motion.drive");
                g.set_pos(drive, Pos { x: x + 450.0, y });
                g.set_param(drive, "channel", CH_SIZE);
                g.set_param(drive, "mode", COMBINE_MULTIPLY);
                wire(g, prox, 0, drive, 0)?;
                wire(g, inv, 0, drive, 1)?;
                drive
            }

            // HIDE: `falloff = overlap`, e o cull INVERTIDO mantém quem tem ~zero.
            3 => {
                let (prox, attr) = measure(g, head, x, y, "overlap")?;
                let drive = g.add_node("motion.drive");
                g.set_pos(drive, Pos { x: x + 300.0, y });
                g.set_param(drive, "channel", CH_FALLOFF);
                g.set_param(drive, "mode", COMBINE_SET);
                wire(g, prox, 0, drive, 0)?;
                wire(g, attr, 0, drive, 1)?;

                let cull = g.add_node("motion.cull");
                g.set_pos(cull, Pos { x: x + 450.0, y });
                g.set_param(cull, "mode", CULL_MODE_FALLOFF);
                g.set_param(cull, "amount", HIDE_EPS);
                g.set_param(cull, "invert", 1.0);
                wire(g, drive, 0, cull, 0)?;
                cull
            }

            // A CONTAGEM: o tamanho DEIXA de ser o gradiente e vira um medidor de
            // quantos vizinhos cada peça toca — por isso `Set`, e não `Multiply`.
            _ => {
                let (prox, attr) = measure(g, head, x, y, "neighbours")?;
                let meter = g.add_node("value.map_range");
                g.set_pos(meter, Pos { x: x + 300.0, y });
                g.set_param(meter, "in_lo", 0.0);
                g.set_param(meter, "in_hi", COUNT_HI);
                g.set_param(meter, "out_lo", COUNT_SIZE_LO);
                g.set_param(meter, "out_hi", COUNT_SIZE_HI);
                wire(g, attr, 0, meter, 0)?;

                let drive = g.add_node("motion.drive");
                g.set_pos(drive, Pos { x: x + 450.0, y });
                g.set_param(drive, "channel", CH_SIZE);
                g.set_param(drive, "mode", COMBINE_SET);
                wire(g, prox, 0, drive, 0)?;
                wire(g, meter, 0, drive, 1)?;
                drive
            }
        };

        sinks.push(sink(g, tail, 1500.0, y)?);
    }

    Some(sinks)
}

pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1  CONTROLE  (o tamanho cresce da esquerda p/ a direita: as da direita SOBREPOEM)",
    "2  SCALE por composicao  (as grandes encolhem ate' apenas se tocar; as pequenas ficam)",
    "3  CONTROLE  (a mesma fileira)",
    "4  HIDE por composicao  (quem se sobrepoe SAI: sobra o punhado livre da esquerda)",
    "5  CONTROLE  (a mesma fileira)",
    "6  CONTAGEM de vizinhos  (o tamanho vira um medidor: livre pequeno, amontoado inchado)",
];

#[cfg(test)]
#[path = "motion_state_conferencia_demos_proximity_tests.rs"]
mod tests;
