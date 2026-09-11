//! **A ORDEM** — a cena `=63` (doc 89, folha 08: a direção arbitrária e a chave como campo).
//!
//! Ordenar não muda onde as peças estão; muda **quem é a primeira**. Isso é invisível até
//! alguém a jusante ler a ordem — então as três bandas passam o mesmo grid pelo mesmo
//! `motion.tint` em gradiente, que lê as colunas de identidade. A COR é a ordem.
//!
//! | banda | chave |
//! |---|---|
//! | 1 | `X` (o eixo de sempre) — a cor corre da esquerda para a direita |
//! | 2 | `X` com `Axis Angle 35°` — ela corre na DIAGONAL |
//! | 3 | `Weight`, alimentada por um `value.noise` — ela corre por um campo autorado |
//!
//! ⚠️ **A banda 2 custava TRÊS nós antes** (`rotate(θ) → sort(X) → rotate(−θ)`), e a 3 não
//! era exprimível: não havia porta por onde uma chave arbitrária entrasse.
//!
//! ⚠️ **A cena depende do `reindex` do `motion.sort`, e não o autora** — de propósito. Ela
//! larga o nó como o artista o larga, então o que ela pinta é o DEFAULT. Na primeira versão
//! (2026-08-19) as três bandas saíam com a MESMA pintura, e o Enio viu-o antes de qualquer
//! gate: o `motion.tint` em gradiente lê a coluna `Index`, o `sort` levava-a consigo, e a
//! ordenação nunca chegava ao pixel. Se um dia o default mudar, esta cena volta a ser o
//! sintoma — que é a única razão para não o fixar aqui.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 3.4;
/// O lado da grelha de cada banda.
const SIDE: f32 = 7.0;
/// O PASSO da grelha — centro a centro, que é o que o `gap_*` do `motion.grid` significa.
const PITCH: f32 = 0.42;
/// **O LADO DE CADA PEÇA, e ele TEM de caber no passo.**
///
/// ⚠️ **Sem isto a cena mostra OCLUSÃO em vez de ordem, e foi o segundo smoke que o Enio
/// reprovou (2026-08-19).** Uma instância sem coluna `size` é desenhada com
/// [`SIZE_IDENTITY`](ph2d_nodegraph::attr::SIZE_IDENTITY) = **1,0 unidade de mundo** — 2,4×
/// o passo desta grelha, ou seja **5,7 peças empilhadas em cada ponto**. Nas bandas 1 e 2
/// isso não aparece porque quem tapa é o vizinho ESPACIAL, que tem quase a mesma cor; na
/// banda 3 a ordem é embaralhada, a metade amarela (desenhada por último) cobre a azul, e o
/// resultado é um campo quase todo amarelo com manchas — a oclusão a ler-se como se a
/// ordenação estivesse errada. *Numa cena cujo assunto é a ORDEM DE SAÍDA, a ordem de
/// desenho é a mesma coisa: ela não pode ter permissão de esconder nada.*
const PIECE: f32 = 0.34;
/// O ângulo da banda diagonal, em graus.
const DIAGONAL: f32 = 35.0;
/// Os índices do enum `key` do `motion.sort` (ver `ph2d_node_motion_sort`).
const KEY_X: f32 = 1.0;
const KEY_WEIGHT: f32 = 5.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Uma banda: grelha → (peso opcional →) `motion.sort` → `motion.tint` gradiente → saída.
fn band(g: &mut Graph, key: f32, axis: f32, weighted: bool, y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 0.0, y });
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", PITCH);
    g.set_param(grid, "gap_y", PITCH);

    // A peça encolhida para caber no passo — ver [`PIECE`]. Fica ANTES da ordenação de
    // propósito: é uma propriedade da fonte, e assim a coluna `size` também é permutada,
    // que é o que uma peça que viaja inteira tem de fazer.
    let fit = g.add_node("motion.scale");
    g.set_pos(
        fit,
        Pos {
            x: 220.0,
            y: y - 90.0,
        },
    );
    g.set_param(fit, "amount", PIECE);
    wire(g, grid, 0, fit, 0)?;

    let sort = g.add_node("motion.sort");
    g.set_pos(sort, Pos { x: 440.0, y });
    g.set_param(sort, "key", key);
    g.set_param(sort, "axis_angle", axis);
    wire(g, fit, 0, sort, 0)?;

    if weighted {
        // ⚠️ **A chave é um CAMPO agora** — um ruído espacial, que nenhum dos cinco modos
        // do enum sabe exprimir.
        let noise = g.add_node("value.noise");
        g.set_pos(
            noise,
            Pos {
                x: 220.0,
                y: y + 90.0,
            },
        );
        g.set_param(noise, "frequency", 0.5);
        g.set_param(noise, "speed", 0.0);
        g.set_param(noise, "space", 1.0); // amostra por POSIÇÃO, não por índice
        wire(g, grid, 0, noise, 0)?;
        wire(g, noise, 0, sort, 1)?;
    }

    // O gradiente lê `Index`/`Count` — é ele que torna a ORDEM visível.
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 660.0, y });
    g.set_param(tint, "mode", 1.0); // Gradient
    g.set_param(tint, "r", 0.1);
    g.set_param(tint, "g", 0.12);
    g.set_param(tint, "b", 0.35);
    g.set_param(tint, "r2", 1.0);
    g.set_param(tint, "g2", 0.75);
    g.set_param(tint, "b2", 0.2);
    wire(g, sort, 0, tint, 0)?;
    Some(tint)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_sortkey_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(3);
    for (row, (key, axis, weighted)) in [
        (KEY_X, 0.0, false),
        (KEY_X, DIAGONAL, false),
        (KEY_WEIGHT, 0.0, true),
    ]
    .into_iter()
    .enumerate()
    {
        let gy = row as f32 * 260.0;
        let head = band(g, key, axis, weighted, gy)?;
        let mv = g.add_node("motion.move");
        g.set_pos(mv, Pos { x: 880.0, y: gy });
        g.set_param(mv, "dy", BAND_DY - row as f32 * BAND_DY);
        wire(g, head, 0, mv, 0)?;
        let out = g.add_node("motion.output");
        g.set_pos(out, Pos { x: 1100.0, y: gy });
        wire(g, mv, 0, out, 0)?;
        sinks.push(out);
    }
    Some(sinks)
}

/// Os rótulos das três bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ORDEM por X -- a cor corre da esquerda para a direita",
        "ORDEM por X com Axis Angle 35 -- ela corre na DIAGONAL",
        "ORDEM por um CAMPO (value.noise na porta Weight) -- ela serpenteia",
    ]
    .into_iter()
    .enumerate()
}

/// O ângulo que a banda 2 autora — para a mensagem citar o número da cena.
pub(crate) fn diagonal() -> f32 {
    DIAGONAL
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_sortkey_tests.rs"]
mod tests;
