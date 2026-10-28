use std::collections::BTreeMap;

use ph2d_kanban_model::{
    Actor, CardRef, Due, FieldDef, FieldId, FieldKind, Hue, KanbanOp, Meta, Record, mirror_card,
};

use super::*;
use crate::test_support::{DAY, NOW, add_card, add_label, add_list, me, utc};

fn due(at: i64, done: bool) -> Option<Due> {
    Some(Due {
        at: Moment(at),
        done,
        remind_before_min: None,
    })
}

/// Um Kanban com quatro cartões variados na lista «To Do».
fn scene() -> (KanbanSet, KanbanId, ListId, [LabelId; 2]) {
    let mut set = KanbanSet::default();
    let k = set.create("Short film");
    let l = add_list(&mut set, k, "To Do");
    let anim = add_label(&mut set, k, "Animação", Hue::Green);
    let story = add_label(&mut set, k, "Story", Hue::Purple);
    add_card(&mut set, k, l, "Animate shot 12", Moment(NOW.0), |c| {
        c.labels = vec![anim];
        c.members = vec![me()];
        c.due = due(NOW.0 - DAY, false);
    });
    add_card(
        &mut set,
        k,
        l,
        "Clean up shot 08",
        Moment(NOW.0 - 10 * DAY),
        |c| {
            c.labels = vec![anim, story];
            c.members = vec![Member("Mia".to_owned())];
            c.due = due(NOW.0 + DAY / 2, false);
        },
    );
    add_card(
        &mut set,
        k,
        l,
        "Timing notes",
        Moment(NOW.0 - 40 * DAY),
        |c| {
            c.due = due(NOW.0 + 20 * DAY, true);
        },
    );
    add_card(&mut set, k, l, "Sound pass", Moment(NOW.0 - 3 * DAY), |c| {
        c.due = due(NOW.0 + 5 * DAY, false);
    });
    (set, k, l, [anim, story])
}

fn shown(set: &KanbanSet, k: KanbanId, l: ListId, f: &Filter) -> Vec<String> {
    filter_entries(f, set, k, l, NOW, &utc(), &me())
        .into_iter()
        .map(|e| set.card(e.shows(k)).unwrap().title.clone())
        .collect()
}

#[test]
fn an_empty_filter_shows_everything() {
    let (set, k, l, _) = scene();
    let f = Filter::default();
    assert!(f.is_empty());
    assert_eq!(shown(&set, k, l, &f).len(), 4);
    assert_eq!(count(&f, set.get(k).unwrap(), NOW, &utc(), &me()), 4);
}

#[test]
fn keyword_matches_title_labels_members_and_fields_without_accents() {
    let (mut set, k, l, _) = scene();
    let f = |s: &str| Filter {
        keyword: s.to_owned(),
        ..Filter::default()
    };
    assert_eq!(
        shown(&set, k, l, &f("SHOT")),
        ["Animate shot 12", "Clean up shot 08"]
    );
    assert_eq!(
        shown(&set, k, l, &f("animacao")).len(),
        2,
        "nome da etiqueta, sem acento"
    );
    assert_eq!(
        shown(&set, k, l, &f("mia")),
        ["Clean up shot 08"],
        "nome do membro"
    );
    assert_eq!(
        shown(&set, k, l, &f("timing sound")).len(),
        2,
        "palavras soltas: qualquer"
    );
    assert!(
        shown(&set, k, l, &f(r#""timing sound""#)).is_empty(),
        "frase: exacta"
    );
    let fid = FieldId(set.get(k).unwrap().mint());
    let def = FieldDef {
        id: fid,
        name: "Cena".to_owned(),
        kind: FieldKind::Text,
        on_front: false,
        pos: ph2d_kanban_model::FracKey::between(None, None),
        meta: Meta::default(),
    };
    set.apply(k, vec![KanbanOp::put(Record::Field(def))]);
    add_card(&mut set, k, l, "Untitled", NOW, |c| {
        c.fields = BTreeMap::from([(fid, ph2d_kanban_model::FieldValue::Text("Floresta".into()))]);
        c.description = "floresta na descrição".to_owned();
    });
    assert_eq!(
        shown(&set, k, l, &f("floresta")),
        ["Untitled"],
        "campo personalizado"
    );
    assert!(
        shown(&set, k, l, &f("descricao")).is_empty(),
        "a descrição é da pesquisa"
    );
}

#[test]
fn members_no_members_and_mine() {
    let (set, k, l, _) = scene();
    let f = Filter {
        mine: true,
        ..Filter::default()
    };
    assert_eq!(shown(&set, k, l, &f), ["Animate shot 12"]);
    let f = Filter {
        no_members: true,
        ..Filter::default()
    };
    assert_eq!(shown(&set, k, l, &f), ["Timing notes", "Sound pass"]);
    let f = Filter {
        no_members: true,
        members: [Member("Mia".to_owned())].into(),
        ..Filter::default()
    };
    assert_eq!(
        shown(&set, k, l, &f).len(),
        3,
        "qualquer: Mia OU sem membros"
    );
    let f = Filter {
        mode: MatchMode::Exact,
        ..f
    };
    assert!(
        shown(&set, k, l, &f).is_empty(),
        "exacta: Mia E sem membros"
    );
}

#[test]
fn due_windows_status_and_the_any_or_exact_combiner() {
    let (set, k, l, [anim, story]) = scene();
    let by = |due: &[DueWindow]| Filter {
        due: due.iter().copied().collect(),
        ..Filter::default()
    };
    assert_eq!(
        shown(&set, k, l, &by(&[DueWindow::Overdue])),
        ["Animate shot 12"]
    );
    assert_eq!(
        shown(&set, k, l, &by(&[DueWindow::NextDay])),
        ["Clean up shot 08"]
    );
    assert_eq!(
        shown(&set, k, l, &by(&[DueWindow::NextWeek])),
        ["Clean up shot 08", "Sound pass"]
    );
    assert_eq!(
        shown(&set, k, l, &by(&[DueWindow::NextMonth])).len(),
        2,
        "o concluído não está em janela nenhuma"
    );
    assert!(shown(&set, k, l, &by(&[DueWindow::NoDates])).is_empty());
    let f = Filter {
        complete: true,
        ..Filter::default()
    };
    assert_eq!(shown(&set, k, l, &f), ["Timing notes"]);
    // Entre categorias é sempre E; dentro, qualquer/exacta.
    let f = Filter {
        labels: [anim, story].into(),
        due: [DueWindow::NextDay].into(),
        ..Filter::default()
    };
    assert_eq!(shown(&set, k, l, &f), ["Clean up shot 08"]);
    let f = Filter {
        labels: [anim, story].into(),
        mode: MatchMode::Exact,
        ..Filter::default()
    };
    assert_eq!(shown(&set, k, l, &f), ["Clean up shot 08"]);
    let f = Filter {
        no_labels: true,
        ..Filter::default()
    };
    assert_eq!(shown(&set, k, l, &f), ["Timing notes", "Sound pass"]);
    assert_eq!(f.selected(), 1);
}

#[test]
fn next_month_is_a_calendar_month_in_the_zone() {
    let z = utc();
    // 31/jan 12:00 → o fim da janela é 28/fev 12:00 (2027 não é bissexto).
    let jan31 =
        ph2d_kanban_dates::instant(CivilDate::new(2027, 1, 31).unwrap(), 12, 0, &z).unwrap();
    let feb28 =
        ph2d_kanban_dates::instant(CivilDate::new(2027, 2, 28).unwrap(), 12, 0, &z).unwrap();
    let mut c = Card::new(
        ph2d_kanban_model::CardId(1),
        ListId(1),
        ph2d_kanban_model::FracKey::between(None, None),
        "x",
        Moment(0),
    );
    c.due = due(feb28, false);
    assert!(due_in(DueWindow::NextMonth, &c, Moment(jan31), &z));
    c.due = due(feb28 + 60_000, false);
    assert!(!due_in(DueWindow::NextMonth, &c, Moment(jan31), &z));
}

#[test]
fn activity_windows_read_the_cards_last_activity() {
    let (mut set, k, l, _) = scene();
    let f = |w| Filter {
        activity: [w].into(),
        ..Filter::default()
    };
    assert_eq!(
        shown(&set, k, l, &f(ActivityWindow::LastWeek)),
        ["Animate shot 12", "Sound pass"]
    );
    assert_eq!(
        shown(&set, k, l, &f(ActivityWindow::IdleFourWeeks)),
        ["Timing notes"]
    );
    // Um gesto registado hoje no cartão velho tira-o do «sem atividade».
    let id = set.get(k).unwrap().cards_in(l)[2].id;
    let mut c = set.get(k).unwrap().cards[&id].clone();
    c.title = "Timing notes v2".to_owned();
    let actor = Actor { by: me(), at: NOW };
    ph2d_kanban_model::apply_logged(
        set.doc_mut(k).unwrap(),
        vec![KanbanOp::put(Record::card(c))],
        &actor,
    );
    assert!(shown(&set, k, l, &f(ActivityWindow::IdleFourWeeks)).is_empty());
}

#[test]
fn a_mirror_passes_when_its_original_does_with_labels_matched_by_name() {
    let (mut set, a, la, [anim, _]) = scene();
    let b = set.create("Other");
    let lb = add_list(&mut set, b, "Inbox");
    let anim_b = add_label(&mut set, b, "Animação", Hue::Green);
    let of = CardRef {
        kanban: a,
        card: set.get(a).unwrap().cards_in(la)[0].id,
    };
    let at = set.place_at(b, lb, 0, None).unwrap();
    for (k, ops) in mirror_card(&set, of, &at).unwrap().into_steps() {
        set.apply(k, ops);
    }
    let f = Filter {
        labels: [anim_b].into(),
        ..Filter::default()
    };
    assert_eq!(shown(&set, b, lb, &f), ["Animate shot 12"]);
    let f = Filter {
        labels: [anim].into(),
        mode: MatchMode::Exact,
        ..Filter::default()
    };
    assert_eq!(shown(&set, a, la, &f).len(), 2);
}
