//! Gates do [`super`] — o `clear` da camada de sprites é DERIVADO do fundo do canvas.

use super::canvas_clear_rgb;
use ph2d_editor::screens::hero::canvas_backdrop;
use ph2d_tokens::overrides::{TokenValue, set_color_override};
use ph2d_tokens::{Color, ColorToken, Theme};

const THEMES: [Theme; 4] = [
    Theme::Forge,
    Theme::Workshop,
    Theme::Sunstone,
    Theme::Blueprint,
];

#[test]
fn the_clear_is_the_canvas_backdrop_byte_for_byte() {
    for theme in THEMES {
        let c = canvas_backdrop(theme);
        let want = (
            f64::from(c.r) / 255.0,
            f64::from(c.g) / 255.0,
            f64::from(c.b) / 255.0,
        );
        assert_eq!(canvas_clear_rgb(theme), want, "{theme:?}");
    }
}

/// ⭐⭐ **A cláusula do report: mudar a cor do canvas tem de mover o canvas.**
///
/// ⚠️ Antes de 2026-09-02 este gate era impossível de escrever — o valor era um literal, e autorar
/// o token movia o resto do app e deixava o canvas onde estava.
#[test]
fn re_dressing_the_canvas_token_moves_the_clear() {
    let theme = Theme::Forge;
    let before = canvas_clear_rgb(theme);
    set_color_override(
        theme,
        ColorToken::Bg1,
        Some(TokenValue::Literal(Color {
            r: 7,
            g: 90,
            b: 130,
            a: 255,
        })),
    )
    .expect("literal nunca fecha ciclo");
    let after = canvas_clear_rgb(theme);
    set_color_override(theme, ColorToken::Bg1, None).expect("restauro");

    assert_ne!(before, after, "o clear tem de seguir a cor autorada");
    assert_eq!(canvas_clear_rgb(theme), before, "e voltar ao de fabrica");
}

/// Controlo: se todos os temas dessem a mesma cor, o gate de cima mediria uma constante.
#[test]
fn a_light_theme_does_not_clear_to_the_dark_backdrop() {
    assert_ne!(
        canvas_clear_rgb(Theme::Forge),
        canvas_clear_rgb(Theme::Sunstone)
    );
}

/// ⛔⛔ **A CERCA DA M14.5, que nunca teve gate.**
///
/// As bordas anti-aliased do chrome estão calibradas contra o fundo legado
/// `(0,047, 0,047, 0,055)` — que é o `Bg1` do Forge dividido por 255. Pôr aqui o `Bg1`
/// *linearizado* (0,012) é a regressão dos *"pixelated borders"* da ronda 2, medida e revertida.
/// ⚠️ Este gate não defende o literal: defende a **distância** a ele. A derivação move o valor em
/// ≤ 3/255 (o literal era uma cópia arredondada), e qualquer troca de conversão salta muito mais.
#[test]
fn the_forge_clear_stays_where_the_chrome_anti_aliasing_was_calibrated() {
    const LEGACY: (f64, f64, f64) = (0.047, 0.047, 0.055);
    const TOLERANCE: f64 = 4.0 / 255.0;
    let (r, g, b) = canvas_clear_rgb(Theme::Forge);
    for (got, want, ch) in [(r, LEGACY.0, 'r'), (g, LEGACY.1, 'g'), (b, LEGACY.2, 'b')] {
        assert!(
            (got - want).abs() <= TOLERANCE,
            "canal {ch}: {got} saiu da calibracao das bordas do chrome ({want} +- {TOLERANCE})"
        );
    }
}
