//! **A LEI DE COR dos botões próprios deste painel** — a metade que se pode afirmar sem pixels.
//!
//! ⚠️ Estes gates são sobre uma função PURA de propósito. O `VectorScene` deste repo não expõe o
//! que foi desenhado, então *«o botão pinta diferente sob o rato»* não é uma pergunta que um teste
//! consiga fazer à cena; o que ele consegue fazer é perguntar à **cor**, que é a única coisa que
//! muda. A outra metade — *o pintor CHAMA esta lei* — é indefensável aqui e vive no arch-gate
//! [`tests/the_audio_editor_asks_the_store_how_its_buttons_look.rs`], porque uma `button` que
//! ignorasse o `action_bg` deixaria todo gate deste ficheiro **verde**.

use super::*;

/// O par de um id que o relógio nunca viu: assente, em repouso.
const AT_REST: (ButtonState, f32) = (ButtonState::Normal, ph2d_editor_core::motion::SETTLED);

fn bg(v: (ButtonState, f32), enabled: bool) -> VelloColor {
    action_bg(
        ColorToken::Bg3,
        ColorToken::BgElev,
        ColorToken::AccentSoft,
        v,
        enabled,
        Theme::default(),
    )
}

fn token(t: ColorToken) -> VelloColor {
    let c = t.resolve(Theme::default());
    VelloColor::from_rgba8(c.r, c.g, c.b, c.a) // LITERAL-COLOR-OK: token-bridge do oráculo
}

/// **Em repouso o painel pinta BYTE A BYTE o que pintava antes desta wave.**
///
/// É a metade que torna a mudança segura: um id sem track publica [`motion::SETTLED`], o
/// `hover_axis` devolve `None` e o chamador cai no token duro. Sem esta afirmação a wave teria de
/// ser julgada só de olho.
#[test]
fn a_button_at_rest_is_the_surface_it_always_was() {
    assert_eq!(bg(AT_REST, true), token(ColorToken::Bg3));
}

/// **E sob o ponteiro ele deixa de ser essa cor** — o defeito que a wave fecha, num número.
///
/// *Mutação que deve sangrar:* `action_bg` devolver sempre o token de repouso.
#[test]
fn a_hovered_button_is_not_the_resting_surface() {
    let hot = bg(
        (ButtonState::Hovered, ph2d_editor_core::motion::SETTLED),
        true,
    );
    assert_ne!(
        hot,
        token(ColorToken::Bg3),
        "o botao continua inerte sob o rato"
    );
    assert_eq!(hot, token(ColorToken::BgElev));
}

/// **O eixo é uma QUANTIDADE, não um degrau:** a meio caminho a cor não é nenhum dos extremos.
///
/// ⚠️ Sem isto, um `action_bg` que ignorasse o `t` e escolhesse pelo ESTADO passaria nos dois
/// gates acima — e a saída do hover seria instantânea, que é a metade do movimento que ninguém vê
/// a faltar até a ver.
#[test]
fn half_way_is_neither_end() {
    let mid = bg((ButtonState::Hovered, 0.5), true);
    assert_ne!(mid, token(ColorToken::Bg3));
    assert_ne!(mid, token(ColorToken::BgElev));
}

/// **`Pressed` é um estado DURO** — meia-pressão não significa nada.
#[test]
fn pressed_is_a_hard_token() {
    assert_eq!(
        bg((ButtonState::Pressed, 0.5), true),
        token(ColorToken::AccentSoft)
    );
}

/// **Um botão DESACTIVADO não acende, mesmo com estado guardado quente.**
///
/// ⚠️ O caso é real e não hipotético: um botão desactivado não regista hit, então o ponteiro não o
/// alcança — mas o `state` no store pode ter ficado `Hovered` do quadro em que ele ainda estava
/// vivo, e sem a saída dura ele acenderia **sozinho** ao ser desactivado sob o cursor.
///
/// *Mutação que deve sangrar:* remover o `if !enabled` de [`action_bg`].
#[test]
fn a_disabled_button_stays_cold_even_with_a_stale_hot_state() {
    assert_eq!(
        bg(
            (ButtonState::Hovered, ph2d_editor_core::motion::SETTLED),
            false
        ),
        token(ColorToken::Bg3)
    );
}

/// **Um toggle ACESO sobe outro eixo, e a alternativa era pior que nada.**
///
/// A família engatada é a `Accent` do catálogo (`Accent → AccentHover`); usar o eixo do solto
/// (`Bg3 → BgElev`) faria o hover **escurecer** a peça mais clara da tela.
///
/// ⚠️ **Ele pergunta ao [`toggle_tokens`], não a tokens escritos aqui — e essa linha é o gate.** A
/// primeira versão passava os quatro tokens à mão a [`action_bg`], e a mutação que punha o toggle
/// aceso no eixo do solto **sobreviveu à suíte inteira**: um oráculo que declara a resposta que
/// julga não está a olhar para a escolha que o produto faz.
///
/// *Mutação que deve sangrar:* o braço `(true, true)` de [`toggle_tokens`] usar os tokens do solto.
#[test]
fn an_engaged_toggle_brightens_instead_of_dimming() {
    let (rest, hot_t, press, _fg) = toggle_tokens(true, true);
    let hot = action_bg(
        rest,
        hot_t,
        press,
        (ButtonState::Hovered, ph2d_editor_core::motion::SETTLED),
        true,
        Theme::default(),
    );
    assert_eq!(hot, token(ColorToken::AccentHover));
    assert_ne!(hot, token(ColorToken::BgElev));
    // E o repouso dele continua a ser o Accent que sempre foi.
    let cold = action_bg(rest, hot_t, press, AT_REST, true, Theme::default());
    assert_eq!(cold, token(ColorToken::Accent));
}

/// **Um toggle SOLTO é a família neutra**, e um DESACTIVADO não tem eixo nenhum.
///
/// *Mutação que deve sangrar:* o braço `(_, false)` de [`toggle_tokens`] dar um tom quente.
#[test]
fn a_loose_toggle_is_the_neutral_family_and_a_disabled_one_has_no_axis() {
    let (rest, hot, _, fg) = toggle_tokens(false, true);
    assert_eq!(
        (rest, hot, fg),
        (ColorToken::Bg3, ColorToken::BgElev, ColorToken::Text1)
    );
    let (rest, hot, press, fg) = toggle_tokens(true, false);
    assert_eq!(
        (rest, hot, press, fg),
        (
            ColorToken::Bg3,
            ColorToken::Bg3,
            ColorToken::Bg3,
            ColorToken::Text2
        ),
        "um toggle inerte mantem a superficie e perde so o contraste"
    );
}
