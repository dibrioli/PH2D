//! Os gates da **Fase A** do ADR-0128 (o Blend Object vivo): a cadeia multi-forma e a cor OKLab.
//!
//! Arquivo irmão do [`super::tests`] pelo teto de LOC (600).

use super::*;
use ph2d_vec_scene::{ShapeKind, cook};

fn square(c: [f64; 2], r: f64) -> VecPath {
    cook(
        ShapeKind::Rectangle,
        [c[0] - r, c[1] - r],
        [c[0] + r, c[1] + r],
        &[],
    )
}

/// O centroide amostrado por arco (o "onde a forma está").
fn centroid(p: &VecPath) -> [f64; 2] {
    let o = Outline::of(p).unwrap();
    let (mut x, mut y) = (0.0, 0.0);
    for k in 0..256 {
        let pt = o.at(f64::from(k) / 256.0);
        x += pt.x;
        y += pt.y;
    }
    [x / 256.0, y / 256.0]
}

/// **A CADEIA multi-forma** (ADR-0128): N fontes, ligadas pairwise, só os intermediários.
///
/// Três quadrados a x=0, 6, 12 com 2 passos cada gap ⇒ 2 gaps × 2 = 4 passos. Os do 1º gap
/// caminham entre 0 e 6 (centros 2, 4); os do 2º, entre 6 e 12 (centros 8, 10). As FONTES não
/// entram (elas se desenham sozinhas).
#[test]
fn a_chain_blends_the_shapes_pairwise_in_order() {
    let shapes = [
        square([0.0, 0.0], 1.0),
        square([6.0, 0.0], 1.0),
        square([12.0, 0.0], 1.0),
    ];
    let out = chain(&shapes, 2);
    assert_eq!(
        out.len(),
        4,
        "2 gaps × 2 passos = 4 intermediários (as fontes não entram)"
    );
    let xs: Vec<f64> = out.iter().map(|p| centroid(p)[0]).collect();
    for (got, want) in xs.iter().zip([2.0, 4.0, 8.0, 10.0]) {
        assert!(
            (got - want).abs() < 1e-6,
            "os centros da cadeia deviam ser [2,4,8,10], são {xs:?}"
        );
    }
}

/// **Um elo degenerado não quebra a cadeia** — o par com uma forma inválida é pulado, os outros
/// saem.
#[test]
fn a_degenerate_link_is_skipped_not_fatal() {
    let shapes = [
        square([0.0, 0.0], 1.0),
        VecPath::default(), // inválida (menos de 2 vértices)
        square([12.0, 0.0], 1.0),
    ];
    // gap 0→1 e 1→2 degeneram (a do meio é inválida); a cadeia sai VAZIA, sem panicar.
    assert!(chain(&shapes, 3).is_empty());

    // Com a do meio válida, os dois gaps saem.
    let ok = [
        square([0.0, 0.0], 1.0),
        square([6.0, 0.0], 1.0),
        VecPath::default(),
    ];
    assert_eq!(chain(&ok, 2).len(), 2, "só o 1º gap é válido");
}

/// **OKLab NÃO passa por cinza lamacento** — a vantagem sobre o Blend do Illustrator.
///
/// O Illustrator interpola em device-space, e o meio de dois matizes opostos (azul ↔ amarelo) cai
/// num **cinza morto** (~128,128,128). Em OKLab o caminho tem croma o tempo todo. O gate mede a
/// saturação do meio: ela não pode desabar.
#[test]
fn opposite_hues_do_not_pass_through_muddy_grey() {
    let mut a = square([0.0, 0.0], 1.0);
    let mut b = square([6.0, 0.0], 1.0);
    a.fill = Some(Paint::solid(Rgba8::new(0, 0, 255, 255))); // azul
    b.fill = Some(Paint::solid(Rgba8::new(255, 255, 0, 255))); // amarelo

    let mid = match morph(&a, &b, 0.5).unwrap().fill {
        Some(Paint::Solid(c)) => c,
        ref other => panic!("sem sólido: {other:?}"),
    };
    // Saturação grosseira: (max − min) dos canais. Cinza morto ≈ 0; um tom com croma é alto.
    let (hi, lo) = (mid.r.max(mid.g).max(mid.b), mid.r.min(mid.g).min(mid.b));
    let chroma = i32::from(hi) - i32::from(lo);
    assert!(
        chroma > 60,
        "o meio de azul↔amarelo tem croma {chroma} (r={},g={},b={}) — desabou para cinza, como o \
         device-space do Illustrator; o OKLab devia manter a cor viva",
        mid.r,
        mid.g,
        mid.b
    );
}
