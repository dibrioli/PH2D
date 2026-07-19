//! Gates for the Colorize engine (`docs/Flip/09 §3`, §7.3).
//!
//! The load-bearing gate is `the_colour_cut_hugs_the_ink_not_the_midpoint`: it is what only
//! LazyBrush does — the boundary between two colours is *attracted to the line*, not drawn
//! halfway between the scribbles. It is red-first-able by construction: strip the ink from
//! the energy (`V_pq` uniform) and the split falls at the scribble midpoint, failing the
//! assertion. The flow solver itself is pinned by `flow_tests.rs` (BK ≡ Edmonds–Karp); these
//! gates pin the WIRING — energy assembly, the guloso multiway, the geometry — on a fixture
//! that contains the phenomenon.

use super::{Scribble, colorize};
use ph2d_core::Vec2;
use ph2d_flip_fill::{FillResult, signed_area};

/// A closed box with an internal vertical divider at `divider_x`, broken by a gap
/// `(gap.0, gap.1)` in the middle — the line-art the colour cut must respect.
fn boxed_with_divider(divider_x: f32, gap: (f32, f32)) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    vec![
        (
            vec![
                Vec2::new(0.1, 0.1),
                Vec2::new(0.9, 0.1),
                Vec2::new(0.9, 0.9),
                Vec2::new(0.1, 0.9),
            ],
            vec![0.0; 4],
            true,
        ),
        (
            vec![Vec2::new(divider_x, 0.1), Vec2::new(divider_x, gap.0)],
            vec![0.0; 2],
            false,
        ),
        (
            vec![Vec2::new(divider_x, gap.1), Vec2::new(divider_x, 0.9)],
            vec![0.0; 2],
            false,
        ),
    ]
}

fn max_x(f: &FillResult) -> f32 {
    f.outer.iter().fold(f32::MIN, |m, p| m.max(p.x))
}
fn centroid_x(f: &FillResult) -> f32 {
    f.outer.iter().map(|p| p.x).sum::<f32>() / f.outer.len() as f32
}
fn area(f: &FillResult) -> f32 {
    signed_area(&f.outer).abs()
}

/// The defining LazyBrush property: with the divider OFF-CENTER at x=0.7, the boundary
/// between the two colours sits AT the ink (x≈0.7) — so the left colour owns most of the box
/// — and NOT halfway between the scribbles (x≈0.55). An energy that ignored the ink would
/// split at the midpoint and fail here.
#[test]
fn the_colour_cut_hugs_the_ink_not_the_midpoint() {
    let strokes = boxed_with_divider(0.7, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.25, 0.3), Vec2::new(0.25, 0.7)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.85, 0.3), Vec2::new(0.85, 0.7)],
            width: 0.0,
        },
    ];
    let regions = colorize(&strokes, &scribbles, 80.0);
    assert_eq!(regions.len(), 2, "two colours → two regions");

    let r0 = &regions.iter().find(|r| r.label == 0).expect("label 0").fill;
    let r1 = &regions.iter().find(|r| r.label == 1).expect("label 1").fill;

    let b = max_x(r0);
    assert!(
        (0.63..=0.77).contains(&b),
        "the left colour must reach the ink at 0.7, not the scribble midpoint 0.55 (got {b})"
    );
    assert!(
        area(r0) > area(r1),
        "the off-center divider gives the left colour more area ({} vs {})",
        area(r0),
        area(r1)
    );
}

/// A gap in the divider does NOT leak the colour across it — the cut jumps the gap in white
/// (paying its width) rather than routing a whole colour through it. Symmetric divider ⇒ two
/// comparable regions, one per side.
#[test]
fn a_gap_in_the_divider_does_not_leak_the_colour() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.28, 0.3), Vec2::new(0.28, 0.7)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.72, 0.3), Vec2::new(0.72, 0.7)],
            width: 0.0,
        },
    ];
    let regions = colorize(&strokes, &scribbles, 80.0);
    assert_eq!(regions.len(), 2, "two comparable regions, one per side");

    let r0 = &regions.iter().find(|r| r.label == 0).expect("label 0").fill;
    let r1 = &regions.iter().find(|r| r.label == 1).expect("label 1").fill;
    assert!(centroid_x(r0) < 0.5, "label 0 on the left");
    assert!(centroid_x(r1) > 0.5, "label 1 on the right");

    let (a0, a1) = (area(r0), area(r1));
    let ratio = a0.min(a1) / a0.max(a1);
    assert!(
        ratio > 0.55,
        "the gap must not leak a colour: areas {a0} vs {a1}"
    );
}

/// HR-5: two identical clicks give the same drawing — same geometry, to the vertex.
#[test]
fn the_colorize_is_deterministic() {
    let strokes = boxed_with_divider(0.6, (0.45, 0.55));
    let scribbles = vec![
        Scribble {
            label: 0,
            points: vec![Vec2::new(0.25, 0.3), Vec2::new(0.25, 0.7)],
            width: 0.0,
        },
        Scribble {
            label: 1,
            points: vec![Vec2::new(0.8, 0.3), Vec2::new(0.8, 0.7)],
            width: 0.0,
        },
    ];
    let a = colorize(&strokes, &scribbles, 80.0);
    let b = colorize(&strokes, &scribbles, 80.0);
    assert_eq!(
        a.len(),
        2,
        "the fixture must actually colour (else 0==0 is vacuous)"
    );
    assert_eq!(a.len(), b.len());
    for (ra, rb) in a.iter().zip(&b) {
        assert_eq!(ra.label, rb.label);
        assert_eq!(ra.fill.outer, rb.fill.outer, "geometry must be identical");
    }
}

/// **O que o artista PINTA é o que SEMEIA.** Um TOQUE curto com pincel grosso tem eixo de
/// poucos pixels, e uma semente desse tamanho DEGENERA o corte: o mínimo passa a ser "cercar
/// o pixel" (perímetro pequeno em papel caro) em vez de correr pela tinta, e a região não
/// colore. Com a espessura semeando, o mesmo toque funciona. É o red-first do campo `width`:
/// zerá-lo derruba a metade `fat`.
#[test]
fn a_short_fat_dab_seeds_its_region_where_a_thin_one_degenerates() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    // Um TOQUE: dois pontos quase coincidentes — o que um clique curto de fato produz.
    let dab = |width: f32| {
        vec![
            Scribble {
                label: 0,
                points: vec![Vec2::new(0.28, 0.50), Vec2::new(0.28, 0.51)],
                width,
            },
            Scribble {
                label: 1,
                points: vec![Vec2::new(0.72, 0.50), Vec2::new(0.72, 0.51)],
                width,
            },
        ]
    };
    let thin = colorize(&strokes, &dab(0.0), 80.0);
    let fat = colorize(&strokes, &dab(0.12), 80.0);
    assert_eq!(
        fat.len(),
        2,
        "um toque com pincel grosso tem de colorir os dois lados (got {})",
        fat.len()
    );
    assert!(
        thin.len() < 2,
        "se o eixo puro já colorisse, a largura não seria load-bearing (fino {})",
        thin.len()
    );
}

/// No lines or no scribbles → nothing (a colour with no line to bound it is not a region).
#[test]
fn no_lines_or_no_scribbles_yields_nothing() {
    let strokes = boxed_with_divider(0.5, (0.45, 0.55));
    assert!(colorize(&strokes, &[], 80.0).is_empty());
    let scr = vec![Scribble {
        label: 0,
        points: vec![Vec2::new(0.5, 0.5)],
        width: 0.0,
    }];
    assert!(colorize(&[], &scr, 80.0).is_empty());
}
