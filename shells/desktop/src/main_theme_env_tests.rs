//! Os testes do `crate::theme::resolve_theme` que viviam no fim do `main.rs` (o módulo continua a ser
//! `theme_env_tests`, logo os nomes dos testes não mudam). Saíram pelo tecto de LOC do `main.rs`.

use crate::theme::resolve_theme;
use ph2d_tokens::{Theme, UiLook};

/// ⛔ **O default segue a aparência** — o smoke de 2026-09-04 abriu no `forge` com o
/// redesenho ligado porque esta função devolvia `Forge` sem perguntar.
#[test]
fn unset_defaults_to_the_looks_own_theme() {
    assert_eq!(resolve_theme(None, UiLook::Classic), Theme::Forge);
    assert_eq!(resolve_theme(None, UiLook::Redesign), Theme::Dark);
    assert_eq!(
        resolve_theme(None, UiLook::Redesign),
        Theme::default_for(UiLook::Redesign),
        "uma lei, duas portas"
    );
}

/// Todo id das DUAS famílias resolve, em qualquer aparência — um nome explícito é uma escolha.
#[test]
fn every_theme_id_resolves_under_both_looks() {
    for look in [UiLook::Classic, UiLook::Redesign] {
        for theme in Theme::ALL {
            assert_eq!(resolve_theme(Some(theme.id()), look), theme, "{look:?}");
        }
    }
}

#[test]
fn unknown_falls_back_to_the_looks_default() {
    assert_eq!(
        resolve_theme(Some("dracula"), UiLook::Classic),
        Theme::Forge
    );
    assert_eq!(
        resolve_theme(Some("dracula"), UiLook::Redesign),
        Theme::Dark
    );
}
