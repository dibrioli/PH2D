//! Ciclo 18 (doc 127): a COR do Blend e do Avg num espaço perceptual.

use super::*;
use ph2d_color::perceptual::{self as pc, MixSpace};

#[test]
fn blend_and_avg_mix_the_tint_perceptually() {
    let (ca, cb) = ([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 0.5]);
    let cols = [Column::Vec4(vec![ca]), Column::Vec4(vec![cb])];
    let Some(Column::Vec4(b)) = cor_perceptual(
        MODE_BLEND,
        &cols,
        &[0.25],
        &[1.0, 1.0],
        2.0,
        1,
        MixSpace::OklchLonger,
    ) else {
        panic!("blend")
    };
    assert_eq!(b[0], pc::mix(ca, cb, 0.25, MixSpace::OklchLonger));
    // O Avg com pesos 1 e 3: a média ponderada em OKLab é a mistura OKLab a `t = 3/4`.
    let Some(Column::Vec4(m)) =
        cor_perceptual(MODE_AVG, &cols, &[], &[1.0, 3.0], 4.0, 1, MixSpace::Oklab)
    else {
        panic!("avg")
    };
    let e = pc::mix(ca, cb, 0.75, MixSpace::Oklab);
    for k in 0..4 {
        assert!(
            (m[0][k] - e[k]).abs() < 1e-5,
            "avg {k}: {} × {}",
            m[0][k],
            e[k]
        );
    }
    // Os modos que não misturam (Add…) não são deste caminho.
    assert!(cor_perceptual(MODE_ADD, &cols, &[], &[1.0, 1.0], 2.0, 1, MixSpace::Oklab).is_none());
}
