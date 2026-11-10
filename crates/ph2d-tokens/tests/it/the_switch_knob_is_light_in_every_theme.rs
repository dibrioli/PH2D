//! O disco do interruptor LIGADO é claro em todos os temas (macOS/iOS), e o hover clareia num tema
//! escuro (Material 3). Porquê: `ph2d_tokens::switch_knob` / `HOVER_LAYER_ALPHA`.

use ph2d_tokens::{ColorToken, HOVER_LAYER_ALPHA, Theme, switch_knob};

/// A luminância WCAG acima da qual uma cor lê como CLARA (≈ L* 76, um cinzento claro).
const CLARO: f64 = 0.5;

#[test]
fn the_switch_knob_is_light_in_every_theme() {
    for theme in Theme::ALL {
        let knob = switch_knob(theme);
        let l = knob.relative_luminance();
        assert!(
            l >= CLARO,
            "{theme:?}: o disco ligado lê escuro (luminância {l:.3})"
        );
        let track = ColorToken::Accent.resolve(theme);
        assert!(
            knob.relative_luminance() > track.relative_luminance(),
            "{theme:?}: o disco não se destaca do acento (é ele o mais claro)"
        );
    }
}

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
