//! A linha de base que [`ph2d_text::cap_centre_px`] pressupõe é a que o layout dá, nas três fontes da
//! interface e nos tamanhos de texto do app: é nela que o texto de cartão se centra.

use ph2d_text::TextSystem;
use ph2d_tokens::{UiFont, UiTextStyle};

#[test]
fn the_cap_centre_sits_on_the_baseline_parley_lays_out() {
    let mut ts = TextSystem::without_system_fonts();
    for font in UiFont::ALL {
        ph2d_text::set_active_text_style(UiTextStyle {
            font,
            ..UiTextStyle::default()
        });
        for px in [10.0_f32, 11.0, 12.0, 13.0, 16.0] {
            let layout = ts.layout("Hx", px, f32::INFINITY);
            let base = layout.lines().next().expect("uma linha").metrics().baseline;
            let meio = ph2d_text::cap_centre_px(px);
            // A letra alta de qualquer das três fontes tem entre `0,66` e `0,74 em`.
            let (min, max) = (base - 0.37 * px, base - 0.33 * px);
            assert!(
                (min..=max).contains(&meio),
                "{font:?} a {px}: base {base}, meio {meio}"
            );
        }
    }
    ph2d_text::set_active_text_style(UiTextStyle::default());
}
