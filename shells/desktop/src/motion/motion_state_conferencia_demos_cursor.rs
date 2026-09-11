//! **O CAMPO SEGUE O RATO** — a cena `=65` (doc 89, folha 08: a célula do `followMouse`).
//!
//! Duas fileiras da mesma grelha, pelo mesmo `motion.falloff`, pelo mesmo `motion.scale`. A
//! única diferença é de onde o CENTRO do campo vem.
//!
//! | banda | o centro do falloff |
//! |---|---|
//! | 1 | **dirigido** pelas duas saídas de um `value.cursor` — o campo anda com o rato |
//! | 2 | o ponto autorado de sempre — parado, e é o controle |
//!
//! ⚠️ **A cena existe para provar uma COMPOSIÇÃO, não um knob.** A célula pedia um
//! `followMouse` no falloff; o que shipou foi um produtor de valor (`value.cursor`) e a rota
//! do **param dirigido** (doc 58), que serve *qualquer* parâmetro de *qualquer* nó. Se um dia
//! alguém puser o toggle no falloff, esta cena é o que mostra que ele já não era preciso.
//!
//! ⚠️ **O par não separa num quadro parado**: com o rato na origem as duas fileiras coincidem,
//! de propósito. O que separa é o MOVIMENTO — por isso o gate coze a cena com o cursor
//! publicado em dois sítios e afirma que a banda 1 muda e a banda 2 não.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O passo da grelha.
const PITCH: f32 = 1.3;
/// O lado da peça em repouso. ⚠️ `PIECE · GROWTH` tem de caber em [`PITCH`] — no pico do
/// campo as peças crescem, e é lá que elas se tapariam.
const PIECE: f32 = 0.4;
/// Quanto uma peça no centro do campo cresce.
const GROWTH: f32 = 2.5;
/// O lado da grelha de cada banda.
const SIDE: f32 = 9.0;
/// O raio do campo.
const RADIUS: f32 = 3.0;
/// O centro autorado da banda de controle.
const STILL: [f32; 2] = [-2.0, 0.0];
/// O vão entre as duas bandas.
const BAND_DY: f32 = 7.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Uma banda: grelha → falloff → escala mascarada pelo campo → tinta → saída.
///
/// Com `follow` ligada, o `center_x`/`center_y` do falloff são **dirigidos** pelas saídas
/// `x`/`y` de um `value.cursor`; desligada, ficam nos números autorados.
fn band(g: &mut Graph, follow: bool, y: f32) -> Option<NodeId> {
    let dy = y_offset(y);
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y });
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", PITCH);
    g.set_param(grid, "gap_y", PITCH);

    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 200.0, y });
    g.set_param(fit, "amount", PIECE);
    wire(g, grid, 0, fit, 0)?;

    // ⚠️ **A banda vai para o seu sítio ANTES do campo, e isso é correção e não arrumação.**
    // O cursor chega em coordenadas de MUNDO; se o `motion.move` viesse depois, o campo
    // compararia posições de um quadro e o artista apontaria noutro — o inchaço nasceria
    // deslocado do ponteiro pelo tamanho exacto do deslocamento da banda. *Um campo dirigido
    // pelo rato tem de agir no MESMO quadro em que o rato é lido.*
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 300.0, y });
    g.set_param(mv, "dy", dy);
    wire(g, fit, 0, mv, 0)?;

    let field = g.add_node("motion.falloff");
    g.set_pos(field, Pos { x: 400.0, y });
    g.set_param(field, "radius", RADIUS);
    g.set_param(field, "center_x", STILL[0]);
    // O controle põe o seu inchaço DENTRO da própria fileira — o centro autorado viaja com a
    // banda, senão ele cairia sobre a vizinha.
    g.set_param(field, "center_y", STILL[1] + dy);
    wire(g, mv, 0, field, 0)?;

    if follow {
        // ⚠️ **UM nó, DUAS saídas** — o `value.cursor` é o primeiro do repo com mais de uma,
        // e é o que evita que seguir um centro 2D custe dois nós para sempre.
        let cursor = g.add_node("value.cursor");
        g.set_pos(
            cursor,
            Pos {
                x: 200.0,
                y: y - 110.0,
            },
        );
        g.drive_param(field, "center_x", (cursor, 0)).ok()?;
        g.drive_param(field, "center_y", (cursor, 1)).ok()?;
    }

    // O campo tem de CHEGAR a alguma coisa que o olho leia: a escala honra a coluna
    // `falloff`, então as peças debaixo do campo incham.
    let grow = g.add_node("motion.scale");
    g.set_pos(grow, Pos { x: 600.0, y });
    g.set_param(grow, "amount", GROWTH);
    wire(g, field, 0, grow, 0)?;

    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 780.0, y });
    g.set_param(tint, "mode", 1.0); // Gradient — as peças grandes também mudam de cor
    g.set_param(tint, "r", 0.36);
    g.set_param(tint, "g", 0.48);
    g.set_param(tint, "b", 0.9);
    g.set_param(tint, "r2", 1.0);
    g.set_param(tint, "g2", 0.78);
    g.set_param(tint, "b2", 0.28);
    wire(g, grow, 0, tint, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1100.0, y });
    wire(g, tint, 0, out, 0)?;
    Some(out)
}

/// A banda de cima sobe meio vão, a de baixo desce meio vão.
fn y_offset(editor_y: f32) -> f32 {
    if editor_y == 0.0 {
        BAND_DY * 0.5
    } else {
        -BAND_DY * 0.5
    }
}

/// Monta a cena. Devolve os sinks: `[segue o rato, parada]`.
pub(crate) fn build_cursor_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![band(g, true, 0.0)?, band(g, false, 320.0)?])
}

/// Os rótulos das duas bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "SEGUE O RATO -- o centro do campo e' dirigido pelo value.cursor",
        "PARADA (o controle) -- o mesmo campo, com o centro autorado",
    ]
    .into_iter()
    .enumerate()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_cursor_tests.rs"]
mod tests;
