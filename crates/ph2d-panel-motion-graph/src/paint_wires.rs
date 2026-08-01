//! **Os fios do grafo F2, e os hits deles** — irmão do [`crate::paint`].
//!
//! ⚠️ Este módulo nasceu de um cap de LOC, não de um redesenho: o `paint` bateu os 200
//! do painel e o arquivo os 600, e o que o empurrou por cima **não foi código novo** —
//! foi o `rustfmt` re-quebrando uma chamada de 7 argumentos ao rebasear (o contexto ao
//! redor mudou, a decisão de quebra mudou, +8 linhas). O corte, porém, é por
//! RESPONSABILIDADE, e é isso que o torna um módulo em vez de um pedaço arrancado: aqui
//! se responde *"que fios existem, quais estão acesos, e onde se clica neles"* — nenhum
//! card, nenhum backdrop.

use crate::geom::View;
use crate::hits::push_wire_hits;
use crate::paint::draw_wire;
use crate::snapshot::GraphViewSnapshot;
use crate::state::MotionGraphPanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;
use std::collections::BTreeSet;

/// Os argumentos por-frame do [`draw_wires`] — um struct e não onze parâmetros soltos,
/// porque todos são a MESMA coisa (o que este frame sabe) e uma lista posicional de onze
/// é onde dois deles trocam de lugar sem o compilador reclamar (os dois `Option` de tupla
/// e os dois `BTreeSet` são intercambiáveis por TIPO).
pub(crate) struct WirePass<'a> {
    pub(crate) state: &'a MotionGraphPanelState,
    pub(crate) snap: &'a GraphViewSnapshot,
    pub(crate) view: &'a View,
    pub(crate) theme: Theme,
    pub(crate) rect: Rect,
    pub(crate) hovered: Option<NodeId>,
    pub(crate) detached: Option<(u32, u16)>,
    pub(crate) live: &'a BTreeSet<u32>,
    pub(crate) focus: &'a Option<BTreeSet<u32>>,
}

/// **Os fios, e os hits deles.**
///
/// Extraído do [`paint`] porque ele bateu o cap de 200 LOC do painel — ⚠️ e o que o
/// empurrou por cima **não foi código novo**: foi o `rustfmt` re-quebrando uma chamada de
/// 7 argumentos ao rebasear (o contexto ao redor mudou, a decisão de quebra mudou, +8
/// linhas). O corte, porém, é por RESPONSABILIDADE: este laço responde *"que fios existem,
/// quais estão acesos, e onde se clica neles"*, e não toca card nenhum.
///
/// ⚠️ O `live`/`focus` entram por REFERÊNCIA e não são recomputados aqui: os dois são
/// computados UMA vez por paint de propósito, porque os cards os leem depois pelo `veiled`
/// — um ramo morto tem de apagar como UMA coisa, e duas derivações divergiriam no dia em
pub(crate) fn draw_wires(
    p: WirePass<'_>,
    ctx: &mut PaintCtx,
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
) {
    for e in &p.snap.edges {
        if p.detached == Some((e.to_node, e.to_port)) {
            continue;
        }
        let is_hovered = p.hovered == Some(crate::hits::wire_hit_id(e.to_node, e.to_port));
        // A SELECTED wire wears the hover highlight persistently — the visible affordance for the
        // click-then-Delete idiom (the alt-click Disconnect had none). Hover is transient (under
        // the cursor), so off-cursor only the selected wire stays lit.
        let is_selected = p.state.selected_wire == Some((e.to_node, e.to_port));
        // A wire is drawn full-strength only if it is live AND (with a selection up) inside the
        // influence. The wire and the cards it joins fade together — the whole point of the
        // reading is that a region of the canvas recedes as ONE region.
        let bright = crate::flow::edge_is_live(p.live, e.to_node)
            && p.focus
                .as_ref()
                .is_none_or(|f| crate::flow::edge_in_influence(f, e.from_node, e.to_node));
        let lit = is_hovered || is_selected;
        draw_wire(ctx, p.snap, e, p.view, p.theme, lit, bright);
        push_wire_hits(hits, p.snap, e, p.view, p.rect);
    }
}
