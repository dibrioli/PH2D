//! Graph-canvas hit rects (split from `paint` for the panel LOC cap).
//!
//! Builds the `GraphSurface` hit rects the M0 dispatch routes gestures through
//! — background, wires, node bodies, sockets — and registers them.
//!
//! **Every content hit rect is clipped to the canvas.** The graph draws inside a
//! Vello clip layer (`paint`), so content the user panned/zoomed past the panel
//! edge is invisible; an unclipped hit rect for it would stay clickable — an
//! invisible target floating over the scene, stealing the click from the
//! viewport. Clipping the geometry and the hit rects from the same `canvas`
//! keeps "what you see" and "what you can click" the same set.

use crate::geom::{self, View, socket_center};
use crate::paint::{fnv_id, wire_handle, wire_hit_polyline};
use crate::snapshot::{GraphEdgeView, GraphNodeView, GraphViewSnapshot};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{GraphHitKind, InteractiveState};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;

/// How many polyline points a wire's hit path is sampled at (segments = N-1);
/// each segment becomes one padded hit rect sharing the wire's id. Denser than
/// the draw sampling so the padded segment boxes hug the curve (fewer gaps /
/// off-curve false hits) now that the band is wider.
/// Half-thickness (screen px) of a wire's hit rects — a forgiving hover / alt-
/// click zone (a full band of `2 × WIRE_HIT_R`, independent of zoom).
pub(crate) const WIRE_HIT_R: f32 = 8.0; // LITERAL-PX-OK: wire hit half-thickness

// Portal-badge geometry for `pre` edges — shared by paint and the hit path so
// the clickable thing is exactly the visible thing (docs/Motion Nodes/03: a
// `pre` edge draws as badges at its sockets, never as a spline).
pub(crate) const PRE_BADGE_R: f32 = 6.0; // LITERAL-PX-OK: portal badge outer radius
pub(crate) const PRE_BADGE_OFF: f32 = 15.0; // LITERAL-PX-OK: badge centre offset out from its socket
const PRE_BADGE_DROP: f32 = 11.0; // LITERAL-PX-OK: source badge drop below the socket line
const PRE_BADGE_HIT_PAD: f32 = 3.0; // LITERAL-PX-OK: extra hit slop around a badge

/// The badge centre(s) standing in for a `pre` edge's spline: one just outside
/// the source's output socket, one just outside the target's input socket. A
/// self-loop (the sequential-node template) collapses to the input-side badge
/// alone — one glyph on the card, not a hairpin.
///
/// The source badge drops BELOW the socket line: the same output usually also
/// feeds a forward wire (integrate → tint), and a badge sitting on that
/// spline would read as a bead on the wrong edge. The target badge stays on
/// the line — a plumbed head has no other incoming wire by construction.
pub(crate) fn pre_badge_centers(
    p0: (f32, f32),
    p3: (f32, f32),
    zoom: f32,
    self_loop: bool,
) -> [Option<(f32, f32)>; 2] {
    let src = (p0.0 + PRE_BADGE_OFF * zoom, p0.1 + PRE_BADGE_DROP * zoom);
    let dst = (p3.0 - PRE_BADGE_OFF * zoom, p3.1);
    [(!self_loop).then_some(src), Some(dst)]
}

/// `r` intersected with `canvas`, or `None` when they do not overlap. The clip
/// that hides content past the panel edge must hide its hit rect too (see the
/// module docs).
pub(crate) fn clip_rect(r: Rect, canvas: Rect) -> Option<Rect> {
    let x0 = r.x.max(canvas.x);
    let y0 = r.y.max(canvas.y);
    let x1 = (r.x + r.w).min(canvas.x + canvas.w);
    let y1 = (r.y + r.h).min(canvas.y + canvas.h);
    (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// Push a node body's hit rect, clipped to the canvas (skipped when fully off).
///
/// **A ghost registers a body too** (doc 57, Enio smoke 2026-07-13: *"o primeiro e
/// último nó do grupo não pode ser movido, mas deveria poder para arrumar melhor os
/// nós"*). It used to register none, on the theory that dragging it would move a node
/// on a canvas the artist is not looking at — but that theory is wrong: a node has
/// exactly ONE position, and the artist looking at the ghost IS looking at the node.
/// Blender and Houdini both let you move the boundary proxy from inside the group,
/// and for the same reason: you cannot tidy a room whose doorways are nailed down.
///
/// What a ghost still cannot do is **be deleted from here** — the shell refuses that
/// with a toast (`intents`), because deleting it would reach into a canvas the artist
/// is not on.
pub(crate) fn push_card_hit(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    body: Rect,
    canvas: Rect,
) {
    if let Some(r) = clip_rect(body, canvas) {
        hits.push((
            node_hit_id(n.id),
            GraphHitKind::Node { node: n.id as u64 },
            r,
        ));
    }
}

/// **OS ALVOS DAS ROWS DE PARAM** (ciclo 1) — a faixa inteira de cada row, que é também o que
/// se arrasta (o número do Blender: pega-se em qualquer ponto da fileira).
///
/// ⚠️ **Empurradas DEPOIS do corpo do cartão**, e por isso ganham-lhe o gesto: `register_hits`
/// é *«na ordem dada — a última ganha»*. Com os params dentro do nó, arrastar o NÓ passa a ser
/// pelo cabeçalho e pelas folgas entre as fileiras. É o preço declarado de ter os controlos ali,
/// e é o que o Blender faz.
///
/// ⚠️ **Nada é registado abaixo de [`geom::param_row_is_grabbable`]** — um alvo que não se vê é
/// um alvo invisível, e roubaria o arrasto do nó sem dar nada em troca.
pub(crate) fn push_param_row_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    view: &crate::geom::View,
    canvas: Rect,
) {
    if !crate::geom::param_row_is_grabbable(view) {
        return;
    }
    for i in 0..crate::geom::band_len(n) {
        match crate::geom::band_at(n, i) {
            // Um param DIRIGIDO por fio não se arrasta: o número vem de fora, e um alvo que não
            // obedece ao dedo é a mentira que a row dirigida do painel já aprendeu a não contar.
            Some(crate::geom::BandRow::Param(k)) if n.params[k].driven => continue,
            // Um cabeçalho de secção É um alvo: o clique dobra. Mesma variante de hit, mesma
            // coordenada — quem decide o que fazer é o `band_at` do lado do gesto.
            Some(_) => {}
            None => break,
        }
        let Ok(row) = u16::try_from(i) else { continue };
        if let Some(r) = clip_rect(crate::geom::param_row_rect(n, view, i), canvas) {
            // ⭐ Uma row de COR regista-se com o id da AMOSTRA (que a shell atribuiu, com o nó
            // lá dentro): é ele que o selector de cor reconhece. As outras usam o id derivado
            // da posição, como sempre.
            let id = match crate::geom::band_at(n, i) {
                Some(crate::geom::BandRow::Param(k)) => n.params[k]
                    .swatch_id
                    .map_or_else(|| param_row_hit_id(n.id, row), NodeId),
                _ => param_row_hit_id(n.id, row),
            };
            hits.push((
                id,
                GraphHitKind::ParamRow {
                    node: n.id as u64,
                    row,
                },
                r,
            ));
        }
    }
}

/// O id de a11y de uma row de param — derivado, como todo id deste painel.
fn param_row_hit_id(node: u32, row: u16) -> NodeId {
    crate::paint::fnv_id(&format!("motion-graph/param-row/{node}/{row}"))
}

/// Push a backdrop's hit rects: its **header** (select + drag the group) and its
/// two bottom-corner **grippers** (resize — either corner, like every panel in the
/// app), all clipped to the canvas. The BODY is deliberately not registered — it
/// must stay click-through so the nodes it frames, and the empty canvas around
/// them, keep receiving clicks and box-selects (see `backdrop`'s module docs). The
/// grippers are pushed after the header so they win the corners they overlap it in
/// a short region; all three lose to the wires and cards registered after them.
pub(crate) fn push_backdrop_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    b: &crate::snapshot::GraphBackdropView,
    view: &View,
    canvas: Rect,
) {
    if let Some(r) = clip_rect(crate::backdrop::header_rect(b, view), canvas) {
        hits.push((
            backdrop_hit_id(b.id, "header"),
            GraphHitKind::Backdrop { id: b.id as u64 },
            r,
        ));
    }
    let (bl, br) = crate::backdrop::grip_rects(b, view);
    for (rect, left, part) in [(bl, true, "grip_bl"), (br, false, "grip_br")] {
        if let Some(r) = clip_rect(rect, canvas) {
            hits.push((
                backdrop_hit_id(b.id, part),
                GraphHitKind::BackdropResize {
                    id: crate::backdrop::resize_handle(b.id, left),
                },
                r,
            ));
        }
    }
}

/// The a11y/hit id of one part of a backdrop (`header` / `grip_bl` / `grip_br`).
fn backdrop_hit_id(id: u32, part: &str) -> NodeId {
    fnv_id(&format!("motion_graph/backdrop/{id}/{part}"))
}

/// Push a wire's hit rects (all sharing one id encoding the target input) —
/// several padded segment boxes along the flattened bezier, clipped to `canvas`.
pub(crate) fn push_wire_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    canvas: Rect,
) {
    let Some((p0, p3)) = crate::paint::wire_endpoints(snap, e, view) else {
        return;
    };
    let id = wire_hit_id(e.to_node, e.to_port);
    let kind = GraphHitKind::Wire {
        edge: wire_handle(e.to_node, e.to_port),
    };
    // A `pre` edge has no spline — its hit surface is its badge(s).
    if e.delayed {
        let r = (PRE_BADGE_R + PRE_BADGE_HIT_PAD) * view.zoom;
        let self_loop = e.from_node == e.to_node;
        for (cx, cy) in pre_badge_centers(p0, p3, view.zoom, self_loop)
            .into_iter()
            .flatten()
        {
            let rect = Rect::new(cx - r, cy - r, 2.0 * r, 2.0 * r);
            if let Some(rect) = clip_rect(rect, canvas) {
                hits.push((id, kind, rect));
            }
        }
        return;
    }
    let pts = wire_hit_polyline(p0, p3, view.zoom);
    for seg in pts.windows(2) {
        let (ax, ay) = seg[0];
        let (bx, by) = seg[1];
        let r = Rect::new(
            ax.min(bx) - WIRE_HIT_R,
            ay.min(by) - WIRE_HIT_R,
            (ax - bx).abs() + 2.0 * WIRE_HIT_R,
            (ay - by).abs() + 2.0 * WIRE_HIT_R,
        );
        if let Some(r) = clip_rect(r, canvas) {
            hits.push((id, kind, r));
        }
    }
}

/// Push the header preview-position toggle's hit rect (doc 86) — the small square that moves the
/// stamp above/below. **Registered AFTER the card body** (last wins, see [`register_hits`]) so a
/// click on the button flips the preview instead of selecting the card. Only a node with a stamp
/// has one ([`geom::preview_toggle_rect`] returns `None` otherwise), so a plain card never grows a
/// dead control.
pub(crate) fn push_preview_toggle_hit(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    view: &View,
    canvas: Rect,
) {
    let Some(rect) = geom::preview_toggle_rect(n, view) else {
        return;
    };
    if let Some(r) = clip_rect(rect, canvas) {
        hits.push((
            preview_toggle_hit_id(n.id),
            GraphHitKind::PreviewToggle { node: n.id as u64 },
            r,
        ));
    }
}

/// Push the ⚠ inert-warning badge's hit rect (ADR-0155) — the corner pip on a node
/// the diagnoser flagged inert. **Registered AFTER the card body** (last wins) so a
/// click on the badge asks for the fix instead of selecting the card. Only an inert
/// node has one ([`geom::inert_badge_rect`] returns `None` otherwise), so a healthy
/// card never grows a dead control.
pub(crate) fn push_inert_badge_hit(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    view: &View,
    canvas: Rect,
) {
    let Some(rect) = geom::inert_badge_rect(n, view) else {
        return;
    };
    if let Some(r) = clip_rect(rect, canvas) {
        hits.push((
            inert_badge_hit_id(n.id),
            GraphHitKind::InertBadge { node: n.id as u64 },
            r,
        ));
    }
}

/// Push a node's input + output socket hit rects (square, fixed screen size),
/// clipped to the canvas.
pub(crate) fn push_socket_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    view: &View,
    canvas: Rect,
) {
    for (i, _) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        if let Some(r) = clip_rect(socket_hit_rect(cx, cy), canvas) {
            hits.push((
                sock_in_id(n.id, i),
                GraphHitKind::SocketIn {
                    node: n.id as u64,
                    port: i as u16,
                },
                r,
            ));
        }
    }
    for (i, _) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        if let Some(r) = clip_rect(socket_hit_rect(cx, cy), canvas) {
            hits.push((
                sock_out_id(n.id, i),
                GraphHitKind::SocketOut {
                    node: n.id as u64,
                    port: i as u16,
                },
                r,
            ));
        }
    }
}

/// ⭐⭐⭐ **O BALÃO DO SOCKET SOB O RATO — e SÓ dele** (estudo do Mini Cavalry, doc 99 §10c).
///
/// ⛔⛔ **A 1.ª versão punha um balão em TODOS os sockets, a cada quadro** — e o
/// `paint_hover_tooltip` lê exactamente **um**, o do `hot_id`. Eram ~45 `String` construídas e
/// deitadas fora por quadro numa cena típica, contra **uma**; e a casa põe as dela **uma vez**
/// (`&'static str` no `pre_populate`), o que devia ter-me feito olhar. *A superfície que
/// consome um valor diz quantos deles vale a pena produzir.*
///
/// ⭐ **E fica mais CERTO, não só mais barato:** a resposta é derivada da **própria lista de
/// hits** — se um socket não tem hit, o `hot_id` nunca é o dele, logo não há como um balão
/// existir sem o socket. A concordância deixa de ser uma lei a manter e passa a ser
/// construção. *Uma lista, e agora nem sequer uma segunda condição.*
///
/// ⚠️ **Não limpa o balão anterior**: os ids de socket são determinísticos por `(nó, porta)`,
/// então o mapa cresce até ao tamanho do documento e não além. Um balão obsoleto nunca é lido
/// (sem hit não há `hot`), e limpá-lo custaria a travessia que esta função existe para evitar.
pub(crate) fn register_hot_tip(
    ctx: &mut PaintCtx,
    nodes: &[GraphNodeView],
    hits: &[(NodeId, GraphHitKind, Rect)],
) {
    let Some(hot) = ctx.host.store_mut().hot_id() else {
        return;
    };
    if let Some(text) = tip_for_hot(nodes, hits, hot) {
        ctx.host.store_mut().set_tooltip(hot, text);
    }
}

/// **A DECISÃO, sem o `PaintCtx`** — que é o que a torna gateável.
///
/// ⚠️ *Uma decisão enterrada num método que precisa de um contexto de host não tem gate
/// possível* — a mesma frase que o `motion_leaf_images` desta casa já escreveu sobre a
/// aritmética dele. O `register_hot_socket_tip` fica com três linhas de plumbing; a lei mora
/// aqui.
///
/// `None` quando o id quente não está na lista de hits (logo não é um socket desta corrida),
/// quando é outra superfície do canvas, ou quando a porta já não existe no snapshot.
#[must_use]
pub(crate) fn tip_for_hot(
    nodes: &[GraphNodeView],
    hits: &[(NodeId, GraphHitKind, Rect)],
    hot: NodeId,
) -> Option<String> {
    let (_, kind, _) = hits.iter().find(|(id, _, _)| *id == hot)?;
    let (node, port, entrada) = match kind {
        GraphHitKind::SocketIn { node, port } => (*node, *port as usize, true),
        GraphHitKind::SocketOut { node, port } => (*node, *port as usize, false),
        _ => return None, // outra superfície: o balão daquele id é de quem o registou
    };
    let n = nodes.iter().find(|n| u64::from(n.id) == node)?;
    let ports = if entrada { &n.inputs } else { &n.outputs };
    Some(crate::paint::socket_tip(ports.get(port)?))
}

fn socket_hit_rect(cx: f32, cy: f32) -> Rect {
    Rect::new(
        cx - geom::SOCKET_HIT_R,
        cy - geom::SOCKET_HIT_R,
        2.0 * geom::SOCKET_HIT_R,
        2.0 * geom::SOCKET_HIT_R,
    )
}

/// Register the background + wire + node + socket `GraphSurface` hit rects (in
/// the order given — last wins) so the M0 dispatch routes gestures here, and
/// republish the canvas rect for the wheel-zoom hit-test. Keyboard focus is
/// cursor-gated by the shell, so it is NOT set here.
pub(crate) fn register_hits(ctx: &mut PaintCtx, rect: Rect, hits: &[(NodeId, GraphHitKind, Rect)]) {
    let parent = ids::MOTION_GRAPH_PANEL;
    {
        let store = ctx.host.store_mut();
        for (id, kind, _) in hits {
            store.register(
                *id,
                InteractiveState::GraphSurface {
                    parent,
                    kind: *kind,
                    canvas: rect,
                },
            );
        }
        store.set_graph_canvas(parent, rect);
    }
    let hit_index = ctx.host.hit_index_mut();
    for (id, _, r) in hits {
        hit_index.register(*id, *r);
    }
}

pub(crate) fn bg_hit_id() -> NodeId {
    fnv_id("motion_graph/bg")
}
fn node_hit_id(node: u32) -> NodeId {
    fnv_id(&format!("motion_graph/node/{node}"))
}
fn preview_toggle_hit_id(node: u32) -> NodeId {
    fnv_id(&format!("motion_graph/preview_toggle/{node}"))
}
fn inert_badge_hit_id(node: u32) -> NodeId {
    fnv_id(&format!("motion_graph/inert_badge/{node}"))
}
fn sock_in_id(node: u32, port: usize) -> NodeId {
    fnv_id(&format!("motion_graph/sock_in/{node}/{port}"))
}
fn sock_out_id(node: u32, port: usize) -> NodeId {
    fnv_id(&format!("motion_graph/sock_out/{node}/{port}"))
}
/// The inline rename box (doc 61). One id — only one thing is being renamed at a time, and the
/// box is over it.
pub(crate) fn rename_id() -> NodeId {
    fnv_id("motion_graph/rename")
}

/// A caixa de escrever o número de um param (ciclo 1). **Um id**, pela mesma razão do
/// [`rename_id`]: só um número está a ser escrito de cada vez, e a caixa está por cima dele.
pub(crate) fn param_edit_id() -> NodeId {
    fnv_id("motion_graph/param_edit")
}

pub(crate) fn wire_hit_id(to_node: u32, to_port: u16) -> NodeId {
    fnv_id(&format!("motion_graph/wire/{to_node}/{to_port}"))
}

#[cfg(test)]
#[path = "hits_tests.rs"]
mod tests;

/// ⭐⭐ **AS AMOSTRAS DE COR DE UM CARTÃO SÃO SELECTORES** — a marca que faz um clique nelas
/// abrir o selector OKLCH, exactamente como na row do painel.
///
/// ⚠️ **São DUAS coisas e não uma:** o alvo (o `register` que [`register_hits`] faz, com o id
/// que a shell atribuiu) e a **marca** aqui. Sem a marca a amostra é pintada, é clicável, e não
/// abre nada — a forma do controlo morto que este painel já pagou no cabeçalho de secção.
pub(crate) fn register_card_swatches(ctx: &mut PaintCtx, snap: &GraphViewSnapshot) {
    let store = ctx.host.store_mut();
    for n in &snap.nodes {
        for p in &n.params {
            let (Some(id), Some(rgba)) = (p.swatch_id, p.swatch) else {
                continue;
            };
            // ⚠️ **A SEMENTE é a terceira coisa, e sem ela o selector abre CINZENTO.** O
            // `pointer_down` lê `widget_color(id)` para o abrir na cor que lá está, e o
            // omisso é `0x888888`: o artista clicaria num vermelho e veria o selector abrir
            // num cinzento — e o primeiro toque escreveria esse cinzento no nó.
            store.set_widget_color(NodeId(id), rgba);
            store.register_picker_swatch(NodeId(id));
        }
    }
}
