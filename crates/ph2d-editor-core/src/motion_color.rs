//! **O que se FAZ com o escalar do substrato: a cor.**
//!
//! Irmão do [`super::motion`] e não continuação dele. O pai integra um número ao longo do tempo
//! (a mola, os papéis, o relógio); este ficheiro responde à pergunta seguinte — *dado esse
//! número, que TINTA sai?* — e é a fronteira onde o movimento encontra o design system.
//!
//! O corte nasceu no tecto de 700 LOC e a linha escolheu-se sozinha: nada aqui sabe o que é uma
//! track, e nada no pai sabe o que é um `ColorToken`.

use super::SETTLED;

/// **A PORTA ÚNICA da mistura de cor de token.** Um `t` fora de `[0,1]` é clampado aqui, e não em
/// cada chamador — a segunda cópia de um clamp é a que alguém esquece.
///
/// ⚠️ Mistura em **sRGB directo**, de propósito: estas são duas tintas de UI vizinhas na mesma
/// família de token (repouso → hover), e uma travessia OKLab entre elas custaria a dependência do
/// espaço de cor num caminho que corre por widget, por quadro, para uma diferença que ninguém
/// distingue em dois tons adjacentes. *Se um dia a mistura for entre tons distantes, esta é a
/// linha que muda — e é uma linha só.*
#[must_use]
pub fn blend_token_color(
    rest: Option<ph2d_tokens::Color>,
    hot: Option<ph2d_tokens::Color>,
    t: f32,
) -> Option<ph2d_tokens::Color> {
    let t = t.clamp(0.0, 1.0);
    match (rest, hot) {
        (None, None) => None,
        // ⚠️ Um lado ausente é **transparente**, não "a outra cor": um botão Default em repouso
        // não tem fundo, e o hover dele tem de EMERGIR do nada em vez de aparecer de repente.
        (Some(a), None) => Some(fade(a, 1.0 - t)),
        (None, Some(b)) => Some(fade(b, t)),
        (Some(a), Some(b)) => Some(ph2d_tokens::Color {
            r: mix(a.r, b.r, t),
            g: mix(a.g, b.g, t),
            b: mix(a.b, b.b, t),
            a: mix(a.a, b.a, t),
        }),
    }
}

/// **O eixo do hover com a sua GUARDA** — a mistura `repouso → hover`, ou `None` quando este
/// estado não é uma quantidade.
///
/// ⚠️ **Uma pergunta, um sítio.** Três widgets já a fazem (`Button::bg_color`, o tint do
/// `IconButton`, a caixa do `Checkbox`) e a lei tem DUAS metades que é preciso não separar: só
/// estados **macios** entram no eixo (`Pressed`/`Focused`/`Disabled` não são uma *fracção* de
/// nada — meia-desactivação não significa coisa alguma), e o neutro [`SETTLED`] sai por aqui como
/// `None`, para o chamador cair no **token duro** e pintar byte a byte o mundo pré-substrato.
///
/// ⚠️ **Sem degrau na fronteira:** um id genuinamente assente no hover publica `1.0` e sai pelo
/// token duro `hot`, que é exactamente `lerp(rest, hot, 1)`.
#[must_use]
pub fn hover_axis(
    soft: bool,
    t: f32,
    rest: Option<ph2d_tokens::Color>,
    hot: Option<ph2d_tokens::Color>,
) -> Option<ph2d_tokens::Color> {
    if !soft || t >= SETTLED {
        return None;
    }
    blend_token_color(rest, hot, t)
}

/// **O tom QUENTE da família de um tom de repouso** — `Accent → AccentHover`, `Danger →
/// DangerSoft`, `Warn → WarnSoft`, e qualquer superfície → `BgElev`.
///
/// ⚠️ **Ela existe porque há chamadores cujo tom de repouso é um PARÂMETRO.** O `paint_toggle` do
/// Audio Mixer recebe o `active_bg` de quem o chama (`Danger` no Mute, `Warn` no Solo, `Accent`
/// nas master-fx), então ele não pode declarar o par quente: uma tabela escrita no chamador
/// cresceria com cada chamador novo, e o quarto nasceria sem ela. Quem sabe o quente é a FAMÍLIA.
///
/// ⚠️ **Ela NÃO subsume o `Button::bg_token`, e o motivo é geométrico:** aquele mapeia
/// `(kind, state)` e os kinds *ghost* (`Default`/`IconOnly`) têm repouso **`None`** — não há tom
/// de que derivar. As duas leis coexistem e o gate
/// `the_family_hover_map_agrees_with_the_button_kinds` prova que **concordam onde ambas
/// respondem**; quem um dia as unificar faz o `bg_token` perguntar aqui.
///
/// O fallback é `BgElev` de propósito: é o que o catálogo dá a toda superfície neutra, e um tom
/// desconhecido é uma superfície até prova em contrário.
#[must_use]
pub fn hover_of(rest: ph2d_tokens::ColorToken) -> ph2d_tokens::ColorToken {
    use ph2d_tokens::ColorToken as T;
    match rest {
        T::Accent => T::AccentHover,
        T::Danger => T::DangerSoft,
        T::Warn => T::WarnSoft,
        _ => T::BgElev,
    }
}

/// **O tom da família sob PRESSÃO** — o irmão duro do [`hover_of`].
///
/// ⚠️ `Danger` e `Warn` devolvem o MESMO tom que o hover, e isso é o catálogo, não preguiça: o
/// `Button::bg_token` dá `DangerSoft` para `Pressed | Hovered` num só braço. Uma família com um
/// tom macio só tem um degrau abaixo do cheio.
#[must_use]
pub fn pressed_of(rest: ph2d_tokens::ColorToken) -> ph2d_tokens::ColorToken {
    use ph2d_tokens::ColorToken as T;
    match rest {
        T::Accent => T::AccentPress,
        T::Danger => T::DangerSoft,
        T::Warn => T::WarnSoft,
        _ => T::AccentSoft,
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn mix(a: u8, b: u8, t: f32) -> u8 {
    (f32::from(a) + (f32::from(b) - f32::from(a)) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn fade(c: ph2d_tokens::Color, t: f32) -> ph2d_tokens::Color {
    ph2d_tokens::Color {
        a: (f32::from(c.a) * t).round().clamp(0.0, 255.0) as u8,
        ..c
    }
}
