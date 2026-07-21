//! O gate do **FANTASMA do offset extremo** (report 2026-07-20: *"muda em tempo real para
//! round mas não muda para Miter e Bevel"*).
//!
//! Com caneta `2|d|` maior que o próprio laço, o contorno interno da banda degenerava e o
//! refugo atravessava o sweep: encolher além da morte da forma RESSUSCITAVA uma ilha
//! (Round/Bevel: 12 verts, área 2,52 a `d=−4`, onde o Miter corretamente devolvia nada), e
//! crescer no extremo inflava a área (30,7 onde a resposta exata é 19,8). No app, com o `d`
//! comitado no extremo, cada join devolvia um refugo DIFERENTE — uns cliques "mudavam" e
//! outros não, que é o report ao pé da letra. O fix é o `drop_phantoms` na porta única
//! `loop_region` (expand.rs).
//!
//! As DUAS metades, de propósito ([[feedback_absence_gate_needs_a_presence_sibling]]): a
//! ausência do fantasma E a presença do resultado legítimo — um filtro guloso demais
//! passaria na 1ª metade apagando offsets reais.

use ph2d_vec_boolean::{area, offset_path};
use ph2d_vec_scene::{Contour, LineJoin, OffsetSide, VecPath, VecVertex};

/// O donut do smoke 17/18 — retângulo 2,4×2,4 com furo quadrado 1,4 (EvenOdd), a forma em
/// que o report foi decodificado. A fixture TEM de conter o fenômeno
/// ([[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]]): o fantasma só
/// nasce quando `2|d|` engole o laço, e o furo pequeno é quem o produz primeiro.
fn donut() -> VecPath {
    let rect = [[2.8, -1.2], [5.2, -1.2], [5.2, 1.2], [2.8, 1.2]]
        .map(VecVertex::corner)
        .to_vec();
    let hole = [[3.3, -0.7], [4.7, -0.7], [4.7, 0.7], [3.3, 0.7]]
        .map(VecVertex::corner)
        .to_vec();
    let mut p = VecPath {
        verts: rect,
        closed: true,
        ..VecPath::default()
    };
    p.subpaths = vec![Contour::new_closed(hole)];
    p.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
    p
}

const JOINS: [LineJoin; 3] = [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];

/// Encolher além da morte da forma aniquila — em TODO join, e monotonicamente.
///
/// A forma tem meia-largura 1,2: a `|d| ≥ 2` nada pode sobrar. Antes do fix, Round e Bevel
/// devolviam uma ilha que CRESCIA conforme o artista encolhia mais (−2 vazio, −3 área 1,24,
/// −4 área 2,52) — não-monotônico, e diferente por join.
#[test]
fn a_shrink_past_the_shapes_death_annihilates_in_every_join() {
    let src = donut();
    for join in JOINS {
        for d in [-2.0, -3.0, -4.0] {
            let out = offset_path(&src, d, join, OffsetSide::Both);
            assert!(
                out.is_empty(),
                "shrink at d={d} with {join:?} must annihilate; got {} path(s), area {}",
                out.len(),
                out.iter().map(|p| area(p).abs()).sum::<f64>()
            );
        }
    }
}

/// Crescer no extremo preserva a identidade do CANCELAMENTO: a área que o arco/chanfro
/// perde nas quinas de FORA é ganha nas do FURO (as duas crescem com o mesmo `d`), então
/// Round e Bevel têm de dar a MESMA área que o Miter — que é a exata, `3,8 + 4d`.
///
/// Antes do fix, um FURO-fantasma nascia dentro do furo crescido e Round/Bevel reportavam
/// 30,7 contra os 19,8 do Miter a `d=+4`.
#[test]
fn a_grow_at_extreme_d_keeps_the_corner_cancel_identity() {
    let src = donut();
    for d in [2.0, 3.0, 4.0] {
        let exact = 3.8 + 4.0 * d;
        for join in JOINS {
            let out = offset_path(&src, d, join, OffsetSide::Both);
            let a: f64 = out.iter().map(|p| area(p).abs()).sum();
            assert!(
                (a - exact).abs() < 0.02,
                "grow at d=+{d} with {join:?}: area {a:.4}, expected {exact:.4} (cancel identity)"
            );
        }
    }
}

/// A metade de PRESENÇA: um offset legítimo sobrevive ao filtro, com a área exata.
///
/// Encolher 0,5 deixa a moldura fina (área 1,8 = 1,4² − 0,4²); crescer 0,5 dá 5,8. Um
/// filtro guloso (ou de limiar errado) passaria nos gates de ausência apagando isto.
#[test]
fn a_legitimate_offset_survives_the_phantom_filter() {
    let src = donut();
    for join in JOINS {
        for (d, exact) in [(-0.5, 1.8), (0.5, 5.8), (1.0, 7.8)] {
            let out = offset_path(&src, d, join, OffsetSide::Both);
            let a: f64 = out.iter().map(|p| area(p).abs()).sum();
            assert!(
                !out.is_empty() && (a - exact).abs() < 0.02,
                "offset at d={d} with {join:?}: area {a:.4}, expected {exact:.4}"
            );
        }
    }
}
