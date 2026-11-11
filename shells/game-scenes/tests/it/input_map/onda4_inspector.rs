//! ⭐ **O Inspector segue a aba, ponta a ponta** (onda 4, 7b, auditoria): pelas portas reais da [`Mesa`] — o ponteiro
//! sobre uma linha da List, o clique, o `frame` da família — o Inspector mostra ESSA acção no mesmo quadro; com a aba
//! fechada, nada (`None`: o Inspector de sempre).

use super::Mesa;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::ListHit;
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_input::inspected::{InputMapPane, current_input_map};

fn shown_name() -> Option<String> {
    match current_input_map() {
        Some(InputMapPane::Action(a)) => Some(a.name),
        _ => None,
    }
}

#[test]
fn clicking_a_list_row_shows_that_action_and_closing_the_tab_clears_it() {
    let mut m = Mesa::aberta(View::List);
    // A List sem escolha: a frase.
    assert_eq!(current_input_map(), Some(InputMapPane::Pick));
    let map = m.tab().doc.snap.map.clone();
    // A segunda acção, para que «a primeira por omissão» não passe por acaso.
    let alvo = map.actions()[1].clone();
    let r = m
        .tab()
        .doc
        .rect_of(ListHit::Row(alvo.id).id())
        .expect("a linha pintada");
    let at = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    m.hover(at);
    let _ = m.click(at);
    assert_eq!(shown_name(), Some(alvo.name.clone()));
    // Outra linha ⇒ no mesmo quadro do clique, a outra acção.
    let outra = map.actions()[0].clone();
    let r = m
        .tab()
        .doc
        .rect_of(ListHit::Row(outra.id).id())
        .expect("a linha pintada");
    let at = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    m.hover(at);
    let _ = m.click(at);
    assert_eq!(shown_name(), Some(outra.name));
    // A aba fechada ⇒ o Inspector de sempre.
    m.tab_mut().close(ph2d_app_input_map::TAB_ID);
    m.frame();
    assert_eq!(current_input_map(), None);
    // Controlo: reaberta pelo menu, volta a mostrar a acção escolhida.
    m.menu();
    assert!(m.active());
    assert!(current_input_map().is_some());
}
