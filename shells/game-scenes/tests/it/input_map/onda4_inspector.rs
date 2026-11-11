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
    m.frame();
    let map = m.tab().doc.snap.map.clone();
    // A List sem escolha: a primeira acção, a linha que a List marca como aberta (uma fonte com a Keyboard).
    assert_eq!(shown_name(), Some(map.actions()[0].name.clone()));
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

/// ⭐ Foto da cena 9 (A4): a linha com manche (Dead zone e Press at à vista) escolhida ⇒ o Inspector mostra-a com o
/// cartão Stick, com os MESMOS valores da linha. Controlo: um mapa sem acções ⇒ a frase.
#[test]
fn the_stick_row_shows_its_stick_card_and_an_empty_map_shows_the_sentence() {
    use ph2d_input::{Binding, GamepadAxis};
    let mut m = Mesa::aberta(View::List);
    // A cena 9: o move_left com o manche (a linha que mostra Dead zone e Press at).
    let ml = m.hero.input_map.id("move_left").expect("fábrica");
    if let Some(a) = m.hero.input_map.get_mut(ml) {
        a.bindings.push(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: false,
        });
        a.set_zone(0.15, 0.5);
    }
    m.frame();
    m.frame();
    let map = m.tab().doc.snap.map.clone();
    let stick = map
        .actions()
        .iter()
        .find(|a| a.is_analog())
        .expect("uma acção com manche")
        .clone();
    let r = m
        .tab()
        .doc
        .rect_of(ListHit::Row(stick.id).id())
        .expect("a linha pintada");
    let at = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    m.hover(at);
    let _ = m.click(at);
    let Some(InputMapPane::Action(a)) = current_input_map() else {
        panic!("a acção");
    };
    assert_eq!(a.name, stick.name);
    let (dead, press) = a.stick.expect("o cartão Stick");
    assert_eq!(dead, format!("{:.2}", stick.dead_zone));
    assert_eq!(press, format!("{:.2}", stick.press_point));
    // Controlo: sem acções, a frase.
    m.hero.input_map = ph2d_input::InputMap::new();
    m.frame();
    m.frame();
    assert_eq!(current_input_map(), Some(InputMapPane::Pick));
}
