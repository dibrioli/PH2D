//! **Arch-gate (F19): no Preview o painel MOSTRA e não edita** — decisão do dono, 18/10.
//!
//! A lei e a lista do que é VISTA moram na família (`ph2d_app_vec::ui_preview_lock`, com gates
//! próprios); aqui mede-se só que a SHELL a chama na porta certa. O despacho do ponteiro exige
//! janela e `gfx`, e nenhum teste de comportamento o alcança — o precedente é o
//! `the_width_reaches_the_selection_when_it_is_authored`.

const PONTEIRO: &str = include_str!("../../src/input_dispatch/despacho_clique_flip.rs");
const TECLADO: &str = include_str!("../../src/input_dispatch/keyboard.rs");

/// O `Down` sobre um painel trancado é recusado ANTES de chegar ao hero, e só com o Preview ligado.
#[test]
fn the_pointer_down_asks_the_family_lock_while_the_preview_is_on() {
    let i = PONTEIRO
        .find("ph2d_app_vec::ui_preview_lock::recusa(")
        .expect("a shell deixou de perguntar à trava do Preview: o painel volta a editar o que o leave desfaz");
    let guarda = &PONTEIRO[PONTEIRO[..i]
        .rfind("if ")
        .expect("a chamada está numa guarda")..i];
    assert!(
        guarda.contains("self.ui_preview.is_on()"),
        "a trava corre fora do Preview"
    );
    assert!(
        guarda.contains("kind == PointerKind::Down"),
        "a trava deixou de ser no Down"
    );
    let fecho = &PONTEIRO[i..];
    let corpo = &fecho[..fecho
        .find("return true;")
        .expect("o Down recusado é consumido")];
    assert!(
        corpo.contains("panel.vector.states.preview.locked"),
        "o Down recusado não avisa o artista — a edição morre em silêncio"
    );
}

/// Nenhum campo guarda o foco com o Preview ligado — uma tecla não pode editar o que o `leave` desfaz.
#[test]
fn no_field_keeps_the_focus_while_the_preview_is_on() {
    let i = TECLADO
        .find("h.store.set_focus(None);")
        .expect("o teclado voltou a alcançar um campo focado durante o Preview");
    let guarda = &TECLADO[TECLADO[..i]
        .rfind("if ")
        .expect("a limpeza está numa guarda")..i];
    assert!(
        guarda.contains("self.ui_preview.is_on()"),
        "a limpeza do foco corre fora do Preview"
    );
    assert!(
        i < TECLADO
            .find("forward_key_to_hero(")
            .expect("o teclado chega ao hero"),
        "o foco é limpo DEPOIS de a tecla já ter chegado ao campo"
    );
}
