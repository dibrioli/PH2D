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

// ═══════════════════════════════════════════════════════════════════════════
// A família NUMÉRICA (plano UI/UX W4c.1) — os mesmos gates, na outra grandeza.
// ═══════════════════════════════════════════════════════════════════════════

use ph2d_panel_tokens::TokensIntent;
use ph2d_panel_tokens::state::push_intent_for_tests as push;
use ph2d_tokens::NumToken;
use ph2d_tokens::num_overrides::{
    NumValue, clear_num_overrides, num_override, num_overrides, set_num_override,
};

fn fresh_both() {
    clear_color_overrides();
    clear_num_overrides();
}

/// Uma tela sem picker nenhum — os intents numéricos não passam por ele.
fn plain_hero() -> HeroScreen {
    HeroScreen::new(ph2d_editor::NodeId(1))
}

/// **Um número digitado chega à camada.**
#[test]
fn a_num_set_reaches_the_layer() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    push(TokensIntent::NumSet { row: 0, px: 13.0 });
    assert!(dispatch(&mut hero, &mut toasts()));
    assert_eq!(NumToken::ALL[0].px(theme), 13.0);
    fresh_both();
}

/// ⚠️ **Só escreve quando MUDA**, e o oráculo é o valor EFETIVO — a mesma lei do read-back do
/// picker. Sem ela, um chip que espelha o efetivo marcaria o projeto sujo por o artista lhe tocar.
#[test]
fn a_num_set_that_matches_the_effective_value_writes_nothing() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    let token = NumToken::ALL[0];
    push(TokensIntent::NumSet {
        row: 0,
        px: token.px(theme),
    });
    assert!(
        !dispatch(&mut hero, &mut toasts()),
        "escrever o valor que ja' estava marcou o projeto como sujo"
    );
    assert_eq!(
        num_override(theme, token),
        None,
        "o slot foi AUTORADO com o valor de fabrica — soltar e autorar-o-mesmo nao sao a mesma coisa"
    );
    fresh_both();
}

/// **Digitar um número numa linha que SEGUE outra QUEBRA o elo** — mesmo com o número a coincidir.
#[test]
fn typing_a_number_on_a_linked_row_breaks_the_link() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    let (a, b) = (0usize, 3usize);
    set_num_override(
        theme,
        NumToken::ALL[a],
        Some(NumValue::Alias(NumToken::ALL[b])),
    )
    .expect("a fixture nao fecha laco");
    // Digita EXACTAMENTE o número que o elo já mostrava.
    let shown = NumToken::ALL[a].px(theme);
    push(TokensIntent::NumSet { row: a, px: shown });
    assert!(
        dispatch(&mut hero, &mut toasts()),
        "digitar sobre uma linha ligada tem de QUEBRAR o elo"
    );
    assert!(
        matches!(
            num_override(theme, NumToken::ALL[a]),
            Some(NumValue::Literal(_))
        ),
        "a linha continua a SEGUIR outra depois de o artista escrever um numero"
    );
    fresh_both();
}

/// **Um número que não é um comprimento vira TOAST e não escreve nada.**
#[test]
fn a_refused_number_is_said_out_loud_and_writes_nothing() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    let before = num_overrides();
    let mut q = toasts();
    push(TokensIntent::NumSet { row: 0, px: -5.0 });
    assert!(
        !dispatch(&mut hero, &mut q),
        "a recusa nao pode marcar o projeto como sujo"
    );
    assert_eq!(q.len(), 1, "a recusa foi SILENCIOSA");
    assert_eq!(num_overrides(), before, "a recusa escreveu na tabela");
    assert_eq!(NumToken::ALL[0].px(theme), NumToken::ALL[0].factory_px());
    fresh_both();
}

/// **O elo numérico chega à camada, e um laço vira toast.**
#[test]
fn a_num_link_reaches_the_layer_and_a_loop_is_said_out_loud() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    push(TokensIntent::NumLink { from: 0, to: 3 });
    assert!(dispatch(&mut hero, &mut toasts()));
    assert!(matches!(
        num_override(theme, NumToken::ALL[0]),
        Some(NumValue::Alias(_))
    ));
    // Fechar 3 -> 0 fecha o laço.
    let mut q = toasts();
    push(TokensIntent::NumLink { from: 3, to: 0 });
    assert!(!dispatch(&mut hero, &mut q), "o laco foi ACEITE");
    assert_eq!(q.len(), 1, "a recusa foi SILENCIOSA");
    fresh_both();
}

/// **O Reset numérico solta o slot** (nunca congela o valor de fábrica nele).
#[test]
fn a_num_reset_releases_the_slot() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    set_num_override(theme, NumToken::ALL[0], Some(NumValue::Literal(13.0))).unwrap();
    push(TokensIntent::NumReset(0));
    assert!(dispatch(&mut hero, &mut toasts()));
    assert_eq!(num_override(theme, NumToken::ALL[0]), None);
    fresh_both();
}

/// ⚠️ **O *Reset This Mode* limpa as DUAS famílias — e só ESTE modo.**
///
/// As duas metades falham por defeitos opostos: esquecer a família numérica deixa a escala de pé
/// depois de um reset que se anuncia total (a metade que ninguém procura), e limpar os quatro modos
/// leva trabalho que o artista não está a olhar.
#[test]
fn reset_all_clears_both_families_of_this_mode_only() {
    fresh_both();
    let mut hero = plain_hero();
    let theme = hero.theme;
    let other = if theme == Theme::Forge {
        Theme::Workshop
    } else {
        Theme::Forge
    };
    put(theme, ColorToken::ALL[0], Some(Color::from_hex(0x00FF00)));
    put(other, ColorToken::ALL[0], Some(Color::from_hex(0x00FF00)));
    set_num_override(theme, NumToken::ALL[0], Some(NumValue::Literal(13.0))).unwrap();
    set_num_override(other, NumToken::ALL[0], Some(NumValue::Literal(13.0))).unwrap();

    push(TokensIntent::ResetAll);
    assert!(dispatch(&mut hero, &mut toasts()));

    assert_eq!(
        num_override(theme, NumToken::ALL[0]),
        None,
        "o Reset This Mode deixou a ESCALA de pe' — a metade que ninguem procura"
    );
    assert_eq!(
        ph2d_tokens::overrides::color_override(theme, ColorToken::ALL[0]),
        None,
        "o Reset This Mode deixou a COR de pe'"
    );
    assert!(
        num_override(other, NumToken::ALL[0]).is_some()
            && ph2d_tokens::overrides::color_override(other, ColorToken::ALL[0]).is_some(),
        "o Reset This Mode apagou trabalho de OUTRO modo"
    );
    fresh_both();
}
