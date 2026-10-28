//! W11 do Kanban — **o cartão que abre um script**: o pedido do verso (`kanban_links::open`) chega à
//! aba do script pelo mesmo caminho do «Edit script» (a aba certa, com o texto, activa); um script
//! que o projecto já não tem não abre nada.

use crate::Mesa;
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::kanban_links::{Family, Target, open, take};

#[test]
fn a_card_opens_the_tab_of_its_script() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "-- bob\n"), ("ana.luau", "-- ana\n")]);
    m.quadro();
    assert!(
        m.docs().tabs().is_empty(),
        "controlo: sem pedido não há aba"
    );
    open(&Target::Script(ids[1].0));
    m.quadro();
    let tabs = m.docs().tabs();
    assert_eq!(
        tabs.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(),
        ["ana.luau"]
    );
    assert_eq!(
        m.hero.documents.surface_active().map(|t| t.id),
        Some(ids[1].0),
        "a aba aberta fica activa"
    );
    assert!(take(Family::Script).is_empty(), "drenado");

    open(&Target::Script(9_999));
    m.quadro();
    assert_eq!(m.docs().tabs().len(), 1, "um script apagado não abre aba");
}
