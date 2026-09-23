//! ⭐⭐ **O MODO DE UMA LINHA** (`PH2D_MOTION_OBJ_SMOKE=15`) — doc 118 §9–§10: a sombra de um
//! `fx.drop_shadow` com o modo `Multiply`, nas DUAS médias, sobre um cenário.
//!
//! # O que está na tela
//!
//! Um cenário azul-claro (uma sprite do MUNDO) e, por cima, duas colunas com duas fileiras cada —
//! em cima uma IMAGEM, em baixo uma FORMA, as duas com uma sombra cinzenta OPACA, e uma faixa escura do cenário a passar por baixo de cada sombra:
//!
//! - à **esquerda** o CONTROLO: a sombra em `Sink` (o modo da saída, `Mix`) — ela TAPA a faixa;
//! - à **direita** a mesma sombra em `Multiply` — ela ESCURECE o que está por baixo, e a faixa vê-se através dela.
//!
//! ⚠️ **As duas fileiras respondem à mesma regra, e cada uma prova uma wave:** a FORMA vai ao Vello,
//! que até à W8 desenhava a sombra dela em `Normal` (a `VectorInstance` não levava o modo da linha);
//! a IMAGEM vai ao passe de sprites pela rota de CPU (o `fx.drop_shadow` só tem lowering de CPU), que
//! até à W10 a desenhava POR BAIXO do cenário — ela nem se via.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O índice `Multiply` no seletor `Shadow Blend` (o `0` é `Sink` e daí em diante é `modo + 1`).
pub(super) const SOMBRA_MULTIPLY: f32 = 4.0;
/// As colunas: o modo da sombra e o `x` da coluna, em mundo.
pub(super) const COLUNAS: [(f32, f32); 2] = [(0.0, -2.4), (SOMBRA_MULTIPLY, 2.4)];
/// As fileiras: a da imagem e a da forma, em `y` de mundo.
const FILEIRA_IMAGEM: f32 = 1.3;
const FILEIRA_FORMA: f32 = -1.3;
/// O lado de uma peça e a distância da sombra — grande o bastante para metade dela cair FORA da peça,
/// sobre o cenário, que é onde o modo se lê.
const LADO: f32 = 1.4;
const DISTANCIA: f32 = 0.55;
/// O cenário: por baixo das duas colunas.
const CENARIO: [f32; 2] = [8.6, 5.4];
const CENARIO_COR: [f32; 4] = [0.6, 0.85, 1.0, 1.0];
/// A cor da sombra, opaca e NEUTRA: um cinzento não se confunde com o laranja da imagem nem com o
/// magenta da forma, e deixa a faixa por baixo ser a única coisa que muda entre as colunas.
const SOMBRA_COR: [f32; 3] = [0.45, 0.45, 0.45];
/// ⭐ **A FAIXA escura do cenário que passa por BAIXO de cada sombra** (medido na 2.ª foto: sobre um
/// cenário liso o `Multiply` só muda um pouco a COR da sombra, e ninguém lê isso como «ver através»).
/// Em `x` ela cai DENTRO da sombra e FORA da peça (a peça cobre `±LADO/2`; a sombra anda `DISTANCIA`
/// a `315°`, logo o lado direito dela vai a `LADO/2 + DISTANCIA·cos 45°`) — em `Sink` a sombra
/// esconde-a, em `Multiply` ela vê-se através.
const FAIXA_DX: f32 = 0.85;
const FAIXA_LARGURA: f32 = 0.14;
const FAIXA_COR: [f32; 4] = [0.12, 0.2, 0.5, 1.0];

const _: () = assert!(
    FAIXA_DX + FAIXA_LARGURA / 2.0 < LADO / 2.0 + DISTANCIA * 0.707
        && FAIXA_DX - FAIXA_LARGURA / 2.0 > LADO / 2.0,
    "a faixa tem de cair dentro da sombra e fora da peca -- senao a cena nao mostra nada"
);

/// Prólogo: o cenário, o objecto que a fileira das imagens projecta, e o grafo.
pub(super) fn run(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::atlas(ph2d_render::WHITE_TILE_KEY, CENARIO, CENARIO_COR),
        Name::new("Scenery"),
    ));
    // As faixas: DEPOIS do cenário, logo por cima dele (e por baixo do Motion, que desenha por cima
    // do mundo — doc 118 §10).
    for &(_, x) in &COLUNAS {
        cx.sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(x + FAIXA_DX, 0.0)),
            Sprite::atlas(
                ph2d_render::WHITE_TILE_KEY,
                [FAIXA_LARGURA, CENARIO[1]],
                FAIXA_COR,
            ),
            Name::new("Stripe"),
        ));
    }
    // O objecto da fileira das imagens — FORA do cenário, onde ele se vê sozinho (a lição da `=14`).
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(-7.0, FILEIRA_IMAGEM)),
        Sprite::atlas(super::DEMO_TILE_KEY, [LADO, LADO], [1.0, 1.0, 1.0, 1.0]),
        Name::new(super::OBJECT),
    ));
    let sinks = build(&mut cx.motion.doc.graph, super::OBJECT);
    cx.motion.sinks.extend(sinks);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =15] O MODO DE UMA LINHA. Por cima do cenario azul ha' 2 colunas; em cada \
         uma, em cima uma IMAGEM e em baixo uma FORMA, as duas com uma SOMBRA cinzenta, e uma \
         FAIXA azul-escura do cenario passa por baixo de cada sombra. (1) A coluna da ESQUERDA e' o \
         controlo: a sombra em 'Sink' TAPA a faixa. (2) A da DIREITA e' a mesma sombra em \
         'Multiply': ela ESCURECE o que esta' por baixo, e a faixa ve'-se atraves dela. (3) A imagem e a \
         forma da mesma coluna tem de fazer o MESMO. Clique no cartao 'Drop Shadow' de uma coluna e \
         troque o 'Shadow Blend' para ver a sombra mudar."
    );
}

/// As quatro saídas: `(modo da sombra, x) × (imagem, forma)`.
pub(super) fn build(graph: &mut Graph, objecto: &str) -> Vec<NodeId> {
    let img = graph.add_node("source.object");
    graph.set_text_param(img, "object", objecto);
    graph.set_pos(img, Pos { x: 0.0, y: -300.0 });
    graph.set_label(img, "The Image");
    let forma = graph.add_node("source.shape");
    graph.set_param(forma, "kind", 0.0); // o círculo
    // ⚠️ O `size` de um círculo é o RAIO (medido na 1.ª foto: o disco saiu com o dobro da imagem).
    graph.set_param(forma, "size", LADO / 2.0);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL, 1.0);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_R, 0.9);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_G, 0.3);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_B, 0.65);
    graph.set_param(forma, ph2d_node_motion_shape::param::FILL_A, 1.0);
    graph.set_pos(forma, Pos { x: 0.0, y: 60.0 });
    graph.set_label(forma, "The Shape");
    let mut sinks = Vec::new();
    for (k, &(modo, x)) in COLUNAS.iter().enumerate() {
        for (fonte, y, dy) in [(img, FILEIRA_IMAGEM, 0.0), (forma, FILEIRA_FORMA, 360.0)] {
            let gx = 260.0 + k as f32 * 520.0;
            sinks.push(celula(graph, fonte, modo, [x, y], [gx, -300.0 + dy]));
        }
    }
    // Nasce com a sombra da coluna `Multiply` (formas) escolhida: o `Shadow Blend` é a linha que o
    // roteiro manda trocar. O `sinks[3]` é a SAÍDA; a sombra é o nó dois antes dela.
    ph2d_panel_motion_graph::request_graph_selection(vec![sombra_de(graph, sinks[3]).0]);
    sinks
}

/// O `fx.drop_shadow` que alimenta uma saída (`sombra → move → output`).
pub(super) fn sombra_de(g: &Graph, saida: NodeId) -> NodeId {
    let entrada = |n: NodeId| {
        g.edges()
            .iter()
            .find(|e| e.to == (n, 0))
            .map(|e| e.from.0)
            .expect("ligada")
    };
    entrada(entrada(saida))
}

/// Uma célula: `fonte → fx.drop_shadow → move → output`.
fn celula(g: &mut Graph, fonte: NodeId, modo: f32, onde: [f32; 2], pos: [f32; 2]) -> NodeId {
    let sombra = g.add_node("fx.drop_shadow");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (i, n) in [sombra, mv, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: pos[0] + i as f32 * 150.0,
                y: pos[1],
            },
        );
    }
    for (a, b) in [(fonte, sombra), (sombra, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("liga");
    }
    g.set_param(sombra, "distance", DISTANCIA);
    // ⚠️ Não laranja (medido na 1.ª foto): a imagem da cena é um ladrilho LARANJA, e uma sombra da
    // mesma cor confundia-se com a peça que a projecta.
    g.set_param(sombra, "r", SOMBRA_COR[0]);
    g.set_param(sombra, "g", SOMBRA_COR[1]);
    g.set_param(sombra, "b", SOMBRA_COR[2]);
    g.set_param(sombra, "a", 1.0);
    g.set_param(sombra, ph2d_node_fx_drop_shadow::SHADOW_BLEND, modo);
    g.set_param(mv, "dx", onde[0]);
    g.set_param(mv, "dy", onde[1]);
    out
}

#[cfg(test)]
#[path = "motion_object_smoke_linha_tests.rs"]
mod tests;
