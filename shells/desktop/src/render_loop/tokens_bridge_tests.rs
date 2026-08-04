//! Gates da **ponte de tokens** (plano UI/UX W6, degrau 1).
//!
//! ⚠️ Estes nasceram de DUAS mutações sobreviventes: com os quatro arch-gates verdes, o *Reset
//! This Mode* podia apagar os quatro modos e o read-back do picker podia escrever a cada frame —
//! nenhum gate via nem um nem outro, porque a ponte vive no laço de frame.
//!
//! E ela é dirigível **headless**: `HeroScreen::new` é construtor puro (o próprio doc dele o diz),
//! e o estado do picker é semeável no store. Um arch-gate sobre o fonte teria sido a resposta
//! preguiçosa a *"isto exige janela"* — e a janela não é exigida.

use ph2d_editor::interaction::InteractiveState;
use ph2d_editor::screens::hero::HeroScreen;
use ph2d_editor::widget::{ChannelMode, Harmony, InterpolationMode};
use ph2d_tokens::color::{Color, ColorValue};
use ph2d_tokens::overrides::{clear_color_overrides, color_overrides, set_color_override};
use ph2d_tokens::{ColorToken, Theme};

use super::dispatch;

/// Uma tela com o picker aberto sobre a swatch da linha `row`, com a cor `rgba` escolhida.
fn hero_with_picker_on(row: usize, rgba: [u8; 4]) -> HeroScreen {
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    hero.store.register(
        ph2d_editor::ids::INSP_BLENDER_PICKER,
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
            hsv_h: 0.0,
            hsv_s: 0.0,
            harmony: Harmony::None,
        },
    );
    hero.store
        .set_picker_target(Some(ph2d_editor::ids::tokens_swatch_id(row)));
    hero
}

/// **A cor escolhida no picker chega à camada** — a metade que faz o app re-vestir.
#[test]
fn the_picked_colour_reaches_the_layer() {
    clear_color_overrides();
    let mine = [0x00, 0xFF, 0x00, 0xFF];
    let mut hero = hero_with_picker_on(0, mine);
    assert!(dispatch(&mut hero), "a ponte nao reportou a mudanca");
    assert_eq!(
        ColorToken::ALL[0].resolve(hero.theme),
        Color {
            r: 0,
            g: 255,
            b: 0,
            a: 255
        }
    );
    clear_color_overrides();
}

/// **A ponte só escreve quando o valor MUDA.**
///
/// ⚠️ O picker publica o valor a cada frame em que está aberto; escrever sempre marcaria o projeto
/// como sujo por o artista estar a OLHAR para a cor. O oráculo é o retorno da ponte no 2º frame —
/// `false` quer dizer *"nada a gravar"*.
#[test]
fn the_bridge_writes_only_when_the_value_moves() {
    clear_color_overrides();
    let mut hero = hero_with_picker_on(0, [0x00, 0xFF, 0x00, 0xFF]);
    assert!(dispatch(&mut hero), "o 1o frame tinha de escrever");
    assert!(
        !dispatch(&mut hero),
        "a ponte re-escreveu a MESMA cor — todo frame com o picker aberto sujaria o projeto"
    );
    clear_color_overrides();
}

/// **O *Reset This Mode* apaga SÓ o modo vigente.**
///
/// ⚠️ O artista vê um modo de cada vez; limpar os quatro levaria uma re-vestida que ele não está a
/// olhar, e nada na tela diria que ela se foi.
#[test]
fn resetting_the_mode_leaves_the_other_modes_alone() {
    clear_color_overrides();
    let c = Color {
        r: 1,
        g: 2,
        b: 3,
        a: 255,
    };
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let here = hero.theme;
    let other = if here == Theme::Sunstone {
        Theme::Forge
    } else {
        Theme::Sunstone
    };
    set_color_override(here, ColorToken::ALL[0], Some(c));
    set_color_override(other, ColorToken::ALL[1], Some(c));

    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::ResetAll);
    assert!(dispatch(&mut hero));

    let left = color_overrides();
    assert_eq!(left.len(), 1, "o Reset levou o que nao era dele: {left:?}");
    assert_eq!(left[0].theme, other);
    clear_color_overrides();
}

/// **O Reset de uma linha solta O token daquela linha** — e nenhum outro.
#[test]
fn resetting_a_row_releases_that_token_only() {
    clear_color_overrides();
    let c = Color {
        r: 1,
        g: 2,
        b: 3,
        a: 255,
    };
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let theme = hero.theme;
    set_color_override(theme, ColorToken::ALL[0], Some(c));
    set_color_override(theme, ColorToken::ALL[3], Some(c));

    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::Reset(3));
    assert!(dispatch(&mut hero));

    let left = color_overrides();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].token, ColorToken::ALL[0]);
    clear_color_overrides();
}
