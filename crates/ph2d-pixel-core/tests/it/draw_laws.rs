//! **P1: as leis que o MVP não tem** (cantos redondos, polígono, pincel em linha e rodado,
//! diagonais, ladrilho por eixo, tolerância do balde, spray) — cada uma com o seu controlo.

use std::collections::BTreeSet;

use ph2d_pixel_core::brush::{Brush, BrushShape};
use ph2d_pixel_core::fill::{Bucket, BucketSample, bucket_cells, flood};
use ph2d_pixel_core::raster::{
    Cell, polygon_cells, rect_cells, rounded_rect_cells, snap_pixel_angle, square_box,
};
use ph2d_pixel_core::symmetry::{Mirrors, Symmetry, Tiling, expand, symmetry_images};
use ph2d_pixel_core::{Background, ColorMode, History, NewSprite, Paint, Sprite, Stroke};

fn set(f: impl FnOnce(&mut dyn FnMut(i32, i32))) -> BTreeSet<Cell> {
    let mut out = BTreeSet::new();
    f(&mut |x, y| {
        out.insert((x, y));
    });
    out
}

/// O conjunto é um laço fechado 8-conectado: cada célula tem ≥ 2 vizinhas no conjunto.
fn closed(s: &BTreeSet<Cell>) -> bool {
    s.iter().all(|&(x, y)| {
        let n = (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter(|&(dx, dy)| (dx, dy) != (0, 0) && s.contains(&(x + dx, y + dy)))
            .count();
        n >= 2
    })
}

#[test]
fn a_rounded_rectangle_is_the_rectangle_at_radius_zero_and_symmetric_otherwise() {
    for (x1, y1) in [(9, 6), (12, 12), (3, 20), (1, 1)] {
        let plain = set(|cb| rect_cells(0, 0, x1, y1, false, cb));
        assert_eq!(
            set(|cb| rounded_rect_cells(0, 0, x1, y1, 0, false, cb)),
            plain
        );
        for r in 1..6 {
            let o = set(|cb| rounded_rect_cells(0, 0, x1, y1, r, false, cb));
            let f = set(|cb| rounded_rect_cells(0, 0, x1, y1, r, true, cb));
            assert!(closed(&o), "{x1}×{y1} r={r} abre: {o:?}");
            assert!(o.is_subset(&f), "o cheio contém o contorno");
            assert!(
                o.iter()
                    .all(|&(x, y)| (0..=x1).contains(&x) && (0..=y1).contains(&y)),
                "dentro da bbox"
            );
            for &(x, y) in &o {
                assert!(
                    o.contains(&(x1 - x, y)) && o.contains(&(x, y1 - y)),
                    "simétrico"
                );
            }
            if x1 >= 4 && y1 >= 4 {
                assert!(!o.contains(&(0, 0)), "{x1}×{y1} r={r}: o canto é redondo");
            }
        }
    }
}

#[test]
fn a_polygon_is_closed_and_its_interior_is_solid_by_row() {
    let tri = [(0, 0), (12, 3), (4, 10)];
    let (o, inner) = polygon_cells(&tri);
    let os: BTreeSet<Cell> = o.iter().copied().collect();
    for v in tri {
        assert!(os.contains(&v), "o vértice {v:?}");
    }
    assert!(closed(&os), "o contorno fecha");
    let all: BTreeSet<Cell> = os
        .union(&inner.iter().copied().collect())
        .copied()
        .collect();
    for y in 0..=10 {
        let row: Vec<i32> = all.iter().filter(|c| c.1 == y).map(|c| c.0).collect();
        if let (Some(&a), Some(&b)) = (row.first(), row.last()) {
            assert_eq!(row.len() as i32, b - a + 1, "linha {y} sem buraco");
        }
    }
    assert!(inner.contains(&(5, 4)), "o meio é interior");
    // Controlo: um polígono degenerado (três pontos numa reta) não tem interior.
    assert!(polygon_cells(&[(0, 0), (5, 0), (9, 0)]).1.is_empty());
    // Côncavo: o «C» não enche a boca.
    let c = [
        (0, 0),
        (10, 0),
        (10, 2),
        (2, 2),
        (2, 8),
        (10, 8),
        (10, 10),
        (0, 10),
    ];
    let (_, ci) = polygon_cells(&c);
    assert!(!ci.contains(&(6, 5)), "a boca do C fica vazia");
    assert!(ci.contains(&(1, 5)), "a espinha do C enche");
}

#[test]
fn the_line_brush_and_the_rotated_square_keep_their_size() {
    for d in 1..=12u32 {
        let line = Brush {
            shape: BrushShape::Line,
            size: d,
            angle: 0,
        }
        .offsets();
        assert_eq!(line.len() as u32, d, "linha 0° de {d}");
        assert!(line.iter().all(|c| c.1 == 0), "na linha da âncora");
        let up = Brush {
            shape: BrushShape::Line,
            size: d,
            angle: 90,
        }
        .offsets();
        assert!(
            up.iter().all(|c| c.0 == 0) && up.len() as u32 == d,
            "90° de {d}: {up:?}"
        );
        let diag = Brush {
            shape: BrushShape::Line,
            size: d,
            angle: 45,
        }
        .offsets();
        assert!(
            diag.windows(2).all(|w| (w[0].0 - w[1].0).abs() == 1),
            "45° anda na diagonal"
        );
        let sq = |angle| {
            Brush {
                shape: BrushShape::Square,
                size: d,
                angle,
            }
            .offsets()
        };
        assert_eq!(sq(90), sq(0), "um quarto de volta é o mesmo quadrado");
        let n0 = sq(0).len() as f64;
        let n30 = sq(30).len() as f64;
        assert!(
            (n30 - n0).abs() <= n0.max(4.0) * 0.6,
            "rodado guarda a área ({n0} → {n30})"
        );
    }
}

#[test]
fn diagonal_symmetry_is_an_involution_and_h_plus_d_gives_eight() {
    let d1 = Symmetry::centered(
        Mirrors {
            d1: true,
            ..Mirrors::default()
        },
        16,
        16,
    );
    assert_eq!(
        symmetry_images(d1, (3, 9))
            .iter()
            .map(|i| i.0)
            .collect::<Vec<_>>(),
        vec![(3, 9), (9, 3)]
    );
    assert_eq!(symmetry_images(d1, (5, 5)).len(), 1, "sobre a diagonal");
    let d2 = Symmetry::centered(
        Mirrors {
            d2: true,
            ..Mirrors::default()
        },
        16,
        16,
    );
    assert_eq!(
        symmetry_images(d2, (3, 9)).last().map(|i| i.0),
        Some((6, 12))
    );
    let all = Symmetry::centered(
        Mirrors {
            h: true,
            d1: true,
            ..Mirrors::default()
        },
        16,
        16,
    );
    assert_eq!(symmetry_images(all, (2, 5)).len(), 8, "o grupo inteiro");
    let both = Symmetry::centered(
        Mirrors {
            d1: true,
            d2: true,
            ..Mirrors::default()
        },
        16,
        16,
    );
    assert_eq!(symmetry_images(both, (2, 5)).len(), 4);
    // O eixo a meio pixel: com axis2 ímpar (7,5 numa largura de 16) a imagem de 7 é 7.
    let half = Symmetry {
        mirrors: Mirrors {
            h: true,
            ..Mirrors::default()
        },
        axis2_x: 15,
        axis2_y: 16,
    };
    assert_eq!(symmetry_images(half, (7, 0)).len(), 1);
    assert_eq!(symmetry_images(half, (3, 0))[1].0, (11, 0));
}

#[test]
fn tiling_wraps_only_the_axis_it_names() {
    let offs = [(0, 0), (1, 0), (0, 1)];
    let run = |t: Tiling| {
        let mut v = Vec::new();
        expand((7, 4), &offs, Symmetry::default(), t, (8, 5), |x, y, _| {
            v.push((x, y))
        });
        v
    };
    assert_eq!(run(Tiling::default()), vec![(7, 4)], "corta os dois");
    assert_eq!(run(Tiling { x: true, y: false }), vec![(7, 4), (0, 4)]);
    assert_eq!(run(Tiling { x: false, y: true }), vec![(7, 4), (7, 0)]);
    assert_eq!(
        run(Tiling { x: true, y: true }),
        vec![(7, 4), (0, 4), (7, 0)]
    );
}

fn sprite(mode: ColorMode, w: u32, h: u32) -> Sprite {
    Sprite::new(NewSprite {
        width: w,
        height: h,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn paint(s: &mut Sprite, h: &mut History, cells: &[Cell], v: u32) {
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut st = Stroke::new(l, f, v, Paint::default());
    st.set_shape(s, cells, &[]);
    if let Some(op) = st.op(s) {
        h.commit(s, op).expect("commit");
    }
}

#[test]
fn the_bucket_respects_tolerance_contiguity_connectivity_and_sample() {
    let mut s = sprite(ColorMode::Rgba, 8, 8);
    let mut h = History::default();
    let key = (s.layers()[0].id, s.frames()[0].id);
    let red = u32::from_le_bytes([200, 0, 0, 255]);
    let red2 = u32::from_le_bytes([210, 0, 0, 255]);
    let wall = u32::from_le_bytes([0, 0, 255, 255]);
    paint(&mut s, &mut h, &[(0, 0), (1, 0)], red);
    paint(&mut s, &mut h, &[(2, 0)], red2);
    paint(&mut s, &mut h, &[(6, 6)], red);
    let opt = |tolerance, contiguous, eight| Bucket {
        tolerance,
        contiguous,
        eight,
        sample: BucketSample::Layer,
    };
    assert_eq!(bucket_cells(&s, key, (0, 0), opt(0, true, false)).len(), 2);
    assert_eq!(
        bucket_cells(&s, key, (0, 0), opt(10, true, false)).len(),
        3,
        "tolerância 10"
    );
    assert_eq!(
        bucket_cells(&s, key, (0, 0), opt(0, false, false)).len(),
        3,
        "global"
    );
    // 4 × 8: uma parede em diagonal separa no 4, vaza no 8.
    let diag: Vec<Cell> = (0..8).map(|k| (k, 7 - k)).collect();
    let mut t = sprite(ColorMode::Rgba, 8, 8);
    paint(&mut t, &mut h, &diag, wall);
    let tk = (t.layers()[0].id, t.frames()[0].id);
    let four = bucket_cells(&t, tk, (0, 0), opt(0, true, false)).len();
    let eight = bucket_cells(&t, tk, (0, 0), opt(0, true, true)).len();
    assert_eq!(four, 28, "o triângulo de cima");
    assert_eq!(eight, 64 - 8, "o 8 atravessa a diagonal");
    // Visible: o que se vê decide (sem camada por cima, igual à Layer).
    let vis = Bucket {
        sample: BucketSample::Visible,
        ..opt(0, true, false)
    };
    assert_eq!(bucket_cells(&t, tk, (0, 0), vis).len(), four);
    // Indexed com tolerância 0 compara o ÍNDICE.
    let ix = sprite(ColorMode::Indexed, 4, 4);
    let ik = (ix.layers()[0].id, ix.frames()[0].id);
    assert_eq!(bucket_cells(&ix, ik, (0, 0), opt(0, true, false)).len(), 16);
    assert!(
        bucket_cells(&ix, ik, (9, 9), opt(0, true, false)).is_empty(),
        "fora"
    );
}

#[test]
fn the_bucket_fills_a_4096_square_without_recursion() {
    let n = 4096u32;
    let mut count = 0usize;
    flood(n, n, (2048, 2048), false, |_, _| true, |_, _| count += 1);
    assert_eq!(count, (n * n) as usize);
    // Um labirinto de colunas força a pilha a crescer em largura e altura.
    let mut c2 = 0usize;
    flood(
        n,
        n,
        (0, 0),
        false,
        |x, y| x % 2 == 0 || y == 0 || y == n as i32 - 1,
        |_, _| c2 += 1,
    );
    assert!(c2 > (n * n / 2) as usize);
}

#[test]
fn the_same_spray_gesture_gives_the_same_cells_and_stays_in_the_disc() {
    let s = sprite(ColorMode::Rgba, 64, 64);
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let run = |at: Cell| {
        let mut st = Stroke::begin(&s, l, f, 1, Paint::default(), at);
        st.spray(&s, at, 8, 40);
        st.spray(&s, (at.0 + 3, at.1), 8, 40);
        st.cells().collect::<Vec<_>>()
    };
    assert_eq!(run((30, 30)), run((30, 30)), "o mesmo gesto");
    assert_ne!(run((30, 30)), run((31, 30)), "outro gesto, outro sorteio");
    let cells = run((30, 30));
    assert!(cells.len() > 20, "o jato espalha: {}", cells.len());
    assert!(cells.iter().all(|&(x, y)| {
        let d1 = f64::from(x - 30).hypot(f64::from(y - 30));
        let d2 = f64::from(x - 33).hypot(f64::from(y - 30));
        d1.min(d2) <= 8.5
    }));
}

#[test]
fn pixel_perfect_retraction_never_erases_a_cell_another_point_painted() {
    let s = sprite(ColorMode::Rgba, 16, 16);
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let pp = Paint {
        pixel_perfect: true,
        ..Paint::default()
    };
    // Volta ao (1,0) depois de o largar como cotovelo: a 2.ª passagem fica.
    let mut st = Stroke::begin(&s, l, f, 1, pp, (0, 0));
    for p in [(1, 0), (1, 1), (2, 1), (2, 2)] {
        st.to(&s, p.0, p.1);
    }
    assert_eq!(st.value_at(1, 0), None, "o cotovelo saiu");
    assert_eq!(st.value_at(1, 1), Some(1));
    // Controlo: sem pixel-perfect o cotovelo fica.
    let mut raw = Stroke::begin(&s, l, f, 1, Paint::default(), (0, 0));
    for p in [(1, 0), (1, 1)] {
        raw.to(&s, p.0, p.1);
    }
    assert_eq!(raw.value_at(1, 0), Some(1));
    // Pincel maior que 1: o pixel-perfect não se aplica (Aseprite).
    let big = Paint {
        pixel_perfect: true,
        brush: Brush {
            size: 3,
            ..Brush::default()
        },
        ..Paint::default()
    };
    let mut b = Stroke::begin(&s, l, f, 1, big, (4, 4));
    for p in [(5, 4), (5, 5)] {
        b.to(&s, p.0, p.1);
    }
    assert_eq!(b.value_at(5, 4), Some(1));
}

#[test]
fn a_shape_redrawn_replaces_the_previous_preview_and_commits_one_step() {
    let mut s = sprite(ColorMode::Rgba, 16, 16);
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut st = Stroke::new(l, f, 7, Paint::default());
    let big = set(|cb| rect_cells(0, 0, 10, 10, false, cb));
    let small = set(|cb| rect_cells(0, 0, 3, 3, false, cb));
    st.set_shape(&s, &big.iter().copied().collect::<Vec<_>>(), &[]);
    st.set_shape(&s, &small.iter().copied().collect::<Vec<_>>(), &[]);
    assert_eq!(
        st.cells().collect::<BTreeSet<_>>(),
        small,
        "o arrasto encolheu a forma"
    );
    let mut h = History::default();
    let op = st.op(&s).expect("op");
    h.commit(&mut s, op).expect("commit");
    assert!(h.undo(&mut s));
    assert!(!h.undo(&mut s), "um passo só");
}

#[test]
fn shift_squares_the_box_and_snaps_lines_to_pixel_angles() {
    assert_eq!(square_box(2, 2, 7, 4), (7, 7));
    assert_eq!(square_box(2, 2, -3, 4), (-3, 7));
    assert_eq!(snap_pixel_angle((0, 0), (10, 1)), (10, 0));
    assert_eq!(snap_pixel_angle((0, 0), (9, 5)), (10, 5), "2:1");
    assert_eq!(snap_pixel_angle((0, 0), (-7, -6)), (-7, -7), "45°");
    assert_eq!(snap_pixel_angle((0, 0), (1, 9)), (0, 9));
}
