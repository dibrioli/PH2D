//! Motion-graph editor paint (Motion Nodes M1.E1–E7, Phase 1b).
//!
//! Reads the published [`GraphViewSnapshot`] + the ephemeral `Panel::State`
//! (pan/zoom/selection/drag/menu) and draws the graph into the docked
//! `motion_graph` region: background + grid, wires (under), then node cards with
//! category-tinted headers, domain-colored sockets, an Accent ring on the
//! selection, the in-progress wire ghost, and the add-node popup.
//!
//! **The canvas content is drawn inside a Vello clip layer.** Pan and zoom move
//! the graph freely in its own space, so a card (or a wire, or the drag ghost
//! that follows a captured pointer) routinely lands past the panel edge; without
//! the clip it paints straight over the scene viewport above. The split chrome
//! (divider + toolbar) is drawn OUTSIDE the clip, since the divider line
//! straddles the panel's own top edge. Hit rects are clipped to the same rect by
//! [`crate::hits`], so what is invisible is also unclickable.
//!
//! Hit rects are registered background → wires → node bodies → sockets (last
//! wins, so a socket beats the node body and a node beats a wire), and the canvas
//! rect is published so the M0 dispatch routes gestures back here. All colors are
//! tokens (HR-15); the add-menu is hit-tested panel-side against `Background`
//! gestures (no menu-specific `GraphHitKind`). Sizes are logical.

#[path = "paint_menu.rs"]
mod paint_menu;
use paint_menu::draw_menu;
#[path = "paint_breadcrumb.rs"]
mod paint_breadcrumb;
/// A faixa de params do cartão (ciclo 1, doc 103) — irmão por RESPONSABILIDADE: este
/// ficheiro desenha o que um cartão É, aquele o que ele CONTROLA.
#[path = "paint_card_params.rs"]
pub(crate) mod paint_card_params;
#[path = "paint_inert_badge.rs"]
mod paint_inert_badge;
#[path = "paint_port_label.rs"]
mod paint_port_label;

/// O gate de PIXEL da faixa de params — o que os seis de geometria não podiam ver.
#[cfg(test)]
#[path = "paint_card_params_tests.rs"]
mod paint_card_params_tests;
/// A ESPÉCIE e o PAPEL: a cor de um socket e o selo do cabeçalho — irmão cortado no teto de
/// LOC, por responsabilidade (ver o cabeçalho dele).
#[path = "paint_role.rs"]
mod paint_role;
#[path = "paint_stamp.rs"]
mod paint_stamp;
#[path = "paint_wire.rs"]
mod paint_wire;
#[path = "paint_wires.rs"]
mod paint_wires;
use paint_card_params::draw_card_params;
use paint_inert_badge::draw_inert_badge;
use paint_port_label::draw_port_labels;
pub use paint_port_label::{PortLabel, input_label_budget_px};
pub(crate) use paint_role::socket_tip;
use paint_role::{role_glyph, role_inset_px, socket_token};
#[path = "paint_grid.rs"]
mod paint_grid;
#[path = "paint_overlays.rs"]
mod paint_overlays;
use paint_grid::draw_grid;
use paint_overlays::draw_canvas_overlays;
/// **COMO SE DESENHA UM CARTÃO** — irmão cortado quando a ÁRVORE COMBINADA passou o teto.
#[path = "paint_card.rs"]
mod paint_card;

/// **A CÁPSULA** — o desenho de um nó abaixo do limiar do texto (ordem do dono, 2026-09-19).
/// Irmão do [`paint_card`] por RESPONSABILIDADE: aquele desenha o que se LÊ de perto, este o que
/// se RECONHECE de longe.
#[path = "paint_capsula.rs"]
pub(crate) mod paint_capsula;
use paint_card::draw_card;

/// **COMO SE DESENHA UM PINO** — irmão cortado no tecto de LOC (600) e por RESPONSABILIDADE:
/// este ficheiro desenha o CARTÃO, aquele o que se pendura na borda dele.
///
/// ⚠️ **DUAS linhas fizeram este corte, e a fusão textual guardou as DUAS declarações** — a
/// `line/UIUX` (a porta da moldura do tema empurrou o `paint.rs` para fora do teto) e a
/// `line/motion-value` (os params dentro do cartão, pela mesma razão). Os dois ficheiros tinham
/// as MESMAS três funções; o que colidiu foi o `mod` + o `use`, e só o compilador o viu.
#[path = "paint_socket.rs"]
mod paint_socket;
use paint_socket::{highlight_socket, paint_socket_glyph, port_out_domain};
use paint_stamp::{draw_preview, draw_preview_toggle};
pub(crate) use paint_wire::{
    WireEmphasis, detached_edge, draw_wire, draw_wire_ghost, draws_wire_ghost, wire_endpoints,
    wire_hit_polyline, wire_polyline, wires_crossed,
};
use paint_wires::{WirePass, draw_wires};

use crate::geom::{self, View, card_h, socket_center};
use crate::hits::{
    bg_hit_id, push_backdrop_hits, push_card_hit, push_inert_badge_hit, push_param_row_hits,
    push_preview_toggle_hit, push_socket_hits, register_hits, register_hot_tip,
};
use crate::snapshot::{
    GraphNodeView, GraphViewSnapshot, PortView, SocketGlyph, current_snapshot, socket_glyph,
};
use crate::state::{MotionGraphPanelState, ViewState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::paint::paint_text_title_elided;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, rect_to_vello, resolve, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::paint_shapes::fill_diamond;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::NodeUiCategory;
use ph2d_nodegraph::port::Domain;
use ph2d_tokens::{ColorToken, Theme};

// Draw-only metrics (the shared graph metrics live in `geom`). These are
// logical units in the graph's OWN coordinate space (scaled by `zoom` at paint),
// not chrome design tokens — the token system (Spacing/Radius/TypeToken/…) covers
// panel chrome, and a node-graph canvas has its own geometry (like the sprite /
// vector canvases). Hence `LITERAL-PX-OK` per line.
const CARD_RADIUS: f32 = 7.0; // LITERAL-PX-OK: node card corner radius
const BYPASS_STRIKE_W: f32 = 2.5; // LITERAL-PX-OK: the strike across a muted card
const SOCKET_R: f32 = 5.0; // LITERAL-PX-OK: socket dot radius
const FIT_PAD: f32 = 44.0; // LITERAL-PX-OK: fit-view margin
const TARGET_RING_PAD: f32 = 3.0; // LITERAL-PX-OK: compatible-target ring beyond the socket
pub(super) const TITLE_PAD_X: f32 = 8.0; // LITERAL-PX-OK: card title left inset
const TITLE_PAD_Y: f32 = 5.0; // LITERAL-PX-OK: card title top inset
const TITLE_SIZE: f32 = 13.0; // LITERAL-PX-OK: card title font size
pub(super) const TITLE_INSET_R: f32 = 12.0; // LITERAL-PX-OK: card title right inset
const READOUT_SIZE: f32 = 11.0; // LITERAL-PX-OK: inline readout font size (below the title's)
const READOUT_PAD_Y: f32 = 4.0; // LITERAL-PX-OK: inline readout top inset within its row
const PREVIEW_RADIUS: f32 = 3.0; // LITERAL-PX-OK: postage-stamp window corner radius
const PREVIEW_INSET: f32 = 4.0; // LITERAL-PX-OK: margin between the stamp's points and its frame
const PREVIEW_DOT_R: f32 = 1.3; // LITERAL-PX-OK: postage-stamp point radius
const PREVIEW_DOT_MIN: f32 = 0.6; // LITERAL-PX-OK: point radius floor, so a zoomed-out dot survives
const PREVIEW_MIN_H: f32 = 18.0; // LITERAL-PX-OK: below this the stamp is sub-pixel mush; draw the empty window
const STACK_OFFSET: f32 = 3.0; // LITERAL-PX-OK: per-step offset of a collapsed card's stack
const WIRE_W_DELAYED: f32 = 1.6; // LITERAL-PX-OK: hover-ghost stroke width revealing a pre pair
const WIRE_W_HOVER: f32 = 4.0; // LITERAL-PX-OK: hovered wire stroke width (targeted for alt-click)
const PRE_RING_W: f32 = 1.4; // LITERAL-PX-OK: portal badge ring stroke width
const PRE_DOT_R: f32 = 2.2; // LITERAL-PX-OK: portal badge inner dot radius
const GHOST_W: f32 = 2.2; // LITERAL-PX-OK: in-progress ghost wire stroke width
const BAND_W: f32 = 1.0; // LITERAL-PX-OK: rubber-band border stroke width
const KNIFE_W: f32 = 1.6; // LITERAL-PX-OK: knife stroke width
const WIRE_TANGENT: f32 = 20.0; // LITERAL-PX-OK: horizontal bezier handle length
const ZOOM_FIT_MIN: f32 = 0.35; // LITERAL-PX-OK: auto-fit zoom floor
const ZOOM_FIT_MAX: f32 = 1.2; // LITERAL-PX-OK: auto-fit zoom ceiling
// Add-node popup draw metrics.
const MENU_RADIUS: f32 = 6.0; // LITERAL-PX-OK: popup corner radius
const MENU_HEADER_PAD_X: f32 = 4.0; // LITERAL-PX-OK: header text x-pad beyond MENU_PAD
const MENU_HEADER_PAD_Y: f32 = 4.0; // LITERAL-PX-OK: header text y-pad
const MENU_HEADER_SIZE: f32 = 12.0; // LITERAL-PX-OK: header font size
const MENU_DOT_X: f32 = 7.0; // LITERAL-PX-OK: row category-dot x-inset
const MENU_DOT_R: f32 = 4.0; // LITERAL-PX-OK: row category-dot radius
const MENU_ROW_TEXT_X: f32 = 18.0; // LITERAL-PX-OK: row label x-inset
const MENU_ROW_TEXT_Y: f32 = 3.0; // LITERAL-PX-OK: row label y-inset
const MENU_ROW_SIZE: f32 = 13.0; // LITERAL-PX-OK: row label font size
const MENU_ROW_TEXT_INSET_R: f32 = 20.0; // LITERAL-PX-OK: row label right inset

/// **O frame do editor de grafo**, na ordem em que ele é montado: dobra os gestos deste frame
/// no estado, publica a seleção, e então desenha — fundo e grade, backdrops, fios, cards,
/// sockets — registrando os hits do menos prioritário ao mais (o último vence). Os overlays
/// transientes saem por [`draw_canvas_overlays`], ainda dentro do clip; o chrome do split e o
/// breadcrumb ficam FORA dele, porque a linha do divisor monta na borda do próprio painel.
///
/// ⚠️ Este doc-comment estava ÓRFÃO (uma linha solta, `que uma delas ganhasse um caso
/// especial.`, cauda de uma frase cuja cabeça alguma edição comeu). Um fragmento não diz
/// nada sobre o que a função faz, e é pior que comentário nenhum na porta de entrada do
/// painel — reescrito, não apagado.
pub(crate) fn paint(state: &mut MotionGraphPanelState, ctx: &mut PaintCtx) {
    let rect = ctx.layout.motion_graph;
    let theme = ctx.host.theme();
    let snap = current_snapshot();

    // **A new level is a new canvas** (doc 57): entering a group, or walking the
    // breadcrumb out of one, re-fits. The members sit where they always sat — the
    // fold moves nothing — so a view zoomed in on the card would open the group
    // showing a corner of it, or nothing at all. Blender and Houdini both re-frame
    // on the way in; so does this.
    // ⭐ **O eco da última largada**, lido UMA vez por quadro — a shell já o entrega resolvido
    // em intensidade, porque o único relógio que o painel recebe no retrato é o PLAYHEAD.
    state.piscada_viva = crate::snapshot::current_graph_flash();
    state.sync_level(snap.level);
    // Fit on first sight (then the user owns pan/zoom; F re-fits). A MANUAL fit with
    // a selection frames it, not the whole graph (`fit_selection`); every auto-fit
    // (first sight, level change) leaves that flag false and so frames everything.
    if !state.fitted && !snap.nodes.is_empty() {
        let scope = state.fit_selection.then_some(&state.selected);
        state.view = fit(&snap, rect, scope);
        state.fitted = true;
        state.fit_selection = false;
    }

    // ⭐ **A BANDA que o divisor parte** — a régua da fracção, publicada pelo layout. ⛔ Não é
    // `center_viewport + rect`: com a timeline docada dentro do split essa soma não é a banda, e
    // era daí que vinham o offset e o tremor do arrasto (ver `HeroLayout::split_band`).
    let band = ctx.layout.split_band;

    // Fold this frame's gestures/zoom/keys into the state before drawing.
    crate::interact::process(state, ctx, rect, band, &snap);

    // Publish the selection so the shell bridge can build the params snapshot for
    // the selected node (M1.P1) — or for the selected backdrop (F2: the params
    // panel shows the properties of whatever ONE subject is selected). Cheap: a
    // small Vec, only while the tool is up.
    crate::snapshot::set_graph_selection(state.selected.iter().copied().collect());
    crate::snapshot::set_graph_backdrop_selection(state.selected_backdrop);
    // ⭐ **E o param em ARRASTO** — o canal que faz um gizmo de canvas acender enquanto a mão
    // mexe num knob do cartão (ver `snapshot::set_graph_param_scrub`). Publicado AQUI, logo a
    // seguir ao `interact::process`, para ser o estado DESTE quadro e não o do anterior.
    crate::snapshot::set_graph_param_scrub(match state.interaction {
        crate::state::Interaction::ScrubParam { node, param, .. } => Some((node, param)),
        _ => None,
    });

    let view = View::new(rect, state.view);

    // Hit rects, lowest-priority first (last registered wins): background →
    // backdrop headers/grippers → wires → node bodies → sockets. So a socket beats
    // the node body it sits on, a node body beats a wire behind it, a wire beats a
    // backdrop's header, and everything beats empty canvas. Each is clipped to
    // `rect` by `hits`, matching the paint clip below.
    let mut hits: Vec<(NodeId, GraphHitKind, Rect)> =
        vec![(bg_hit_id(), GraphHitKind::Background, rect)];

    // ── Canvas content: clipped to the panel, so a card / wire panned past the
    // edge never paints over the scene viewport above. ──────────────────────
    ctx.scene.push_clip(&rect_to_vello(rect));
    fill_rounded_rect(ctx.scene, rect, 0.0, resolve(ColorToken::GraphBg, theme));
    draw_grid(ctx, rect, theme);

    // Backdrops: behind the wires and cards (they are the wallpaper of the group).
    // Only the header + gripper get hit rects — the BODY is click-through, so a
    // click or box-select over a backdrop still reaches the nodes and the canvas
    // beneath it (see `backdrop`'s module docs).
    for b in &snap.backdrops {
        let selected = state.selected_backdrop == Some(b.id);
        crate::backdrop::draw(ctx, b, &view, theme, selected);
        push_backdrop_hits(&mut hits, b, &view, rect);
    }

    // Wires under the cards. The hovered wire (the hit-index id under the cursor,
    // tracked by the shell's free-move hover) is drawn emphasised so it reads as
    // the alt-click delete target.
    let hovered = ctx.host.store().hot_id();
    // The wire whose END is being dragged off its input is NOT drawn (doc 45): the artist is
    // visibly pulling it out of that socket, and leaving it plugged in would say the gesture
    // had not taken. Its hit rects go with it — you cannot alt-click a wire you are holding.
    let detached = detached_edge(state);
    // **What the cook actually pulls** (F3): reachability backwards from the sinks. Computed
    // ONCE per paint and read by both the wires and the cards, so a dead branch fades as one
    // thing — a veiled card still trailing a full-strength wire would be worse than no veil.
    let live = crate::flow::live_set(&snap);
    // **What the selection touches** — its ancestors and its descendants (F3). `None` when
    // nothing is selected: an empty selection dims NOTHING (a canvas that goes grey the moment
    // you click empty space would punish the most common gesture there is).
    let focus =
        (!state.selected.is_empty()).then(|| crate::flow::influence_set(&snap, &state.selected));
    let veiled = |id: u32| !live.contains(&id) || focus.as_ref().is_some_and(|f| !f.contains(&id));
    draw_wires(
        WirePass {
            state,
            snap: &snap,
            view: &view,
            theme,
            rect,
            hovered,
            detached,
            live: &live,
            focus: &focus,
        },
        ctx,
        &mut hits,
    );
    // Cards, collecting body hits as we draw them. A card whose rect does not touch the panel is
    // SKIPPED entirely: the clip layer already hides it, but Vello still has to bound and bin
    // every path inside it — panning a big graph would pay for cards nobody can see. (Its hit
    // rects are clipped away by `hits` anyway, so what is invisible stays unclickable.)
    let on_screen: Vec<&GraphNodeView> = snap
        .nodes
        .iter()
        .filter(|n| touches(geom::card_rect(n, &view), rect))
        .collect();
    for n in &on_screen {
        // A GHOST is always veiled: it is not part of this level, and the veil is the
        // whole message (doc 57). It never counts as "inert" or "out of the influence"
        // — those are readings about the graph, and a ghost is a reading about the
        // BOUNDARY.
        let dim = n.kind == crate::snapshot::NodeViewKind::Ghost || veiled(n.id);
        // `draw_card` also draws this node's ⚠ inert badge (ADR-0155) on its corner.
        let body = draw_card(ctx, state, n, &view, theme, dim);
        push_card_hit(&mut hits, n, body, rect);
        // ⚠️ **Depois do corpo, para lhes GANHAR o gesto** — ver `push_param_row_hits`.
        push_param_row_hits(&mut hits, n, &view, rect);
    }
    // Sockets + the header toggle + the inert badge last, so all three beat the card body
    // under them (doc 86; ADR-0155).
    for n in &on_screen {
        push_socket_hits(&mut hits, n, &view, rect);
        push_preview_toggle_hit(&mut hits, n, &view, rect);
        push_inert_badge_hit(&mut hits, n, &view, rect);
    }
    draw_canvas_overlays(ctx, state, &snap, &view, theme, rect);
    ctx.scene.pop_layer();

    // Split chrome (E9): the draggable divider at the scene boundary + the
    // SplitH / SplitV / Fit toolbar. Drawn + hit-registered above the graph
    // content so they win a click there, and OUTSIDE the clip — the divider line
    // straddles the panel's own top edge and the clip would halve it.
    crate::paint_chrome::draw_split_chrome(
        ctx,
        rect,
        band,
        theme,
        &mut hits,
        crate::paint_chrome::ChromeState {
            knife_armed: state.knife_armed,
            probe_armed: state.probe_armed,
            group_verb: crate::interact::group_verb(state),
            // ADR-0155: the shell owns the flag; the chip reads the published value.
            node_help: crate::snapshot::node_help(),
        },
    );
    // The breadcrumb (doc 57), top-left: where you are, and every way back out. Drawn
    // only when you are somewhere — at the root there is nothing to walk back to.
    paint_breadcrumb::draw(ctx, rect, theme, &snap, &mut hits);
    // While the add-menu is open, a full-canvas Background shield registered LAST
    // makes every click resolve as Background — so a menu row drawn over a card /
    // socket still reaches the menu, and a click off the menu dismisses it
    // (`interact::apply_background`).
    if state.menu.is_some() || state.editor.is_some() {
        hits.push((bg_hit_id(), GraphHitKind::Background, rect));
    }

    register_hits(ctx, rect, &hits);
    crate::hits::register_card_swatches(ctx, &snap, &view, rect);
    // O balão do socket sob o rato — UM por quadro, derivado da lista de hits acima.
    register_hot_tip(ctx, &snap.nodes, &hits);

    // The rename box (doc 61) is drawn OVER the thing's title, after everything else and after
    // the hit registration — it is a widget, not a graph hit, and it registers itself with the
    // panel's widget index like the menu's search field does.
    crate::rename::paint(state, ctx, rect, &snap, &view);
    // ⭐ **A caixa de escrever um número** (ciclo 1) — pela mesma porta e pela mesma razão que
    // a de renomear: é um widget sobre o cartão, registado DEPOIS dos hits do grafo, para o
    // clique dentro dela chegar a ela e não à row que está por baixo.
    crate::param_edit::paint(state, ctx, &snap, &view);
    // ⭐⭐ **A janela do editor rico** — por cima de tudo, e com os widgets dela registados
    // DEPOIS do escudo de fundo, para o clique numa alça chegar à alça e o clique fora fechar.
    crate::param_editor::paint(state, ctx, rect, theme);
}

/// Scale + center a bounding box into `rect`. `selection = Some(ids)` frames only
/// those nodes (Frame Selected); `None` frames the whole graph. A selection that
/// matches no VISIBLE node (stale, or on another level) falls back to the whole
/// graph — a Fit that framed empty space would strand the artist looking at nothing.
pub(crate) fn fit(
    snap: &GraphViewSnapshot,
    rect: Rect,
    selection: Option<&std::collections::BTreeSet<u32>>,
) -> ViewState {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let mut framed = 0usize;
    for n in &snap.nodes {
        if let Some(sel) = selection
            && !sel.contains(&n.id)
        {
            continue;
        }
        min_x = min_x.min(n.x);
        min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + geom::CARD_W);
        max_y = max_y.max(n.y + card_h(n));
        framed += 1;
    }
    // The selection named nothing on this level: frame the whole graph instead of a
    // degenerate empty box.
    if framed == 0 && selection.is_some() {
        return fit(snap, rect, None);
    }
    let bw = (max_x - min_x).max(1.0);
    let bh = (max_y - min_y).max(1.0);
    let zoom = ((rect.w - 2.0 * FIT_PAD) / bw)
        .min((rect.h - 2.0 * FIT_PAD) / bh)
        .clamp(ZOOM_FIT_MIN, ZOOM_FIT_MAX); // CLAMP-OK: const bounds, min<max, non-NaN
    ViewState {
        pan_x: (rect.w - bw * zoom) * 0.5 - min_x * zoom,
        pan_y: (rect.h - bh * zoom) * 0.5 - min_y * zoom,
        zoom,
    }
}

/// Do two rects overlap at all? (A card that merely touches the panel edge is still drawn: half a
/// card is what tells the artist there is more canvas that way.)
fn touches(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// The strike across a muted card: a single line from the body's top-left corner to its
/// bottom-right. It SPANS the card — the "off" gesture reads only if it crosses the whole thing,
/// not as a dot or a stub. A pure function so the geometry is gate-able without a Vello scene.
pub(crate) fn bypass_strike(body: Rect) -> [(f32, f32); 2] {
    [(body.x, body.y), (body.x + body.w, body.y + body.h)]
}

/// FNV-1a-64 of `key` — the runtime sibling of `hash_node_id` (which is
/// `&'static str`-only) for the dynamic per-element hit ids. Shared with
/// `paint_chrome` (the split divider / toolbar hit ids).
pub(crate) fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}
/// Opaque `Wire { edge }` handle = the target input `(to_node, to_port)`, which
/// uniquely identifies the edge (one edge per input). Decoded in `interact` for
/// `Disconnect`.
pub(crate) fn wire_handle(to_node: u32, to_port: u16) -> u64 {
    ((to_node as u64) << 16) | (to_port as u64)
}
/// Inverse of [`wire_handle`].
pub(crate) fn wire_target(handle: u64) -> (u32, u16) {
    ((handle >> 16) as u32, (handle & 0xffff) as u16)
}

/// The `node-cat-*` header/dot tint for a category (the colour that *teaches the library map*, plan §2.4).
/// The single source — the graph menu row dot, and now the shell's full-screen palette model, both call it.
pub fn cat_token(c: NodeUiCategory) -> ColorToken {
    match c {
        NodeUiCategory::Source => ColorToken::NodeCatSource,
        NodeUiCategory::Distribute => ColorToken::NodeCatDistribute,
        NodeUiCategory::Transform => ColorToken::NodeCatTransform,
        NodeUiCategory::Focus => ColorToken::NodeCatFocus,
        NodeUiCategory::Fx => ColorToken::NodeCatFx,
        NodeUiCategory::Output => ColorToken::NodeCatOutput,
        NodeUiCategory::Utility => ColorToken::NodeCatUtility,
    }
}

pub(super) fn domain_token(d: Domain) -> ColorToken {
    match d {
        Domain::Instances => ColorToken::PortInstances,
        Domain::Vector => ColorToken::PortVector,
        Domain::Field => ColorToken::PortField,
        Domain::Signal => ColorToken::PortSignal,
        Domain::Control => ColorToken::PortControl,
    }
}

/// Flatten a cubic Bézier into `n` polyline points.
fn cubic_polyline(
    p0: (f32, f32),
    c1: (f32, f32),
    c2: (f32, f32),
    p3: (f32, f32),
    n: usize,
) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|k| {
            let t = k as f32 / n as f32;
            let u = 1.0 - t;
            // Cubic Bézier Bernstein basis: B(t) = u³·p0 + 3u²t·c1 + 3ut²·c2 + t³·p3.
            let three = 3.0; // LITERAL-PX-OK: Bernstein-basis coefficient (math, not design)
            let (a, b, c, d) = (u * u * u, three * u * u * t, three * u * t * t, t * t * t);
            (
                a * p0.0 + b * c1.0 + c * c2.0 + d * p3.0,
                a * p0.1 + b * c1.1 + c * c2.1 + d * p3.1,
            )
        })
        .collect()
}
