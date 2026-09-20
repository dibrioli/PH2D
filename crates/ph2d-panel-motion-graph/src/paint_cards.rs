//! **Os cartões do grafo, e os hits deles** — irmão do [`crate::paint`] e gémeo do
//! [`crate::paint::paint_wires`].
//!
//! ⚠️ Este módulo nasceu de um cap de LOC pela MESMA razão que o irmão dos fios (o `paint`
//! bateu os 200 do painel), e o corte é por RESPONSABILIDADE: aqui se responde *«que cartões
//! estão no ecrã, de que lado sai a moldura do retrato de cada um, e onde se clica neles»* —
//! nenhum fio, nenhum backdrop, nenhuma moldura de painel.
//!
//! ⭐ **A ordem dos dois laços é LOAD-BEARING:** os corpos primeiro e os sockets, o botão do
//! retrato e o selo depois, para que os três GANHEM o gesto ao corpo que está debaixo deles
//! (doc 86; ADR-0155).

use crate::geom::{self, View};
use crate::hits::{
    push_card_hit, push_inert_badge_hit, push_param_row_hits, push_preview_toggle_hit,
    push_socket_hits,
};
use crate::paint::{draw_card, touches};
use crate::snapshot::{GraphNodeView, GraphViewSnapshot, NodeViewKind};
use crate::state::MotionGraphPanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;
use std::collections::BTreeSet;

/// Os argumentos por-frame do [`draw_cards`] — um struct pela mesma razão do irmão dos fios:
/// todos são a MESMA coisa (o que este quadro sabe), e numa lista posicional os dois conjuntos
/// de ids trocam de lugar sem o compilador reclamar.
pub(crate) struct CardPass<'a> {
    pub(crate) state: &'a MotionGraphPanelState,
    pub(crate) snap: &'a GraphViewSnapshot,
    pub(crate) view: &'a View,
    pub(crate) theme: Theme,
    pub(crate) rect: Rect,
    /// Os nós que um sink alcança — quem está fora é velado (F3).
    pub(crate) live: &'a BTreeSet<u32>,
    /// A influência do que está seleccionado, quando há selecção.
    pub(crate) focus: &'a Option<BTreeSet<u32>>,
}

pub(crate) fn draw_cards(
    p: CardPass,
    ctx: &mut PaintCtx,
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
) {
    let veiled =
        |id: u32| !p.live.contains(&id) || p.focus.as_ref().is_some_and(|f| !f.contains(&id));
    // A card whose rect does not touch the panel is SKIPPED entirely: the clip layer already
    // hides it, but Vello still has to bound and bin every path inside it — panning a big graph
    // would pay for cards nobody can see. (Its hit rects are clipped away by `hits` anyway, so
    // what is invisible stays unclickable.)
    let on_screen: Vec<&GraphNodeView> = p
        .snap
        .nodes
        .iter()
        .filter(|n| touches(geom::card_rect(n, p.view), p.rect))
        .collect();
    // ⭐⭐ **De que lado sai a moldura de cada retrato** — a lei, UMA vez por quadro e sobre a
    // tela INTEIRA (nunca sobre os cartões visíveis: um cartão fora do ecrã continua a ser o
    // vizinho de coluna que decide o lado de quem está dentro). Ver `geom::retratos_em_cima`.
    let retratos_em_cima = geom::retratos_em_cima_por_id(&p.snap.nodes);
    for n in &on_screen {
        // A GHOST is always veiled: it is not part of this level, and the veil is the whole
        // message (doc 57). It never counts as "inert" or "out of the influence" — those are
        // readings about the graph, and a ghost is a reading about the BOUNDARY.
        let dim = n.kind == NodeViewKind::Ghost || veiled(n.id);
        // `draw_card` also draws this node's ⚠ inert badge (ADR-0155) on its corner.
        let lado = p.state.preview_position(n.id, &retratos_em_cima);
        let body = draw_card(ctx, p.state, n, p.view, p.theme, dim, lado);
        push_card_hit(hits, n, body, p.rect);
        // ⚠️ **Depois do corpo, para lhes GANHAR o gesto** — ver `push_param_row_hits`.
        push_param_row_hits(hits, n, p.view, p.rect);
    }
    for n in &on_screen {
        push_socket_hits(hits, n, p.view, p.rect);
        push_preview_toggle_hit(hits, n, p.view, p.rect);
        push_inert_badge_hit(hits, n, p.view, p.rect);
    }
}
