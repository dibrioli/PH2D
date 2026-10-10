//! Ciclo 18 (doc 127): a COR do crossfade num espaço perceptual — a mistura da casa, só na `tint`.

use super::*;
use ph2d_color::perceptual::{MixSpace, mix};

fn lado(p: [f32; 2], c: [f32; 4]) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![p]))
        .with("tint", Column::Vec4(vec![c]))
}

#[test]
fn the_tint_crossfades_in_the_chosen_space_and_the_rest_does_not_move() {
    let (ca, cb) = ([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]);
    let (a, b) = (lado([0.0, 0.0], ca), lado([2.0, 4.0], cb));
    for esp in [
        MixSpace::Oklab,
        MixSpace::OklchShorter,
        MixSpace::OklchLonger,
    ] {
        let out = morph_stream_no_espaco(&a, &b, &[0.3], esp);
        let Some(Column::Vec4(t)) = out.get("tint") else {
            panic!("tint")
        };
        assert_eq!(t[0], mix(ca, cb, 0.3, esp), "{esp:?}");
        let Some(Column::Vec2(p)) = out.get("P") else {
            panic!("P")
        };
        assert_eq!(p[0], [0.6, 1.2], "a posicao continua linear");
    }
    // `Linear` é o morph de sempre, ao bit.
    assert_eq!(
        morph_stream_no_espaco(&a, &b, &[0.3], MixSpace::Linear).get("tint"),
        morph_stream(&a, &b, &[0.3]).get("tint")
    );
}
