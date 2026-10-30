//! As cenas 2 e 4 do Kanban como modelo puro (movidas da `ph2d-app-kanban`, 30/10): o quadro grande
//! e o verso rico de «Animate shot 12».

use crate::{Scene, people, write};
use ph2d_kanban_model::{
    Activity, ActivityId, CardId, Comment, CommentId, Cover, CoverFill, Due, FieldDef, FieldId,
    FieldKind, FieldOption, FieldValue, Happened, Hue, KanbanId, KanbanSet, Member, Meta, Moment,
    OptionId, Reaction, Record, Shade, Subject,
};

/// A cena 2: um quadro grande.
pub fn big(set: &mut KanbanSet) -> KanbanId {
    write(set, "Big board \u{2014} 2 000 cards", |s| {
        let tag = s.label("Shot", Hue::Purple, Shade::Normal);
        for l in 0..8 {
            let list = s.list(&format!("Sequence {}", l + 1), None, None);
            for c in 0..250 {
                s.card(
                    list,
                    &format!("Shot {:03} \u{2014} layout, anim, light", c + 1),
                    |card| {
                        if c % 3 == 0 {
                            card.labels = vec![tag];
                        }
                        if c % 5 == 0 {
                            card.members = people(&["Enio"]);
                        }
                    },
                );
            }
        }
    })
}

/// A cena 4: «Animate shot 12» com tudo o que o verso mostra (a maquete `Back.dc.html`). As horas
/// contam-se a partir de `now` (ms) (o prazo fica atrasado, o comentário «2 hours ago»).
pub fn rich_back(set: &mut KanbanSet, id: KanbanId, now: i64) -> Option<CardId> {
    const HOUR: i64 = 3_600_000;
    let k = set.doc_mut(id)?;
    let card = k
        .cards
        .values()
        .find(|c| c.title == "Animate shot 12")?
        .clone();
    let story = k.labels.values().find(|l| l.name == "Story")?.id;
    let doing = card.list;
    let todo = k.lists.values().find(|l| l.title == "To Do")?.id;
    let mut s = Scene::new(k.mint() - 1);
    let priority = FieldId(s.id());
    let high = OptionId(1);
    let estimate = FieldId(s.id());
    let shot_no = FieldId(s.id());
    for (f, name, kind) in [
        (
            priority,
            "Priority",
            FieldKind::Dropdown(vec![
                FieldOption {
                    id: OptionId(0),
                    text: "Low".into(),
                    color: Some(Hue::Green),
                },
                FieldOption {
                    id: high,
                    text: "High".into(),
                    color: Some(Hue::Orange),
                },
            ]),
        ),
        (estimate, "Estimate (days)", FieldKind::Number),
        (shot_no, "Shot", FieldKind::Text),
    ] {
        let pos = s.pos();
        s.put(Record::Field(FieldDef {
            id: f,
            name: name.into(),
            kind,
            on_front: false,
            pos,
            meta: Meta::default(),
        }));
    }
    let mut c = card.clone();
    c.labels.push(story);
    c.members = people(&["Enio", "Rui"]);
    c.cover = Some(Cover {
        fill: CoverFill::Color(Hue::Sky),
        full: false,
    });
    c.start = Some(Moment(now - 6 * 24 * HOUR));
    c.due = Some(Due {
        at: Moment(now - 3 * 24 * HOUR),
        done: false,
        remind_before_min: None,
    });
    c.description = "Fox jumps the log, lands, looks back. **Hold the look for 12 frames** before \
the cut \u{2014} the music hits there.\n\n- Reference: [fox_jump_ref.mp4](https://example.com/fox_jump_ref.mp4)\n\
- Timing sheet in the Storyboard tab\n"
        .into();
    c.fields.insert(priority, FieldValue::Option(high));
    c.fields.insert(estimate, FieldValue::Number(3.0));
    c.fields.insert(shot_no, FieldValue::Text("12".into()));
    s.put(Record::Card(c.into()));
    let lists: Vec<_> = k
        .checklists
        .values()
        .filter(|x| x.card == card.id)
        .cloned()
        .collect();
    for mut cl in lists {
        cl.title = "Animation pass".into();
        let items: Vec<_> = k
            .items
            .values()
            .filter(|i| i.checklist == cl.id)
            .cloned()
            .collect();
        let names = [
            "Blocking",
            "Key poses",
            "Breakdowns",
            "In-betweens",
            "Polish and overlap",
        ];
        for (mut it, name) in items.into_iter().zip(names) {
            it.text = name.into();
            match name {
                "In-betweens" => {
                    it.due = Some(Moment(now + 24 * HOUR));
                    it.member = Some(Member("Enio".into()));
                }
                "Polish and overlap" => it.member = Some(Member("Rui".into())),
                _ => {}
            }
            s.put(Record::CheckItem(it));
        }
        s.put(Record::Checklist(cl));
    }
    let comment = CommentId(s.id());
    s.put(Record::Comment(Comment {
        id: comment,
        card: card.id,
        author: Member("Rui".into()),
        at: Moment(now - 2 * HOUR),
        text: "@Ana can you check the timing on 12C? The landing feels one frame late.".into(),
        edited: None,
        reactions: vec![Reaction {
            emoji: "\u{1F44D}".into(),
            by: people(&["Ana", "Enio"]),
        }],
        meta: Meta::default(),
    }));
    let moved = ActivityId(s.id());
    s.put(Record::Activity(Activity {
        id: moved,
        subject: Subject::Card(card.id),
        by: Member("Ana".into()),
        at: Moment(now - 20 * HOUR),
        what: Happened::Moved {
            from: todo,
            to: doing,
        },
        meta: Meta::default(),
    }));
    ph2d_kanban_model::apply_batch(k, s.into_ops());
    Some(card.id)
}
