//! W11 do Kanban — **o cartão que abre uma macro**: o pedido do verso (`kanban_links::open`) é
//! drenado pelo painel ao pintar e a macro fica escolhida na Biblioteca (com os verbos dela); uma
//! macro que a biblioteca já não tem não escolhe nada.

use super::seam_macros::{Palco, info};
use ph2d_editor_core::kanban_links::{Family, Target, open, pending};
use ph2d_editor_core::macro_edits::{FamiliaDaMacro, MacroDaBibliotecaView, MacroPanelInfo};
use ph2d_panel_macros::ids;

fn com(nomes: &[&str]) -> MacroPanelInfo {
    MacroPanelInfo {
        biblioteca: nomes
            .iter()
            .map(|n| MacroDaBibliotecaView {
                nome: (*n).into(),
                familias: vec![FamiliaDaMacro::Motion],
                duracao_s: 1.0,
                tela: None,
            })
            .collect(),
        ..info(None, None)
    }
}

#[test]
fn a_card_that_opens_a_macro_chooses_it_in_the_library() {
    open(&Target::Macro("Vento".into()));
    let mut p = Palco::novo(com(&["Arvore", "Vento"]));
    assert!(!pending(Family::Macro), "o painel drenou o pedido");
    assert_eq!(p.st.lib_escolhida.as_deref(), Some("Vento"));
    p.pinta();
    assert!(
        p.rect(ids::MACROS_LIB_APPLY_NEW).is_some(),
        "os verbos da escolhida aparecem"
    );
}

#[test]
fn a_macro_gone_from_the_library_chooses_nothing() {
    open(&Target::Macro("Gone".into()));
    let p = Palco::novo(com(&["Arvore"]));
    assert_eq!(p.st.lib_escolhida, None);
    assert_eq!(p.st.lib_pedida, None, "o pedido cai");
}
