//! W23 B5(1) — o segmento do meio do cotovelo no modelo: alinhado com os pontos de ajuste, gravado
//! (formato 14) e no registo ao vivo dos pontos; W23 B5(2) — os textos ao longo da linha
//! (formato 15) e os registos deles.

use super::*;
use crate::{Field, Rgba};

fn arrow(waypoints: Vec<[f64; 2]>) -> Connector {
    let ink = Rgba([0, 0, 0, 255]);
    let mut c = Connector::new(
        End::Free([0.0, 0.0]),
        End::Free([400.0, 300.0]),
        Route::Elbow,
        Style::new(None, Some(ink), ink),
    );
    c.waypoints = waypoints;
    c
}

/// ⭐ Inserir um ponto de ajuste parte o trecho — os dois meios voltam ao de nascença, os outros
/// trechos ficam com o seu; tirar funde. Nunca há mais meios que trechos, nem `None` à cauda.
#[test]
fn a_waypoint_split_keeps_each_other_leg_its_own_middle() {
    let mut c = arrow(vec![[200.0, 0.0]]);
    c.set_mid(0, Some(0.25));
    c.set_mid(1, Some(0.75));
    c.insert_waypoint(1, [300.0, 100.0]);
    assert_eq!(
        c.mids,
        vec![Some(0.25)],
        "o trecho 1 partiu-se: os dois de nascença"
    );
    assert_eq!(c.mid(2), None);
    c.set_mid(2, Some(0.6));
    c.remove_waypoint(0);
    assert_eq!(
        c.mids,
        vec![None, Some(0.6)],
        "os trechos 0 e 1 fundiram-se"
    );
    c.set_mid(1, None);
    assert!(c.mids.is_empty(), "sem `None` à cauda");
    c.set_mid(5, Some(0.1));
    assert!(
        c.mids.is_empty(),
        "nenhum trecho 5 numa seta de dois trechos"
    );
}

/// ⭐ Os pontos de ajuste e os meios viajam no MESMO registo: dois colegas não os desalinham.
#[test]
fn the_waypoints_field_carries_the_middles() {
    let mut doc = crate::BoardDoc::default();
    let mut a =
        crate::Element::new_connector(doc.mint_id(), doc.z_on_top(), arrow(vec![[200.0, 0.0]]));
    a.connector_mut().unwrap().set_mid(1, Some(0.8));
    let bytes = a.read(Field::Waypoints).expect("uma seta tem o registo");
    let mut b = crate::Element::new_connector(doc.mint_id(), doc.z_on_top(), arrow(Vec::new()));
    assert!(b.write(Field::Waypoints, &bytes));
    assert_eq!(b.connector().unwrap().waypoints, vec![[200.0, 0.0]]);
    assert_eq!(b.connector().unwrap().mids, vec![None, Some(0.8)]);
}

/// ⭐ Os textos e o modo de os deitar são DOIS registos: um colega que deita os textos e outro que
/// escreve num deles ficam os dois.
#[test]
fn the_captions_and_their_turn_are_separate_fields() {
    let mut a = arrow(Vec::new());
    a.captions = vec![Caption::new("sim", 0.2)];
    let mut doc = crate::BoardDoc::default();
    let el = crate::Element::new_connector(doc.mint_id(), doc.z_on_top(), a);
    let mut turned = el.clone();
    turned.connector_mut().unwrap().turn = TextTurn::Level;
    assert_eq!(turned.changed_fields(&el), vec![Field::TextTurn]);
    let mut written = el.clone();
    written.connector_mut().unwrap().captions[0].text = "não".into();
    assert_eq!(written.changed_fields(&el), vec![Field::Captions]);
    let mut both = turned.clone();
    assert!(both.write(Field::Captions, &written.read(Field::Captions).unwrap()));
    let c = both.connector().unwrap();
    assert_eq!(
        (c.turn, c.captions[0].text.as_str()),
        (TextTurn::Level, "não")
    );
    // E o `turn` também viaja sozinho pelo fio (mutação B10 do B5(2): o `write` dele não escrevia).
    let mut level = el.clone();
    assert!(level.write(Field::TextTurn, &turned.read(Field::TextTurn).unwrap()));
    assert_eq!(level.connector().unwrap().turn, TextTurn::Level);
    assert!(level.changed_fields(&turned).is_empty());
}
