use super::*;
use crate::{KanbanOp, Record, RecordId};

fn hex_fixture(text: &str) -> Vec<u8> {
    let hex: String = text.lines().filter(|l| !l.starts_with('#')).collect();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// O primeiro cartão do primeiro Kanban.
fn first(set: &KanbanSet) -> CardRef {
    let k = set.iter().next().unwrap();
    let list = k.lists_in_order()[0].id;
    CardRef {
        kanban: k.id,
        card: k.cards_in(list)[0].id,
    }
}

fn link(set: &mut KanbanSet, at: CardRef, target: Target, name: &str) -> ProjectLinkId {
    let k = set.get(at.kanban).unwrap();
    let id = ProjectLinkId(k.mint());
    let l = ProjectLink::new(
        id,
        at.card,
        k.pos_after_links(at.card),
        target,
        name,
        Moment(9),
    );
    let undo = set.apply(at.kanban, vec![KanbanOp::put(Record::ProjectLink(l))]);
    assert_eq!(undo, vec![KanbanOp::Delete(RecordId::ProjectLink(id))]);
    id
}

/// O v5 cheio mais uma ligação de cada tipo (o que o v6 trouxe).
pub(crate) fn full_v6() -> (KanbanSet, CardRef) {
    let mut set = crate::agenda::tests::full_v5();
    let at = first(&set);
    for (t, n) in [
        (Target::Object(41), "Player"),
        (Target::Script(7), "player.luau"),
        (Target::Macro("Walk cycle".into()), "Walk cycle"),
        (
            Target::Span {
                object: Some(41),
                strip: Some(3),
                from: 12,
                to: 48,
            },
            "Jump",
        ),
        (
            Target::Board {
                board: 2,
                element: Some(90),
            },
            "Ideas",
        ),
    ] {
        link(&mut set, at, t, n);
    }
    (set, at)
}

#[test]
fn v5_bytes_read_as_v6_equal() {
    let bytes = hex_fixture(include_str!("../tests/fixtures/kanban_set_v5_full.hex"));
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 5);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    let today = crate::agenda::tests::full_v5();
    assert!(
        back.iter().eq(today.iter()),
        "o degrau v5 → v6 dá os mesmos Kanbans"
    );
    assert_eq!(back.agenda(), today.agenda(), "e a mesma agenda");
    assert!(back.iter().all(|k| k.project_links.is_empty()));
    let again = KanbanSet::from_bytes(&back.to_bytes().unwrap()).unwrap();
    assert_eq!(again, back, "e, regravado em v6, continua igual");
}

#[test]
fn the_v6_bytes_round_trip_every_kind_of_project_link() {
    let (set, at) = full_v6();
    let bytes = set.to_bytes().unwrap();
    assert!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0 >= 6);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert!(back.iter().eq(set.iter()));
    assert_eq!(back.agenda(), set.agenda());
    let k = back.get(at.kanban).unwrap();
    let names: Vec<&str> = k
        .links_of(at.card)
        .iter()
        .map(|l| l.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["Player", "player.luau", "Walk cycle", "Jump", "Ideas"],
        "pela ordem em que se ligaram"
    );
}

/// O sentido inverso: o objecto responde também pelos trechos dele; o Quadro inteiro por cada
/// elemento; um cartão apagado deixa de referir; a ligação apagada também.
#[test]
fn cards_linking_answers_the_reverse_question() {
    let (mut set, at) = full_v6();
    let other = set.create("Outro");
    let l = crate::test_support::list(set.get(other).unwrap(), "L");
    set.apply(other, vec![l]);
    let lo = set.get(other).unwrap().lists_in_order()[0].id;
    let c = crate::test_support::card(set.get(other).unwrap(), lo, "Two");
    set.apply(other, vec![c]);
    let two = CardRef {
        kanban: other,
        card: set.get(other).unwrap().cards_in(lo)[0].id,
    };
    let span = link(
        &mut set,
        two,
        Target::Span {
            object: Some(41),
            strip: None,
            from: 0,
            to: 5,
        },
        "Intro",
    );
    assert_eq!(set.cards_linking(&Target::Object(41)), vec![at, two]);
    assert_eq!(set.cards_linking(&Target::Script(7)), vec![at]);
    assert_eq!(set.cards_linking(&Target::Script(8)), vec![]);
    let whole = Target::Board {
        board: 2,
        element: None,
    };
    assert_eq!(set.cards_linking(&whole), vec![at]);
    let other_el = Target::Board {
        board: 2,
        element: Some(91),
    };
    assert_eq!(set.cards_linking(&other_el), vec![], "outro elemento não");
    set.apply(other, vec![KanbanOp::Delete(RecordId::ProjectLink(span))]);
    assert_eq!(set.cards_linking(&Target::Object(41)), vec![at]);
    set.apply(at.kanban, vec![KanbanOp::Delete(RecordId::Card(at.card))]);
    assert_eq!(set.cards_linking(&Target::Object(41)), vec![]);
}

/// Pôr e tirar uma ligação é um passo de desfazer cada; o id não colide com outra tabela.
#[test]
fn a_project_link_is_one_undo_step_and_mints_in_the_shared_id_space() {
    let (mut set, at) = full_v6();
    let k = set.get(at.kanban).unwrap();
    let id = k.links_of(at.card)[0].id;
    assert!(k.uses(id.0));
    let before = k.clone();
    let undo = set.apply(at.kanban, vec![KanbanOp::Delete(RecordId::ProjectLink(id))]);
    assert_eq!(set.get(at.kanban).unwrap().links_of(at.card).len(), 4);
    set.apply(at.kanban, undo);
    let k = set.get(at.kanban).unwrap();
    assert_eq!(k.links_of(at.card).len(), 5);
    assert_eq!(
        k.project_links[&id].target,
        before.project_links[&id].target
    );
}

/// Mover um cartão para outro Kanban leva as ligações (e tira-as da origem); copiá-lo copia-as.
#[test]
fn moving_or_copying_a_card_carries_its_project_links() {
    use crate::transfer::tests::two;
    let (mut set, a, b) = two();
    let k = set.get(a).unwrap();
    let src = CardRef {
        kanban: a,
        card: k.cards_in(k.lists_in_order()[0].id)[0].id,
    };
    link(&mut set, src, Target::Script(7), "player.luau");
    let blist = set.get(b).unwrap().lists_in_order()[0].id;
    let to = set.place_at(b, blist, 0, None).unwrap();
    let plan = crate::copy_card(
        &set,
        src,
        &to,
        crate::CopyKeep::default(),
        "Copy",
        Moment(1),
    );
    let mut h = std::collections::BTreeMap::new();
    let me = crate::transfer::tests::me();
    set.apply_plan(plan.unwrap(), &mut h, 1, &me);
    assert_eq!(
        set.cards_linking(&Target::Script(7)).len(),
        2,
        "a cópia também liga"
    );
    let to = set.place_at(b, blist, 0, None).unwrap();
    set.apply_plan(crate::move_card(&set, src, &to).unwrap(), &mut h, 2, &me);
    let now = set.cards_linking(&Target::Script(7));
    assert_eq!(now.len(), 2);
    assert!(now.iter().all(|r| r.kanban == b), "a origem já não refere");
}
