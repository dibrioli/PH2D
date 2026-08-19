//! **O ESPAÇO DO CAMPO** — a cena `=60`, a folha 06 linha 20 (o **último P1** dela).
//!
//! Quatro blocos com o **mesmo** `motion.noise`: mesma semente, mesma amplitude, mesma
//! oitava, mesma escala. O que muda é **onde o campo é amostrado**.
//!
//! 1. **CONTROLE** — o campo de sempre: manchas redondas.
//! 2. **RODADO 45°** — as mesmas manchas, na diagonal.
//! 3. **COMPRIMIDO no Y** — `uniform` desligado e o eixo Y com escala própria e maior: o
//!    mesmo passo de mundo cobre mais campo, então as manchas ficam **baixas e largas** —
//!    listras deitadas. ⚠️ *Escala maior = feição menor*, o contrário do que o nome sugere.
//! 4. **OS DOIS** — comprimido e **depois** rodado: as listras saem **na diagonal**. É a
//!    banda que prova a ORDEM — se o nó rodasse primeiro, elas sairiam deitadas como as da
//!    banda 3.
//!
//! ## ⚠️ O campo é o TAMANHO do ponto, e a primeira versão desta cena falhou por isso
//!
//! O smoke do Enio reprovou-a com *"não tem nada girado nem na diagonal"*, e a medição achou
//! **três números errados ao mesmo tempo** — todos do mesmo tipo, e nenhum visível num teste:
//!
//! | | medido na v1 | consequência |
//! |---|---|---|
//! | tamanho do ponto ÷ vão | **1,0 ÷ 0,32 = 3,1×** | ⚠️ um sprite sem coluna `size` desenha a **1,0** (o `SIZE_IDENTITY` do shell): os quadrados cobriam 3× o vizinho e o bloco era uma **placa sólida** |
//! | deslocamento ÷ vão | **1,31×** | as peças cruzavam as fileiras vizinhas |
//! | manchas no bloco | **2,5** | duas manchas não mostram rotação — não há padrão para virar |
//!
//! E um quarto, de desenho: o campo empurrava só em **Y**, e *"o padrão girou"* lido a partir
//! de pontos que sobem e descem é quase ilegível. A cura foi mudar o que o campo escreve: ele
//! agora dirige o **`Size`**, e o bloco vira um **retrato do campo** em pontos. Uma rotação de
//! 45° numa imagem é óbvia; num deslocamento vertical, não é.
//!
//! *Um gate mede o que a cena PRODUZ; só o olho mede o que ela MOSTRA.*
//!
//! ⚠️ **Isto julga-se PARADO.** O `speed` é zero nas quatro.
//!
//! ⚠️ **O que a cena NÃO tem, e é decisão MEDIDA:** o *offset* do campo já sai de
//! `motion.move(+d) → noise → motion.move(−d)` (a pose volta com `|Δx| = 0`), e o *scale
//! uniforme* **já era o param `scale`** — o sanduíche `motion.transform(s) … (1/s)` é
//! bit-a-bit `scale·s` com a amplitude dividida por `s` (pior `|Δy|`: **0,000000**). Uma
//! banda a mostrar qualquer um dos dois ensinaria um knob que não existe
//! (`measure_noise_space`).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Lado de cada bloco, em pontos.
const SIDE: f32 = 21.0;
/// O vão entre pontos, em unidades de mundo. O bloco mede `(21−1)·0,26 = 5,20`.
const GAP: f32 = 0.26;
/// O tamanho BASE de um ponto, escrito por um `motion.scale` antes do ruído.
///
/// ⚠️ **Ele existe porque a ausência da coluna `size` não é «pequeno», é `1,0`** — o
/// `SIZE_IDENTITY` que o shell passa ao `lower_to_instances`. Sem esta linha o bloco é uma
/// placa de quadrados sobrepostos, que foi como a v1 chegou ao smoke.
const DOT: f32 = 0.16;
/// Quanto o campo soma ao tamanho. ⚠️ `DOT + AMPLITUDE = 0,25` contra um vão de `0,26`:
/// **o maior ponto ainda não toca o vizinho** (0,96 do vão), e o contraste entre o menor e o
/// maior é **3,6×**, que é o que faz o campo se ler como imagem.
const AMPLITUDE: f32 = 0.09;
/// A escala espacial do campo. ⚠️ Escolhida pelo NÚMERO DE MANCHAS, não pelo gosto:
/// `5,20 · 0,77 = 4,0` células de ruído atravessam o bloco. Com as **2,5** da v1 não havia
/// padrão suficiente para uma rotação se ver.
const SCALE: f32 = 0.77;
/// A escala do eixo Y quando `uniform` está desligado — **4×** o `SCALE`, para «listra» se
/// ler como listra (medido: a razão de variação `dx/dy` cai de 0,976 para 0,341).
const SCALE_Y: f32 = SCALE * 4.0;
/// O ângulo das bandas 2 e 4, em graus. ⚠️ **45 e não 90:** com 90° um campo isotrópico
/// parece o mesmo (as manchas trocam de eixo e o olho não tem referência).
const TURN: f32 = 45.0;
/// Meia distância entre os centros dos blocos. Os quatro em **2×2** ocupam `12 × 12` de
/// mundo — dentro do que as cenas aprovadas desta conferência já ocupavam.
const BAND: f32 = 3.4;

fn wire(g: &mut Graph, from: NodeId, to: NodeId) -> Option<()> {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .ok()
}

/// Um bloco de `SIDE × SIDE` pontos, já encolhidos ao tamanho base.
fn block(g: &mut Graph, x: f32, y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x, y });
    for (k, v) in [
        ("rows", SIDE),
        ("cols", SIDE),
        ("gap_x", GAP),
        ("gap_y", GAP),
    ] {
        g.set_param(grid, k, v);
    }
    let sc = g.add_node("motion.scale");
    g.set_pos(sc, Pos { x: x + 200.0, y });
    g.set_param(sc, "amount", DOT);
    wire(g, grid, sc)?;
    Some(sc)
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
    // ⚠️ **Size, não Y.** O campo É a imagem — ver a nota do cabeçalho.
    g.set_param(n, "channel", 3.0);
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
fn place(g: &mut Graph, head: NodeId, dx: f32, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", dy);
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
    // Em **2×2**: os quatro cabem na tela e cada um tem os outros três por perto para
    // comparar. Numa fileira de quatro o olho compara só com o vizinho.
    for (i, (rotation, stretched)) in [(0.0, false), (TURN, false), (0.0, true), (TURN, true)]
        .into_iter()
        .enumerate()
    {
        let gy = i as f32 * 220.0;
        let src = block(g, 0.0, gy)?;
        let ns = noise(g, src, rotation, stretched, 440.0, gy)?;
        let dx = if i % 2 == 0 { -BAND } else { BAND };
        let dy = if i < 2 { BAND } else { -BAND };
        sinks.push(place(g, ns, dx, dy, 660.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "EM CIMA 'A ESQUERDA -- CONTROLE: manchas redondas",
        "EM CIMA 'A DIREITA  -- RODADO 45 graus: as MESMAS manchas, na diagonal",
        "EM BAIXO 'A ESQUERDA -- COMPRIMIDO no Y: listras DEITADAS",
        "EM BAIXO 'A DIREITA  -- OS DOIS: listras na DIAGONAL (a ordem que o no' aplica)",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem da cena cita: `(ângulo, escala, escala do Y)`.
pub(crate) fn knobs() -> (f32, f32, f32) {
    (TURN, SCALE, SCALE_Y)
}

/// O lado do bloco, em pontos — o gate precisa dele para andar na grade.
///
/// ⚠️ `#[cfg(test)]` porque **só o gate o lê**: a mensagem da cena cita o ângulo e as escalas
/// (o `knobs`), e um acessor público que ninguém chama fora do teste é código morto que o
/// clippy do ship apanha.
#[cfg(test)]
pub(crate) fn side() -> usize {
    SIDE as usize
}

/// O vão entre pontos — a régua contra a qual o gate mede se eles se tocam.
#[cfg(test)]
pub(crate) fn gap() -> f32 {
    GAP
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_field_space_tests.rs"]
mod tests;
