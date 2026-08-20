//! **A FAMÍLIA DOS CAMPOS** — a cena `=66` (doc 89, folha 10: o anel, a força com sinal e o
//! truncamento).
//!
//! Três pares, um por nó. O mesmo grafo dos dois lados de cada par; só um número muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `field.radial_sweep` | o disco de sempre | **um `Inner Radius`** — um ANEL |
//! | `field.box` | `Strength 1`, a máscara de sempre | **`Strength −1`** — o campo EMPURRA |
//! | `field.combine` | `Add` com `Clamp` — a soma satura | **`Clamp` desligado** — ela passa de 1 |
//!
//! ⚠️ **O campo é invisível: o que se vê é o CONSUMIDOR.** Cada banda passa a máscara a um
//! `motion.scale`, que a lê como `1 + (amount − 1)·falloff` — então onde o campo vale 1 a peça
//! dobra, onde vale 0 ela fica, e onde vale **2** (o que só existe com o truncamento
//! desligado) ela triplica. É essa terceira figura que o par do `Clamp` existe para mostrar.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O passo da grelha.
const PITCH: f32 = 1.0;
/// O lado da peça em repouso. ⚠️ No pico o campo pode valer **2** (soma sem truncar), e o
/// `motion.scale` leva a peça a `1 + (2 − 1)·2 = 3` vezes — `PIECE · 3` tem de caber no passo.
const PIECE: f32 = 0.3;
/// Quanto uma peça no cheio do campo cresce.
const GROWTH: f32 = 2.0;
/// O lado da grelha de cada banda.
const SIDE: f32 = 9.0;
/// O raio externo dos campos radiais.
const RADIUS: f32 = 4.2;
/// O raio interno que a banda do anel autora.
const INNER: f32 = 2.4;
/// O vão entre as duas colunas e entre as três linhas.
///
/// ⚠️ **Medidos contra o LADO da banda, que é `(SIDE − 1) · PITCH = 8`** — não escolhidos por
/// gosto. Com um vão vertical menor que isso as linhas montavam umas por cima das outras na
/// mesma coluna (a primeira versão usava `5,4` e as linhas 1 e 2 partilhavam a faixa
/// `y ∈ [1,4; 4]`), e duas bandas sobrepostas leem-se como uma banda com o dobro das peças.
const GAP_X: f32 = 5.6;
const GAP_Y: f32 = 9.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// A grelha da banda, já encolhida para caber no passo **e já no seu quadrante**.
///
/// ⚠️ **O deslocamento vem ANTES do campo, e isto é correção e não arrumação.** Todo
/// modificador desta família honra a coluna `falloff` — o `motion.move` inclusive. Um
/// `move` colocado depois do campo é **mascarado por ele**: as peças no cheio do campo
/// andam o vão inteiro e as de fora ficam onde estavam, e a banda **espalha-se** em vez de
/// se mudar. A primeira versão desta cena fazia isso, e os quatro gates apanharam-no antes
/// do Enio (medido: um vão de `5,6` produzia um deslocamento efectivo de `4,6`, que é o vão
/// menos a peça que não andou). *Um campo dirigido por coordenadas de mundo age no MESMO
/// quadro em que a banda já está.*
fn source(g: &mut Graph, y: f32, at: [f32; 2]) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y });
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", PITCH);
    g.set_param(grid, "gap_y", PITCH);
    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 140.0, y });
    g.set_param(fit, "amount", PIECE);
    let _ = wire(g, grid, 0, fit, 0);
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 260.0, y });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    let _ = wire(g, fit, 0, mv, 0);
    mv
}

/// O consumidor: a máscara vira TAMANHO e a banda ganha cor. O layout já aconteceu, na
/// [`source`] — ver o ⚠️ dela.
fn finish(g: &mut Graph, head: NodeId, rgb: [f32; 3], ey: f32) -> Option<NodeId> {
    let grow = g.add_node("motion.scale");
    g.set_pos(grow, Pos { x: 660.0, y: ey });
    g.set_param(grow, "amount", GROWTH);
    wire(g, head, 0, grow, 0)?;

    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 800.0, y: ey });
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    wire(g, grow, 0, tint, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1080.0, y: ey });
    wire(g, tint, 0, out, 0)?;
    Some(out)
}

/// **O ANEL**: um `field.radial_sweep` de disco cheio, com ou sem buraco.
fn ring_band(g: &mut Graph, inner: f32, ey: f32, at: [f32; 2]) -> Option<NodeId> {
    let src = source(g, ey, at);
    let f = g.add_node("field.radial_sweep");
    g.set_pos(f, Pos { x: 400.0, y: ey });
    g.set_param(f, "center_x", at[0]);
    g.set_param(f, "center_y", at[1]);
    g.set_param(f, "radius", RADIUS);
    g.set_param(f, "inner_radius", inner);
    g.set_param(f, "end_angle", 360.0); // o disco inteiro — a lei aqui é RADIAL
    g.set_param(f, "soft", 0.12);
    wire(g, src, 0, f, 0)?;
    Some(f)
}

/// **A FORÇA COM SINAL**: o mesmo `field.box`, com a máscara ou com o empurrão.
fn strength_band(g: &mut Graph, strength: f32, ey: f32, at: [f32; 2]) -> Option<NodeId> {
    let src = source(g, ey, at);
    let f = g.add_node("field.box");
    g.set_pos(f, Pos { x: 400.0, y: ey });
    g.set_param(f, "center_x", at[0]);
    g.set_param(f, "center_y", at[1]);
    g.set_param(f, "width", 4.0);
    g.set_param(f, "height", 4.0);
    g.set_param(f, "soft", 0.6);
    g.set_param(f, "strength", strength);
    wire(g, src, 0, f, 0)?;
    Some(f)
}

/// **O TRUNCAMENTO**: duas caixas que se cruzam, somadas.
///
/// ⚠️ As duas TÊM de se sobrepor — é só no cruzamento que a soma passa de 1, e é só ali que o
/// toggle muda um número. Duas caixas separadas dariam o mesmo desenho dos dois lados.
fn clamp_band(g: &mut Graph, clamp: f32, ey: f32, at: [f32; 2]) -> Option<NodeId> {
    let src = source(g, ey, at);
    let mut leg = |dx: f32, dy: f32, py: f32| {
        let b = g.add_node("field.box");
        g.set_pos(b, Pos { x: 400.0, y: py });
        g.set_param(b, "width", 5.0);
        g.set_param(b, "height", 5.0);
        g.set_param(b, "soft", 1.2);
        g.set_param(b, "center_x", at[0] + dx);
        g.set_param(b, "center_y", at[1] + dy);
        wire(g, src, 0, b, 0);
        b
    };
    let a = leg(-1.4, 0.0, ey);
    let b = leg(1.4, 0.0, ey + 90.0);
    let c = g.add_node("field.combine");
    g.set_pos(c, Pos { x: 540.0, y: ey });
    g.set_param(c, "mode", 1.0); // Add
    g.set_param(c, "clamp", clamp);
    wire(g, a, 0, c, 0)?;
    wire(g, b, 0, c, 1)?;
    Some(c)
}

/// Monta a cena. Devolve os sinks: anel(disco, anel) · força(+1, −1) · soma(trunca, livre).
pub(crate) fn build_field_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let rgb = [[0.46, 0.72, 1.0], [1.0, 0.74, 0.3], [0.62, 1.0, 0.66]];
    let mut sinks = Vec::with_capacity(6);
    for (row, pair) in [[0.0, INNER], [1.0, -1.0], [1.0, 0.0]]
        .into_iter()
        .enumerate()
    {
        for (col, v) in pair.into_iter().enumerate() {
            let ey = (row * 2 + col) as f32 * 260.0;
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y - row as f32 * GAP_Y,
            ];
            let head = match row {
                0 => ring_band(g, v, ey, at)?,
                1 => strength_band(g, v, ey, at)?,
                _ => clamp_band(g, v, ey, at)?,
            };
            sinks.push(finish(g, head, rgb[row], ey)?);
        }
    }
    Some(sinks)
}

/// Os rótulos das seis bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ANEL, esquerda -- o disco de sempre: as pecas do meio crescem",
        "ANEL, direita -- Inner Radius: o meio volta ao normal e sobra um ANEL",
        "FORCA, esquerda -- Strength 1: a caixa e' quem cresce",
        "FORCA, direita -- Strength -1: a caixa fica quieta e o RESTO cresce mais",
        "SOMA, esquerda -- Add com Clamp: o cruzamento das duas caixas satura",
        "SOMA, direita -- Clamp desligado: o cruzamento passa de 1 e cresce o dobro",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (INNER, GROWTH)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_field_tests.rs"]
mod tests;
