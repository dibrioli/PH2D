//! Gates da pintura/hit das faixas da pilha (split do `stack_lane_paint.rs` pelo cap de 600 LOC
//! dos painéis; declarado lá como irmão via `#[path]`, então `super` é `stack_lane_paint`).
//!
//! Tudo aqui é sobre a **ORDEM e a existência dos hits** — as duas coisas que nenhuma pintura
//! headless alcança e que, quando erram, erram em silêncio: um grip que outro retângulo cobriu,
//! ou um grip travado que ainda aceita o clique.

use super::*;
use crate::graph::TimeView;
use crate::stack_ease_grip::{EASE_IN, EASE_OUT, blend_px, ease_grips, strip_grips};

/// Last registration wins, so this is what a click at `(x, y)` resolves to.
fn hit(plan: &[(u64, u8, Rect)], x: f32, y: f32) -> Option<(u64, u8)> {
    plan.iter()
        .rev()
        .find(|(_, _, r)| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
        .map(|(s, e, _)| (*s, *e))
}

fn band() -> Rect {
    Rect::new(100.0, 50.0, 400.0, 200.0)
}

/// Os dois grips de fade de um strip, como o `paint_lane` os constrói. `fades` são as larguras
/// das cunhas em PIXELS (o que o `blend_px` mede).
fn grips(
    id: u64,
    body: Rect,
    fades: (f32, f32),
    locked: (bool, bool),
) -> Vec<(u64, u8, Rect, bool)> {
    match ease_grips(body, fades.0, fades.1) {
        Some((a, b)) => vec![(id, EASE_IN, a, locked.0), (id, EASE_OUT, b, locked.1)],
        None => Vec::new(),
    }
}

/// …e o caso comum: nenhuma fade autorada ainda (as alças em repouso).
fn rest_grips(id: u64, body: Rect, locked: (bool, bool)) -> Vec<(u64, u8, Rect, bool)> {
    grips(id, body, (0.0, 0.0), locked)
}

/// **Each operation is its OWN corner** (Enio, 2026-07-16): the GREEN top corners STRETCH
/// (codes 5/6), the RED bottom corners TRIM (codes 0/1). Top-half vs bottom-half of each
/// edge, and the fade handle rests just inside them, full height, so all three are
/// grabbable without a modifier.
#[test]
fn the_corners_split_stretch_top_from_trim_bottom_and_the_fade_grip_survives() {
    let body = Rect::new(150.0, 60.0, 100.0, 20.0); // [150, 250), y [60, 80), mid 70
    let plan = hit_plan(&[(1, body)], &rest_grips(1, body, (false, false)), band());

    // Left edge: TOP is stretch-start (green), BOTTOM is trim-start (red).
    assert_eq!(hit(&plan, 152.0, 62.0), Some((1, 5)), "top-left STRETCHES");
    assert_eq!(hit(&plan, 152.0, 77.0), Some((1, 0)), "bottom-left TRIMS");
    // Right edge: TOP stretch-end, BOTTOM trim-end.
    assert_eq!(hit(&plan, 248.0, 62.0), Some((1, 6)), "top-right STRETCHES");
    assert_eq!(hit(&plan, 248.0, 77.0), Some((1, 1)), "bottom-right TRIMS");
    // The fade handle rests just inside the start/end corners, and is grabbable over the
    // WHOLE height (the smoke rule: a thin tab on a short strip could be seen, not grabbed).
    // With EDGE_W = 12 the rest fade grips sit at [162, 174] (in) and [226, 238] (out).
    assert_eq!(hit(&plan, 168.0, 62.0), Some((1, EASE_IN)), "fade-in, top");
    assert_eq!(hit(&plan, 168.0, 77.0), Some((1, EASE_IN)), "…and bottom");
    assert_eq!(hit(&plan, 232.0, 62.0), Some((1, EASE_OUT)), "fade-out");
    assert_eq!(
        hit(&plan, 200.0, 70.0),
        Some((1, 2)),
        "between the handles the strip is BODY"
    );
}

/// **A fade the OVERLAP owns is not the strip's to author** (Unity's rule). The grip is
/// still painted — the artist must see that the edge is spoken for — but it registers
/// no hit, because a dimmed control that still dispatches is a control that lies.
///
/// FALSIFIED by dropping the `locked` test in `hit_plan`: the grip would take the click
/// and write an `ease_in` that `blend_in` then ignores, so the drag would do NOTHING and
/// the number would sit in the document, wrong, waiting to surface the day the neighbour
/// moves away.
#[test]
fn a_fade_defined_by_a_neighbour_is_painted_but_not_grabbable() {
    let body = Rect::new(150.0, 60.0, 100.0, 20.0);
    // The START is locked (a neighbour overlaps it); the END is the strip's own.
    let plan = hit_plan(&[(1, body)], &rest_grips(1, body, (true, false)), band());

    assert!(
        !plan.iter().any(|(_, e, _)| *e == EASE_IN),
        "the locked fade-in grip must not be registered at all"
    );
    assert_eq!(
        hit(&plan, 168.0, 62.0),
        Some((1, 2)),
        "…so where the (locked) fade-in would rest is just the strip's body"
    );
    assert!(
        plan.iter().any(|(_, e, _)| *e == EASE_OUT),
        "and the edge the strip DOES own keeps its grip"
    );
}

/// A strip too narrow to hold both corners AND both fade grips offers **no fade grip** —
/// rather than one that lands on a corner and steals it. A strip you cannot fade at this
/// zoom is honest; a strip you can no longer TRIM or STRETCH is a bug.
#[test]
fn a_strip_too_narrow_for_the_fade_keeps_its_corners() {
    let tiny = Rect::new(150.0, 60.0, 20.0, 20.0); // < 2 * (EASE_REST_X + EASE_W)
    assert!(ease_grips(tiny, 0.0, 0.0).is_none());
    let plan = hit_plan(&[(1, tiny)], &rest_grips(1, tiny, (false, false)), band());
    assert!(
        !plan.iter().any(|(_, e, _)| *e == EASE_IN || *e == EASE_OUT),
        "no fade grips on a strip this small"
    );
    // Both corners of both edges live: stretch on top (5/6), trim on the bottom (0/1).
    assert_eq!(hit(&plan, 151.0, 62.0), Some((1, 5)), "top-left stretches");
    assert_eq!(hit(&plan, 151.0, 77.0), Some((1, 0)), "bottom-left trims");
    assert_eq!(hit(&plan, 168.0, 62.0), Some((1, 6)), "top-right stretches");
    assert_eq!(hit(&plan, 168.0, 77.0), Some((1, 1)), "bottom-right trims");
}

/// **The crossfade's own grip must be reachable.** Two overlapping strips ARE a
/// crossfade (ADR-0115 R1), and the first thing you do after making one is grab
/// the left strip's END edge to tune it. Registering each strip whole put the
/// right strip's body on top of that edge, and the drag slid the neighbour
/// instead. Bodies first, edges after — every edge outranks every box.
#[test]
fn the_left_strips_end_edge_survives_the_overlap_that_makes_the_crossfade() {
    let a = Rect::new(150.0, 60.0, 100.0, 20.0); // A: [150, 250)
    let b = Rect::new(200.0, 60.0, 100.0, 20.0); // B: [200, 300) — 50 px of overlap
    let plan = hit_plan(&[(1, a), (2, b)], &[], band());

    assert_eq!(
        hit(&plan, 248.0, 70.0),
        Some((1, 1)),
        "A's end grip, deep inside B's body, must still be A's END edge"
    );
    assert_eq!(
        hit(&plan, 202.0, 70.0),
        Some((2, 0)),
        "and B's start grip is B's START edge"
    );
    assert_eq!(
        hit(&plan, 230.0, 70.0),
        Some((2, 2)),
        "between them: B's body"
    );
}

/// A strip that starts left of the view has an x far outside the time area. Its
/// unclipped body used to blanket the whole label column — and, registered after
/// them, win the clicks of the lane's name, weight, mute and "+ strip".
#[test]
fn a_strip_that_starts_before_the_view_never_reaches_the_label_column() {
    let straddling = Rect::new(-300.0, 60.0, 600.0, 20.0); // starts far left
    let plan = hit_plan(&[(1, straddling)], &[], band());

    assert!(
        hit(&plan, 20.0, 70.0).is_none(),
        "nothing of the strip may be clickable left of the time area"
    );
    assert_eq!(
        hit(&plan, 150.0, 70.0),
        Some((1, 2)),
        "inside the band it is still the strip's body"
    );
    assert!(
        !plan.iter().any(|(_, e, _)| *e == 0),
        "and its START grip is off-screen: a grip you cannot see must not be \
         grabbable, or the strip gets trimmed blind"
    );
}

/// Nothing registers outside the band, ever — the same rule that keeps a lane
/// scrolled half under the ruler from stealing the clicks of "+ Lane".
#[test]
fn no_hit_rect_ever_escapes_the_band() {
    let over = Rect::new(120.0, 20.0, 600.0, 20.0); // above the band, and too wide
    for (_, _, r) in hit_plan(&[(1, over)], &[], band()) {
        let b = band();
        assert!(
            r.x >= b.x && r.y >= b.y && r.x + r.w <= b.x + b.w && r.y + r.h <= b.y + b.h,
            "{r:?} escapes {b:?}"
        );
    }
}

/// **As duas alças NUNCA se atropelam** — nem no ponto em que as duas cunhas se encontram.
///
/// Um fade-in e um fade-out que quase se tocam no meio (a forma mais ordinária que existe)
/// punham os dois retângulos no MESMO lugar: como o hit index é last-wins e o OUT é registrado
/// por último, agarrar a ponta VISÍVEL do fade-in arrastava o fade-OUT. Mais fundo, os grips se
/// CRUZAVAM (o de entrada pintado à direita do de saída).
///
/// Por isso os dois são colocados JUNTOS (`ease_grips` devolve o par): quando as pontas se
/// encontram, eles se sentam lado a lado em torno do ponto médio — na ordem certa, nunca no
/// mesmo pixel. FALSIFICADO colocando um de cada vez.
#[test]
fn the_two_fade_grips_never_land_on_the_same_pixel_or_swap_sides() {
    let body = Rect::new(150.0, 60.0, 100.0, 20.0);
    // Varre TODAS as combinações de cunha, inclusive as que se cruzam fundo.
    for i in 0..=40 {
        for j in 0..=40 {
            let (in_px, out_px) = (i as f32 * 3.0, j as f32 * 3.0);
            let Some((a, b)) = ease_grips(body, in_px, out_px) else {
                continue;
            };
            assert!(
                a.x + a.w <= b.x + 1e-4,
                "in={in_px} out={out_px}: as alças se atropelam ({a:?} vs {b:?})"
            );
            for g in [a, b] {
                assert!(
                    g.x >= body.x && g.x + g.w <= body.x + body.w,
                    "in={in_px} out={out_px}: a alça {g:?} saiu do strip {body:?}"
                );
            }
        }
    }
}

/// **Uma alça de fade não pode roubar o grip de canto do VIZINHO.**
///
/// Strips se sobrepõem — é isso que é o crossfade. Então a alça de um pode cair em cima do grip
/// de canto do outro e, registrada depois de todo canto, ganhar o clique: o artista puxaria a
/// borda de B e autoraria uma fade em A. O `ease_grips` só protege as bordas do PRÓPRIO strip;
/// a recusa do vizinho mora aqui, no `hit_plan`, que é quem vê os dois.
#[test]
fn a_fade_grip_never_steals_a_neighbours_corner_grip() {
    // A começa antes; B entra por cima (o overlap É o crossfade).
    let a = Rect::new(150.0, 60.0, 120.0, 20.0); // [150, 270)
    let b = Rect::new(156.0, 60.0, 120.0, 20.0); // [156, 276) — a borda de B cai DENTRO de A
    let mut eases = rest_grips(1, a, (false, false));
    eases.extend(rest_grips(2, b, (false, false)));
    let plan = hit_plan(&[(1, a), (2, b)], &eases, band());

    // Os cantos iniciais de B ocupam [156, 162). Nada de A (nem sua alça de fade) pode
    // roubá-los: STRETCH no topo, TRIM embaixo.
    for x in [157.0_f32, 159.0, 161.0] {
        assert_eq!(
            hit(&plan, x, 62.0),
            Some((2, 5)),
            "x={x}: o canto de cima de B continua sendo STRETCH de B"
        );
        assert_eq!(
            hit(&plan, x, 77.0),
            Some((2, 0)),
            "x={x}: e o de baixo, TRIM de B"
        );
    }
}

/// **The trim preview marks the REMOVED span, and only while cutting.** A trim drag that
/// shortens a strip red-hatches the frames it drops (from the edge at drag-start to where
/// it is now); a trim that REVEALS more clip, or any non-trim gesture, hatches nothing.
#[test]
#[allow(clippy::field_reassign_with_default)] // the drag is re-set per case; struct-update hides it
fn the_trim_preview_marks_only_the_removed_span_and_only_when_cutting() {
    use crate::state::{StripDrag, TimelinePanelState};
    use ph2d_timeline::{StripId, StripLoop, StripView};
    let view = TimeView {
        time_x: 100.0,
        right: 500.0,
        view_start: 0.0,
        px_per_s: 100.0,
    };
    let sv = |t_start: f64, t_end: f64| StripView {
        id: StripId(7),
        clip_name: "X".into(),
        container: None,
        t_start,
        t_end,
        blend_in: 0.0,
        blend_out: 0.0,
        lead_in: 0.0,
        lead_out: 0.0,
        marks: [0.0; 4],
        ease_locked_in: false,
        ease_locked_out: false,
        loop_mode: StripLoop::Once,
        speed: 1.0,
    };
    let drag = |edge: u8| {
        Some(StripDrag {
            lane: 0,
            id: StripId(7),
            edge,
            start_x: 0.0,
            start_span: (1.0, 3.0), // the edges at drag-start
            start_ease: 0.0,
        })
    };
    let body = Rect::new(view.x(1.5), 60.0, 100.0, 20.0);
    let mut st = TimelinePanelState::default();

    // Trim-start dragged INWARD (now 1.5 > original 1.0): the cut is the removed [1.0, 1.5].
    st.strip_drag = drag(0);
    let cutting = sv(1.5, 3.0);
    let cut = trim_cut_region(&st, 0, &[(&cutting, body)], view).expect("a cut");
    assert!(
        (cut.x - view.x(1.0)).abs() < 1e-4 && (cut.x + cut.w - view.x(1.5)).abs() < 1e-4,
        "the hatch spans exactly the removed [1.0, 1.5]: {cut:?}"
    );
    assert!(
        (cut.y - body.y).abs() < 1e-4 && (cut.h - body.h).abs() < 1e-4,
        "…at the strip's own height"
    );

    // Trim-start dragged OUTWARD (now 0.5 < original 1.0) reveals clip — no cut.
    let revealing = sv(0.5, 3.0);
    assert!(
        trim_cut_region(&st, 0, &[(&revealing, body)], view).is_none(),
        "revealing more clip removes nothing"
    );

    // A BODY slide (edge 2) is not a trim — even one that slides the strip LEFT, so its
    // new end sits before the original: nothing is CUT, the whole clip just moved. (This
    // is the case that makes the edge filter load-bearing: a moved strip's `else` branch
    // would otherwise hatch a phantom cut.)
    st.strip_drag = drag(2);
    let slid = sv(0.5, 2.5);
    assert!(
        trim_cut_region(&st, 0, &[(&slid, body)], view).is_none(),
        "a body slide is not a cut, even sliding left"
    );

    // And a fade drag (edge 3) is not a trim either.
    st.strip_drag = drag(3);
    assert!(
        trim_cut_region(&st, 0, &[(&cutting, body)], view).is_none(),
        "only a trim drag previews a cut"
    );
}

/// `blend_px` mede a cunha que o painel desenha — é o que ancora a alça na PONTA dela. Sem isto,
/// os gates acima passariam com uma alça que nunca sai do repouso.
#[test]
fn the_grip_rides_the_wedge_tip() {
    let view = TimeView {
        time_x: 100.0,
        right: 500.0,
        view_start: 0.0,
        px_per_s: 100.0,
    };
    assert!(
        (blend_px(view, 1.0, 0.5) - 50.0).abs() < 1e-4,
        "meio segundo a 100 px/s é 50 px de cunha"
    );
    // …e a alça segue a ponta: 50 px de cunha põem o grip 50 px dentro do strip.
    let body = Rect::new(150.0, 60.0, 200.0, 20.0);
    let (a, _) = ease_grips(body, 50.0, 0.0).expect("cabe");
    assert!(
        (a.x - (body.x + 50.0)).abs() < 1e-4,
        "a alça tem de estar NA ponta da cunha, não no repouso: {a:?}"
    );
}

/// **Uma alça de fade um pouco PARA FORA da strip continua agarrável** (Enio, 2026-07-19:
/// *"a alça do fade não pode ser movida se ficar um pouco para fora da strip"*).
///
/// Um `lead` menor que a largura da própria alça punha o grip em cima da quina da PRÓPRIA
/// strip, e o `hit_plan` DERRUBA toda alça que cobre uma quina — então o artista via a alça
/// e não conseguia pegá-la (e alargar as quinas para 12 px piorou isso). O grip agora para
/// na borda do corpo: a quina fica inteira DENTRO, a alça inteira FORA, e as duas se
/// registram.
#[test]
fn an_outward_fade_grip_just_past_the_edge_is_still_grabbable() {
    let body = Rect::new(150.0, 60.0, 100.0, 20.0); // [150, 250)
    // Um lead de 3 px — bem menor que a largura da alça.
    let (a, b) = strip_grips(body, 3.0, 0.0, 0.0, 3.0).expect("grips");
    assert!(
        a.x + a.w <= body.x + 1e-3,
        "o grip de entrada fica FORA do corpo: {a:?}"
    );
    assert!(
        b.x >= body.x + body.w - 1e-3,
        "o grip de saída fica FORA do corpo: {b:?}"
    );

    let eases = vec![(1, EASE_IN, a, false), (1, EASE_OUT, b, false)];
    let plan = hit_plan(&[(1, body)], &eases, band());
    assert!(
        plan.iter().any(|(_, e, _)| *e == EASE_IN),
        "a alça de entrada tem de estar REGISTRADA (não derrubada pela quina): {plan:?}"
    );
    assert!(
        plan.iter().any(|(_, e, _)| *e == EASE_OUT),
        "a alça de saída tem de estar REGISTRADA: {plan:?}"
    );
    // …e as quinas seguem inteiras — a alça não roubou o trim.
    assert_eq!(
        hit(&plan, 155.0, 77.0),
        Some((1, 0)),
        "trim do início intacto"
    );
    assert_eq!(hit(&plan, 245.0, 77.0), Some((1, 1)), "trim do fim intacto");
}

/// **O código que o REALCE pergunta é o que o hit index REGISTRA.** O hover pinta a quina em
/// Accent quando o ponteiro está nela, resolvendo o id por `corner_hit_edge`; se esse
/// mapeamento divergir do que o `hit_plan` arquiva, uma quina acende pelo alvo da vizinha —
/// o gizmo mentindo sobre o que o clique vai fazer.
#[test]
fn the_corner_highlight_asks_for_the_edge_the_hit_plan_registers() {
    let body = Rect::new(150.0, 60.0, 100.0, 20.0);
    let plan = hit_plan(&[(1, body)], &[], band());
    for (stretch, edge, x, y) in [
        (true, 0u8, 152.0, 62.0),  // topo-esquerda: stretch-start
        (true, 1u8, 248.0, 62.0),  // topo-direita: stretch-end
        (false, 0u8, 152.0, 77.0), // base-esquerda: trim-start
        (false, 1u8, 248.0, 77.0), // base-direita: trim-end
    ] {
        let registered = hit(&plan, x, y).expect("a quina está registrada").1;
        assert_eq!(
            crate::strip_paint::corner_hit_edge(stretch, edge),
            registered,
            "o realce da quina (stretch={stretch}, edge={edge}) pergunta por um código \
             diferente do que o hit_plan registrou"
        );
    }
}
