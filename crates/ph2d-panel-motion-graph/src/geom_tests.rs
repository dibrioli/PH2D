//! Unit tests for [`super`] (`geom.rs`) — extracted to a sibling (`#[path]`) so geom.rs stays under the panel LOC cap. Pure relocation of the `#[cfg(test)] mod tests` block.
use super::*;
use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView};
use crate::state::Menu;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

fn node_with_inputs(id: u32, x: f32, n_in: usize) -> GraphNodeView {
    GraphNodeView {
        kind: crate::snapshot::NodeViewKind::Node,
        id,
        display_name: "n".into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y: 0.0,
        inputs: (0..n_in)
            .map(|_| PortView {
                name: "i",
                domain: Domain::Instances,
                dim: Dim::Scalar,
                clock: Clock::Frame,
            })
            .collect(),
        outputs: vec![],
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: Vec::new(),
        sections: Vec::new(),
    }
}

fn one_node(node: GraphNodeView) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![node],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    }
}

fn node_with_outputs(id: u32, x: f32, n_out: usize) -> GraphNodeView {
    let mut n = node_with_inputs(id, x, 0);
    n.outputs = (0..n_out)
        .map(|_| PortView {
            name: "o",
            domain: Domain::Instances,
            dim: Dim::Scalar,
            clock: Clock::Frame,
        })
        .collect();
    n
}

#[test]
fn nearest_input_socket_resolves_the_right_port() {
    let snap = one_node(node_with_inputs(7, 100.0, 2));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    // Input 0 center: (100, HEADER_H + ROW_H*0.5) = (100, 37).
    let (n0, p0) = socket_center(&snap.nodes[0], &view, false, 0);
    assert_eq!(nearest_input_socket(&snap, &view, n0, p0), Some((7, 0)));
    // Input 1 center: (100, HEADER_H + ROW_H*1.5) = (100, 59).
    let (n1, p1) = socket_center(&snap.nodes[0], &view, false, 1);
    assert_eq!(nearest_input_socket(&snap, &view, n1, p1), Some((7, 1)));
    // Far from any socket → no snap.
    assert_eq!(nearest_input_socket(&snap, &view, 400.0, 400.0), None);
}

/// **A near-miss SNAPS to the socket** — a drop 15 px above input 0 (past the 9 px exact
/// hit box, inside the 22 px magnet) still lands on it. FALSIFIED by shrinking `SNAP_R` to
/// `SOCKET_HIT_R`: 15 > 9, so the near-miss resolves to nothing and the wire is dropped.
#[test]
fn a_near_miss_snaps_to_the_socket() {
    let snap = one_node(node_with_inputs(7, 100.0, 2));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    // Input 0 center (100, 37); 15 px above it — only input 0 is within reach.
    assert_eq!(
        nearest_input_socket(&snap, &view, 100.0, 22.0),
        Some((7, 0))
    );
}

/// Between two in-range sockets the magnet takes the **nearer** one — a drop at y=54 is
/// 17 px from input 0 and 5 px from input 1. FALSIFIED by returning the first socket found
/// rather than the closest: input 0 (found first, still in range) would win.
#[test]
fn snap_takes_the_nearest_of_two_sockets() {
    let snap = one_node(node_with_inputs(7, 100.0, 2));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    assert_eq!(
        nearest_input_socket(&snap, &view, 100.0, 54.0),
        Some((7, 1))
    );
}

/// The output magnet reads **output** sockets (right edge), not inputs — the mirror used by
/// the backward wire drag. FALSIFIED by resolving against the input side, which this node
/// does not have.
#[test]
fn the_output_magnet_reads_output_sockets() {
    let snap = one_node(node_with_outputs(9, 100.0, 1));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let (ox, oy) = socket_center(&snap.nodes[0], &view, true, 0);
    assert_eq!(
        nearest_output_socket(&snap, &view, ox + 12.0, oy),
        Some((9, 0))
    );
    assert_eq!(nearest_input_socket(&snap, &view, ox + 12.0, oy), None);
}

/// **A readout makes the card taller — and the HIT geometry grows with it.**
///
/// `card_h` is the one source of truth for both the paint and the hit-test, which is
/// why the readout's row is added here and not in `paint`. FALSIFIED by adding the row
/// only where the card is drawn: the card would gain a dead strip along its bottom edge
/// — a place that looks like the node and does not answer the mouse.
#[test]
fn a_readout_grows_the_card_and_its_hit_rect_together() {
    let bare = node_with_inputs(1, 0.0, 2);
    let mut with_readout = node_with_inputs(2, 0.0, 2);
    with_readout.readout = Some("12 inst".into());

    assert_eq!(
        card_h(&with_readout) - card_h(&bare),
        ROW_H,
        "one row taller"
    );

    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let (bare_r, read_r) = (card_rect(&bare, &view), card_rect(&with_readout, &view));
    assert_eq!(read_r.h - bare_r.h, ROW_H, "…and so is what the mouse hits");

    // A point in the readout's own row is INSIDE the card that has one, and outside the
    // card that does not. That is the strip that would have gone dead.
    let y = bare_r.y + bare_r.h + ROW_H * 0.5;
    let band = Rect::new(0.0, y - 1.0, 10.0, 2.0);
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![bare, with_readout],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    };
    assert_eq!(
        nodes_in_box(&snap, &view, band),
        vec![2],
        "only the taller card reaches down into that row"
    );
}

/// A **preview-CAPABLE** node: its primary output is a positional motion stream
/// (`Instances`/`Vec2`, the `P` column the stamp scatters). This is what owns a moldura by TYPE,
/// whatever it emits this frame (doc 86 B1) — the fixture MUST carry the output, not just content,
/// or it does not contain the phenomenon.
fn positional(id: u32, x: f32) -> GraphNodeView {
    let mut n = node_with_inputs(id, x, 2);
    n.outputs = vec![PortView {
        name: "P",
        domain: Domain::Instances,
        dim: Dim::Vec2,
        clock: Clock::Frame,
    }];
    n
}

fn stamped(id: u32, x: f32) -> GraphNodeView {
    let mut n = positional(id, x);
    n.preview = Some(vec![[0.0, 0.0], [1.0, 1.0]]);
    n
}

/// **The preview left the card** (doc 86): a node WITH a stamp is no taller than one
/// without — the in-card strip is gone. FALSIFIED by `card_h` still reserving the preview
/// band (the card would keep a dead strip where the stamp used to sit).
#[test]
fn the_card_no_longer_reserves_in_card_preview_height() {
    assert_eq!(
        card_h(&stamped(2, 0.0)),
        card_h(&node_with_inputs(1, 0.0, 2)),
        "the stamp no longer lives in the card"
    );
}

/// **The moldura sits OUTSIDE the card, above or below by position** (doc 86). Below → under
/// the card bottom; Above → over the header; neither overlaps the card rect, and `pos` moves
/// it. FALSIFIED by a frame drawn inside the body, or by ignoring `pos`.
#[test]
fn the_preview_frame_is_external_above_or_below() {
    let n = stamped(1, 100.0);
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let card = card_rect(&n, &view);
    let below = preview_frame_rect(&n, &view, PreviewPos::Below).unwrap();
    let above = preview_frame_rect(&n, &view, PreviewPos::Above).unwrap();
    assert!(below.y >= card.y + card.h, "below sits under the card");
    assert!(above.y + above.h <= card.y, "above sits over the card");
    assert!(
        preview_frame_rect(&node_with_inputs(2, 0.0, 2), &view, PreviewPos::Below).is_none(),
        "no positional output, no frame"
    );
}

/// **B1: the moldura survives an empty stream** (doc 86 §10). `motion.emitter` is stateless, so
/// its live count crosses zero and `preview` winks to `None` — but its output PORT stays `Vec2`,
/// so the frame AND its position toggle must stay put (the flicker was tying their existence to
/// the content). FALSIFIED by keying the frame off `n.preview` again: the empty tick returns
/// `None` and the whole moldura blinks Some↔None frame to frame.
#[test]
fn the_preview_moldura_survives_an_empty_stream() {
    let mut n = positional(1, 100.0);
    n.preview = None; // this tick the stateless emitter emitted nothing
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    assert!(
        preview_frame_rect(&n, &view, PreviewPos::Below).is_some(),
        "a preview-capable node keeps its frame even with no content this tick"
    );
    assert!(
        preview_toggle_rect(&n, &view).is_some(),
        "and keeps its position toggle"
    );
}

/// **The moldura does not move when the content winks** (doc 86 B1 — the measured invariant of the
/// fix). The SAME positional node, once empty and once full, yields the byte-identical frame rect:
/// the box neither vanishes nor shifts as the stateless emitter's count crosses zero. FALSIFIED by
/// any dependence of the frame's GEOMETRY on `preview`.
#[test]
fn the_preview_moldura_is_identical_whether_empty_or_full() {
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let mut empty = positional(1, 100.0);
    empty.preview = None;
    let mut full = positional(1, 100.0);
    full.preview = Some(vec![[0.0, 0.0], [1.0, 1.0]]);
    for pos in [PreviewPos::Below, PreviewPos::Above] {
        assert_eq!(
            preview_frame_rect(&empty, &view, pos),
            preview_frame_rect(&full, &view, pos),
            "the frame is the same box empty or full"
        );
    }
    assert_eq!(
        preview_toggle_rect(&empty, &view),
        preview_toggle_rect(&full, &view),
        "and so is its position toggle"
    );
}

/// **A value node grows no moldura** (doc 86). Its output is `Scalar` — its stamp is the number
/// it already shows, and an empty box under it would promise a picture that is not coming. The
/// slot is a fact of the TYPE, so even a value node carrying stray `preview` content gets none.
/// FALSIFIED by offering a frame on every card (`has_preview_slot` always true).
#[test]
fn a_value_node_has_no_preview_moldura() {
    // `node_with_outputs` gives `Scalar` outputs — a value node.
    let mut n = node_with_outputs(2, 0.0, 1);
    n.preview = Some(vec![[0.0, 0.0]]); // even with stray content, a value has no slot
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    assert!(preview_frame_rect(&n, &view, PreviewPos::Below).is_none());
    assert!(preview_toggle_rect(&n, &view).is_none());
}

/// **The header toggle exists only with a preview, and lives in the header** (doc 86). No
/// stamp → nothing to reposition → no button. FALSIFIED by a button offered on every card.
#[test]
fn the_toggle_lives_in_the_header_only_with_a_preview() {
    let n = stamped(1, 100.0);
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let btn = preview_toggle_rect(&n, &view).expect("a stamped node has the toggle");
    let (hx, hy) = view.pt(n.x, n.y);
    assert!(
        btn.y >= hy && btn.y + btn.h <= hy + HEADER_H,
        "inside the header band"
    );
    assert!(
        btn.x + btn.w <= hx + CARD_W && btn.x >= hx,
        "at the header's right, inside the card"
    );
    assert!(
        preview_toggle_rect(&node_with_inputs(2, 0.0, 2), &view).is_none(),
        "no positional output, no toggle"
    );
}

#[test]
fn menu_panel_clamps_into_the_canvas() {
    let canvas = Rect::new(0.0, 0.0, 300.0, 200.0);
    // Opened past the right/bottom edge → clamped fully inside.
    let menu = Menu {
        scroll: 0.0,
        screen: (290.0, 190.0),
        spawn: (0.0, 0.0),
        // Any body — clamping is geometry, not content. (The node library moved to the shell
        // palette; the local popups left are read by eye.)
        body: crate::state::MenuBody::NodeActions {
            multi: false,
            group: false,
        },
    };
    let p = menu_panel(&menu, 3, canvas);
    assert!(p.x + p.w <= canvas.x + canvas.w + 0.01);
    assert!(p.y + p.h <= canvas.y + canvas.h + 0.01);
    assert!(p.x >= canvas.x && p.y >= canvas.y);
}

// ─────────────────────────────────────────────────────────────────────────────
// CICLO 1 — **os params vivem no cartão** (doc 103; decisão do Enio, 2026-09-05).
// ─────────────────────────────────────────────────────────────────────────────

fn hint(label: &'static str) -> ph2d_node_registry::ParamUiHint {
    ph2d_node_registry::ParamUiHint {
        param: "p",
        label,
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ph2d_node_registry::ParamWidget::Slider,
    }
}

fn with_params(mut n: GraphNodeView, k: usize) -> GraphNodeView {
    n.params = (0..k)
        .map(|_| crate::snapshot::CardParam {
            hint: hint("Rows"),
            value: 3.0,
            driven: false,
            swatch: None,
        })
        .collect();
    n
}

/// **O cartão RESERVA a faixa dos seus params** — uma fileira por param, entre os sockets e o
/// readout. FALSIFICADO por `card_param_rows` devolver `0`: o cartão volta à altura de antes e
/// as rows passam a ser desenhadas por cima do readout e fora do corpo.
#[test]
fn the_card_reserves_a_band_for_its_params() {
    let nu = node_with_inputs(1, 0.0, 2);
    let base = card_h(&nu);
    for k in [1usize, 3, 8, 24] {
        assert_eq!(
            card_h(&with_params(node_with_inputs(1, 0.0, 2), k)),
            base + k as f32 * ROW_H,
            "um cartao com {k} params e' {k} fileiras mais alto"
        );
    }
}

/// **A ALTURA NÃO DEPENDE DO ZOOM.** [`card_h`] é espaço de GRAFO: um cartão que encolhesse ao
/// afastar faria os hit-rects saltarem debaixo do dedo a meio de um pinch — e o LOD só decide
/// se o CONTEÚDO da row é desenhado. FALSIFICADO por `card_h` passar a consultar o zoom.
#[test]
fn the_card_height_does_not_follow_the_zoom() {
    let n = with_params(node_with_inputs(1, 0.0, 2), 5);
    // `card_h` não recebe a vista, e é essa a prova: se um dia receber, este teste não compila,
    // que é o aviso certo (a lei muda de forma, não de valor).
    let h = card_h(&n);
    assert!(h > 0.0, "a altura existe e e' do grafo, nao do ecra");
}

/// **A ROW DE PARAM CAI DENTRO DO CARTÃO** — o rect que o pintor usa é o mesmo que um hit-test
/// usará, e ele tem de estar dentro do corpo. FALSIFICADO por trocar o `param_band_top` pela
/// soma antiga (a faixa passaria a começar em cima do readout).
#[test]
fn every_param_row_falls_inside_its_card() {
    let n = with_params(node_with_inputs(7, 40.0, 2), 6);
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let (cx, cy) = view.pt(n.x, n.y);
    let ch = card_h(&n) * view.zoom;
    for i in 0..n.params.len() {
        let r = param_row_rect(&n, &view, i);
        assert!(r.y >= cy, "a row {i} comeca abaixo do topo do cartao");
        assert!(
            r.y + r.h <= cy + ch + f32::EPSILON,
            "a row {i} acaba dentro do cartao (fundo {} contra {})",
            r.y + r.h,
            cy + ch
        );
        assert!(r.x >= cx && r.w <= CARD_W * view.zoom + f32::EPSILON);
    }
}

/// **AS ROWS NÃO SE SOBREPÕEM** — cada uma ocupa exactamente uma fileira. FALSIFICADO por o
/// passo do `param_row_rect` deixar de ser `ROW_H`.
#[test]
fn param_rows_stack_without_overlapping() {
    let n = with_params(node_with_inputs(7, 0.0, 1), 4);
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    for i in 1..n.params.len() {
        let a = param_row_rect(&n, &view, i - 1);
        let b = param_row_rect(&n, &view, i);
        assert!(
            (b.y - (a.y + a.h)).abs() < 1e-3,
            "a row {i} comeca onde a anterior acaba"
        );
    }
}

/// **O READOUT FICA ABAIXO DOS PARAMS** — senão o número que o nó produziu é escrito por cima
/// dos botões dele. FALSIFICADO por `readout_top` voltar a somar só os sockets.
#[test]
fn the_readout_sits_below_the_param_band() {
    let n = with_params(node_with_inputs(1, 0.0, 2), 3);
    assert_eq!(
        readout_top(&n),
        param_band_top(&n) + 3.0 * ROW_H,
        "o readout comeca depois da ultima row de param"
    );
    assert!(readout_top(&n) > param_band_top(&n));
}

/// ⭐⭐ **O LOD É DO TEXTO, e a BARRA pinta-se sempre.**
///
/// ⛔ A primeira versão escondia a row inteira abaixo do limiar e o smoke do Enio devolveu
/// *«tudo em branco»*: a cena abre a `zoom ≈ 0,5`, a faixa ficava reservada (a altura não segue
/// o zoom) e nada era desenhado nela. O texto some (`11 px × zoom ≥ 9 px` ⇒ `zoom ≥ 0,818`);
/// a barra e o nível, não.
///
/// FALSIFICADO por `param_text_is_drawn` devolver sempre `true` (o rótulo vira uma mancha
/// cinzenta e paga-se o texto num grafo afastado) ou sempre `false` (nenhum número se lê).
#[test]
fn only_the_text_of_a_param_row_follows_the_zoom() {
    let at = |z: f32| {
        let vs = ViewState {
            zoom: z,
            ..ViewState::default()
        };
        param_text_is_drawn(&View::new(Rect::new(0.0, 0.0, 800.0, 600.0), vs))
    };
    assert!(!at(0.25), "afastado, o rotulo e' uma mancha — nao se escreve");
    assert!(!at(0.5), "o zoom com que a cena de smoke abre");
    assert!(!at(0.8), "logo abaixo do limiar (0,818) ainda nao");
    assert!(at(0.83), "logo acima, sim");
    assert!(at(1.0));
    assert!(at(2.5));
}

/// ⭐⭐ **A FAIXA INTERCALA CABEÇALHOS E ROWS, e é UMA coordenada** — o pintor, o hit-test e o
/// gesto falam todos em índice de faixa. FALSIFICADO por `band_at` contar sem os cabeçalhos:
/// um clique cai uma fileira ao lado no primeiro nó com secções.
#[test]
fn the_band_interleaves_section_headers_with_the_rows() {
    let mut n = with_params(node_with_inputs(1, 0.0, 1), 5);
    n.sections = vec![
        crate::snapshot::CardSection {
            title: "Shape",
            at: 0,
            open: true,
            hidden: 0,
        },
        crate::snapshot::CardSection {
            title: "Advanced",
            at: 3,
            open: true,
            hidden: 0,
        },
    ];
    assert_eq!(band_len(&n), 7, "5 rows + 2 cabecalhos");
    use crate::geom::BandRow::{Header, Param};
    let esperado = [
        Header(0),
        Param(0),
        Param(1),
        Param(2),
        Header(1),
        Param(3),
        Param(4),
    ];
    for (i, e) in esperado.iter().enumerate() {
        assert_eq!(band_at(&n, i), Some(*e), "fileira {i}");
    }
    assert_eq!(band_at(&n, 7), None, "depois do fim nao ha' fileira");
    // E a altura conta as sete.
    assert_eq!(
        card_h(&n),
        card_h(&node_with_inputs(1, 0.0, 1)) + 7.0 * ROW_H
    );
}
