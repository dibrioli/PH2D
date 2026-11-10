//! O hover clareia num tema escuro e escurece num claro (Material 3). Porquê:
//! `ph2d_tokens::HOVER_LAYER_ALPHA`.

use ph2d_tokens::{ColorToken, HOVER_LAYER_ALPHA, Theme};

#[test]
fn the_hover_layer_lightens_a_dark_surface_and_darkens_a_light_one() {
    let over = |theme: Theme| {
        let (s, t) = (
            ColorToken::BgElev.resolve(theme),
            ColorToken::Text1.resolve(theme),
        );
        let mix = |a: u8, b: u8| f32::from(a) + (f32::from(b) - f32::from(a)) * HOVER_LAYER_ALPHA;
        let lit = (mix(s.r, t.r) + mix(s.g, t.g) + mix(s.b, t.b)) / 3.0;
        (
            lit,
            (f32::from(s.r) + f32::from(s.g) + f32::from(s.b)) / 3.0,
        )
    };
    for theme in Theme::ALL {
        let (lit, base) = over(theme);
        if theme.is_dark() {
            assert!(
                lit > base + 1.0,
                "{theme:?}: o hover não clareia ({base} → {lit})"
            );
        } else {
            assert!(
                lit < base - 1.0,
                "{theme:?}: o hover não escurece ({base} → {lit})"
            );
        }
    }
}
