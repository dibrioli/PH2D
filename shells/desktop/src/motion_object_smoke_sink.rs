//! **A cena do ESTILO DO SINK** (`PH2D_MOTION_OBJ_SMOKE=9`, doc 89 folha 17) — quatro
//! pares, e cada par muda **um** param do `motion.output`.
//!
//! | fileira | esquerda | direita |
//! |---|---|---|
//! | pivô | `Pivot X = 0` — cada cópia gira no PRÓPRIO centro | `Pivot X = 0,5` — o ponto de giro salta para a aresta |
//! | sub-UV | a arte inteira, quatro vezes | **um QUARTO dela** em cada cópia |
//! | filtro | `Linear` — o pedaço ampliado sai borrado | **`Nearest`** — sai em blocos duros |
//! | ordem | `Texture` — os dois materiais REAGRUPAM | **`Stream`** — a ordem das linhas ganha |
//!
//! ⚠️ **Esta cena precisa de DOIS objectos com texturas diferentes**, e é por isso que ela
//! vive aqui e não no roteador de `PH2D_GPU_COOK_DEMO`: os demos daquele roteador
//! amostram **um ladrilho BRANCO opaco** (`init.rs`, o `motion_default_uv`), e sobre um
//! ladrilho chapado o filtro, o sub-UV e a mídia mista são todos **invisíveis** — três
//! fileiras que passariam verdes e mudas.
//!
//! ⚠️ **A estrela é VECTORIAL de propósito.** A membrana assa-a numa tile, então ela tem
//! **texels a sério** — que é o que faz `Nearest` e `Linear` diferirem. Os ladrilhos do
//! átlas de demo são cores CHAPADAS: ampliá-los 10× não mostra filtro nenhum.
//!
//! ⚠️ **A ordem só se vê com o stream a ALTERNAR de textura.** Duas cadeias concatenadas
//! dariam `A,A,A,B,B,B` — e aí a ordem das linhas e o agrupamento por textura são a MESMA
//! coisa, e o par sairia igual dos dois lados. Por isso as duas grelhas nascem
//! **entrelaçadas em `x`** e passam por um `motion.sort(key = X)`: a fileira fica
//! `A,B,A,B,A,B`, as cópias sobrepõem-se, e aí `Texture` empilha um material inteiro por
//! cima do outro enquanto `Stream` faz a escada da esquerda para a direita.

use super::{DEMO_TILE_KEY, OBJECT};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O nome do SEGUNDO objecto — o que dá a fileira da ordem a sua outra textura.
pub(crate) const CHIP: &str = "Chip";

/// O `channel` do `motion.oscillator` que escreve `rot` (a escada de `channel_column`).
const OSC_CHANNEL_ROT: f32 = 2.0;

/// O centro em `x` de cada coluna, e o `y` de cada fileira.
const COL_X: f32 = 3.5;
const ROW_Y: [f32; 4] = [3.1, 1.0, -1.2, -3.4];

fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16) {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .expect("connect");
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], y: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

/// `source.object(name) → duplicator ← grid` — a metade comum de toda fileira.
/// Devolve o `duplicator` (a cabeça da cadeia).
fn stamped(g: &mut Graph, name: &str, grid: &[(&str, f32)], y: f32) -> NodeId {
    let src = node(g, "source.object", &[], y, 0.0);
    g.set_text_param(src, "object", name);
    let gr = node(g, "motion.grid", grid, y + 70.0, 0.0);
    let dup = node(g, "motion.duplicator", &[], y, 210.0);
    wire(g, src, 0, dup, 0);
    wire(g, gr, 0, dup, 1);
    dup
}

/// Põe a cadeia no seu quadrante e termina-a num `motion.output` com o estilo dado.
///
/// ⚠️ **A colocação entra ANTES do sink e DEPOIS de tudo o que é campo** — a lei que a
/// cena `=73` pagou: um deslocamento multiplicado por uma máscara estica a fileira por
/// cima das vizinhas. Aqui não há máscara, e a ordem é a mesma de propósito.
fn sink(g: &mut Graph, head: NodeId, row: usize, right: bool, style: &[(&str, f32)]) -> NodeId {
    let y = row as f32 * 240.0;
    let mv = node(
        g,
        "motion.move",
        &[
            ("dx", if right { COL_X } else { -COL_X }),
            ("dy", ROW_Y[row]),
        ],
        y,
        420.0,
    );
    wire(g, head, 0, mv, 0);
    let out = node(g, "motion.output", style, y, 600.0);
    wire(g, mv, 0, out, 0);
    out
}

/// **Fileira 1 — o PIVÔ.** Cinco cópias, cada uma com a sua rotação (estática: o
/// `frequency = 0` faz a fase ser só o escalonamento por índice, então o relógio não
/// mexe nada). Com o pivô ao centro elas giram no lugar; com ele na aresta, o ponto de
/// giro salta e a fileira abre-se em leque.
fn row_pivot(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        let dup = stamped(
            g,
            OBJECT,
            &[("rows", 1.0), ("cols", 5.0), ("gap_x", 1.35)],
            0.0,
        );
        let osc = node(
            g,
            "motion.oscillator",
            &[
                ("channel", OSC_CHANNEL_ROT),
                ("amplitude", 70.0),
                ("frequency", 0.0),
                ("phase_stagger", 0.21),
            ],
            0.0,
            320.0,
        );
        wire(g, dup, 0, osc, 0);
        let style: &[(&str, f32)] = if right { &[("pivot_x", 0.5)] } else { &[] };
        sinks.push(sink(g, osc, 0, right, style));
    }
}

/// **Fileira 2 — o SUB-UV.** Quatro cópias; à direita cada uma mostra um quarto da arte
/// (`stagger = 1` ⇒ a célula anda uma por elemento).
fn row_sub_uv(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        let dup = stamped(
            g,
            OBJECT,
            &[("rows", 1.0), ("cols", 4.0), ("gap_x", 1.35)],
            240.0,
        );
        let head = if right {
            let uv = node(
                g,
                "motion.sub_uv",
                &[("cols", 2.0), ("rows", 2.0), ("stagger", 1.0)],
                240.0,
                320.0,
            );
            wire(g, dup, 0, uv, 0);
            uv
        } else {
            dup
        };
        sinks.push(sink(g, head, 1, right, &[]));
    }
}

/// **Fileira 3 — o FILTRO.** UMA cópia grande de um pedaço pequeno da arte: a ampliação
/// é o que torna o filtro visível, e o sub-UV é o que fabrica a ampliação.
///
/// ⚠️ **A célula é a mesma dos dois lados** — o que muda é só o sampler. Uma célula
/// diferente faria o par mostrar duas artes e ninguém saberia a que atribuir a diferença.
fn row_filter(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        let dup = stamped(g, OBJECT, &[("rows", 1.0), ("cols", 1.0)], 480.0);
        let uv = node(
            g,
            "motion.sub_uv",
            &[("cols", 10.0), ("rows", 10.0), ("cell", 44.0)],
            480.0,
            320.0,
        );
        wire(g, dup, 0, uv, 0);
        let big = node(g, "motion.scale", &[("amount", 3.4)], 480.0, 380.0);
        wire(g, uv, 0, big, 0);
        // 1 = Nearest · 2 = Linear (a escada dos tags de `FilterMode`).
        sinks.push(sink(
            g,
            big,
            2,
            right,
            &[("filter", if right { 1.0 } else { 2.0 })],
        ));
    }
}

/// **Fileira 4 — a ORDEM.** Duas grelhas ENTRELAÇADAS em `x`, juntadas e ordenadas por
/// `x`, de modo que o stream alterne de textura cópia a cópia.
fn row_sort(g: &mut Graph, sinks: &mut Vec<NodeId>) {
    for right in [false, true] {
        // As duas grelhas partilham o passo e diferem por meio passo — é isso que as
        // entrelaça depois do `sort`.
        let a = stamped(
            g,
            OBJECT,
            &[("rows", 1.0), ("cols", 3.0), ("gap_x", 1.1)],
            720.0,
        );
        let b = stamped(
            g,
            CHIP,
            &[("rows", 1.0), ("cols", 3.0), ("gap_x", 1.1)],
            790.0,
        );
        let off = node(g, "motion.move", &[("dx", 0.55)], 790.0, 320.0);
        wire(g, b, 0, off, 0);
        let mix = node(g, "motion.combine", &[], 720.0, 360.0);
        wire(g, a, 0, mix, 0);
        wire(g, off, 0, mix, 1);
        // `key = 1` é o X (a escada do `motion.sort`).
        let srt = node(g, "motion.sort", &[("key", 1.0)], 720.0, 400.0);
        wire(g, mix, 0, srt, 0);
        sinks.push(sink(
            g,
            srt,
            3,
            right,
            &[("sort", if right { 1.0 } else { 0.0 })],
        ));
    }
}

/// O SEGUNDO objecto da cena — um sprite de ladrilho chapado, que é a OUTRA textura.
pub(super) fn spawn_chip(sim: &mut ph2d_ecs::SimWorld) {
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::atlas(DEMO_TILE_KEY, [0.85, 0.85], [1.0, 1.0, 1.0, 1.0]),
        Name::new(CHIP),
    ));
}

/// Monta as oito cadeias e devolve os oito sinks — **função PURA sobre o grafo**, para o
/// gate a poder montar sem `AppGfx` nenhum.
pub(crate) fn build_sink_style_graph(g: &mut Graph) -> Vec<NodeId> {
    let mut sinks = Vec::with_capacity(8);
    row_pivot(g, &mut sinks);
    row_sub_uv(g, &mut sinks);
    row_filter(g, &mut sinks);
    row_sort(g, &mut sinks);
    sinks
}

/// A cena do modo `=9`, montada no frame 6 (a entidade da estrela nasce no `sync`).
pub(super) fn run(gfx: &mut crate::AppGfx) {
    let sinks = build_sink_style_graph(&mut gfx.motion.doc.graph);
    gfx.motion.sinks.extend(sinks);
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =9] O SINK ganhou o ESTILO DE DESENHO (doc 89 folha 17).
  Quatro fileiras, cada uma com um par -- ESQUERDA = como era, DIREITA = o param novo.
  1 PIVO   : a esquerda cada estrela gira no proprio centro; a direita o ponto de
             giro esta' na aresta, e a fileira abre-se em leque.
  2 SUB-UV : a esquerda quatro estrelas inteiras; a direita cada copia mostra um
             QUARTO da arte (2x2).
  3 FILTRO : o MESMO pedacinho ampliado -- a esquerda `Linear` (borrado), a direita
             `Nearest` (blocos duros). E' o modo de pixel-art.
  4 ORDEM  : estrelas e quadrados alternados e sobrepostos -- a esquerda `Texture`
             (um material inteiro por cima do outro), a direita `Stream` (a escada
             da esquerda para a direita, que e' a ordem das linhas).
  > clique num no' Output e mexa em Pivot X / Filter / Sort.
  (!) DEU ERRADO se algum par sair igual dos dois lados, ou se alguma fileira sumir."
    );
}

#[cfg(test)]
#[path = "motion_object_smoke_sink_tests.rs"]
mod tests;
