//! Gates da pintura/hit das faixas da pilha (split do `stack_lane_paint.rs` pelo cap de 600 LOC
//! dos painéis; declarado lá como irmão via `#[path]`, então `super` é `stack_lane_paint`).
//!
//! Tudo aqui é sobre a **ORDEM e a existência dos hits** — as duas coisas que nenhuma pintura
//! headless alcança e que, quando erram, erram em silêncio: um grip que outro retângulo cobriu,
//! ou um grip travado que ainda aceita o clique.

use super::*;

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

/// The two grips of a strip with no fade authored yet, as `paint_lane` builds them.
fn rest_grips(id: u64, body: Rect, locked: (bool, bool)) -> Vec<(u64, u8, Rect, bool)> {
    [(EASE_IN, locked.0), (EASE_OUT, locked.1)]
        .into_iter()
        .filter_map(|(edge, lock)| ease_grip(body, 0.0, edge).map(|r| (id, edge, r, lock)))
        .collect()
}

/// **B4: the fade grip is reachable, and it does not eat the trim grip.**
///
/// A strip ALONE on a lane could not fade at all before this — `ease_in`/`ease_out`
/// existed, the evaluator honoured them, and nothing wrote them. The grip that writes
/// them sits on the strip's top corner, one trim-grip-width in, so both are grabbable:
/// the fade must not cost the artist the trim.
#[test]
fn the_fade_grip_is_reachable_and_the_trim_grip_survives_it() {
    let body = Rect::new(150.0, 60.0, 100.0, 20.0); // [150, 250)
    let plan = hit_plan(&[(1, body)], &rest_grips(1, body, (false, false)), band());

    assert_eq!(
        hit(&plan, 152.0, 62.0),
        Some((1, 0)),
        "the START corner is still the TRIM grip — the fade handle rests clear of it"
    );
    assert_eq!(
        hit(&plan, 159.0, 62.0),
        Some((1, EASE_IN)),
        "…and just inside it, at the top, is the FADE-IN grip"
    );
    assert_eq!(
        hit(&plan, 248.0, 62.0),
        Some((1, 1)),
        "mirror at the end: the TRIM grip"
    );
    assert_eq!(
        hit(&plan, 240.0, 62.0),
        Some((1, EASE_OUT)),
        "…and the FADE-OUT grip inside it"
    );
    assert_eq!(
        hit(&plan, 200.0, 62.0),
        Some((1, 2)),
        "between them, the top of the strip is still the BODY: the fade grips are \
         corners, not a bar across the whole top edge"
    );
    assert_eq!(
        hit(&plan, 159.0, 76.0),
        Some((1, 2)),
        "and below the grip, the same x is the BODY — the slide keeps most of the height"
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
        hit(&plan, 159.0, 62.0),
        Some((1, 2)),
        "…so that corner is just the strip's body"
    );
    assert!(
        plan.iter().any(|(_, e, _)| *e == EASE_OUT),
        "and the edge the strip DOES own keeps its grip"
    );
}

/// A strip too narrow to hold both trim grips AND both fade grips offers **no fade
/// grip** — rather than one that lands on top of a trim grip and steals it. A strip you
/// cannot fade at this zoom is honest; a strip you can no longer TRIM is a bug.
#[test]
fn a_strip_too_narrow_for_both_grips_keeps_the_trim_and_drops_the_fade() {
    let tiny = Rect::new(150.0, 60.0, 20.0, 20.0); // < 2 * (EASE_REST_X + EASE_W)
    assert!(ease_grip(tiny, 0.0, EASE_IN).is_none());
    let plan = hit_plan(&[(1, tiny)], &rest_grips(1, tiny, (false, false)), band());
    assert!(
        !plan.iter().any(|(_, e, _)| *e == EASE_IN || *e == EASE_OUT),
        "no fade grips on a strip this small"
    );
    assert_eq!(hit(&plan, 151.0, 62.0), Some((1, 0)), "the trim grip lives");
    assert_eq!(
        hit(&plan, 169.0, 62.0),
        Some((1, 1)),
        "…and so does the other"
    );
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
