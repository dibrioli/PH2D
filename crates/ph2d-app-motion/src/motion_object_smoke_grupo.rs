//! ⭐⭐⭐ **A MISTURA EM GRUPO** (`PH2D_MOTION_OBJ_SMOKE=14`) — os três alcances do doc 118, nas
//! duas médias, sobre um CENÁRIO que se vê (ordem do dono: *«todas as opções possíveis sem excluir
//! nenhuma, com seletor de modo no nó»* e *«Igual, como nas formas»*).
//!
//! # O que está na tela
//!
//! Um cenário (uma imagem grande do mundo, fora do Motion) e, por cima, quatro COLUNAS — da
//! esquerda para a direita: o CONTROLO em `Normal`, e o `Multiply` em `Everything`, `Copies` e
//! `Scene`. Cada coluna tem duas fileiras: em cima quatro cópias de uma IMAGEM que se sobrepõem, em
//! baixo quatro cópias de uma FORMA. Uma saída (`Output`) por célula, porque o alcance é do NÓ.
//!
//! ⚠️ **As cópias SOBREPÕEM-SE de propósito** — o doc 118 §1 põe a diferença entre os alcances
//! onde duas se empilham: `Everything` escurece a sobreposição duas vezes, `Scene` uma vez, e
//! `Copies` deixa o cenário intacto por baixo das cópias.
//!
//! ⚠️ **O cenário é uma SPRITE do mundo e não um nó**: é isso que prova a W2 (o mundo por baixo da
//! cena vectorial). Um cenário feito de formas misturar-se-ia mesmo sem ela.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_eval_motion::BlendWith;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O tag de `Multiply` no seletor do `motion.output`.
const MULTIPLY: f32 = 3.0;
/// As colunas: o alcance (ou `None` = o controlo em `Normal`) e o `x` da coluna, em mundo.
pub(super) const COLUNAS: [(Option<BlendWith>, f32); 4] = [
    (None, -4.8),
    (Some(BlendWith::Everything), -1.6),
    (Some(BlendWith::Copies), 1.6),
    (Some(BlendWith::Scene), 4.8),
];
/// As fileiras: a das imagens e a das formas, em `y` de mundo.
pub(super) const FILEIRA_IMAGEM: f32 = 1.3;
pub(super) const FILEIRA_FORMA: f32 = -1.3;
/// O lado de uma cópia e o passo da grelha 2×2 — MENOR que o lado, para as cópias se sobreporem.
const LADO_COPIA: f32 = 1.1;
const PASSO: f32 = 0.7;

const _: () = assert!(
    PASSO < LADO_COPIA,
    "sem sobreposicao os tres alcances desenham o mesmo -- a cena deixa de ensinar"
);
/// O cenário: por baixo das TRÊS colunas que misturam — e NÃO da de controlo.
///
/// ⚠️⚠️ **Medido nas fotos da cena:** com o cenário também por baixo do controlo, as quatro imagens
/// em `Normal` (que vão ao passe de sprites) ficavam TAPADAS por ele — o passe desenhou as 4
/// (contadas) e o cenário, uma sprite do mundo, ficou por cima; nem um `ZIndexOverride(-1)` no
/// cenário as trouxe à frente. É a ordem entre as sprites do mundo e as do Motion, que esta wave
/// não toca e não investigou (doc 118 §6). ⇒ o controlo fica no chão do canvas, onde se vê.
pub(super) const CENARIO: [f32; 2] = [9.8, 5.4];
/// O centro do cenário em `x`: do meio da coluna `Everything` ao fim da `Scene`.
const CENARIO_X: f32 = 1.6;
/// A cor do cenário (a tinta do ladrilho branco).
const CENARIO_COR: [f32; 4] = [0.6, 0.85, 1.0, 1.0];

/// Prólogo: o cenário e o objecto que as imagens carimbam.
pub(super) fn run(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(CENARIO_X, 0.0)),
        // ⚠️ Um azul CLARO sobre o ladrilho branco: o `Multiply` escurece-o para uma cor que não é a
        // das cópias (laranja) nem a das formas (magenta) — as três leituras não se confundem.
        Sprite::atlas(ph2d_render::WHITE_TILE_KEY, CENARIO, CENARIO_COR),
        Name::new("Scenery"),
    ));
    // O objecto que a fileira das imagens carimba — posto FORA do cenário (à esquerda, na altura
    // da fileira das imagens), onde ele se vê sozinho e não entra em mistura nenhuma. ⚠️ Em cima
    // do cenário ele ficava cortado pela borda do canvas (medido na 1.ª foto).
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(-8.4, FILEIRA_IMAGEM)),
        // ⚠️ Do MESMO lado da forma: as duas fileiras sobrepõem-se por igual.
        Sprite::atlas(
            super::DEMO_TILE_KEY,
            [LADO_COPIA, LADO_COPIA],
            [1.0, 1.0, 1.0, 1.0],
        ),
        Name::new(super::OBJECT),
    ));
    let sinks = build(&mut cx.motion.doc.graph, super::OBJECT);
    cx.motion.sinks.extend(sinks);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =14] A MISTURA EM GRUPO. Por cima do cenario ha' 4 colunas; em cada \
         uma, em cima 4 copias de uma IMAGEM e em baixo 4 copias de uma FORMA, que se sobrepoem. \
         (1) A coluna da ESQUERDA e' o controlo, em 'Normal'. (2) As outras tres estao em \
         'Multiply', e o que as separa e' a linha 'Blend With' do cartao 'Output': 'Everything' \
         (cada copia escurece TUDO por baixo -- o cenario e as copias anteriores: a sobreposicao \
         fica MAIS escura), 'Copies' (as copias escurecem-se so' entre si e o cenario fica \
         INTACTO por baixo delas), 'Scene' (as copias juntam-se e o grupo escurece o cenario UMA \
         vez: a sobreposicao nao escurece mais). (3) Imagens e formas de cada coluna tem de \
         seguir a MESMA regra. Clique numa saida e troque o 'Blend With' para ver a coluna mudar."
    );
}

/// Os oito sinks: `(alcance, x) × (imagem, forma)`, e as fontes partilhadas.
pub(super) fn build(graph: &mut Graph, objecto: &str) -> Vec<NodeId> {
    let img = graph.add_node("source.object");
    graph.set_text_param(img, "object", objecto);
    graph.set_pos(img, Pos { x: 0.0, y: -420.0 });
    graph.set_label(img, "The Image");
    let forma = graph.add_node("source.shape");
    graph.set_param(forma, "kind", 0.0); // o círculo
    graph.set_param(forma, "size", LADO_COPIA);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL, 1.0);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_R, 0.9);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_G, 0.3);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_B, 0.65);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_A, 1.0);
    graph.set_pos(forma, Pos { x: 0.0, y: -60.0 });
    graph.set_label(forma, "The Shape");
    let grelha = graph.add_node("motion.grid");
    graph.set_param(grelha, "rows", 2.0);
    graph.set_param(grelha, "cols", 2.0);
    graph.set_param(grelha, "gap_x", PASSO);
    graph.set_param(grelha, "gap_y", PASSO);
    graph.set_pos(grelha, Pos { x: 0.0, y: -240.0 });
    let mut sinks = Vec::new();
    for (k, &(alcance, x)) in COLUNAS.iter().enumerate() {
        for (fonte, y, dy) in [(img, FILEIRA_IMAGEM, 0.0), (forma, FILEIRA_FORMA, 360.0)] {
            let gx = 260.0 + k as f32 * 420.0;
            sinks.push(celula(
                graph,
                fonte,
                grelha,
                alcance,
                [x, y],
                [gx, -420.0 + dy],
            ));
        }
    }
    // Nasce com a saída da coluna `Everything` (imagens) escolhida: o `Blend With` é a linha
    // que o roteiro manda trocar.
    ph2d_panel_motion_graph::request_graph_selection(vec![sinks[2].0]);
    sinks
}

/// Uma célula: `fonte → duplicator ← grelha → move → output`.
fn celula(
    g: &mut Graph,
    fonte: NodeId,
    grelha: NodeId,
    alcance: Option<BlendWith>,
    onde: [f32; 2],
    pos: [f32; 2],
) -> NodeId {
    let dup = g.add_node("motion.duplicator");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    g.set_pos(
        dup,
        Pos {
            x: pos[0],
            y: pos[1],
        },
    );
    g.set_pos(
        mv,
        Pos {
            x: pos[0] + 130.0,
            y: pos[1],
        },
    );
    g.set_pos(
        out,
        Pos {
            x: pos[0] + 260.0,
            y: pos[1],
        },
    );
    let mut liga = |a: NodeId, ap: u16, b: NodeId, bp: u16| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("liga");
    };
    liga(fonte, 0, dup, 0);
    liga(grelha, 0, dup, 1);
    liga(dup, 0, mv, 0);
    liga(mv, 0, out, 0);
    g.set_param(mv, "dx", onde[0]);
    g.set_param(mv, "dy", onde[1]);
    if let Some(com) = alcance {
        g.set_param(out, ph2d_eval_motion::SINK_BLEND_PARAM, MULTIPLY);
        let tag = BlendWith::ALL
            .iter()
            .position(|c| *c == com)
            .expect("um dos três");
        g.set_param(out, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, tag as f32);
    }
    out
}

#[cfg(test)]
#[path = "motion_object_smoke_grupo_tests.rs"]
mod tests;
