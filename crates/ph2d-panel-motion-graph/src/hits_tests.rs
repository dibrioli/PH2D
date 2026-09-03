//! Os gates do que é **agarrável** no canvas do grafo — cortados do `hits.rs` no teto de LOC
//! do painel (600).
//!
//! ⚠️ **O corte é o mais barato que havia e é por RESPONSABILIDADE na mesma:** o pai responde
//! *«que rectângulos existem, e em que ordem?»* e este *«e ele acerta?»*. O pai cresce quando o
//! canvas ganha uma superfície; este quando alguém acha uma forma nova de ela falhar.

use super::*;
use crate::snapshot::PortView;
use crate::state::ViewState;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const CANVAS: Rect = Rect {
    x: 0.0,
    y: 100.0,
    w: 400.0,
    h: 200.0,
};

fn node(id: u32, x: f32, y: f32) -> GraphNodeView {
    GraphNodeView {
        kind: crate::snapshot::NodeViewKind::Node,
        id,
        display_name: "n".into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y,
        inputs: vec![PortView {
            name: "i",
            domain: Domain::Instances,
            dim: Dim::Vec2,
            clock: Clock::Frame,
        }],
        outputs: vec![],
        readout: None,
        drops: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
    }
}

#[test]
fn clip_rect_intersects_and_rejects_disjoint() {
    // Fully inside → unchanged.
    let inside = Rect::new(10.0, 120.0, 50.0, 50.0);
    assert_eq!(clip_rect(inside, CANVAS), Some(inside));
    // Straddling the top edge → cropped to the canvas.
    let straddle = clip_rect(Rect::new(10.0, 80.0, 50.0, 50.0), CANVAS).unwrap();
    assert_eq!((straddle.y, straddle.h), (100.0, 30.0));
    // Fully above the canvas → gone.
    assert_eq!(clip_rect(Rect::new(10.0, 0.0, 50.0, 50.0), CANVAS), None);
    // Touching the edge exactly (zero area) → gone, not a degenerate hit.
    assert_eq!(clip_rect(Rect::new(10.0, 50.0, 50.0, 50.0), CANVAS), None);
}

#[test]
fn a_card_panned_off_canvas_registers_no_hit() {
    // The defect this guards: content scrolled/zoomed past the panel edge is
    // clipped away visually, but an unclipped hit rect would still swallow
    // clicks meant for the scene behind it.
    let mut hits = Vec::new();
    push_card_hit(
        &mut hits,
        &node(7, 0.0, 0.0),
        Rect::new(10.0, -60.0, 190.0, 58.0),
        CANVAS,
    );
    assert!(hits.is_empty(), "an off-canvas card is not clickable");

    // A card straddling the edge stays clickable, but only over its visible part.
    push_card_hit(
        &mut hits,
        &node(8, 0.0, 0.0),
        Rect::new(10.0, 70.0, 190.0, 58.0),
        CANVAS,
    );
    let (_, _, r) = hits[0];
    assert_eq!((r.y, r.h), (100.0, 28.0), "only the visible band is hit");
}

/// **A ghost is grabbable** (doc 57, Enio smoke 2026-07-13: *"o primeiro e último
/// nó do grupo não pode ser movido, mas deveria poder"*). It is a real node with
/// exactly ONE position, and the artist looking at the ghost IS looking at the
/// node — so it can be dragged, to tidy the room it flanks. Blender and Houdini
/// both move their boundary proxies from inside the group, for the same reason:
/// you cannot tidy a room whose doorways are nailed down.
///
/// The one line it does not cross is DELETE — refused shell-side with a toast,
/// because that would reach into a canvas the artist is not on.
#[test]
fn a_ghost_can_be_grabbed_and_wired_like_the_node_it_is() {
    let ghost = GraphNodeView {
        kind: crate::snapshot::NodeViewKind::Ghost,
        ..node(9, 10.0, 10.0)
    };
    let view = View::new(CANVAS, ViewState::default());

    let mut body = Vec::new();
    push_card_hit(
        &mut body,
        &ghost,
        Rect::new(10.0, 120.0, 190.0, 58.0),
        CANVAS,
    );
    assert_eq!(body.len(), 1, "you can grab it and move it out of the way");

    let mut sockets = Vec::new();
    push_socket_hits(&mut sockets, &ghost, &view, CANVAS);
    assert_eq!(
        sockets.len(),
        1,
        "and its sockets are live: you can wire it"
    );
}

/// A `pre` edge's hit surface is its badge pair, not a spline band: two
/// compact rects for a normal delayed edge, ONE for the self-loop template,
/// and the sampled-polyline rects only for plain forward wires. This is
/// what keeps the portal badges clickable exactly where they draw
/// (docs/Motion Nodes/03).
#[test]
fn a_pre_edge_hits_as_badges_not_as_a_spline() {
    let mk_edge = |from: u32, to: u32, delayed: bool| GraphEdgeView {
        from_node: from,
        from_port: 0,
        to_node: to,
        to_port: 0,
        delayed,
        out_domain: Domain::Instances,
    };
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![node(1, 20.0, 50.0), node(2, 240.0, 50.0)],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    };
    let view = View::new(CANVAS, ViewState::default());

    let mut plain = Vec::new();
    push_wire_hits(&mut plain, &snap, &mk_edge(1, 2, false), &view, CANVAS);
    let mut badges = Vec::new();
    push_wire_hits(&mut badges, &snap, &mk_edge(1, 2, true), &view, CANVAS);
    let mut self_loop = Vec::new();
    push_wire_hits(&mut self_loop, &snap, &mk_edge(2, 2, true), &view, CANVAS);

    assert!(
        plain.len() > badges.len(),
        "a forward wire hits along its whole band ({}), a pre edge only at its badges ({})",
        plain.len(),
        badges.len()
    );
    assert_eq!(badges.len(), 2, "one badge per end of the pre edge");
    assert_eq!(self_loop.len(), 1, "the self-loop collapses to one badge");
    // Both badge rects are compact squares, not wire-band slabs.
    for (_, _, r) in &badges {
        assert!(r.w <= 2.0 * (PRE_BADGE_R + 4.0) && r.h <= 2.0 * (PRE_BADGE_R + 4.0));
    }
}

/// **The preview toggle is a hit only over a preview-capable card** (doc 86). A node whose
/// primary output is positional (`Instances`/`Vec2`) registers one `PreviewToggle` rect (so a
/// click reaches the dispatch), whatever it emits this frame (doc 86 B1); a value card
/// registers none (no dead control). FALSIFIED by offering the hit unconditionally.
#[test]
fn the_preview_toggle_registers_a_hit_only_over_a_stamped_card() {
    let view = View::new(CANVAS, ViewState::default());
    let mut stamped = node(1, 10.0, 10.0);
    // Preview-capability is the OUTPUT TYPE, not the content — this is what a click needs.
    stamped.outputs = vec![PortView {
        name: "P",
        domain: Domain::Instances,
        dim: Dim::Vec2,
        clock: Clock::Frame,
    }];
    stamped.preview = None; // and it stays hittable on an empty tick (the B1 flicker)
    let mut hits = Vec::new();
    push_preview_toggle_hit(&mut hits, &stamped, &view, CANVAS);
    assert_eq!(hits.len(), 1, "a preview-capable node exposes the toggle");
    assert!(
        matches!(hits[0].1, GraphHitKind::PreviewToggle { node } if node == 1),
        "and it carries this node's id"
    );

    let mut none = Vec::new();
    push_preview_toggle_hit(&mut none, &node(2, 10.0, 10.0), &view, CANVAS);
    assert!(
        none.is_empty(),
        "a card with no positional output has no toggle to click"
    );
}

/// **The ⚠ inert badge is a hit only over an inert node** (ADR-0155): an inert card
/// registers one `InertBadge` rect (so the click reaches the dispatch); a healthy card
/// registers none (no dead control — the [`push_preview_toggle_hit`] discipline).
/// FALSIFIED by offering the hit unconditionally.
#[test]
fn the_inert_badge_is_a_hit_only_over_an_inert_node() {
    let view = View::new(CANVAS, ViewState::default());
    let mut inert = node(1, 10.0, 10.0);
    inert.inert = true;
    let mut hits = Vec::new();
    push_inert_badge_hit(&mut hits, &inert, &view, CANVAS);
    assert_eq!(hits.len(), 1, "an inert node exposes the badge");
    assert!(
        matches!(hits[0].1, GraphHitKind::InertBadge { node } if node == 1),
        "and it carries this node's id"
    );

    let mut none = Vec::new();
    push_inert_badge_hit(&mut none, &node(2, 10.0, 10.0), &view, CANVAS);
    assert!(none.is_empty(), "a healthy node has no badge to click");
}

#[test]
fn sockets_off_canvas_register_no_hit() {
    // Pan the view far above the canvas: the node's socket is off-screen.
    let view = View::new(
        CANVAS,
        ViewState {
            pan_x: 0.0,
            pan_y: -500.0,
            zoom: 1.0,
        },
    );
    let mut hits = Vec::new();
    push_socket_hits(&mut hits, &node(1, 0.0, 0.0), &view, CANVAS);
    assert!(hits.is_empty(), "off-canvas sockets are not grabbable");

    // Panned back in, the socket registers.
    let view = View::new(CANVAS, ViewState::default());
    push_socket_hits(&mut hits, &node(1, 10.0, 10.0), &view, CANVAS);
    assert_eq!(hits.len(), 1);
}

/// ⭐⭐⭐ **O BALÃO SAI DA LISTA DE HITS, e é isso que o torna impossível de discordar dela**
/// (estudo do Mini Cavalry, doc 99 §10c).
///
/// ⛔⛔ **A 1.ª versão construía um balão por socket por quadro** e mantinha a concordância por
/// LEI (o mesmo laço, a mesma condição). Hoje ela é por CONSTRUÇÃO: o texto é procurado na
/// própria lista de hits, então um socket sem hit não tem por onde produzir um.
///
/// As três metades mordem: o socket com hit responde · um id que não está na lista não · e uma
/// superfície do canvas que não é socket também não (o balão daquele id pertence a quem o
/// registou, e sobrescrevê-lo apagaria o dele).
#[test]
fn the_hot_tip_answers_for_a_socket_and_for_a_card() {
    let view = View::new(CANVAS, ViewState::default());
    let n = node(1, 10.0, 10.0);
    let mut hits = Vec::new();
    push_socket_hits(&mut hits, &n, &view, CANVAS);
    assert_eq!(hits.len(), 1, "a fixtura tem de conter o fenomeno");
    let nodes = [n];

    let tip = tip_for_hot(&nodes, &hits, hits[0].0).expect("o socket com hit responde");
    assert!(
        tip.contains(" · "),
        "o balao diz o NOME e a ESPECIE, separados: {tip:?}"
    );

    assert!(
        tip_for_hot(&nodes, &hits, bg_hit_id()).is_none(),
        "um id que nao esta' na lista de hits nao produz balao nenhum"
    );

    // ⭐⭐⭐ **O CARTÃO diz o que o nó DEITA FORA** (doc 99 §10d) — a pergunta que custou os
    // três reports de 2026-09-01 e que nenhuma superfície do app respondia.
    let mut com_cartao = hits.clone();
    let card = node_hit_id(1);
    com_cartao.push((card, GraphHitKind::Node { node: 1 }, CANVAS));
    let mut n_perde = node(1, 10.0, 10.0);
    n_perde.drops = Some("id · vel".to_string());
    assert_eq!(
        tip_for_hot(&[n_perde], &com_cartao, card).as_deref(),
        Some("drops id · vel"),
        "o cartao de um no' que perde colunas DIZ quais"
    );
    // ⚠️ **E o SILÊNCIO é a resposta honesta para quem não perde nada** — um balão a dizer
    // «drops nothing» em 134 cartões seria o mesmo ruído que a lei do rótulo já recusou.
    assert!(
        tip_for_hot(&[node(1, 10.0, 10.0)], &com_cartao, card).is_none(),
        "um no' que nao perde nada nao diz nada"
    );

    // O fundo ESTÁ na lista, e mesmo assim não é um socket.
    let mut com_fundo = hits.clone();
    com_fundo.push((bg_hit_id(), GraphHitKind::Background, CANVAS));
    assert!(
        tip_for_hot(&nodes, &com_fundo, bg_hit_id()).is_none(),
        "uma superficie que nao e' socket nao pode roubar o balao dela"
    );
}
