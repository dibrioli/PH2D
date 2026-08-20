//! **OS DOIS DEFEITOS DE JUNÇÃO** — a cena `=62` (doc 89, folha 08).
//!
//! As duas células desta cena descrevem **comportamento errado**, não knob ausente, e é por
//! isso que vêm antes do resto da folha.
//!
//! **(1) A ESCALA DO PONTO nunca chegava ao carimbo.** O `motion.duplicator` somava `P` e
//! `rot` e deitava fora **toda** outra coluna do ponto — então um espalhamento que já produziu
//! um `size` por ponto perdia-o ao carimbar. Medido: pontos com `size = [0, 4, 8, 12]` davam
//! uma saída **sem coluna `size` nenhuma**. As três referências são unânimes (Houdini
//! `pscale`, Blender o socket `Scale`, Cavalry `Shape Scale`).
//!
//! **(2) O `motion.combine` não renumerava.** Cada fonte escreve o seu `Index = 0..n−1` e
//! `Count = n`, e o `concat` copia-os verbatim — então `grid(9) + grid(4)` devolve **13**
//! linhas com `Index = [0..8, 0..3]`: as duas colunas de identidade **mentem**, e todo efeito
//! dirigido por índice a jusante lê a lista como se fossem duas.
//!
//! ⚠️ **A banda que revela o (2) precisa de um consumidor de ÍNDICE, e nem todos são.** O
//! `motion.color_ramp` com o `t` desligado usa a posição da linha, não a coluna — ele mostraria
//! um degradê perfeito sobre a lista mentirosa. Quem lê a COLUNA é o `motion.tint` em modo
//! gradiente, e é por isso que ele está aqui.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 3.2;
/// Quantos pontos a fileira do carimbo tem.
const STAMPS: f32 = 7.0;
/// As duas grelhas que a junção mistura — 9 + 4 = 13 linhas.
const LEFT: f32 = 3.0;
const RIGHT: f32 = 2.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Termina a banda num `motion.output`, deslocada para o seu lugar.
fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 220.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// **A banda do CARIMBO**: uma fileira de pontos cujo `size` cresce ao longo dela, carimbada
/// com um quadrado. `point_scale` decide se essa escala chega ao carimbo.
fn stamp_band(g: &mut Graph, point_scale: f32, y: f32) -> Option<NodeId> {
    // A forma: um ponto só, que o carimbo replica.
    let shape = g.add_node("motion.grid");
    g.set_pos(shape, Pos { x: 0.0, y });
    g.set_param(shape, "rows", 1.0);
    g.set_param(shape, "cols", 1.0);

    // Os pontos, e a escala POR PONTO que um espalhamento produz.
    let pts = g.add_node("motion.grid");
    g.set_pos(
        pts,
        Pos {
            x: 0.0,
            y: y + 90.0,
        },
    );
    g.set_param(pts, "rows", 1.0);
    g.set_param(pts, "cols", STAMPS);
    g.set_param(pts, "gap_x", 1.3);
    let ramp = g.add_node("value.instance_field");
    g.set_pos(
        ramp,
        Pos {
            x: 220.0,
            y: y + 90.0,
        },
    );
    g.set_param(ramp, "mode", 1.0); // Ramp 0..1
    wire(g, pts, 0, ramp, 0)?;
    let sized = g.add_node("motion.drive");
    g.set_pos(
        sized,
        Pos {
            x: 440.0,
            y: y + 90.0,
        },
    );
    g.set_param(sized, "channel", 3.0); // Size
    g.set_param(sized, "mode", 1.0); // Set
    g.set_param(sized, "scale", 1.6);
    wire(g, pts, 0, sized, 0)?;
    wire(g, ramp, 0, sized, 1)?;

    let dup = g.add_node("motion.duplicator");
    g.set_pos(dup, Pos { x: 660.0, y });
    g.set_param(dup, "point_scale", point_scale);
    wire(g, shape, 0, dup, 0)?;
    wire(g, sized, 0, dup, 1)?;
    Some(dup)
}

/// **A banda da JUNÇÃO**: duas grelhas misturadas e tingidas por um gradiente que lê a coluna
/// `Index`. `reindex` decide se essa coluna diz a verdade sobre a lista junta.
fn join_band(g: &mut Graph, reindex: f32, y: f32) -> Option<NodeId> {
    let a = g.add_node("motion.grid");
    g.set_pos(a, Pos { x: 0.0, y });
    g.set_param(a, "rows", LEFT);
    g.set_param(a, "cols", LEFT);
    g.set_param(a, "gap_x", 0.6);
    g.set_param(a, "gap_y", 0.6);

    let b = g.add_node("motion.grid");
    g.set_pos(
        b,
        Pos {
            x: 0.0,
            y: y + 90.0,
        },
    );
    g.set_param(b, "rows", RIGHT);
    g.set_param(b, "cols", RIGHT);
    g.set_param(b, "gap_x", 0.6);
    g.set_param(b, "gap_y", 0.6);
    let shift = g.add_node("motion.move");
    g.set_pos(
        shift,
        Pos {
            x: 220.0,
            y: y + 90.0,
        },
    );
    g.set_param(shift, "dx", 2.6);
    wire(g, b, 0, shift, 0)?;

    let join = g.add_node("motion.combine");
    g.set_pos(join, Pos { x: 440.0, y });
    g.set_param(join, "reindex", reindex);
    wire(g, a, 0, join, 0)?;
    wire(g, shift, 0, join, 1)?;

    // ⚠️ O `motion.tint` em GRADIENTE lê as colunas `Index`/`Count` — é o consumidor que
    // torna a mentira visível. Preto → laranja ao longo da lista.
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 660.0, y });
    g.set_param(tint, "mode", 1.0); // Gradient
    g.set_param(tint, "r", 0.15);
    g.set_param(tint, "g", 0.15);
    g.set_param(tint, "b", 0.2);
    g.set_param(tint, "r2", 1.0);
    g.set_param(tint, "g2", 0.55);
    g.set_param(tint, "b2", 0.1);
    wire(g, join, 0, tint, 0)?;
    Some(tint)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_join_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    for (row, scale) in [0.0f32, 1.0].into_iter().enumerate() {
        let gy = row as f32 * 260.0;
        let head = stamp_band(g, scale, gy)?;
        sinks.push(place(
            g,
            head,
            BAND_DY * 1.5 - row as f32 * BAND_DY,
            900.0,
            gy,
        )?);
    }
    for (row, re) in [0.0f32, 1.0].into_iter().enumerate() {
        let gy = (row + 2) as f32 * 260.0;
        let head = join_band(g, re, gy)?;
        sinks.push(place(
            g,
            head,
            -BAND_DY * 0.5 - row as f32 * BAND_DY,
            900.0,
            gy,
        )?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CARIMBO, Point Scale 0 -- todos do MESMO tamanho (o defeito)",
        "CARIMBO, Point Scale 1 -- o tamanho de cada ponto CHEGA",
        "JUNCAO, Reindex off -- a cor REINICIA no meio da lista (o defeito)",
        "JUNCAO, Reindex on -- um degrade so', sobre as 13",
    ]
    .into_iter()
    .enumerate()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_join_tests.rs"]
mod tests;
