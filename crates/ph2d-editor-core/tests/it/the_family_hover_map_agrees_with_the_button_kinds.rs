//! **As DUAS leis de *«qual é o tom quente deste tom?»* concordam onde ambas respondem.**
//!
//! O catálogo responde por **KIND** (`Button::bg_token`, um `match (kind, state)`); o
//! [`motion::hover_of`] responde por **FAMÍLIA** (um `match` no tom de repouso). As duas existem
//! porque nenhuma subsume a outra:
//!
//! * o `bg_token` não pode ser derivado da família — os kinds *ghost* (`Default`/`IconOnly`) têm
//!   repouso **`None`**, e não há tom de que derivar;
//! * o `hover_of` não pode ser derivado do kind — o `paint_toggle` do Audio Mixer recebe o tom de
//!   repouso como **PARÂMETRO** (`Danger` no Mute, `Warn` no Solo, `Accent` nas master-fx), e não
//!   há kind que o descreva.
//!
//! ⚠️ **Duas cópias que não se podem fundir são fundíveis numa PROVA.** É o molde do oráculo
//! congelado (`serial_side`, `warp_axis`): a segunda resposta fica, e um gate afirma que ela é a
//! mesma. Sem isto, mover `Accent → AccentHover` no catálogo deixaria o mixer a acender com o tom
//! velho, em silêncio, com as duas suítes verdes.
//!
//! ⚠️ **O controlo positivo é metade do gate:** ele exige ver os pares que JÁ existem. Uma tabela
//! vazia — porque um enum mudou de nome, porque um kind saiu — reporta zero desacordos, que é
//! exactamente o que a concordância reporta.

use ph2d_a11y::NodeId;
use ph2d_editor_core::motion::{hover_of, pressed_of};
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState};
use ph2d_tokens::{ColorToken, Theme};

/// Os kinds cujo repouso é um tom CHEIO — os únicos de que uma família se pode derivar.
const FILLED: [ButtonKind; 2] = [ButtonKind::Accent, ButtonKind::Danger];

fn bg(kind: ButtonKind, state: ButtonState, theme: Theme) -> Option<ph2d_tokens::Color> {
    Button::new(NodeId(1), "x")
        .kind(kind)
        .state(state)
        .bg_color(theme)
}

/// **Para todo kind CHEIO, o quente da família é o quente do catálogo.**
///
/// *Mutação que deve sangrar:* trocar o braço `Accent` de [`hover_of`] (ou o braço
/// `(ButtonKind::Accent, Hovered)` de `bg_token`).
#[test]
fn the_family_hover_map_agrees_with_the_button_kinds() {
    let theme = Theme::default();
    let mut seen = 0;
    for kind in FILLED {
        let rest = bg(kind, ButtonState::Normal, theme).expect("um kind cheio tem repouso");
        // O tom de repouso resolvido, de volta ao TOKEN: a família é indexada por token, e é o
        // token que os dois lados têm em comum.
        let rest_token = match kind {
            ButtonKind::Accent => ColorToken::Accent,
            ButtonKind::Danger => ColorToken::Danger,
            _ => unreachable!("FILLED só tem kinds cheios"),
        };
        assert_eq!(
            rest_token.resolve(theme),
            rest,
            "o repouso do kind deixou de ser o token que este gate assume"
        );
        let hot = bg(kind, ButtonState::Hovered, theme).expect("um kind cheio tem hover");
        assert_eq!(
            hover_of(rest_token).resolve(theme),
            hot,
            "a familia e o catalogo discordam sobre o HOVER de {rest_token:?}"
        );
        let press = bg(kind, ButtonState::Pressed, theme).expect("um kind cheio tem press");
        assert_eq!(
            pressed_of(rest_token).resolve(theme),
            press,
            "a familia e o catalogo discordam sobre o PRESS de {rest_token:?}"
        );
        seen += 1;
    }
    assert_eq!(
        seen,
        FILLED.len(),
        "a varredura ficou vazia — gate a olhar para nada"
    );
}

/// **E o fallback é a superfície neutra**, que é o que o catálogo dá aos kinds *ghost*.
///
/// ⚠️ Um tom que a família não conhece é uma **superfície** até prova em contrário — nunca um
/// `Accent` por omissão, que faria um botão desconhecido acender como se fosse a acção primária.
#[test]
fn an_unknown_rest_tone_is_treated_as_a_surface() {
    let theme = Theme::default();
    assert_eq!(hover_of(ColorToken::Bg3), ColorToken::BgElev);
    assert_eq!(pressed_of(ColorToken::Bg3), ColorToken::AccentSoft);
    // E é exactamente o que o kind ghost usa no hover.
    let ghost_hot = bg(ButtonKind::Default, ButtonState::Hovered, theme).expect("ghost acende");
    assert_eq!(ColorToken::BgElev.resolve(theme), ghost_hot);
}

/// **`Warn` tem família, e ela não é a do `Accent`.**
///
/// O catálogo não tem `ButtonKind::Warn` — o Solo do mixer é o único consumidor —, então este par
/// não pode ser cruzado com o `bg_token` e é pinado aqui, com o motivo: `WarnSoft` existe no
/// palette e é o degrau macio da família.
#[test]
fn warn_has_its_own_soft_step() {
    assert_eq!(hover_of(ColorToken::Warn), ColorToken::WarnSoft);
    assert_eq!(pressed_of(ColorToken::Warn), ColorToken::WarnSoft);
    assert_ne!(hover_of(ColorToken::Warn), hover_of(ColorToken::Accent));
}
