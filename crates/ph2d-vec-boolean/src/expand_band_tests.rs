//! Testes de [`super`] (a banda por peças) — arquivo irmão.

use super::*;
use crate::expand::{Region, flat_lines};
use crate::{Closing, to_bez_with};
use kurbo::Shape;
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{ALL_SHAPES, ShapeKind, cook};

fn loops(band: &BezPath) -> Vec<BezPath> {
    let mut all = Vec::new();
    let mut sub = BezPath::new();
    for el in band.elements() {
        sub.push(*el);
        if *el == PathEl::ClosePath {
            all.push(std::mem::take(&mut sub));
        }
    }
    all
}

/// Toda peça sai com área positiva, nos DOIS sentidos do laço — é o que faz o `NonZero` ser a
/// união. Um laço invertido aqui reabriria o cancelamento que este módulo existe para fechar.
#[test]
fn every_piece_is_positive_whatever_the_loop_direction() {
    let tri = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(0.2, 0.3),
    ];
    for rev in [false, true] {
        let mut poly = tri.to_vec();
        if rev {
            poly.reverse();
        }
        let mut bp = BezPath::new();
        bp.move_to(poly[0]);
        for &p in &poly[1..] {
            bp.line_to(p);
        }
        bp.close_path();
        for join in [LineJoin::Bevel, LineJoin::Miter] {
            let band = pieces_with(&bp, 2.0, join, 4.0, true);
            let all = loops(&band);
            assert_eq!(all.len(), 9, "3 arestas × 2 meios retângulos + 3 junções");
            for l in &all {
                assert!(l.area() > 0.0, "peça negativa ({join:?}, rev={rev})");
            }
        }
    }
}

/// Formas do catálogo onde a banda se DOBRA (vales, setas, balões) e uma lisa — o filtro e as
/// corridas têm de acertar onde o stroker errava. O catálogo inteiro custa ~40 s em debug: ele
/// corre na sonda `#[ignore]` abaixo.
const DOBRAM: &[ShapeKind] = &[
    ShapeKind::Rectangle,
    ShapeKind::Star,
    ShapeKind::ArrowDouble,
    ShapeKind::SpeechRect,
    ShapeKind::Moon,
    ShapeKind::Cross,
    ShapeKind::Bolt,
    ShapeKind::Burst,
    ShapeKind::Heart,
];

/// A base de `k` (cozida na caixa 2×2) e a sua fronteira achatada.
fn base_of(k: ShapeKind) -> Option<(Region, BezPath)> {
    let sh = cook(k, [0.0, 0.0], [2.0, 2.0], &[]);
    if !sh.closed {
        return None;
    }
    let base = Region::of(
        &flat_lines(&to_bez_with(&sh, Closing::Always)),
        LsFillRule::NonZero,
    )?;
    let flat = flat_lines(&base.bez());
    Some((base, flat))
}

fn op_of(d: f64) -> BinaryOp {
    if d > 0.0 {
        BinaryOp::Union
    } else {
        BinaryOp::Difference
    }
}

/// Piso do controle do filtro: a primeira corrida cortou `7 171` peças (11/10).
const CUT_FLOOR: usize = 5_000;

/// ⭐ **O filtro de lado não muda a resposta**: `base ∪/∖ banda` com a banda filtrada tem a área (a
/// `1e-5` dela: o sweep corta as mesmas retas noutra ordem; o achatamento erra 100× isso)
/// da banda INTEIRA, em todo o catálogo, Bevel e Miter, crescendo e encolhendo — e o filtro de
/// facto corta peças (controle: senão o teste passaria com um filtro que não filtra nada).
#[test]
fn the_side_filter_never_changes_the_answer() {
    let mut cut = 0usize;
    for &k in DOBRAM {
        let Some((base, flat)) = base_of(k) else {
            continue;
        };
        for join in [LineJoin::Bevel, LineJoin::Miter] {
            for d in [0.05, 0.4, 1.5, -0.05, -0.3] {
                let area = |keep_all| {
                    let band = pieces_with(&flat, d, join, 4.0, keep_all);
                    let r = Region::of(&band, LsFillRule::NonZero)
                        .and_then(|b| base.combine(&b, op_of(d)))
                        .expect("sweep");
                    (r.bez().area().abs(), loops(&band).len())
                };
                let ((full, n_full), (filtered, n_cut)) = (area(true), area(false));
                cut += n_full - n_cut;
                let rel = (full - filtered).abs() / base.bez().area().abs();
                assert!(
                    rel < 1e-5,
                    "{k:?} {join:?} d={d}: banda inteira {full} × filtrada {filtered}"
                );
            }
        }
    }
    eprintln!("peças cortadas pelo filtro e pelas corridas: {cut}");
    assert!(
        cut > CUT_FLOOR,
        "o filtro cortou só {cut} peças — deixou de filtrar"
    );
}

/// **Sonda** (`--ignored --nocapture`, em release): o catálogo inteiro, a banda por peças contra o
/// stroker da kurbo — a diferença de área (onde o stroker cancelava) e o custo, intercalados no
/// mesmo processo. É a medição citada no doc do módulo.
#[test]
#[ignore = "sonda: catálogo inteiro, ~40 s em debug"]
fn measure_the_band_against_the_stroker_on_the_catalog() {
    use kurbo::{Cap, Join, Stroke};
    let mut tot = [0.0_f64; 2];
    for &k in ALL_SHAPES {
        let Some((base, flat)) = base_of(k) else {
            continue;
        };
        let (mut worst, mut t) = ((0.0_f64, String::new()), [0.0_f64; 2]);
        for (join, kj) in [
            (LineJoin::Bevel, Join::Bevel),
            (LineJoin::Miter, Join::Miter),
        ] {
            for d in [0.05, 0.2, 0.5, 1.0, 2.0, -0.05, -0.2, -0.5] {
                let pen = Stroke::new(2.0 * f64::abs(d))
                    .with_join(kj)
                    .with_caps(Cap::Butt);
                let mut a = [0.0_f64; 2];
                for (v, slot) in a.iter_mut().enumerate() {
                    let t0 = std::time::Instant::now();
                    let raw = if v == 0 {
                        flat_lines(&crate::expand::penned_outline(&base.bez(), &pen))
                    } else {
                        pieces(&flat, d, join, 4.0)
                    };
                    let r = Region::of(&raw, LsFillRule::NonZero)
                        .and_then(|b| base.combine(&b, op_of(d)));
                    t[v] += t0.elapsed().as_secs_f64() * 1e3;
                    *slot = r.map_or(-1.0, |r| r.bez().area().abs());
                }
                let rel = (a[0] - a[1]).abs() / a[1].abs().max(base.bez().area().abs());
                if rel > worst.0 {
                    worst = (rel, format!("{join:?} d={d}"));
                }
            }
        }
        tot[0] += t[0];
        tot[1] += t[1];
        eprintln!(
            "{k:?}: stroker erra {:.3} % ({}) · stroker {:.1} ms · peças {:.1} ms",
            worst.0 * 100.0,
            worst.1,
            t[0],
            t[1]
        );
    }
    eprintln!("TOTAL stroker {:.0} ms · peças {:.0} ms", tot[0], tot[1]);
}
