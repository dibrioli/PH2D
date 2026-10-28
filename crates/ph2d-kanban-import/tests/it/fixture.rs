//! O que a fixture sintética tem de dar, campo a campo (o mapa em `fixtures/…synthetic.md`).

use ph2d_kanban_import::{DirFiles, LossKind, import_trello};
use ph2d_kanban_model::{
    Background, Card, CoverFill, FieldKind, FieldValue, Happened, Hue, Kanban, KanbanId, Link,
    Member, Moment, Shade, Subject,
};

use crate::round_trip::tempdir;
use crate::{fixture_bytes, imported};

fn ms(iso: &str) -> Moment {
    Moment(
        iso.parse::<jiff::Timestamp>()
            .expect("iso")
            .as_millisecond(),
    )
}

fn card<'a>(k: &'a Kanban, title: &str) -> &'a Card {
    k.cards.values().find(|c| c.title == title).expect(title)
}

fn titles(k: &Kanban, list: &str) -> Vec<String> {
    let l = k.lists.values().find(|l| l.title == list).expect(list);
    k.cards_in(l.id).iter().map(|c| c.title.clone()).collect()
}

fn m(s: &str) -> Member {
    Member(s.to_owned())
}

#[test]
fn import_fixture_counts_and_order() {
    let imp = imported();
    let k = &imp.kanban;
    let c = imp.report.counts;
    assert_eq!((c.lists, c.cards, c.labels, c.members), (4, 9, 32, 3));
    assert_eq!(
        (c.checklists, c.items, c.attachments, c.fields),
        (3, 6, 3, 5)
    );
    assert_eq!((c.comments, c.activities, c.files), (2, 24, 0));
    let lists: Vec<_> = k.lists_in_order().iter().map(|l| l.title.clone()).collect();
    assert_eq!(lists, ["A fazer", "Fazendo", "Feito"]);
    assert!(
        k.lists
            .values()
            .any(|l| l.title == "Arquivo antigo" && l.archived)
    );
    assert_eq!(
        titles(k, "A fazer"),
        ["Roteiro, versão \"final\"", "Cena 4 (do item)"]
    );
    assert_eq!(
        titles(k, "Fazendo"),
        ["Modelo: cena nova", "Animatic"],
        "empate: ordem do ficheiro"
    );
    assert_eq!(
        titles(k, "Feito"),
        ["──── Separador ────", "Trilha sonora", "Fundo da cena 7"]
    );
    assert!(card(k, "Rascunho descartado").archived);
    let fazendo = k.lists.values().find(|l| l.title == "Fazendo").expect("l");
    assert_eq!(
        (
            fazendo.card_limit,
            k.lists
                .values()
                .find(|l| l.title == "A fazer")
                .and_then(|l| l.color)
        ),
        (Some(3), Some(Hue::Sky))
    );
    assert_eq!(k.header.background, Background::Gradient(1));
    assert!(k.header.starred);
}

#[test]
fn import_fixture_maps_card_fields() {
    let k = imported().kanban;
    let c = card(&k, "Roteiro, versão \"final\"");
    let label_names: Vec<_> = c.labels.iter().map(|l| k.labels[l].name.clone()).collect();
    assert_eq!(
        label_names,
        ["Bug", "Arte", "Urgente", "", "Sem cor", "Cor estranha"]
    );
    let urgent = k
        .labels
        .values()
        .find(|l| l.name == "Urgente")
        .and_then(|l| l.color)
        .expect("c");
    assert_eq!((urgent.hue, urgent.shade), (Hue::Yellow, Shade::Bold));
    assert!(
        k.labels
            .values()
            .any(|l| l.name == "Cor estranha" && l.color.is_none())
    );
    assert_eq!(c.members, [m("ana"), m("bruno")]);
    assert_eq!(c.votes, [m("ana")]);
    assert_eq!(c.start, Some(ms("2026-09-05T00:00:00Z")));
    let due = c.due.expect("due");
    assert_eq!(
        (due.at, due.done, due.remind_before_min),
        (ms("2026-10-30T15:00:00Z"), false, Some(1440))
    );
    assert_eq!(
        c.cover.map(|c| (c.fill, c.full)),
        Some((CoverFill::Color(Hue::Purple), false))
    );
    assert_eq!(c.location, "Estúdio 2, Lisboa");
    assert_eq!(c.created, ms("2026-09-02T10:00:00Z"));
    assert!(c.description.contains("**terceiro acto**"));
    let by_name = |n: &str| k.fields.values().find(|f| f.name == n).expect(n);
    assert_eq!(
        c.fields[&by_name("Cliente").id],
        FieldValue::Text("Canal Lua".into())
    );
    assert_eq!(c.fields[&by_name("Horas").id], FieldValue::Number(12.5));
    assert_eq!(
        c.fields[&by_name("Entrega final").id],
        FieldValue::Date(ms("2026-12-01T18:00:00Z"))
    );
    assert_eq!(
        c.fields[&by_name("Aprovado").id],
        FieldValue::Checkbox(true)
    );
    let FieldKind::Dropdown(opts) = &by_name("Prioridade").kind else {
        panic!("dropdown")
    };
    let texts: Vec<_> = opts.iter().map(|o| (o.text.as_str(), o.color)).collect();
    assert_eq!(
        texts,
        [
            ("Alta", Some(Hue::Red)),
            ("Média", Some(Hue::Yellow)),
            ("Baixa", None)
        ]
    );
    assert_eq!(
        c.fields[&by_name("Prioridade").id],
        FieldValue::Option(opts[0].id)
    );
    assert_eq!(c.fields.len(), 5);
    let atts: Vec<_> = k.attachments.values().filter(|a| a.card == c.id).collect();
    assert!(
        atts.iter().all(|a| matches!(a.link, Link::Url(_))),
        "sem pasta, ficheiros são links"
    );
    assert!(card(&k, "Modelo: cena nova").template);
    assert_eq!(card(&k, "Modelo: cena nova").cover, None);
    let anim = card(&k, "Animatic");
    assert_eq!(
        anim.cover.map(|c| (c.fill, c.full)),
        Some((CoverFill::Color(Hue::Green), true))
    );
    assert!(anim.due.is_some_and(|d| d.done));
    assert_eq!(card(&k, "Fundo da cena 7").labels.len(), 19);
    assert_eq!(card(&k, "Cena 4 (do item)").origin, Some(c.id));
}

#[test]
fn import_fixture_maps_checklists_comments_and_activity() {
    let k = imported().kanban;
    let c = card(&k, "Roteiro, versão \"final\"");
    let mut cls: Vec<_> = k.checklists.values().filter(|cl| cl.card == c.id).collect();
    cls.sort_by(|a, b| a.pos.cmp(&b.pos));
    assert_eq!(
        cls.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
        ["Revisão", "Cenas"]
    );
    let mut items: Vec<_> = k
        .items
        .values()
        .filter(|i| i.checklist == cls[1].id)
        .collect();
    items.sort_by(|a, b| a.pos.cmp(&b.pos));
    let got: Vec<_> = items
        .iter()
        .map(|i| (i.text.as_str(), i.done, i.member.clone(), i.due))
        .collect();
    assert_eq!(
        got,
        [
            ("Cena 1", true, Some(m("ana")), None),
            ("Cena 3", false, None, None),
            (
                "Cena 7",
                false,
                Some(m("bruno")),
                Some(ms("2026-10-15T09:00:00Z"))
            ),
        ]
    );
    let mut comments: Vec<_> = k.comments.values().collect();
    comments.sort_by_key(|c| c.at);
    assert_eq!(comments[0].author, m("carla"));
    assert_eq!(comments[0].text, "@bruno veja o **acto 3**, e @ana também.");
    assert_eq!(comments[0].edited, Some(ms("2026-09-05T10:02:30.500Z")));
    assert!(
        comments
            .iter()
            .all(|x| x.card == c.id && x.reactions.is_empty())
    );
    let of = |s: Subject| k.activities.values().filter(move |a| a.subject == s);
    let anim = card(&k, "Animatic").id;
    let whats: Vec<_> = of(Subject::Card(anim)).map(|a| a.what.clone()).collect();
    assert!(whats.contains(&Happened::Renamed {
        from: "Animatic v0".into()
    }));
    assert!(whats.iter().any(|w| matches!(w, Happened::Moved { .. })));
    assert!(whats.contains(&Happened::MemberAdded(m("carla"))));
    assert!(whats.contains(&Happened::DueDone(true)));
    let roteiro: Vec<_> = of(Subject::Card(c.id)).map(|a| a.what.clone()).collect();
    assert!(roteiro.contains(&Happened::DueSet(Some(ms("2026-10-30T15:00:00Z")))));
    assert!(roteiro.contains(&Happened::StartSet(Some(ms("2026-09-05T00:00:00Z")))));
    assert!(
        roteiro
            .iter()
            .any(|w| matches!(w, Happened::Detached { name, .. } if name == "velho.png"))
    );
    assert!(k.activities.values().any(|a| a.what == Happened::Deleted));
    assert!(of(Subject::Kanban).any(|a| a.what
        == Happened::Renamed {
            from: "Curta".into()
        }));
}

#[test]
fn import_fixture_reports_every_loss() {
    let r = imported().report;
    let n = |k| r.count(k);
    assert_eq!(
        (n(LossKind::PowerUp), n(LossKind::BoardDescription)),
        (2, 1)
    );
    assert_eq!(
        (n(LossKind::UnknownMember), n(LossKind::VoteWithoutMember)),
        (2, 1)
    );
    assert_eq!((n(LossKind::UnknownColor), n(LossKind::Orphan)), (1, 3));
    assert_eq!(
        (
            n(LossKind::CoverImage),
            n(LossKind::CardRole),
            n(LossKind::Subscription)
        ),
        (1, 1, 1)
    );
    assert_eq!(
        (n(LossKind::ActionWithoutAuthor), n(LossKind::BadValue)),
        (1, 3)
    );
    assert_eq!(n(LossKind::UnsupportedAction), 3);
    assert_eq!(n(LossKind::DuplicateId), 0);
}

#[test]
fn import_with_a_folder_brings_the_bytes_and_the_image_cover() {
    let dir = tempdir();
    std::fs::write(dir.join("storyboard.pdf"), b"%PDF storyboard").expect("w");
    let json: serde_json::Value = serde_json::from_slice(&fixture_bytes()).expect("json");
    let capa_id = json["cards"]
        .as_array()
        .and_then(|cs| cs.iter().find(|c| c["name"] == "Modelo: cena nova"))
        .and_then(|c| c["attachments"][0]["id"].as_str())
        .expect("id")
        .to_owned();
    std::fs::create_dir_all(dir.join(&capa_id)).expect("d");
    std::fs::write(dir.join(&capa_id).join("capa.png"), b"\x89PNG capa").expect("w");
    let imp =
        import_trello(&fixture_bytes(), KanbanId(1), Some(&DirFiles(dir.clone()))).expect("i");
    assert_eq!(imp.files.len(), 2);
    assert_eq!(imp.report.count(LossKind::CoverImage), 0);
    assert_eq!(imp.report.count(LossKind::AttachmentFileMissing), 0);
    let cover = card(&imp.kanban, "Modelo: cena nova").cover.expect("capa");
    assert!(matches!(cover.fill, CoverFill::Image(_)) && cover.full);
    std::fs::remove_file(dir.join("storyboard.pdf")).expect("rm");
    let imp =
        import_trello(&fixture_bytes(), KanbanId(1), Some(&DirFiles(dir.clone()))).expect("i");
    assert_eq!(imp.report.count(LossKind::AttachmentFileMissing), 1);
    let _ = std::fs::remove_dir_all(dir);
}
