//! **OS FANTASMAS** — os gates de *que cópias este nó emite* (a ordem, os canais, a aberração,
//! o `falloff`, o caso morto). Cortados do `lib.rs` pelo tecto de LOC (ciclo 7, W1b, doc 112),
//! o mesmo corte que a `lens_tests.rs` já tinha feito.

use super::*;

/// Two elements, one at the origin and one out at `x = 2`, with a size column
/// riding along (so the copies must carry it).
fn pair(tint: [f32; 4]) -> Stream {
    Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [2.0, 0.0]]))
        .with("tint", Column::Vec4(vec![tint, tint]))
        .with("size", Column::Vec2(vec![[0.5, 0.5], [0.5, 0.5]]))
}

fn ps(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}
fn ts(s: &Stream) -> Vec<[f32; 4]> {
    match s.get("tint").unwrap() {
        Column::Vec4(v) => v.clone(),
        _ => panic!("tint"),
    }
}

/// The layout: ghosts FIRST (they must draw behind), the element LAST and
/// **verbatim**. FALSIFIED by any implementation that moves or recolours the
/// element itself — the body of the shape must survive the effect untouched.
#[test]
fn the_ghosts_sit_behind_and_the_element_survives_verbatim() {
    let src = pair([1.0, 1.0, 1.0, 1.0]);
    let out = split(&src, 0.0, 0.1, 0.0, 0.0, 1.0, Lens::CENTRED);
    assert_eq!(out.count(), 6, "two ghosts + the element, per element");

    let (p, t) = (ps(&out), ts(&out));
    // Rows 0-1: the R ghost, displaced +x.
    assert_eq!(p[0], [0.1, 0.0]);
    assert_eq!(p[1], [2.1, 0.0]);
    // Rows 2-3: the G+B ghost, displaced −x.
    assert_eq!(p[2], [-0.1, 0.0]);
    assert_eq!(p[3], [1.9, 0.0]);
    // Rows 4-5: the elements themselves, where they always were.
    assert_eq!(p[4], [0.0, 0.0]);
    assert_eq!(p[5], [2.0, 0.0]);
    assert_eq!(t[4], [1.0; 4], "the element keeps its own colour");
    assert_eq!(t[5], [1.0; 4]);

    // The copies inherit every other column (here: `size`).
    match out.get("size").unwrap() {
        Column::Vec2(v) => assert_eq!(v.len(), 6, "size rode along onto the ghosts"),
        _ => panic!("size"),
    }
}

/// Channel isolation is a MULTIPLY on the element's own tint — so a coloured
/// element throws the fringes its colour actually contains. FALSIFIED by the
/// naive "paint one ghost red and the other cyan", which would give a pure-blue
/// element a red fringe out of nowhere.
#[test]
fn the_channels_are_isolated_out_of_the_elements_own_colour() {
    let out = split(
        &pair([0.0, 0.2, 0.8, 1.0]),
        0.0,
        0.1,
        0.0,
        0.0,
        1.0,
        Lens::CENTRED,
    );
    let t = ts(&out);
    // A blue-ish element has NO red to throw: its R ghost is black.
    assert_eq!(
        t[0],
        [0.0, 0.0, 0.0, 1.0],
        "no red in the source, no red ghost"
    );
    // …and its G+B ghost carries exactly the green and blue it does have.
    assert_eq!(t[2], [0.0, 0.2, 0.8, 1.0]);
    // The two ghosts summed = the source colour (the additive split, recovered).
    let source = ts(&pair([0.0, 0.2, 0.8, 1.0]))[0];
    for ((r, gb), src) in t[0].iter().zip(&t[2]).zip(&source).take(3) {
        assert_eq!(r + gb, *src, "the R and G+B ghosts partition the colour");
    }
}

/// Radial mode: the fringe is ZERO at the centroid and grows with the distance
/// from it (lateral aberration). FALSIFIED by the uniform split, which would
/// displace the centre element too.
#[test]
fn aberration_is_zero_at_the_axis_and_grows_outward() {
    // Three elements at x = 0, 1, 2 → the centroid is x = 1.
    let src = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
    let out = split(&src, 1.0, 0.0, 0.0, 0.5, 1.0, Lens::CENTRED);
    let p = ps(&out);
    // The R ghost of the CENTRE element sits exactly on it — no fringe on axis.
    assert_eq!(p[1], [1.0, 0.0], "zero displacement at the optical axis");
    // The outer ones smear outward, by strength × their distance from it.
    assert_eq!(p[0], [-0.5, 0.0], "1 unit out → 0.5 of fringe");
    assert_eq!(p[2], [2.5, 0.0]);
    // The G+B ghost mirrors them inward.
    assert_eq!(p[3], [0.5, 0.0]);
    assert_eq!(p[5], [1.5, 0.0]);
}

/// `falloff` fades the FRINGES, never the element — so an artist can aberrate a
/// region of the layout and leave the rest clean.
#[test]
fn falloff_fades_the_fringes_and_never_the_element() {
    let src = pair([1.0, 1.0, 1.0, 1.0]).with("falloff", Column::Scalar(vec![0.0, 1.0]));
    let t = ts(&split(&src, 0.0, 0.1, 0.0, 0.0, 1.0, Lens::CENTRED));
    assert_eq!(t[0][3], 0.0, "element 0 is masked out → invisible fringe");
    assert_eq!(t[1][3], 1.0, "element 1 keeps its fringe");
    assert_eq!(t[4], [1.0; 4], "the masked element itself is untouched");
}

/// The effect turns ITSELF off rather than half-drawing: no opacity, or over the
/// instance budget, forwards the input verbatim (same count, same rows).
#[test]
fn a_dead_opacity_or_an_over_budget_stream_forwards_the_input() {
    let src = pair([1.0, 1.0, 1.0, 1.0]);
    let off = split(&src, 0.0, 0.1, 0.0, 0.0, 0.0, Lens::CENTRED);
    assert_eq!(off.count(), 2);
    assert_eq!(ps(&off), ps(&src), "verbatim, not three copies of nothing");

    let huge = Stream::new(MAX_INSTANCES); // 3 × over the ceiling
    assert_eq!(
        split(&huge, 0.0, 0.1, 0.0, 0.0, 1.0, Lens::CENTRED).count(),
        MAX_INSTANCES
    );
    assert_eq!(
        split(&Stream::new(0), 0.0, 0.1, 0.0, 0.0, 1.0, Lens::CENTRED).count(),
        0
    );
}
