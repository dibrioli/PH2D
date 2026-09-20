//! Guards for the wire's FLATTENING (doc 46.1). `super` is `paint::paint_wire`.

use super::*;
use crate::geom::ZOOM_DA_CAPSULA;

/// **A backward wire-draw paints its ghost too.** `draw_wire_ghost` has always handled
/// `DrawWireBack`, but the paint entry decides WHEN to call it, and the predicate it consults must
/// say yes for a backward drag — otherwise the ghost is dead code and the wire stays invisible
/// until it connects (Enio's report). FALSIFIED by dropping the `DrawWireBack` arm of
/// `draws_wire_ghost`: the backward drag then paints nothing, while the forward CONTROL still does.
#[test]
fn a_backward_wire_draw_shows_its_ghost() {
    // The bug: a backward drag out of an empty input must show its ghost.
    assert!(draws_wire_ghost(&Interaction::DrawWireBack {
        to_node: 1,
        to_port: 0,
        cur: (0.0, 0.0),
        target: None,
    }));
    // The control: the forward draw has always shown its ghost.
    assert!(draws_wire_ghost(&Interaction::DrawWire {
        from_node: 1,
        from_port: 0,
        cur: (0.0, 0.0),
        target: None,
        detached: None,
    }));
    // Not "always true": an idle graph paints no ghost.
    assert!(!draws_wire_ghost(&Interaction::Idle));
}

/// The true curve, densely sampled — the oracle the flattening is measured against.
fn true_curve(p0: (f32, f32), p3: (f32, f32), zoom: f32) -> Vec<(f32, f32)> {
    let (c1, c2) = wire_handles(p0, p3, zoom);
    cubic_polyline(p0, c1, c2, p3, 2000)
}

fn point_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let (wx, wy) = (p.0 - a.0, p.1 - a.1);
    let len2 = vx * vx + vy * vy;
    let t = if len2 > 0.0 {
        ((wx * vx + wy * vy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (p.0 - (a.0 + vx * t), p.1 - (a.1 + vy * t));
    (dx * dx + dy * dy).sqrt()
}

/// The worst distance from the true curve to `poly`.
fn max_deviation(poly: &[(f32, f32)], curve: &[(f32, f32)]) -> f32 {
    curve
        .iter()
        .map(|p| {
            poly.windows(2)
                .map(|s| point_to_segment(*p, s[0], s[1]))
                .fold(f32::INFINITY, f32::min)
        })
        .fold(0.0, f32::max)
}

/// **The drawn wire is deep sub-pixel from the curve it claims to be** — at any length, at any
/// zoom.
///
/// And the SECOND half of this guard is the one that matters, because measuring only the chord
/// error would have declared the bug fixed while it was still on screen: the old fixed-20
/// flattening strayed a mere ~0.5 px, and still looked like a chain of sticks. What the eye
/// reads is the vertex DENSITY (the marching dashes are short straights cut from this polyline,
/// so each facet becomes a visibly straight tick). So the guard pins the density too.
#[test]
fn a_wire_is_flattened_to_a_sub_pixel_tolerance_at_any_size() {
    // A long, steep, zoomed-in wire: the worst case, and the one in Enio's screenshot.
    for (p0, p3, zoom) in [
        ((0.0, 0.0), (60.0, 20.0), 1.0),     // a short hop
        ((0.0, 0.0), (420.0, 260.0), 1.0),   // a long diagonal
        ((0.0, 0.0), (420.0, 260.0), 2.5),   // …at full zoom
        ((300.0, 40.0), (60.0, 300.0), 1.8), // backwards (target left of source)
    ] {
        let poly = wire_polyline(p0, p3, zoom);
        let dev = max_deviation(&poly, &true_curve(p0, p3, zoom));
        assert!(
            dev < 0.3,
            "flattened wire strays {dev:.3} px from the curve ({p0:?} -> {p3:?} @ {zoom})"
        );
    }

    // The wire from Enio's screenshot: it must come out substantially DENSER than the 20
    // segments it used to get, or the dashes go on reading as straight sticks.
    let (p0, p3, zoom) = ((0.0, 0.0), (420.0, 260.0), 2.5);
    let n = wire_polyline(p0, p3, zoom).len() - 1;
    assert!(
        n >= 40,
        "a long zoomed wire gets {n} segments - it used to get 20, and it showed"
    );
    // …and the error it buys is an order of magnitude under the old one (~0.5 px, measured).
    let (c1, c2) = wire_handles(p0, p3, zoom);
    let old = max_deviation(
        &cubic_polyline(p0, c1, c2, p3, 20),
        &true_curve(p0, p3, zoom),
    );
    let new = max_deviation(&wire_polyline(p0, p3, zoom), &true_curve(p0, p3, zoom));
    assert!(new * 4.0 < old, "old {old:.3} px -> new {new:.3} px");
}

/// The count comes from the CURVE: a bigger, more curved, more zoomed wire gets more segments,
/// and a stub gets few. (Zoom refines for free — the flattening happens in screen space.)
#[test]
fn the_segment_count_follows_the_curve_not_a_constant() {
    let stub = wire_polyline((0.0, 0.0), (40.0, 0.0), 1.0).len();
    let long = wire_polyline((0.0, 0.0), (420.0, 260.0), 1.0).len();
    let zoomed = wire_polyline((0.0, 0.0), (420.0, 260.0), 2.5).len();
    assert!(stub < long, "a stub needs fewer segments: {stub} vs {long}");
    assert!(
        long < zoomed,
        "zoomed in, the same wire is refined: {long} vs {zoomed}"
    );
    assert!(zoomed <= FLATTEN_MAX + 1, "and the budget is bounded");
}

/// **The coarse HIT boxes still cover the drawn wire.** The hit path samples deliberately
/// coarsely (fat boxes, few of them), which is only legitimate while the chord error stays well
/// inside the box padding — otherwise there would be a strip of the visible wire that is not
/// clickable, and the knife (which rides the DRAWN polyline) would cut wires the hit-test
/// cannot even hover.
#[test]
fn the_coarse_hit_boxes_still_cover_the_drawn_wire() {
    for (p0, p3, zoom) in [
        ((0.0, 0.0), (420.0, 260.0), 1.0),
        ((0.0, 0.0), (420.0, 260.0), 2.5),
        ((300.0, 40.0), (60.0, 300.0), 1.8),
    ] {
        let dev = max_deviation(&wire_hit_polyline(p0, p3, zoom), &true_curve(p0, p3, zoom));
        assert!(
            dev < crate::hits::WIRE_HIT_R,
            "the hit chords stray {dev:.2} px - more than the {} px box padding",
            crate::hits::WIRE_HIT_R
        );
    }
}

/// **A selected wire wears the SELECTION hue, distinct from a hovered one.** A selected wire draws
/// in `Accent` — the same colour a selected node's ring wears (the app's one selection hue) — so it
/// reads as *committed*, not merely *under the cursor*; a hovered-but-unselected wire keeps the
/// bright `Text1` emphasis, and an idle wire keeps its own port-domain hue. Selection wins over
/// hover, exactly as a node's Accent ring does — this is what makes several selected wires read as a
/// SELECTION rather than a row of hovers. FALSIFIED three ways: `Selected` reusing the hover `Text1`
/// (the v1 that made selection and hover indistinguishable once many wires could be lit at once),
/// the `Hover` branch folding away so a hovered wire loses its bright emphasis, or `of` letting hover
/// win over selection.
#[test]
fn a_selected_wire_is_accent_distinct_from_the_hover_hue() {
    let dom = ColorToken::PortInstances; // any port-domain hue
    // Selected → Accent (the selection hue).
    assert_eq!(WireEmphasis::Selected.token(dom), ColorToken::Accent);
    // Hovered but not selected → the bright hover emphasis, the delete-target read.
    assert_eq!(WireEmphasis::Hover.token(dom), ColorToken::Text1);
    // Idle → the wire keeps its own port-domain hue.
    assert_eq!(WireEmphasis::Idle.token(dom), dom);
    // Selection WINS over hover, exactly as a node's Accent ring does.
    assert_eq!(WireEmphasis::of(true, true), WireEmphasis::Selected);
    assert_eq!(WireEmphasis::of(true, false), WireEmphasis::Hover);
    assert_eq!(WireEmphasis::of(false, false), WireEmphasis::Idle);
    assert!(WireEmphasis::Selected.lit() && WireEmphasis::Hover.lit() && !WireEmphasis::Idle.lit());
}

/// **An incompatible drop target reads as DANGER RED, not muted grey** — the ghost wire and
/// the target-socket ring both go red, the affirmative "no" the artist sees before releasing.
/// This is the `connects_directly` verdict (resolved onto the target by the interaction) made
/// visible; a compatible or empty target keeps the domain preview and an Accent ring.
///
/// Reverting the change — Danger back to the old muted `Border`, or the ring back to Accent
/// regardless — sangra here, and `None` (empty space) must stay the neutral preview so hovering
/// nowhere never flashes red.
#[test]
fn an_incompatible_drop_target_is_danger_red() {
    let dom = ColorToken::PortInstances; // any domain hue: the "this is what I'll carry" preview

    // The ghost WIRE: incompatible ⇒ Danger; compatible or empty ⇒ the on-target preview.
    assert_eq!(ghost_wire_token(Some(false), dom), ColorToken::Danger);
    assert_eq!(ghost_wire_token(Some(true), dom), dom);
    assert_eq!(ghost_wire_token(None, dom), dom);
    // A backward drag with nothing found yet has no hue — it stays muted Border, never red.
    assert_eq!(
        ghost_wire_token(None, ColorToken::Border),
        ColorToken::Border
    );

    // The target-socket RING: compatible ⇒ Accent (drop here), incompatible ⇒ Danger (not here).
    assert_eq!(target_ring_token(true), ColorToken::Accent);
    assert_eq!(target_ring_token(false), ColorToken::Danger);

    // The verdict comes straight off the interaction's resolved target tuple.
    assert_eq!(target_compat(&Some((3, 0, false))), Some(false));
    assert_eq!(target_compat(&Some((3, 0, true))), Some(true));
    assert_eq!(target_compat(&None), None);
}

/// **The ghost wire's loose end SNAPS to the target socket** — once the drag locks onto a socket
/// (the same one the magnet in `geom` will drop onto), the wire draws to that socket's CENTRE,
/// not the cursor, so the artist SEES the lock. FALSIFIED three ways: the target is ignored (the
/// end never leaves the cursor), the wrong EDGE is used (an output target drawn on the left), or
/// empty space resolves to a point instead of `None` (the wire would jump to a phantom socket).
#[test]
fn the_ghost_end_snaps_to_the_target_socket() {
    use crate::snapshot::{GraphNodeView, NodeViewKind, PortView};
    use crate::state::ViewState;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};

    let port = || PortView {
        name: "p",
        domain: Domain::Instances,
        dim: Dim::Scalar,
        clock: Clock::Frame,
    };
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![GraphNodeView {
            primary_input: 0,
            kind: NodeViewKind::Node,
            id: 5,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x: 100.0,
            y: 0.0,
            inputs: vec![port()],
            outputs: vec![port()],
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
        }],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    };
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let node = &snap.nodes[0];

    // Locked onto the node's OUTPUT socket (right edge) — the ghost end is that socket's centre.
    assert_eq!(
        ghost_target_center(&Some((5, 0, true)), &snap, &view, true),
        Some(socket_center(node, &view, true, 0)),
    );
    // Locked onto its INPUT (left edge): proves the `output` flag picks the edge, not a constant.
    assert_eq!(
        ghost_target_center(&Some((5, 0, false)), &snap, &view, false),
        Some(socket_center(node, &view, false, 0)),
    );
    // No target, or a node that is not here → None, so the end falls back to the cursor.
    assert_eq!(ghost_target_center(&None, &snap, &view, true), None);
    assert_eq!(
        ghost_target_center(&Some((99, 0, true)), &snap, &view, true),
        None,
    );
}

// ──────────────────────────────────────────────────────────────────────────────────────────────
// A LARGURA DO FIO — ordem do dono (2026-09-20): *«linhas mais grossas»*.
// ──────────────────────────────────────────────────────────────────────────────────────────────

fn vista_do_fio(zoom: f32) -> View {
    View::new(
        Rect::new(0.0, 0.0, 800.0, 600.0),
        crate::state::ViewState {
            zoom,
            ..crate::state::ViewState::default()
        },
    )
}

/// ⭐⭐⭐ **NA CÁPSULA, UM FIO PÁRA DE ENCOLHER** — a ordem do dono, medida no número.
///
/// ⚠️ **A régua é a largura em PÍXEIS DE ECRÃ e não uma razão:** o que o artista se queixou de
/// não ver é uma quantidade de pixels, e `0,54` deles é menos de meio traço.
///
/// FALSIFICADO por devolver `base_w * view.zoom` no braço da cápsula: o fio fino volta a `0,54`
/// px a zoom `0,3`.
#[test]
fn na_capsula_um_fio_para_de_encolher() {
    const FINO: f32 = 1.8; // o `W_THREAD` do `flow`, escrito aqui de propósito: se ele mudar,
    // este gate tem de ser re-lido e não silenciosamente re-escalado.
    for zoom in [0.2_f32, 0.3, 0.5, ZOOM_DA_CAPSULA - 0.01] {
        let v = vista_do_fio(zoom);
        assert_eq!(
            crate::geom::detalhe(&v),
            crate::geom::Detalhe::Capsula,
            "a fixtura tem de estar no regime da CAPSULA a zoom {zoom}"
        );
        let largura = largura_do_fio(FINO, &v);
        assert!(
            (largura - FINO).abs() < 1e-6,
            "a zoom {zoom} o fio mede {largura} e devia parar de encolher em {FINO}"
        );
        // O CONTROLO: sem o piso ele seria isto, e é isto que o dono não via.
        assert!(
            FINO * zoom < 1.3,
            "a fixtura deixou de conter o fenomeno: sem piso o fio mediria {} px a zoom {zoom}",
            FINO * zoom
        );
    }
}

/// ⛔⛔ **ACIMA DO LIMIAR A LARGURA É A DE SEMPRE, AO BIT** — o regime do cartão não muda um
/// pixel, e é isso que separa esta cura de um número novo espalhado pelo painel.
///
/// FALSIFICADO por tirar o `match` e aplicar o piso sempre: a zoom `0,8` o fio passa de `1,44`
/// para `1,80`, num regime sobre o qual ninguém se queixou.
#[test]
fn acima_do_limiar_a_largura_do_fio_e_a_de_sempre() {
    for zoom in [ZOOM_DA_CAPSULA + 0.01, 0.8, 1.0, 1.7, 2.5] {
        let v = vista_do_fio(zoom);
        assert_eq!(
            crate::geom::detalhe(&v),
            crate::geom::Detalhe::Completo,
            "a fixtura tem de estar no regime do CARTAO a zoom {zoom}"
        );
        for base in [1.6_f32, 1.8, 4.0, 5.2] {
            let largura = largura_do_fio(base, &v);
            assert!(
                (largura - base * zoom).abs() < f32::EPSILON,
                "a zoom {zoom} o fio de base {base} mede {largura} e tem de medir {} AO BIT",
                base * zoom
            );
        }
    }
}

/// ⭐⭐ **O CANAL DA MASSA SOBREVIVE AO PISO** — a largura de um fio *diz quantos elementos passam
/// nele*, e um tecto único achataria os dois no mesmo traço.
///
/// FALSIFICADO por `largura.max(PISO_EM_PIXEIS)` com um piso absoluto: a zoom `0,2` o fio fino e
/// o pesado passam a medir o mesmo, e a leitura da massa morre exactamente no zoom em que ela
/// seria mais útil (a cadeia inteira à vista).
#[test]
fn a_massa_do_fio_sobrevive_ao_piso() {
    for zoom in [0.2_f32, 0.3, ZOOM_DA_CAPSULA, 1.0, 2.5] {
        let v = vista_do_fio(zoom);
        let fino = largura_do_fio(crate::flow::wire_width(Some(1)), &v);
        let pesado = largura_do_fio(crate::flow::wire_width(Some(4096)), &v);
        assert!(
            pesado > fino * 2.0,
            "a zoom {zoom} o fio pesado ({pesado}) tem de continuar a ler-se mais grosso que o \
             fino ({fino})"
        );
    }
}

/// ⭐⭐ **TODO TRAÇO DE FIO PASSA PELA PORTA** — o censo que faz a próxima linha herdar a lei em
/// vez de a redescobrir.
///
/// ⚠️ **Ele é TEXTUAL e sabe-se fraco** (*um censo de texto sobrevive a um `if false &&`*): o que
/// os três gates acima medem é a LEI, por valor; o que só um censo vê é um **traço NOVO** que
/// alguém escreva a multiplicar pelo zoom à mão. ⛔ A regra é por NOME de largura de fio, e não
/// por «qualquer `* view.zoom`»: o anel de um badge `pre` multiplica pelo zoom com toda a razão,
/// e uma regra larga demais obrigaria a isentá-lo — que é como um censo morre.
#[test]
fn nenhum_traco_de_fio_escapa_a_porta() {
    let fonte = include_str!("paint_wire.rs");
    for nome in ["GHOST_W", "WIRE_W_DELAYED", "WIRE_W_HOVER"] {
        let agulha = format!("{nome} * view.zoom");
        assert!(
            !fonte.contains(&agulha),
            "«{agulha}» traca um fio sem passar pelo `largura_do_fio` — na capsula ele volta a \
             desaparecer"
        );
    }
    // ⚠️⚠️ **A única multiplicação legítima vive DENTRO da porta, e a régua é o ESCOPO e não uma
    // CONTAGEM.** A 1.ª redacção contava as ocorrências de `base_w * view.zoom` e exigia `1`; ela
    // reprovou sobre produto CERTO, porque *essa cadeia é PREFIXO da outra* —
    // `base_w * view.zoom.max(…)` contém-na à letra. ⛔ Um censo por substring não sabe onde uma
    // expressão acaba; o que ele sabe é onde um BLOCO começa.
    let porta = fonte
        .split_once("pub(crate) fn largura_do_fio")
        .expect("a porta tem de existir neste ficheiro");
    let corpo_da_porta = porta
        .1
        .split_once("\n}\n")
        .expect("a porta tem de fechar")
        .0;
    let fora = porta.0.matches("base_w * view.zoom").count()
        + porta
            .1
            .split_once("\n}\n")
            .expect("a porta tem de fechar")
            .1
            .matches("base_w * view.zoom")
            .count();
    assert_eq!(
        fora, 0,
        "ha' {fora} sitios FORA do `largura_do_fio` a multiplicar a largura pelo zoom a' mao"
    );
    // E o controlo de que a janela não é vazia (senão o zero acima é trivialmente verdadeiro).
    assert!(
        corpo_da_porta.matches("base_w * view.zoom").count() >= 2,
        "a janela da porta esta' vazia — este censo nao esta' a medir a porta"
    );
    // E o controlo POSITIVO: a porta é de facto chamada (senão este censo passaria sobre um
    // ficheiro onde ninguém desenha fio nenhum).
    assert!(
        fonte.matches("largura_do_fio(").count() >= 4,
        "a porta tem de ser chamada pelos tracos do fio — este censo esta' a varrer o vazio"
    );
}
