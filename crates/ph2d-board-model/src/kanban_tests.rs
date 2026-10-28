use super::*;
use crate::{BoardOp, BoardSet, Element, FORMAT_VERSION, Field, Rgba, Style};

fn style() -> Style {
    Style::new(None, None, Rgba([0, 0, 0, 255]))
}

/// Um quadro com uma forma e um Kanban com a vista mexida (rolagem, zoom, uma lista recolhida).
fn set_with_kanban() -> (BoardSet, ElementId) {
    let mut set = BoardSet::default();
    let b = set.create("A".into());
    let doc = &mut set.get_mut(b).unwrap().doc;
    let mut k = BoardKanban::new(KanbanId(7), style());
    k.view.scroll = [12.5, -3.0];
    k.view.zoom = 0.75;
    k.view.collapsed.insert(ListId(4));
    let id = doc.mint_id();
    let el = Element::new_kanban(id, doc.z_on_top(), k, [10.0, 20.0, 800.0, 450.0]);
    BoardOp::Put(el).apply(doc);
    (set, id)
}

#[test]
fn a_kanban_element_round_trips_with_its_view() {
    let (set, id) = set_with_kanban();
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION
    );
    let back = BoardSet::from_bytes(&bytes).unwrap();
    assert_eq!(back, set);
    let k = back.boards()[0].doc.get(id).unwrap().kanban().unwrap();
    assert_eq!(k.kanban, KanbanId(7));
    assert_eq!(k.view.zoom, 0.75);
    assert!(k.view.collapsed.contains(&ListId(4)));
}

/// Os bytes do formato 17 (sem Kanbans) leem-se pelo mesmo leitor e regravam-se no 18.
#[test]
fn format_17_bytes_open_in_18_and_resave_as_18() {
    let mut set = BoardSet::default();
    set.create("Velho".into());
    // A versão é um varint de UM byte à cabeça: o 17 é o 18 com esse byte trocado.
    let mut old = set.to_bytes().unwrap();
    assert_eq!(old[0], 18);
    old[0] = 17;
    assert_eq!(postcard::take_from_bytes::<u32>(&old).unwrap().0, 17);
    let back = BoardSet::from_bytes(&old).unwrap();
    assert_eq!(back.boards(), set.boards());
    let again = back.to_bytes().unwrap();
    assert_eq!(postcard::take_from_bytes::<u32>(&again).unwrap().0, 18);
}

/// Só a REFERÊNCIA é um registo (viaja e desfaz-se); a vista não.
#[test]
fn the_kanban_reference_is_a_field_and_the_view_is_not() {
    let (set, id) = set_with_kanban();
    let a = set.boards()[0].doc.get(id).unwrap().clone();
    let mut b = a.clone();
    b.kanban_mut().unwrap().view.scroll = [99.0, 99.0];
    assert!(a.changed_fields(&b).is_empty());
    b.kanban_mut().unwrap().kanban = KanbanId(8);
    assert_eq!(a.changed_fields(&b), vec![Field::Kanban]);
    let mut c = a.clone();
    assert!(c.write(Field::Kanban, &b.read(Field::Kanban).unwrap()));
    assert_eq!(c.kanban().unwrap().kanban, KanbanId(8));
    assert!(Element::read(&set_shape(), Field::Kanban).is_none());
}

fn set_shape() -> Element {
    let s = crate::Shape {
        kind: crate::ShapeType::Rectangle,
        style: style(),
        text: crate::RichText::default(),
    };
    Element::new_shape(ElementId(1), crate::FracKey::between(None, None), s, [0.0; 4])
}

#[test]
fn moving_the_view_is_not_an_edit() {
    let (mut set, id) = set_with_kanban();
    let doc = &mut set.get_mut(crate::BoardId(1)).unwrap().doc;
    let rev = doc.rev();
    doc.kanban_view_mut(id).unwrap().toggle_list(ListId(9));
    assert_eq!(doc.rev(), rev);
    assert!(doc.get(id).unwrap().kanban().unwrap().view.collapsed.contains(&ListId(9)));
    assert_eq!(doc.kanban_elements(KanbanId(7)), vec![id]);
    assert!(doc.kanban_elements(KanbanId(8)).is_empty());
}
