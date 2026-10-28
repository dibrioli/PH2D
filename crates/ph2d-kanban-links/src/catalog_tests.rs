use super::*;
use ph2d_kanban_model::{Card, FracKey, KanbanId, KanbanSet, List, ListId, Meta};

/// Um Kanban com um cartão e uma ligação a cada tipo.
fn linked() -> (KanbanSet, KanbanId, CardId) {
    let mut set = KanbanSet::default();
    let kid = set.create("K");
    let k = set.get(kid).unwrap();
    let ids = k.mint_n(2);
    let list = List {
        id: ListId(ids[0]),
        title: "L".into(),
        pos: FracKey::between(None, None),
        color: None,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    };
    let card = Card::new(
        CardId(ids[1]),
        list.id,
        FracKey::between(None, None),
        "C",
        Moment(0),
    );
    let c = card.id;
    set.apply(
        kid,
        vec![
            KanbanOp::put(Record::List(list)),
            KanbanOp::put(Record::card(card)),
        ],
    );
    let targets = [
        (Target::Object(41), "Player"),
        (Target::Script(7), "player.luau"),
        (Target::Macro("Walk".into()), "Walk"),
        (
            Target::Span {
                object: Some(41),
                strip: Some(3),
                from: 1,
                to: 9,
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
    ];
    for (t, n) in targets {
        let op = link_op(set.get(kid).unwrap(), c, t, n, Moment(1));
        set.apply(kid, vec![op]);
    }
    (set, kid, c)
}

fn full_catalog() -> Catalog {
    let mut cat = Catalog::default();
    cat.objects.insert(41, "Player".into());
    cat.scripts.insert(7, "player.luau".into());
    cat.macros.insert("Walk".into());
    cat.strips.insert(3, "Jump".into());
    cat.boards.insert(
        2,
        BoardEntry {
            name: "Ideas".into(),
            elements: [90].into(),
        },
    );
    cat
}

fn resolved(set: &KanbanSet, kid: KanbanId, c: CardId, cat: &Catalog) -> Vec<Resolved> {
    let k = set.get(kid).unwrap();
    k.links_of(c).into_iter().map(|l| resolve(cat, l)).collect()
}

#[test]
fn every_kind_resolves_live_with_its_kind() {
    let (set, kid, c) = linked();
    let r = resolved(&set, kid, c, &full_catalog());
    assert!(r.iter().all(|r| r.live));
    let kinds: Vec<Kind> = r.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        [
            Kind::Object,
            Kind::Script,
            Kind::Macro,
            Kind::Span,
            Kind::Element
        ]
    );
}

/// O objecto renomeado resolve-se pelo `StableId`: o nome novo aparece, e o `refresh` grava-o como
/// reserva (um passo que o desfazer leva).
#[test]
fn a_renamed_object_resolves_by_its_stable_id() {
    let (mut set, kid, c) = linked();
    let mut cat = full_catalog();
    cat.objects.insert(41, "Hero".into());
    let r = resolved(&set, kid, c, &cat);
    assert_eq!((r[0].name.as_str(), r[0].live), ("Hero", true));
    let ops = refresh(&cat, set.get(kid).unwrap(), c);
    assert_eq!(ops.len(), 1, "só a ligação ao objecto mudou de nome");
    set.apply(kid, ops);
    assert_eq!(set.get(kid).unwrap().links_of(c)[0].name, "Hero");
    assert!(refresh(&cat, set.get(kid).unwrap(), c).is_empty());
}

/// Alvo apagado = ligação quebrada com o nome de reserva; nunca desaparece.
#[test]
fn a_deleted_target_is_broken_and_keeps_its_fallback_name() {
    let (set, kid, c) = linked();
    let r = resolved(&set, kid, c, &Catalog::default());
    assert_eq!(r.len(), 5, "nenhuma some");
    assert!(r.iter().all(|r| !r.live));
    let names: Vec<&str> = r.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["Player", "player.luau", "Walk", "Jump", "Ideas"]);
    let mut cat = full_catalog();
    cat.boards.get_mut(&2).unwrap().elements.clear();
    let r = resolved(&set, kid, c, &cat);
    assert!(
        !r[4].live,
        "o elemento apagado quebra, o Quadro vivo não basta"
    );
    cat.strips.clear();
    assert!(
        !resolved(&set, kid, c, &cat)[3].live,
        "o clip apagado quebra o trecho"
    );
    assert!(refresh(&Catalog::default(), set.get(kid).unwrap(), c).is_empty());
}
