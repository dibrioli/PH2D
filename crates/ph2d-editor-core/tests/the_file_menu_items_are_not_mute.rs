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
/// ⚠️ Os quatro juntos de propósito: um `else if` mal encadeado faria `Save As…` levantar o `Save`,
/// e o artista perderia o ficheiro anterior sem que nada reprovasse.
///
/// ⛔⛔ **A lista é LITERAL, e isso é uma dívida NOMEADA.** Ela nasceu com três entradas e o
/// *Export SVG…* (2026-09-02) passou por ela **sem a acordar** — o gate ficou verde afirmando sobre
/// três itens de um menu que tinha quatro. *Um censo escrito à mão não vê o item que alguém
/// acrescentou.* Quem juntar um item ao menu Ficheiro TEM de o juntar aqui; a alternativa (derivar
/// a lista dos campos do `FileMenuRequests`) pede uma macro que a casa não tem.
#[test]
fn each_file_menu_item_raises_its_own_flag() {
    for (id, name) in [
        (ids::CTX_MENU_SAVE, "save"),
        (ids::CTX_MENU_SAVE_AS, "save_as"),
        (ids::CTX_MENU_OPEN_PROJECT, "open"),
        (ids::CTX_MENU_EXPORT_SVG, "export_svg"),
    ] {
        let mut h = hero();
        assert!(
            h.apply_event(WidgetEvent::Click(id)),
            "{name}: o clique tem de ser consumido"
        );
        let got = [
            h.file_menu.save,
            h.file_menu.save_as,
            h.file_menu.open,
            h.file_menu.export_svg,
        ];
        let want = match name {
            "save" => [true, false, false, false],
            "save_as" => [false, true, false, false],
            "open" => [false, false, true, false],
            _ => [false, false, false, true],
        };
        assert_eq!(got, want, "{name} levantou a bandeira errada: {got:?}");
    }
}

/// ⭐⭐⭐ **TODO item do menu Ficheiro tem de estar no censo acima** — a rede que apanha o próximo.
///
/// ⚠️ Ele lê as linhas do menu (`ContextMenuKind::SaveMenu`), que é a fonte que o artista vê, e
/// exige que cada uma levante alguma bandeira. *Assim um item novo reprova em vez de passar
/// despercebido, mesmo que ninguém se lembre de editar a lista literal.*
#[test]
fn no_file_menu_row_is_left_out_of_the_census() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_editor_core::screens::hero::menu_rows::menu_rows;
    for (id, label, _) in menu_rows(ContextMenuKind::SaveMenu) {
        let mut h = hero();
        assert!(
            h.apply_event(WidgetEvent::Click(*id)),
            "{label}: o clique tem de ser consumido"
        );
        assert!(
            h.file_menu != Default::default() || h.import_requested,
            "{label} nao levanta bandeira nenhuma — o item e' MUDO"
        );
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
