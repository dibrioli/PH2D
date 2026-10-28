use ph2d_kanban_model::{
    Comment, CommentId, Cover, CoverFill, Due, Hue, KanbanId, KanbanOp, Meta, Record,
};

use super::*;
use crate::test_support::{DAY, NOW, add_card, add_label, add_list, me, utc};

/// Dois Kanbans: «Film» (To Do, Done) e «Music» (Ideas).
fn scene() -> (KanbanSet, KanbanId, KanbanId) {
    let mut set = KanbanSet::default();
    let film = set.create("Film");
    let music = set.create("Music");
    let todo = add_list(&mut set, film, "To Do");
    let done = add_list(&mut set, film, "Done");
    let ideas = add_list(&mut set, music, "Ideas");
    let urgent = add_label(&mut set, film, "Urgente", Hue::Red);
    add_card(&mut set, film, todo, "Forest background", NOW, |c| {
        c.labels = vec![urgent];
        c.members = vec![me()];
        c.due = Some(Due {
            at: Moment(NOW.0 + DAY / 2),
            done: false,
            remind_before_min: None,
        });
    });
    add_card(&mut set, film, todo, "Paint sky", NOW, |c| {
        c.description = "A **forest** at dusk".to_owned();
        c.cover = Some(Cover {
            fill: CoverFill::Color(Hue::Blue),
            full: false,
        });
    });
    let old = add_card(&mut set, film, done, "Old forest", NOW, |c| {
        c.archived = true
    });
    let _ = old;
    let theme = add_card(&mut set, music, ideas, "Theme", NOW, |c| {
        c.members = vec![Member("Mia".to_owned())];
    });
    let id = CommentId(set.get(music).unwrap().mint());
    let cm = Comment {
        id,
        card: theme,
        author: Member("Mia".to_owned()),
        at: NOW,
        text: "Ação na floresta".to_owned(),
        edited: None,
        reactions: Vec::new(),
        meta: Meta::default(),
    };
    set.apply(music, vec![KanbanOp::put(Record::Comment(cm))]);
    (set, film, music)
}

fn titles(set: &KanbanSet, q: &str) -> Vec<String> {
    search(set, q, NOW, &utc(), &me())
        .into_iter()
        .map(|h| set.card(h.card).unwrap().title.clone())
        .collect()
}

#[test]
fn parse_query_reads_the_trello_operators() {
    let q = parse_query(r#"@me -#urgente due:week is:archived list:"To Do" has:cover "a b" x:y"#);
    let terms: Vec<(bool, Term)> = q.into_iter().map(|c| (c.negate, c.term)).collect();
    assert_eq!(
        terms,
        [
            (false, Term::Me),
            (true, Term::Label("urgente".into())),
            (false, Term::Due(DueWindow::NextWeek)),
            (false, Term::Is(Is::Archived)),
            (false, Term::List("to do".into())),
            (false, Term::Has(Has::Cover)),
            (false, Term::Text("a b".into())),
            (false, Term::Text("x:y".into())),
        ]
    );
}

#[test]
fn text_searches_every_kanban_and_ranks_the_title_first() {
    let (set, ..) = scene();
    assert_eq!(titles(&set, "forest"), ["Forest background", "Paint sky"]);
    assert_eq!(
        titles(&set, "floresta"),
        ["Theme"],
        "comentário noutro Kanban"
    );
    assert_eq!(titles(&set, "ACAO"), ["Theme"], "sem caixa nem acento");
    assert!(titles(&set, "forest sky").contains(&"Paint sky".to_owned()));
    assert_eq!(
        titles(&set, "forest sky"),
        ["Paint sky"],
        "cada termo tem de aparecer"
    );
    assert_eq!(
        titles(&set, r#""forest background""#),
        ["Forest background"]
    );
    assert!(titles(&set, "").is_empty());
}

#[test]
fn operators_filter_and_negate() {
    let (set, ..) = scene();
    assert_eq!(titles(&set, "@me"), ["Forest background"]);
    assert_eq!(titles(&set, "@mia"), ["Theme"]);
    assert_eq!(titles(&set, "#urgente"), ["Forest background"]);
    assert_eq!(
        titles(&set, "#red"),
        ["Forest background"],
        "a cor serve de nome"
    );
    assert_eq!(titles(&set, "forest -#urgente"), ["Paint sky"]);
    assert_eq!(titles(&set, "due:day"), ["Forest background"]);
    assert_eq!(titles(&set, "has:cover"), ["Paint sky"]);
    assert_eq!(titles(&set, "has:description"), ["Paint sky"]);
    assert_eq!(titles(&set, "board:music"), ["Theme"]);
    assert_eq!(
        titles(&set, r#"list:"to do" -has:cover"#),
        ["Forest background"]
    );
    assert_eq!(
        titles(&set, "forest is:archived"),
        ["Old forest"],
        "só com is:archived"
    );
    assert_eq!(titles(&set, "is:open").len(), 3);
}

#[test]
fn equal_scores_follow_the_project_order_every_time() {
    let (set, ..) = scene();
    let a = search(&set, "is:open", NOW, &utc(), &me());
    let b = search(&set, "is:open", NOW, &utc(), &me());
    assert_eq!(a, b);
    let names: Vec<String> = a
        .iter()
        .map(|h| set.card(h.card).unwrap().title.clone())
        .collect();
    assert_eq!(names, ["Forest background", "Paint sky", "Theme"]);
}
