// ph2d-chrome-sync:z=181 (dispatch priority, ADR-0107; lower = earlier)
// ⚠️ Logo DEPOIS dos dois cards irmãos (fill/onion, z=180) e MUITO antes das alças de
// canvas (z=230+): um card flutuante tem de engolir o ponteiro antes de a arte por baixo o
// ver, senão arrastar de um círculo a outro MOVE as formas.
//! **O DIAGRAMA da booleana viva** — o card flutuante onde a operação é da LIGAÇÃO.
//!
//! Duas metades, coladas, espelhando [`super::onion_modal`]:
//!
//! * [`paint_bool_graph_modal`] — desenha o card no `store.bool_graph_pos()` (preso ao viewport):
//!   a banda de título com alça e X, um círculo por forma com o nome ao lado, um arco por ligação
//!   com a operação escrita nele, e a faixa de aviso quando o grafo se morde.
//! * [`apply`] — o despacho: o X fecha, o clique na banda é consumido (o arrasto é a máquina da
//!   shell), e o corpo consome tudo o que cai dentro dele.
//!
//! # A geometria mora noutro sítio, de propósito
//!
//! ⚠️ Toda posição vem de [`crate::widget::bool_graph`] — `node_center`, `link_points`, `card_size`.
//! Este arquivo **não calcula uma coordenada**. É o que garante que quem PINTA e quem ACERTA O
//! CLIQUE leem o mesmo mapa: um segundo cálculo divergiria do primeiro no dia em que um
//! espaçamento mudasse, e o artista clicaria ao lado do que vê.
//!
//! # O card publica o RECT QUE DESENHOU
//!
//! ⚠️ `store.set_bool_graph_drawn(...)` é a porta única do acerto do clique. A shell precisa do
//! retângulo para correr o `node_at`/`link_at`, e recalculá-lo a partir do canto pedido
//! significaria repetir a prisão ao viewport — a mesma armadilha das duas contas.
//!
//! # O que ele NÃO faz
//!
//! Não muta o documento. O card empilha **intenções** que a shell drena
//! ([`crate::interaction::WidgetStore::take_bool_graph_intents`]), porque só a shell alcança o ECS.
//! É a lei do painel de vetor (*"a verdade mora no ECS, não aqui"*), e é o que impede duas
//! respostas para *"quais são as ligações deste grupo?"*.

use crate::ids;
use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};
use crate::paint::{
    fill_circle, fill_rounded_rect, paint_text, resolve, stroke_polyline, stroke_rounded_rect,
};
use crate::screens::hero::HeroScreen;
use crate::widget::{
    BoolGraphNode, BoolGraphView, Button, bool_graph_card_size, bool_graph_link_points,
    bool_graph_node_center, bool_graph_node_radius, bool_graph_ring_inner_radius, paint_button,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Largura do X que fecha o card.
const CLOSE_W: f32 = 20.0; // LITERAL-PX-OK: bool-graph modal close-X width
/// Espessura do traço de uma ligação.
const LINK_W: f32 = 2.0; // LITERAL-PX-OK: bool-graph link stroke width
/// Espessura do anel de um círculo.
const RING_W: f32 = 1.5; // LITERAL-PX-OK: bool-graph node ring width
/// Meia-largura da ponta da seta.
const HEAD_W: f32 = 5.0; // LITERAL-PX-OK: bool-graph arrowhead half-width
/// Comprimento da ponta da seta.
const HEAD_L: f32 = 10.0; // LITERAL-PX-OK: bool-graph arrowhead length
/// O recuo horizontal do glifo da operação, em frações da fonte — ele fica centrado no traço.
const GLYPH_NUDGE: f32 = 0.35; // LITERAL-PX-OK: bool-graph op glyph centering, fraction of the font
/// A margem do nome dentro do círculo, em frações do raio.
const LABEL_INSET: f32 = 0.66; // LITERAL-PX-OK: bool-graph in-circle label inset, fraction of radius
/// A altura do número de z acima do centro, em frações do raio.
const BADGE_RISE: f32 = 0.72; // LITERAL-PX-OK: bool-graph z-badge rise, fraction of radius
/// O recuo do número de z à esquerda do centro, em frações do raio.
const BADGE_INSET: f32 = 0.5; // LITERAL-PX-OK: bool-graph z-badge inset, fraction of radius

/// **O nome curto de cada operação**, para caber ao lado de um arco.
///
/// ⚠️ São as quatro de CONJUNTO e mais nada — as quatro receitas não cabem numa ligação (a lei 3
/// do [`crate::widget::bool_graph`]). Um código que este build não conhece desenha `?`, que é a
/// degradação honesta: ele diz *"há uma ligação aqui e eu não sei o que ela faz"* em vez de
/// escolher uma operação por omissão.
fn op_glyph(op: u8) -> &'static str {
    match op {
        0 => "+",
        1 => "-",
        2 => "n",
        3 => "x",
        _ => "?",
    }
}

/// O rótulo por extenso de uma operação — o que a legenda mostra.
fn op_label(op: u8) -> &'static str {
    match op {
        0 => ph2d_i18n::tr("panel.vector.bool.union"),
        1 => ph2d_i18n::tr("panel.vector.bool.subtract"),
        2 => ph2d_i18n::tr("panel.vector.bool.intersect"),
        3 => ph2d_i18n::tr("panel.vector.bool.exclude"),
        _ => "?",
    }
}

/// **O retângulo do card**, preso ao viewport — a conta que o painter faz e publica.
fn card_rect(pos: (f32, f32), view: &BoolGraphView, viewport: Rect) -> Rect {
    let (w, h) = bool_graph_card_size(view);
    let max_x = (viewport.x + viewport.w - w).max(viewport.x);
    let max_y = (viewport.y + viewport.h - h).max(viewport.y);
    Rect::new(
        pos.0.clamp(viewport.x, max_x), // CLAMP-OK: limites ordenados (max_x ≥ viewport.x), sem NaN
        pos.1.clamp(viewport.y, max_y), // CLAMP-OK: idem
        w,
        h,
    )
}

/// Desenha o card no `store.bool_graph_pos()`. No-op quando fechado — e nesse caso publica
/// `None` como rect desenhado, para o acerto do clique não responder sobre um card que não existe.
pub fn paint_bool_graph_modal(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &mut WidgetStore,
    viewport: Rect,
) {
    let Some(pos) = store.bool_graph_pos() else {
        store.set_bool_graph_drawn(None);
        return;
    };
    let view = store.bool_graph_view().clone();
    let rect = card_rect(pos, &view, viewport);
    store.set_bool_graph_drawn(Some(rect));

    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    // ⚠️ O CORPO inteiro engole o ponteiro. Sem isto o clique atravessaria para a arte por baixo, e
    // arrastar de um círculo a outro moveria as FORMAS — o oposto exato do gesto.
    hit_index.register(ids::VECTOR_BOOL_GRAPH_BODY, rect);

    paint_title_band(scene, text_system, theme, hit_index, store, rect);
    paint_links(scene, text_system, theme, &view, rect);
    paint_nodes(scene, text_system, theme, &view, rect);
    paint_footer(scene, text_system, theme, &view, rect);
}

/// A banda de título: alça de arrasto à esquerda, X à direita.
///
/// ⚠️ A alça **para antes** do X para os dois retângulos nunca partilharem um pixel — um *Down* no
/// X fecha, um *Down* na banda arrasta, e sobrepô-los faria o fechar depender de qual handler
/// corre primeiro.
fn paint_title_band(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    rect: Rect,
) {
    let row_h = ROW_H_PX;
    let font = TypeToken::Sm.px();
    let close_x = rect.x + rect.w - CLOSE_W - Spacing::Sm.px();
    let handle = Rect::new(rect.x, rect.y, close_x - rect.x, row_h);
    hit_index.register(ids::VECTOR_BOOL_GRAPH_HANDLE, handle);
    paint_text(
        text_system,
        scene,
        ph2d_i18n::tr("panel.vector.bool.graph.title"),
        rect.x + Spacing::Md.px(),
        rect.y + (row_h - font) * 0.5,
        font,
        handle.w - Spacing::Md.px(),
        resolve(ColorToken::Text1, theme),
    );
    let close_rect = Rect::new(close_x, rect.y, CLOSE_W, row_h);
    hit_index.register(ids::VECTOR_BOOL_GRAPH_CLOSE, close_rect);
    let close = Button::new(ids::VECTOR_BOOL_GRAPH_CLOSE, "X")
        .visual(store.button_visual(ids::VECTOR_BOOL_GRAPH_CLOSE));
    paint_button(&close, close_rect, scene, text_system, theme);
}

/// Os traços, cada um com a seta na ponta de quem RECEBE e a operação escrita no meio.
///
/// ⚠️ Os traços vêm ANTES dos círculos de propósito: a seta encosta na borda do receptor, e
/// desenhá-la por cima do círculo fá-la-ia parecer ENTRAR na forma em vez de chegar a ela.
fn paint_links(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    view: &BoolGraphView,
    rect: Rect,
) {
    let font = TypeToken::Sm.px();
    for link in &view.links {
        let pts = bool_graph_link_points(rect, view, *link);
        let [a, b] = pts[..] else { continue };
        stroke_polyline(scene, &[a, b], LINK_W, resolve(ColorToken::Accent, theme));
        paint_arrowhead(scene, theme, &[a, b]);
        // O verbo vai no MEIO do traço, que é onde ele não disputa espaço com nenhum círculo.
        let mid = (a.0.midpoint(b.0), a.1.midpoint(b.1));
        paint_text(
            text_system,
            scene,
            op_glyph(link.op),
            mid.0 - font * GLYPH_NUDGE,
            mid.1 - font * 0.5,
            font,
            font * 2.0,
            resolve(ColorToken::Text1, theme),
        );
    }
}

/// A ponta da seta, no fim do arco (o lado de quem RECEBE).
fn paint_arrowhead(scene: &mut VectorScene, theme: Theme, pts: &[(f32, f32)]) {
    let n = pts.len();
    let tip = pts[n - 1];
    let prev = pts[n - 2];
    let (dx, dy) = (tip.0 - prev.0, tip.1 - prev.1);
    let len = dx.hypot(dy);
    if len <= f32::EPSILON {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    // A base recua ao longo da direção de chegada; as duas asas abrem na perpendicular.
    let base = (tip.0 - ux * HEAD_L, tip.1 - uy * HEAD_L);
    let wing = |s: f32| (base.0 - uy * HEAD_W * s, base.1 + ux * HEAD_W * s);
    let color = resolve(ColorToken::Accent, theme);
    stroke_polyline(scene, &[wing(1.0), tip, wing(-1.0)], LINK_W, color);
}

/// Os círculos: o disco, o **aro** (a alça de ligar), o nome DENTRO e o número de z.
fn paint_nodes(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    view: &BoolGraphView,
    rect: Rect,
) {
    let r = bool_graph_node_radius();
    let font = TypeToken::Xs.px();
    for (i, node) in view.nodes.iter().enumerate() {
        let (cx, cy) = bool_graph_node_center(rect, view, i);
        // ⚠️ Um nó CONSUMIDO desenha-se apagado: ele não põe nada na tela, e um disco aceso diria o
        // contrário. É a mesma distinção que o mapa faz (lista vazia = desenha nada).
        let fill = if node.consumed {
            ColorToken::Bg3
        } else {
            ColorToken::Accent
        };
        fill_circle(scene, cx, cy, r, resolve(fill, theme));
        // O ARO — a alça de ligar. Ele é desenhado como um anel MAIS CLARO por dentro do disco,
        // para a banda que responde ao arrasto ser visível em vez de ser folclore.
        ring(
            scene,
            cx,
            cy,
            bool_graph_ring_inner_radius(),
            resolve(ColorToken::Border, theme),
        );
        ring(scene, cx, cy, r, resolve(ColorToken::Text1, theme));
        // O NOME, dentro do círculo.
        paint_text(
            text_system,
            scene,
            &node.label,
            cx - r * LABEL_INSET,
            cy - font * 0.5,
            font,
            r * LABEL_INSET * 2.0,
            resolve(ColorToken::Text1, theme),
        );
        // O NÚMERO DE Z, no alto do círculo. ⚠️ É o que sobrou da coluna e é o essencial dela:
        // ligações que chegam ao mesmo nó dobram na ordem de z de quem opera, e sem este número o
        // resultado dependeria de uma coisa que o diagrama não mostra.
        paint_text(
            text_system,
            scene,
            &BoolGraphNode::z_badge(i).to_string(),
            cx - r * BADGE_INSET,
            cy - r * BADGE_RISE,
            font,
            font * 2.0,
            resolve(ColorToken::Text2, theme),
        );
    }
}

/// O anel de um círculo, como uma polilinha fechada — a crate não tem `stroke_circle`, e
/// aproximá-lo aqui evita um primitivo novo no design system para um uso só.
fn ring(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, color: ph2d_vector::Color) {
    /// Quantos segmentos aproximam a circunferência. ⚠️ CONTAGEM, não pixel.
    const SEGS: usize = 24;
    #[allow(clippy::cast_precision_loss)] // índice de segmento, não medida
    let pts: Vec<(f32, f32)> = (0..=SEGS)
        .map(|k| {
            let a = std::f32::consts::TAU * k as f32 / SEGS as f32;
            (r.mul_add(a.cos(), cx), r.mul_add(a.sin(), cy))
        })
        .collect();
    stroke_polyline(scene, &pts, RING_W, color);
}

/// A dica de gesto, ou o aviso de ciclo quando o grafo se morde.
///
/// ⚠️ O ciclo é uma recusa SILENCIOSA no motor (a arte fica como estava, e nada explica por quê).
/// Aceitável para o modelo; ⛔ inaceitável para a janela — é aqui que ele ganha voz.
fn paint_footer(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    view: &BoolGraphView,
    rect: Rect,
) {
    let font = TypeToken::Xs.px();
    let (key, color) = if view.cycle {
        ("panel.vector.bool.graph.cycle", ColorToken::Danger)
    } else if view.nodes.is_empty() {
        ("panel.vector.bool.graph.empty", ColorToken::Text2)
    } else {
        ("panel.vector.bool.graph.hint", ColorToken::Text2)
    };
    paint_text(
        text_system,
        scene,
        ph2d_i18n::tr(key),
        rect.x + Spacing::Md.px(),
        rect.y + rect.h - Spacing::Md.px() - font,
        font,
        rect.w - Spacing::Md.px() * 2.0,
        resolve(color, theme),
    );
}

/// Despacha os eventos do card (ligado em `chrome::dispatch_all`). Só age com o card ABERTO.
///
/// ⚠️ O corpo consome tudo o que cai nele e devolve `true` **sem fazer nada**: o gesto de ligar é
/// uma máquina da shell (ela é quem sabe o que um `VecPathId` significa), e o que este braço faz é
/// impedir o clique de vazar para outra chrome por baixo.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if hero.store.bool_graph_pos().is_none() {
        return false;
    }
    match event {
        WidgetEvent::Click(id) if id == ids::VECTOR_BOOL_GRAPH_CLOSE => {
            hero.store.close_bool_graph();
            true
        }
        WidgetEvent::Click(id)
            if id == ids::VECTOR_BOOL_GRAPH_HANDLE || id == ids::VECTOR_BOOL_GRAPH_BODY =>
        {
            true
        }
        _ => false,
    }
}

/// O rótulo por extenso de uma operação — publicado para a shell mostrar a escolha atual sem uma
/// segunda tabela.
#[must_use]
pub fn bool_graph_op_label(op: u8) -> &'static str {
    op_label(op)
}

#[cfg(test)]
mod tests {
    //! Gates do card. Inline (e não num irmão `_tests.rs`) porque o scanner de chrome trata todo
    //! `chrome/*.rs` como um handler.

    use super::*;
    use crate::widget::{BoolGraphLink, BoolGraphNode};

    fn text_system() -> TextSystem {
        TextSystem::without_system_fonts()
    }

    fn view() -> BoolGraphView {
        BoolGraphView {
            nodes: vec![
                BoolGraphNode {
                    id: 1,
                    label: "A".into(),
                    consumed: false,
                    at: None,
                },
                BoolGraphNode {
                    id: 2,
                    label: "B".into(),
                    consumed: true,
                    at: None,
                },
            ],
            links: vec![BoolGraphLink {
                from: 2,
                to: 1,
                op: 1,
            }],
            cycle: false,
        }
    }

    fn paint(hero: &mut HeroScreen, viewport: Rect) -> HitIndex {
        let mut scene = VectorScene::new();
        let mut ts = text_system();
        let mut hits = HitIndex::default();
        paint_bool_graph_modal(
            &mut scene,
            &mut ts,
            Theme::default(),
            &mut hits,
            &mut hero.store,
            viewport,
        );
        hits
    }

    /// **FECHADO NÃO REGISTA NADA, E NÃO PUBLICA RECT.**
    ///
    /// ⚠️ A segunda metade é a que importa: um rect publicado com o card fechado faria o acerto do
    /// clique responder sobre um card que não está na tela, e o artista veria a arte deixar de
    /// aceitar o ponteiro numa região invisível.
    #[test]
    fn fechado_nao_regista_nem_publica_rect() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        hero.store.set_bool_graph_view(view());
        let hits = paint(&mut hero, Rect::new(0.0, 0.0, 1200.0, 800.0));
        assert!(hits.hit(10.0, 10.0).is_none());
        assert_eq!(hero.store.bool_graph_drawn(), None);
    }

    /// **ABERTO REGISTA O CORPO, A ALÇA E O X — E PUBLICA O RECT QUE DESENHOU.**
    #[test]
    fn aberto_regista_o_card_e_publica_o_rect() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        hero.store.set_bool_graph_view(view());
        hero.store.open_bool_graph(100.0, 60.0);
        let hits = paint(&mut hero, Rect::new(0.0, 0.0, 1200.0, 800.0));
        let rect = hero.store.bool_graph_drawn().expect("publicou o rect");
        assert!(rect.w > 0.0 && rect.h > 0.0);
        // O corpo engole um ponto no meio do card.
        assert!(
            hits.hit(rect.x + rect.w * 0.5, rect.y + rect.h * 0.7)
                .is_some()
        );
        // O X está lá.
        assert_eq!(
            hits.hit(
                rect.x + rect.w - CLOSE_W * 0.5 - Spacing::Sm.px(),
                rect.y + 4.0
            ),
            Some(ids::VECTOR_BOOL_GRAPH_CLOSE)
        );
    }

    /// **A ALÇA NÃO PARTILHA UM PIXEL COM O X.** Sobrepô-los faria o fechar depender de qual
    /// handler corre primeiro — e isso muda com a ordem de registo, que ninguém lê.
    #[test]
    fn a_alca_para_antes_do_x() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        hero.store.set_bool_graph_view(view());
        hero.store.open_bool_graph(100.0, 60.0);
        let hits = paint(&mut hero, Rect::new(0.0, 0.0, 1200.0, 800.0));
        let rect = hero.store.bool_graph_drawn().unwrap();
        let x_esquerda = rect.x + rect.w - CLOSE_W - Spacing::Sm.px();
        assert_eq!(
            hits.hit(x_esquerda - 1.0, rect.y + 4.0),
            Some(ids::VECTOR_BOOL_GRAPH_HANDLE)
        );
        assert_eq!(
            hits.hit(x_esquerda + 1.0, rect.y + 4.0),
            Some(ids::VECTOR_BOOL_GRAPH_CLOSE)
        );
    }

    /// **O CARD FICA DENTRO DO VIEWPORT** mesmo pedido fora dele — e o rect publicado é o PRESO,
    /// não o pedido. Se fosse o pedido, o clique acertaria onde o card não está.
    #[test]
    fn o_rect_publicado_e_o_preso_ao_viewport() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        hero.store.set_bool_graph_view(view());
        hero.store.open_bool_graph(5000.0, 5000.0);
        let vp = Rect::new(0.0, 0.0, 1200.0, 800.0);
        paint(&mut hero, vp);
        let rect = hero.store.bool_graph_drawn().unwrap();
        assert!(
            rect.x + rect.w <= vp.x + vp.w + 0.5 && rect.y + rect.h <= vp.y + vp.h + 0.5,
            "o card saiu do viewport: {rect:?}"
        );
        assert_ne!(hero.store.bool_graph_pos(), Some((rect.x, rect.y)));
    }

    /// **O X FECHA.**
    #[test]
    fn o_x_fecha_o_card() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        hero.store.open_bool_graph(100.0, 60.0);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::VECTOR_BOOL_GRAPH_CLOSE)
        ));
        assert_eq!(hero.store.bool_graph_pos(), None);
    }

    /// **FECHADO, O DESPACHO NÃO CONSOME NADA** — senão o card fechado engoliria cliques do resto
    /// da chrome para sempre.
    #[test]
    fn fechado_o_despacho_nao_consome() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        assert!(!apply(
            &mut hero,
            WidgetEvent::Click(ids::VECTOR_BOOL_GRAPH_CLOSE)
        ));
        assert!(!apply(
            &mut hero,
            WidgetEvent::Click(ids::VECTOR_BOOL_GRAPH_BODY)
        ));
    }

    /// **AS QUATRO OPERAÇÕES DE CONJUNTO TÊM GLIFO PRÓPRIO, E UM CÓDIGO DESCONHECIDO DIZ `?`.**
    ///
    /// ⚠️ A segunda metade é a lei: escolher uma operação por omissão para um código que este build
    /// não conhece faria o diagrama AFIRMAR algo que ele não sabe.
    #[test]
    fn cada_operacao_tem_glifo_e_o_desconhecido_diz_interrogacao() {
        let quatro: Vec<&str> = (0u8..4).map(op_glyph).collect();
        let unicos: std::collections::BTreeSet<&str> = quatro.iter().copied().collect();
        assert_eq!(
            unicos.len(),
            4,
            "duas operações partilham glifo: {quatro:?}"
        );
        for op in [4u8, 7, 200] {
            assert_eq!(op_glyph(op), "?", "o código {op} escolheu uma operação");
        }
    }
}
