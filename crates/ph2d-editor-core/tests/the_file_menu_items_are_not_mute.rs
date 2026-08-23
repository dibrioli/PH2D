//! **Os três itens do menu Ficheiro deixaram de ser mudos** (2026-08-23).
//!
//! ⚠️ `Save`, `Save As…` e `Open Project…` existiam no menu, **consumiam o clique** (devolviam
//! `true`, então o gesto parecia ter acontecido) e não faziam **nada** — o comentário do módulo
//! chamava-lhes *placeholders*. Um botão que consome o clique e não age é pior que um botão
//! ausente: o artista conclui que gravou.
//!
//! ⛔ Este gate afirma o que o painel PROMETE, não o que o disco faz: quem grava é o shell, que é
//! quem tem o disco. A metade de lá tem os gates dela em `shells/desktop/src/project_io_tests.rs`.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::ids;
use ph2d_editor_core::{HeroScreen, NodeId};

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(NodeId(1))
}

/// **Cada item levanta a SUA bandeira, e só a sua.**
///
/// ⚠️ As três juntas de propósito: um `else if` mal encadeado faria `Save As…` levantar o `Save`,
/// e o artista perderia o ficheiro anterior sem que nada reprovasse.
#[test]
fn each_file_menu_item_raises_its_own_flag() {
    for (id, name) in [
        (ids::CTX_MENU_SAVE, "save"),
        (ids::CTX_MENU_SAVE_AS, "save_as"),
        (ids::CTX_MENU_OPEN_PROJECT, "open"),
    ] {
        let mut h = hero();
        assert!(
            h.apply_event(WidgetEvent::Click(id)),
            "{name}: o clique tem de ser consumido"
        );
        let got = [h.file_menu.save, h.file_menu.save_as, h.file_menu.open];
        let want = match name {
            "save" => [true, false, false],
            "save_as" => [false, true, false],
            _ => [false, false, true],
        };
        assert_eq!(got, want, "{name} levantou a bandeira errada: {got:?}");
    }
}

/// **E um clique noutro sítio não levanta nenhuma** — o controlo negativo, que é o que impede o
/// `else` final de ter engolido o mundo.
#[test]
fn an_unrelated_click_raises_nothing() {
    let mut h = hero();
    let _ = h.apply_event(WidgetEvent::Click(ids::TOOL_UNDO));
    assert!(h.file_menu == Default::default());
    assert!(
        !h.import_requested,
        "e o Import continua a ser dele proprio"
    );
}

/// **O `Import…` continua a funcionar** — ele era o único dos quatro que já agia, e a refactoração
/// dos outros três passou pelo mesmo `if`.
#[test]
fn import_still_raises_its_flag() {
    let mut h = hero();
    assert!(h.apply_event(WidgetEvent::Click(ids::CTX_MENU_IMPORT)));
    assert!(h.import_requested);
    assert!(h.file_menu == Default::default());
}
