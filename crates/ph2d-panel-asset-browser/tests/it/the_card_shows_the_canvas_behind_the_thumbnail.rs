//! ⭐⭐ **O FUNDO DE UM CARTÃO É O FUNDO DO CANVAS** — report do dono do produto (2026-09-02):
//! *«seria interessante que o fundo do ícone do asset seja da mesma cor do fundo do canvas mesmo
//! quando se muda a cor do canvas»*.
//!
//! ⚠️ **A cláusula que manda é a segunda.** Um cartão pintado com uma cópia da cor do canvas
//! passaria o primeiro pedido e falharia o segundo em silêncio — e é exactamente o que existia:
//! três sítios respondiam «de que cor é o fundo do canvas?» por conta própria. Estes gates medem a
//! PORTA, não o valor: com o token re-vestido, o cartão tem de andar junto.

use ph2d_panel_asset_browser::card_backdrop::card_backdrop;
use ph2d_tokens::overrides::{TokenValue, set_color_override};
use ph2d_tokens::{Color, ColorToken, Theme};

const THEMES: [Theme; 4] = [
    Theme::Forge,
    Theme::Workshop,
    Theme::Sunstone,
    Theme::Blueprint,
];

/// Uma cor de asset que não se parece com fundo nenhum — se ela aparecer onde não devia, vê-se.
const SWATCH: [u8; 4] = [200, 40, 160, 255];

fn canvas_of(theme: Theme) -> Color {
    ph2d_editor_core::screens::hero::canvas_backdrop(theme)
}

#[test]
fn a_card_with_a_thumbnail_sits_on_the_canvas_colour() {
    for theme in THEMES {
        assert_eq!(
            card_backdrop(theme, SWATCH, true),
            canvas_of(theme),
            "{theme:?}: o fundo do cartao tem de ser o fundo do canvas"
        );
    }
}

/// ⛔ A metade que impede a cura de apagar a wave A2: sem miniatura, a cor dominante É o cartão.
#[test]
fn a_card_without_a_thumbnail_still_shows_the_assets_own_colour() {
    for theme in THEMES {
        let c = card_backdrop(theme, SWATCH, false);
        assert_eq!(
            [c.r, c.g, c.b, c.a],
            SWATCH,
            "{theme:?}: sem miniatura o cartao mostra a cor do proprio asset"
        );
    }
}

/// ⭐⭐ **A cláusula «mesmo quando se muda a cor do canvas».**
///
/// ⚠️ Sem este gate, um cartão que lesse uma CÓPIA do valor passaria os dois de cima.
#[test]
fn re_dressing_the_canvas_token_moves_the_card_with_it() {
    let theme = Theme::Forge;
    let before = card_backdrop(theme, SWATCH, true);
    let painted = Color {
        r: 7,
        g: 90,
        b: 130,
        a: 255,
    };
    assert_ne!(before, painted, "a fixtura tem de mudar alguma coisa");

    set_color_override(theme, ColorToken::Bg1, Some(TokenValue::Literal(painted)))
        .expect("literal nunca fecha ciclo");
    let after = card_backdrop(theme, SWATCH, true);
    set_color_override(theme, ColorToken::Bg1, None).expect("restauro");

    assert_eq!(
        after, painted,
        "o cartao tem de seguir a cor autorada do canvas"
    );
    assert_eq!(
        card_backdrop(theme, SWATCH, true),
        before,
        "e voltar ao de fabrica quando o override sai"
    );
}

/// O controlo da fixtura: se os quatro temas dessem a mesma cor, os gates de cima passariam sobre
/// uma constante e não sobre uma leitura.
#[test]
fn the_four_themes_do_not_share_one_backdrop() {
    let dark = canvas_of(Theme::Forge);
    let light = canvas_of(Theme::Sunstone);
    assert_ne!(
        dark, light,
        "um tema claro e um escuro nao podem ter o mesmo fundo de canvas"
    );
}
