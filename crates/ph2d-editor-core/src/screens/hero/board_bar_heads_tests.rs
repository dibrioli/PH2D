//! W23 B5 — a grelha de TODAS as pontas, pelo caminho real do ecrã.

use super::*;

/// ⭐ Com a seta seleccionada, o «…» das pontas do FIM abre a grelha das quinze; o losango (uma das
/// de UML que a barra estreita não tinha) aplica-se à ponta do fim e a grelha fecha; sem a grelha
/// aberta a ponta de UML não se clica.
#[test]
fn the_more_tips_grid_offers_every_head_of_the_catalog() {
    let mut t = T::new();
    t.two_boxes();
    t.click_bar(Item::Tool(Tool::Connector));
    let (a, b) = (t.at(0.26, 0.45), t.at(0.66, 0.45));
    t.drag(a, b);
    assert!(
        t.find(node(Item::Head(1, Head::Diamond))).is_none(),
        "fechada"
    );
    t.click_bar(Item::MoreHeads(1));
    for h in Head::ALL {
        assert!(
            t.find(node(Item::Head(1, h))).is_some(),
            "{h:?} não está na grelha"
        );
    }
    t.click_bar(Item::Head(1, Head::Diamond));
    assert_eq!(t.arrows()[0].connector().unwrap().heads[1], Head::Diamond);
    assert_eq!(t.arrows()[0].connector().unwrap().heads[0], Head::None);
    assert!(
        t.find(node(Item::Head(1, Head::ZeroOrMany))).is_none(),
        "fechou"
    );
    // A do início abre a sua: as pontas dela são as do início.
    t.click_bar(Item::MoreHeads(0));
    t.click_bar(Item::Head(0, Head::ZeroOrMany));
    assert_eq!(
        t.arrows()[0].connector().unwrap().heads[0],
        Head::ZeroOrMany
    );
}
