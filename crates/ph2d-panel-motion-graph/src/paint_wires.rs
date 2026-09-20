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
use crate::paint::{WireEmphasis, draw_wire};
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
        // A SELECTED wire wears the Accent a selected node's ring wears (the app's one selection
        // hue), so it reads as COMMITTED — distinct from a wire merely under the cursor, which keeps
        // the bright hover emphasis. `draw_wire` folds the two bools into its width. This is the
        // visible affordance for the click-then-Delete idiom (the alt-click Disconnect had none).
        let is_selected = p.state.selected_wires.contains(&(e.to_node, e.to_port));
        // A wire is drawn full-strength only if it is live AND (with a selection up) inside the
        // influence. The wire and the cards it joins fade together — the whole point of the
        // reading is that a region of the canvas recedes as ONE region.
        let bright = crate::flow::edge_is_live(p.live, e.to_node)
            && p.focus
                .as_ref()
                .is_none_or(|f| crate::flow::edge_in_influence(f, e.from_node, e.to_node));
        let emphasis = WireEmphasis::of(is_hovered, is_selected);
        draw_wire(ctx, p.snap, e, p.view, p.theme, emphasis, bright);
        // ⭐⭐⭐ **O REALCE DE UMA LARGADA, por CIMA** — a promessa (enquanto a mão paira) e o
        // eco (a desvanecer) são o MESMO traço com forças diferentes; a decisão vive em
        // [`crate::realce`]. Desenhado como um traço a mais e não dentro do [`draw_wire`]:
        // ele é ADITIVO e temporário, e enfiá-lo na função que decide a cor de toda a vida de um
        // fio misturaria um estado com um acontecimento.
        if let Some(forca) = crate::realce::realce_do_fio(p.state, (e.to_node, e.to_port))
            && let Some((p0, p3)) = crate::paint::wire_endpoints(p.snap, e, p.view)
        {
            ph2d_editor_core::paint::stroke_polyline(
                ctx.scene,
                &crate::paint::wire_polyline(p0, p3, p.view.zoom),
                crate::paint::WIRE_W_HOVER * p.view.zoom,
                ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Success, p.theme)
                    .multiply_alpha(forca),
            );
        }
        push_wire_hits(hits, p.snap, e, p.view, p.rect);
    }
}
