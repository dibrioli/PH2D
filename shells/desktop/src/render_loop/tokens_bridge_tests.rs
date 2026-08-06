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
use ph2d_tokens::overrides::{
    TokenValue, clear_color_overrides, color_overrides, set_color_override,
};
use ph2d_tokens::{ColorToken, Theme};

use super::dispatch;

/// Uma fila de toasts descartável — estes gates medem a CAMADA, e a recusa de laço tem gate
/// próprio (`a_loop_is_refused_at_the_door_and_writes_nothing`, na `ph2d-tokens`).
fn toasts() -> ph2d_editor::ToastQueue {
    ph2d_editor::ToastQueue::default()
}

/// Escrever um LITERAL — o `expect` documenta a propriedade: um literal TERMINA uma cadeia de
/// aliases, então a porta nunca o recusa.
fn put(theme: Theme, token: ColorToken, colour: Option<Color>) {
    set_color_override(theme, token, colour.map(TokenValue::Literal))
        .expect("um literal nunca fecha um laco");
}

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
    assert!(
        dispatch(&mut hero, &mut toasts()),
        "a ponte nao reportou a mudanca"
    );
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
    assert!(
        dispatch(&mut hero, &mut toasts()),
        "o 1o frame tinha de escrever"
    );
    assert!(
        !dispatch(&mut hero, &mut toasts()),
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
    put(here, ColorToken::ALL[0], Some(c));
    put(other, ColorToken::ALL[1], Some(c));

    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::ResetAll);
    assert!(dispatch(&mut hero, &mut toasts()));

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
    put(theme, ColorToken::ALL[0], Some(c));
    put(theme, ColorToken::ALL[3], Some(c));

    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::Reset(3));
    assert!(dispatch(&mut hero, &mut toasts()));

    let left = color_overrides();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].token, ColorToken::ALL[0]);
    clear_color_overrides();
}

/// **O elo do painel chega à camada** — e o token passa a valer o que o alvo vale.
#[test]
fn the_link_intent_reaches_the_layer() {
    clear_color_overrides();
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let theme = hero.theme;
    let (a, b) = (0usize, 3usize);
    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::Link {
        from: a,
        to: b,
    });
    assert!(
        dispatch(&mut hero, &mut toasts()),
        "a ponte nao reportou o elo"
    );
    assert_eq!(
        ColorToken::ALL[a].resolve(theme),
        ColorToken::ALL[b].resolve(theme),
        "o elo nao chegou ao mundo"
    );
    clear_color_overrides();
}

/// **A recusa de um laço FALA** — e não escreve.
///
/// ⚠️ Este é o gate que separa *"a porta recusa"* (que a `ph2d-tokens` já prova) de *"o artista
/// fica a saber"*. Um gesto que não acontece sem nada na tela é indistinguível de um botão
/// quebrado, e é por isso que o oráculo é a FILA DE TOASTS, não o estado da camada.
#[test]
fn a_refused_loop_says_so_and_writes_nothing() {
    clear_color_overrides();
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let mut q = toasts();
    // a -> b passa.
    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::Link {
        from: 0,
        to: 3,
    });
    assert!(dispatch(&mut hero, &mut q));
    assert_eq!(q.len(), 0, "um elo legitimo nao pode reclamar");
    let before = color_overrides();

    // b -> a fecha o laço.
    ph2d_panel_tokens::state::push_intent_for_tests(ph2d_panel_tokens::TokensIntent::Link {
        from: 3,
        to: 0,
    });
    assert!(
        !dispatch(&mut hero, &mut q),
        "a recusa nao pode marcar o projeto como sujo"
    );
    assert_eq!(q.len(), 1, "a recusa foi SILENCIOSA");
    assert_eq!(color_overrides(), before, "a recusa escreveu na tabela");
    clear_color_overrides();
}

/// **Escolher uma cor numa linha que SEGUE outra quebra o elo** — mesmo com a cor a coincidir.
///
/// ⚠️ Sem esta metade o guard *"só escreve quando muda"* apanharia o caso: a linha já MOSTRA a cor
/// do alvo, então picar exactamente essa cor seria um clique sem efeito, com o elo intacto e nada
/// na tela a explicar por quê.
#[test]
fn picking_a_colour_on_a_linked_row_breaks_the_link() {
    clear_color_overrides();
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let theme = hero.theme;
    let (a, b) = (0usize, 3usize);
    set_color_override(
        theme,
        ColorToken::ALL[a],
        Some(TokenValue::Alias(ColorToken::ALL[b])),
    )
    .expect("a fixture nao fecha laco");
    // Pica EXACTAMENTE a cor que o elo já mostrava.
    let shown = ColorToken::ALL[a].resolve(theme);
    let mut hero2 = hero_with_picker_on(a, [shown.r, shown.g, shown.b, shown.a]);
    hero2.theme = theme;
    assert!(
        dispatch(&mut hero2, &mut toasts()),
        "escolher no picker sobre uma linha ligada tem de QUEBRAR o elo"
    );
    assert!(
        matches!(
            ph2d_tokens::overrides::color_override(theme, ColorToken::ALL[a]),
            Some(TokenValue::Literal(_))
        ),
        "a linha continua a SEGUIR outra depois de o artista escolher uma cor"
    );
    let _ = &mut hero;
    clear_color_overrides();
}
