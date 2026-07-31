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
#[path = "paint_stamp.rs"]
mod paint_stamp;
#[path = "paint_wire.rs"]
mod paint_wire;
use paint_stamp::draw_preview;
pub(crate) use paint_wire::{
    detached_edge, draw_wire, draw_wire_ghost, draws_wire_ghost, wire_endpoints, wire_hit_polyline,
    wires_crossed,
};

use crate::geom::{self, View, card_h, socket_center};
use crate::hits::{
    bg_hit_id, push_backdrop_hits, push_card_hit, push_socket_hits, push_wire_hits, register_hits,
};
use crate::snapshot::{
    GraphNodeView, GraphViewSnapshot, PortView, SocketGlyph, current_snapshot, socket_glyph,
};
use crate::state::{Interaction, MotionGraphPanelState, ViewState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_title, rect_to_vello, resolve, stroke_polyline,
    stroke_rounded_rect,
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
const TITLE_PAD_X: f32 = 8.0; // LITERAL-PX-OK: card title left inset
const TITLE_PAD_Y: f32 = 5.0; // LITERAL-PX-OK: card title top inset
const TITLE_SIZE: f32 = 13.0; // LITERAL-PX-OK: card title font size
const TITLE_INSET_R: f32 = 12.0; // LITERAL-PX-OK: card title right inset
const READOUT_SIZE: f32 = 11.0; // LITERAL-PX-OK: inline readout font size (below the title's)
const READOUT_PAD_Y: f32 = 4.0; // LITERAL-PX-OK: inline readout top inset within its row
const PREVIEW_RADIUS: f32 = 3.0; // LITERAL-PX-OK: postage-stamp window corner radius
const PREVIEW_INSET: f32 = 4.0; // LITERAL-PX-OK: margin between the stamp's points and its frame
const PREVIEW_DOT_R: f32 = 1.3; // LITERAL-PX-OK: postage-stamp point radius
const PREVIEW_DOT_MIN: f32 = 0.6; // LITERAL-PX-OK: point radius floor, so a zoomed-out dot survives
const PREVIEW_MIN_H: f32 = 18.0; // LITERAL-PX-OK: below this the stamp is sub-pixel mush; draw the empty window
const GRID_STEP: f32 = 32.0; // LITERAL-PX-OK: background grid spacing
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

pub(crate) fn paint(state: &mut MotionGraphPanelState, ctx: &mut PaintCtx) {
    let rect = ctx.layout.motion_graph;
    let theme = ctx.host.theme();
    let snap = current_snapshot();

    // **A new level is a new canvas** (doc 57): entering a group, or walking the
    // breadcrumb out of one, re-fits. The members sit where they always sat — the
    // fold moves nothing — so a view zoomed in on the card would open the group
    // showing a corner of it, or nothing at all. Blender and Houdini both re-frame
    // on the way in; so does this.
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

    // The scene half of the center split — the divider drag maps the pointer to a
    // split fraction against the full center band (`center` + `rect`), and the
    // toolbar highlights the active orientation (E9).
    let center = ctx.layout.center_viewport;

    // Fold this frame's gestures/zoom/keys into the state before drawing.
    crate::interact::process(state, ctx, rect, center, &snap);

    // Publish the selection so the shell bridge can build the params snapshot for
    // the selected node (M1.P1) — or for the selected backdrop (F2: the params
    // panel shows the properties of whatever ONE subject is selected). Cheap: a
    // small Vec, only while the tool is up.
    crate::snapshot::set_graph_selection(state.selected.iter().copied().collect());
    crate::snapshot::set_graph_backdrop_selection(state.selected_backdrop);

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
    for e in &snap.edges {
        if detached == Some((e.to_node, e.to_port)) {
            continue;
        }
        let is_hovered = hovered == Some(crate::hits::wire_hit_id(e.to_node, e.to_port));
        // A wire is drawn full-strength only if it is live AND (with a selection up) inside the
        // influence. The wire and the cards it joins fade together — the whole point of the
        // reading is that a region of the canvas recedes as ONE region.
        let bright = crate::flow::edge_is_live(&live, e.to_node)
            && focus
                .as_ref()
                .is_none_or(|f| crate::flow::edge_in_influence(f, e.from_node, e.to_node));
        draw_wire(ctx, &snap, e, &view, theme, is_hovered, bright);
        push_wire_hits(&mut hits, &snap, e, &view, rect);
    }
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
        let body = draw_card(ctx, state, n, &view, theme, dim);
        push_card_hit(&mut hits, n, body, rect);
    }
    // Sockets last so they win the overlap with the node edge they sit on.
    for n in &on_screen {
        push_socket_hits(&mut hits, n, &view, rect);
    }
    // Overlays above the cards, still clipped: the in-progress wire ghost (it
    // tracks a CAPTURED pointer, which routinely leaves the panel) and the
    // add-menu (already clamped on-canvas).
    if draws_wire_ghost(&state.interaction) {
        draw_wire_ghost(ctx, &snap, state, &view, theme);
    }
    // The probe readout, over the cards (it is a HUD, not part of the graph).
    if let Some(p) = &snap.probe
        && let Some(n) = snap.nodes.iter().find(|n| n.id == p.node)
    {
        crate::probe::draw(ctx, p, n, &view, theme);
    }
    // The knife stroke — Danger, because it is one: what it crosses gets cut.
    if let Interaction::Knife { anchor, cur } = state.interaction {
        stroke_polyline(
            ctx.scene,
            &[anchor, cur],
            KNIFE_W,
            resolve(ColorToken::Danger, theme),
        );
    }
    // The rubber band (left-drag on empty canvas). Translucent fill — it is drawn
    // OVER the very cards it is selecting, and an opaque one would hide them.
    if let Interaction::BoxSelect { anchor, cur, .. } = state.interaction {
        let band = geom::band_rect(anchor, cur);
        fill_rounded_rect(
            ctx.scene,
            band,
            0.0,
            resolve(ColorToken::GraphMarquee, theme),
        );
        stroke_rounded_rect(
            ctx.scene,
            band,
            0.0,
            BAND_W,
            resolve(ColorToken::Accent, theme),
        );
    }
    if let Some(menu) = &state.menu {
        draw_menu(ctx, menu, &snap, rect, theme);
    }
    ctx.scene.pop_layer();

    // Split chrome (E9): the draggable divider at the scene boundary + the
    // SplitH / SplitV / Fit toolbar. Drawn + hit-registered above the graph
    // content so they win a click there, and OUTSIDE the clip — the divider line
    // straddles the panel's own top edge and the clip would halve it.
    crate::paint_chrome::draw_split_chrome(
        ctx,
        rect,
        center,
        theme,
        &mut hits,
        crate::paint_chrome::ChromeState {
            knife_armed: state.knife_armed,
            probe_armed: state.probe_armed,
            group_verb: crate::interact::group_verb(state),
        },
    );
    // The breadcrumb (doc 57), top-left: where you are, and every way back out. Drawn
    // only when you are somewhere — at the root there is nothing to walk back to.
    paint_breadcrumb::draw(ctx, rect, theme, &snap, &mut hits);
    // While the add-menu is open, a full-canvas Background shield registered LAST
    // makes every click resolve as Background — so a menu row drawn over a card /
    // socket still reaches the menu, and a click off the menu dismisses it
    // (`interact::apply_background`).
    if state.menu.is_some() {
        hits.push((bg_hit_id(), GraphHitKind::Background, rect));
    }

    register_hits(ctx, rect, &hits);

    // The rename box (doc 61) is drawn OVER the thing's title, after everything else and after
    // the hit registration — it is a widget, not a graph hit, and it registers itself with the
    // panel's widget index like the menu's search field does.
    crate::rename::paint(state, ctx, rect, &snap, &view);
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

fn draw_grid(ctx: &mut PaintCtx, rect: Rect, theme: Theme) {
    let grid = resolve(ColorToken::GraphGrid, theme);
    let step = GRID_STEP;
    let mut x = rect.x;
    while x < rect.x + rect.w {
        stroke_polyline(ctx.scene, &[(x, rect.y), (x, rect.y + rect.h)], 1.0, grid);
        x += step;
    }
    let mut y = rect.y;
    while y < rect.y + rect.h {
        stroke_polyline(ctx.scene, &[(rect.x, y), (rect.x + rect.w, y)], 1.0, grid);
        y += step;
    }
}

/// Do two rects overlap at all? (A card that merely touches the panel edge is still drawn: half a
/// card is what tells the artist there is more canvas that way.)
fn touches(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Draw one node card; returns its screen-space body rect (for hit registration).
fn draw_card(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
    veiled: bool,
) -> Rect {
    let (sx, sy) = view.pt(n.x, n.y);
    let w = geom::CARD_W * view.zoom;
    let h = card_h(n) * view.zoom;
    let r = CARD_RADIUS * view.zoom;
    let body = Rect::new(sx, sy, w, h);

    // **A collapsed subgraph draws as a STACK** (doc 57): two cards peeking out from
    // behind the front one. It is the universal idiom for "there is more than one
    // thing here", it needs no icon and no legend, and it is the only difference
    // between a card that opens and a card that does not — which is exactly the
    // difference a double-click depends on the artist knowing.
    if n.kind == crate::snapshot::NodeViewKind::Subgraph {
        for step in [2.0, 1.0] {
            let off = STACK_OFFSET * step * view.zoom;
            let back = Rect::new(sx + off, sy - off, w, h);
            fill_rounded_rect(ctx.scene, back, r, resolve(ColorToken::Bg3, theme));
            stroke_rounded_rect(ctx.scene, back, r, 1.0, resolve(ColorToken::Border, theme));
        }
    }

    fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::Bg2, theme));
    let header = Rect::new(sx, sy, w, geom::HEADER_H * view.zoom);
    fill_rounded_rect(ctx.scene, header, r, resolve(cat_token(n.category), theme));
    stroke_rounded_rect(ctx.scene, body, r, 1.0, resolve(ColorToken::Border, theme));
    if state.selected.contains(&n.id) {
        stroke_rounded_rect(ctx.scene, body, r, 2.0, resolve(ColorToken::Accent, theme));
    }

    paint_text_title(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        sx + TITLE_PAD_X * view.zoom,
        sy + TITLE_PAD_Y * view.zoom,
        TITLE_SIZE * view.zoom,
        w - TITLE_INSET_R * view.zoom,
        resolve(ColorToken::Text1, theme),
    );

    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        paint_socket_glyph(ctx, cx, cy, SOCKET_R * view.zoom, p, theme);
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        paint_socket_glyph(ctx, cx, cy, SOCKET_R * view.zoom, p, theme);
    }

    // The inline readout: what this card produced on this frame's cook, under its sockets.
    // Text2 (the muted tone), not Text1 — it is a live instrument reading, not a label the
    // artist authored, and it must not compete with the node's own name.
    if let Some(text) = &n.readout {
        let row_y = sy + (geom::HEADER_H + geom::card_rows(n) * geom::ROW_H) * view.zoom;
        paint_text_title(
            ctx.text_system,
            ctx.scene,
            text,
            sx + TITLE_PAD_X * view.zoom,
            row_y + READOUT_PAD_Y * view.zoom,
            READOUT_SIZE * view.zoom,
            w - TITLE_INSET_R * view.zoom,
            resolve(ColorToken::Text2, theme),
        );
    }

    draw_preview(ctx, n, view, theme);

    // **Veiled** — the card is not part of what the artist is looking at. Two reasons, ONE
    // veil (a card veiled twice is just darker, and the artist cannot read "why" out of a
    // shade):
    //
    // - **Inert**: no sink reaches it, so the cook never pulls it and nothing it does reaches
    //   the canvas. The reading is REACHABILITY (F3), not "has a readout" (F2): a node cooked
    //   inside a scoped lane (`motion.time_remap`) has no root-lane memo and therefore no
    //   number — it is consumed all the same, and veiling it would be a lie it could not argue
    //   with.
    // - **Out of the influence**: something is selected, and this card neither feeds it nor is
    //   fed by it. Esc drops the selection and the whole canvas comes back.
    //
    // A veil, not a repaint: the card keeps its category colour, its title and its sockets, and
    // stays perfectly grabbable. It recedes; it does not become a different kind of thing.
    // (The selection ring is drawn ABOVE it — a selected node must still look selected.)
    if veiled {
        fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::GraphInert, theme));
        if state.selected.contains(&n.id) {
            stroke_rounded_rect(ctx.scene, body, r, 2.0, resolve(ColorToken::Accent, theme));
        }
    }

    // **Switched OFF** (bypass/mute — H). A bypassed node IS inert — nothing flows through it — so
    // it takes the same veil as an unwired card; the STRIKE, corner to corner, is what says the
    // inertness is DELIBERATE (you muted it) rather than an accident (you forgot to wire it). It
    // sits above the veil and the selection ring so a muted node reads as muted even when selected.
    if n.bypassed {
        fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::GraphInert, theme));
        let strike = bypass_strike(body);
        stroke_polyline(
            ctx.scene,
            &strike,
            BYPASS_STRIKE_W * view.zoom,
            resolve(ColorToken::Text2, theme),
        );
    }
    body
}

/// The strike across a muted card: a single line from the body's top-left corner to its
/// bottom-right. It SPANS the card — the "off" gesture reads only if it crosses the whole thing,
/// not as a dot or a stub. A pure function so the geometry is gate-able without a Vello scene.
pub(crate) fn bypass_strike(body: Rect) -> [(f32, f32); 2] {
    [
        (body.x, body.y),
        (body.x + body.w, body.y + body.h),
    ]
}

/// Draw one socket dot: coloured by [`Domain`], SHAPED by [`Dim`] — a diamond ◇ for a
/// multi-component column, a circle ○ for a single value ([`socket_glyph`]). The one
/// door the input AND output loops go through, so the two can never disagree on how a
/// socket looks (the bug a second copy invites six months on).
fn paint_socket_glyph(ctx: &mut PaintCtx, cx: f32, cy: f32, r: f32, p: &PortView, theme: Theme) {
    let color = resolve(domain_token(p.domain), theme);
    match socket_glyph(p.dim) {
        SocketGlyph::Value => fill_circle(ctx.scene, cx, cy, r, color),
        SocketGlyph::Column => fill_diamond(ctx.scene, cx, cy, r, color),
    }
}

/// A ring around a socket (the drop-target highlight), drawn as a rounded-rect stroke
/// whose corner radius equals its half-side — i.e. a circle — so it reuses
/// `stroke_rounded_rect` (no per-frame trig, HR-5-clean). `token` is `Accent` for a
/// compatible target and `Danger` for an incompatible one: the ring is a circle
/// regardless of the socket's own glyph, because it is a halo, not the socket.
fn highlight_socket(ctx: &mut PaintCtx, cx: f32, cy: f32, r: f32, theme: Theme, token: ColorToken) {
    let d = 2.0 * r;
    stroke_rounded_rect(
        ctx.scene,
        Rect::new(cx - r, cy - r, d, d),
        r,
        2.0,
        resolve(token, theme),
    );
}

/// The source output port's [`Domain`] (ghost color) — the snapshot carries it
/// on the port view.
fn port_out_domain(n: &GraphNodeView, port: u16) -> Domain {
    n.outputs
        .get(port as usize)
        .map(|p| p.domain)
        .unwrap_or(Domain::Instances)
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

pub(crate) fn cat_token(c: NodeUiCategory) -> ColorToken {
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

fn domain_token(d: Domain) -> ColorToken {
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
