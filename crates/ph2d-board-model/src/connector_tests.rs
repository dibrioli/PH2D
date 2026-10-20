//! W23 B5(1) — o segmento do meio do cotovelo no modelo: alinhado com os pontos de ajuste, gravado
//! (formato 14, o 13 lê-se) e no registo ao vivo dos pontos; W23 B5(2) — os textos ao longo da linha
//! (formato 15, o 14 lê-se) e os registos deles.

use super::*;
use crate::{BoardSet, ElementKind, FORMAT_VERSION, Field, Rgba};

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

/// ⭐ Um ficheiro do formato 13 continua a abrir — gravado pelo build de `10c11f8be2` (teste
/// temporário que escreveu `to_bytes`, apagado): «Setas13» com duas caixas e uma seta cotovelo presa
/// às duas, pontas `Circle`/`Arrow`, rótulo «sim» e um ponto de ajuste em (250, 200). Lê-se sem meios
/// fixos, e grava-se no formato de hoje.
#[test]
fn a_format_13_file_still_opens_with_its_elbow_unpinned() {
    let set = BoardSet::from_bytes(V13_BYTES).expect("o formato 13 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Setas13");
    let c = b
        .doc
        .live()
        .find_map(|e| match &e.kind {
            ElementKind::Connector(c) => Some(&**c),
            _ => None,
        })
        .expect("a seta");
    assert_eq!(c.route, Route::Elbow);
    assert_eq!(c.heads, [Head::Circle, Head::Arrow]);
    assert_eq!(c.captions, vec![Caption::new("sim", Caption::MIDDLE)]);
    assert_eq!(c.turn, TextTurn::Level);
    assert_eq!(c.waypoints, vec![[250.0, 200.0]]);
    assert!(c.mids.is_empty());
    let mut set = set;
    let id = set.boards()[0].id;
    let doc = &mut set.get_mut(id).unwrap().doc;
    let mut el = doc
        .live()
        .find(|e| e.connector().is_some())
        .unwrap()
        .clone();
    el.connector_mut().unwrap().set_mid(1, Some(0.3));
    crate::BoardOp::Put(el).apply(doc);
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION
    );
    assert_eq!(
        BoardSet::from_bytes(&bytes).unwrap(),
        set,
        "o de hoje vai e volta"
    );
}

/// Gravados pelo build do formato 13 (`10c11f8be2`).
const V13_BYTES: &[u8] = include_bytes!("../fixtures/format_v13.bin");

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

/// ⭐ W23 B5(2) — um ficheiro do formato 14 continua a abrir — gravado pelo build de `0353869e6d`
/// (teste temporário que escreveu `to_bytes`, apagado): o «Setas13» acima renomeado «Setas14», com o
/// segmento do meio do trecho 1 fixo a 0,3. O rótulo «sim» volta como UM texto a meio, deitado na
/// horizontal como o artista o via, e o meio fixo fica.
#[test]
fn a_format_14_file_opens_with_its_label_as_one_level_caption() {
    let set = BoardSet::from_bytes(V14_BYTES).expect("o formato 14 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Setas14");
    let c = b
        .doc
        .live()
        .find_map(|e| e.connector())
        .expect("a seta")
        .clone();
    assert_eq!(c.captions, vec![Caption::new("sim", Caption::MIDDLE)]);
    assert_eq!(c.turn, TextTurn::Level);
    assert_eq!(c.mids, vec![None, Some(0.3)]);
    assert_eq!(c.waypoints, vec![[250.0, 200.0]]);
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION
    );
    assert_eq!(
        BoardSet::from_bytes(&bytes).unwrap(),
        set,
        "o 15 vai e volta"
    );
}

/// Gravados pelo build do formato 14 (`0353869e6d`).
const V14_BYTES: &[u8] = include_bytes!("../fixtures/format_v14.bin");

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
