//! ⭐⭐⭐ **O redesenho veste a família do Godot — e só ela; o clássico veste a de sempre — e só ela.**
//!
//! Decisão do Enio (2026-09-04): o modelo é o Godot 4.6 «Modern», o cinza e o azul são os dele.
//! Este gate afirma as três costuras entre a decisão e o que o artista vê:
//!
//! 1. o app **abre** num tema derivado quando a aparência é o redesenho, e no `forge` quando é o
//!    clássico;
//! 2. o menu de tema mostra **uma família por aparência** — nunca as duas misturadas;
//! 3. cada linha do menu de tema **muda o tema** para o que diz (pela cadeia real de despacho),
//!    e a marca de estado acende na linha certa.

use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::screens::hero::menu_rows::menu_rows;
use ph2d_editor_core::screens::hero::theme_menu::{THEME_MENU, theme_menu_id};
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_tokens::{Theme, UiLook};

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(NodeId(1))
}

/// O tema de arranque segue a aparência — e é o `Dark` (o *Default* do Godot) no redesenho.
///
/// ⚠️ `HeroScreen::new` lê a aparência do ambiente **uma vez por processo** (`OnceLock`), então
/// este teste não a pode alternar — afirma-se a LEI (`Theme::default_for`) e que o `new` a segue
/// para a aparência que este processo tem.
#[test]
fn the_app_opens_in_the_family_of_its_look() {
    assert_eq!(Theme::default_for(UiLook::Redesign), Theme::Dark);
    assert_eq!(Theme::default_for(UiLook::Classic), Theme::Forge);
    let h = hero();
    let look = ph2d_editor_core::paint::ui_look();
    assert_eq!(
        h.theme,
        Theme::default_for(look),
        "o HeroScreen nao abre no tema da aparencia que o processo tem ({look:?})"
    );
}

/// **O menu de tema mostra uma família por aparência.**
///
/// **Mutação que deve sangrar:** apagar o `if crate::paint::ui_is_redesign()` do braço
/// `ThemeSelector` em `menu_rows` — as duas aparências passariam a mostrar a família clássica, e
/// o redesenho ficaria sem porta para os presets que o dono escolheu.
#[test]
fn the_theme_menu_shows_one_family_per_look() {
    for (look, family) in [
        (UiLook::Redesign, &Theme::MODERN[..]),
        (UiLook::Classic, &Theme::CLASSIC[..]),
    ] {
        ph2d_editor_core::paint::set_ui_look(look);
        let rows: Vec<NodeId> = menu_rows(ContextMenuKind::ThemeSelector)
            .iter()
            .map(|(id, ..)| *id)
            .collect();
        let theme_rows: Vec<Theme> = rows
            .iter()
            .filter_map(|id| THEME_MENU.iter().find(|(m, _)| m == id).map(|(_, t)| *t))
            .collect();
        assert_eq!(
            theme_rows, family,
            "{look:?}: as linhas de tema do menu nao sao a familia da aparencia"
        );
        // E nenhuma da OUTRA família se esconde no menu.
        let other: &[Theme] = if look == UiLook::Redesign {
            &Theme::CLASSIC
        } else {
            &Theme::MODERN
        };
        for t in other {
            assert!(
                !rows.contains(&theme_menu_id(*t)),
                "{look:?}: {t:?} e' da outra familia e esta' no menu"
            );
        }
    }
    ph2d_editor_core::paint::set_ui_look(UiLook::Redesign);
}

/// **Cada linha do menu muda o tema para o que diz — as oito, pela cadeia real.**
#[test]
fn every_theme_row_sets_its_theme_and_the_mark_follows() {
    for (id, want) in THEME_MENU {
        let mut h = hero();
        assert!(
            h.apply_event(WidgetEvent::Click(id)),
            "{want:?}: a linha nao e' consumida por ninguem"
        );
        assert_eq!(h.theme, want, "a linha de {want:?} pos outro tema");
        assert_eq!(
            theme_menu_id(h.theme),
            id,
            "a marca de estado acende noutra linha"
        );
    }
}

/// ⛔ **Um tema moderno não desenha moldura em repouso; um clássico desenha** — a linha que
/// separa «plano» de «com a mesma cara», afirmada na tabela que os quatro pintores de cromo lêem.
#[test]
fn the_modern_family_is_flat_and_the_classic_is_framed() {
    use ph2d_tokens::visuals::{Chrome, Widgets};
    for theme in Theme::CLASSIC {
        assert!(Chrome::of(theme).panel_border.is_visible(), "{theme:?}");
        assert!(
            Widgets::of(theme).inactive.bg_stroke.is_visible(),
            "{theme:?}"
        );
    }
    for theme in [Theme::Dark, Theme::Gray, Theme::Light] {
        assert!(!Chrome::of(theme).panel_border.is_visible(), "{theme:?}");
        assert!(!Chrome::of(theme).field_border.is_visible(), "{theme:?}");
        assert!(
            !Widgets::of(theme).inactive.bg_stroke.is_visible(),
            "{theme:?}"
        );
        assert_eq!(
            Chrome::of(theme).panel_radius,
            4.0,
            "{theme:?}: o raio do Godot"
        );
    }
}
