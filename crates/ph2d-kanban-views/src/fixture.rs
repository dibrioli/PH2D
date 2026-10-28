//! As cenas das vistas para os testes e as fotos: a **cena 1** do Kanban («Short film —
//! production», as mesmas listas, etiquetas, cartões e membros de `ph2d-app-kanban/src/smoke.rs`,
//! que esta crate não pode chamar — o app depende dela) com datas de início e entrega espalhadas
//! pelo mês para o calendário e a linha do tempo, dois campos personalizados para a tabela, e um
//! quadro de `n` cartões para medir.

use std::collections::BTreeMap;

use ph2d_kanban_model::{
    Card, CardId, Due, FieldDef, FieldId, FieldKind, FieldOption, FieldValue, FracKey, Hue,
    KanbanId, KanbanOp, KanbanSet, Label, LabelColor, LabelId, List, ListId, Member, Meta, Moment,
    OptionId, Record, Shade,
};

pub const HOUR: i64 = 3_600_000;
pub const DAY: i64 = 24 * HOUR;

/// Quem escreve uma cena: ids em sequência e as chaves de ordem por ordem de chegada.
pub struct Scene {
    pub ops: Vec<KanbanOp>,
    next: u64,
    pos: Option<FracKey>,
    now: i64,
}

impl Scene {
    fn id(&mut self) -> u64 {
        self.next += 1;
        self.next
    }

    fn pos(&mut self) -> FracKey {
        let p = FracKey::between(self.pos.as_ref(), None);
        self.pos = Some(p.clone());
        p
    }

    pub fn label(&mut self, name: &str, hue: Hue, shade: Shade) -> LabelId {
        let id = LabelId(self.id());
        self.ops.push(KanbanOp::put(Record::Label(Label {
            id,
            name: name.into(),
            color: Some(LabelColor { hue, shade }),
            meta: Meta::default(),
        })));
        id
    }

    pub fn list(&mut self, title: &str, color: Option<Hue>, limit: Option<u32>) -> ListId {
        let id = ListId(self.id());
        let pos = self.pos();
        self.ops.push(KanbanOp::put(Record::List(List {
            id,
            title: title.into(),
            pos,
            color,
            card_limit: limit,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        })));
        id
    }

    pub fn field(&mut self, name: &str, kind: FieldKind) -> FieldId {
        let id = FieldId(self.id());
        let pos = self.pos();
        self.ops.push(KanbanOp::put(Record::Field(FieldDef {
            id,
            name: name.into(),
            kind,
            on_front: false,
            pos,
            meta: Meta::default(),
        })));
        id
    }

    pub fn card(&mut self, list: ListId, title: &str, f: impl FnOnce(&mut Card, i64)) -> CardId {
        let id = CardId(self.id());
        let mut c = Card::new(id, list, self.pos(), title, Moment(self.now - 30 * DAY));
        f(&mut c, self.now);
        self.ops.push(KanbanOp::put(Record::Card(c)));
        id
    }
}

fn people(names: &[&str]) -> Vec<Member> {
    names.iter().map(|n| Member((*n).into())).collect()
}

fn due(at: i64, done: bool) -> Option<Due> {
    Some(Due {
        at: Moment(at),
        done,
        remind_before_min: None,
    })
}

/// Escreve um Kanban novo em `set` com `f`.
pub fn write(set: &mut KanbanSet, name: &str, now: Moment, f: impl FnOnce(&mut Scene)) -> KanbanId {
    let id = set.create(name);
    let Some(k) = set.doc_mut(id) else {
        return id;
    };
    let mut s = Scene {
        ops: Vec::new(),
        next: k.mint() - 1,
        pos: None,
        now: now.0,
    };
    f(&mut s);
    ph2d_kanban_model::apply_batch(k, s.ops);
    id
}

/// ⭐ A cena 1 com datas, a contar de `now` (às 10:00 de um dia, em UTC, para as fotos).
#[allow(clippy::too_many_lines)]
pub fn short_film(set: &mut KanbanSet, now: Moment) -> KanbanId {
    write(set, "Short film \u{2014} production", now, |s| {
        let writing = s.label("Writing", Hue::Green, Shade::Normal);
        let audio = s.label("Audio", Hue::Sky, Shade::Normal);
        let later = s.label("Later", Hue::Black, Shade::Subtle);
        let story = s.label("Story", Hue::Purple, Shade::Normal);
        let urgent = s.label("Urgent", Hue::Red, Shade::Bold);
        let character = s.label("Character", Hue::Pink, Shade::Normal);
        let look = s.label("Look", Hue::Orange, Shade::Subtle);
        let animation = s.label("Animation", Hue::Green, Shade::Bold);
        let light = s.label("Light", Hue::Lime, Shade::Normal);
        let len = s.field("Shot length", FieldKind::Number);
        let prio = s.field(
            "Priority",
            FieldKind::Dropdown(vec![
                FieldOption {
                    id: OptionId(1),
                    text: "High".into(),
                    color: Some(Hue::Red),
                },
                FieldOption {
                    id: OptionId(2),
                    text: "Normal".into(),
                    color: None,
                },
            ]),
        );
        let backlog = s.list("Backlog", None, None);
        let todo = s.list("To Do", None, None);
        let doing = s.list("Doing", Some(Hue::Blue), Some(3));
        let review = s.list("Review", None, None);
        let done = s.list("Done", None, None);
        s.card(backlog, "Write the logline", |c, now| {
            c.labels = vec![writing];
            c.due = due(now + 9 * DAY, false);
        });
        s.card(backlog, "Collect references for the forest", |_, _| {});
        s.card(backlog, "Pick the music", |c, now| {
            c.labels = vec![audio];
            c.members = people(&["Mia"]);
            c.start = Some(Moment(now + 6 * DAY));
            c.due = due(now + 12 * DAY + 7 * HOUR, false);
        });
        s.card(backlog, "Voice casting", |c, _| c.labels = vec![audio, later]);
        s.card(todo, "Storyboard act 1", |c, now| {
            c.labels = vec![story, urgent];
            c.members = people(&["Rui", "Ana"]);
            c.start = Some(Moment(now - 3 * DAY));
            c.due = due(now + 2 * DAY + 7 * HOUR, false);
            c.fields = BTreeMap::from([(prio, FieldValue::Option(OptionId(1)))]);
        });
        s.card(todo, "Design the fox", |c, now| {
            c.labels = vec![character];
            c.members = people(&["Ana"]);
            c.start = Some(Moment(now - DAY));
            c.due = due(now + 4 * DAY, false);
        });
        s.card(todo, "Fox concept sheet", |c, now| {
            c.labels = vec![character, look];
            c.members = people(&["Ana"]);
            c.due = due(now - 2 * DAY, false);
        });
        s.card(todo, "Color script", |c, now| {
            c.labels = vec![look];
            c.start = Some(Moment(now + 3 * DAY));
            c.due = due(now + 8 * DAY, false);
        });
        s.card(doing, "Animate shot 12", |c, now| {
            c.labels = vec![animation];
            c.members = people(&["Enio"]);
            c.start = Some(Moment(now - 5 * DAY));
            c.due = due(now - DAY, false);
            c.fields = BTreeMap::from([
                (len, FieldValue::Number(4.5)),
                (prio, FieldValue::Option(OptionId(1))),
            ]);
        });
        s.card(doing, "Clean up shot 08", |c, now| {
            c.labels = vec![animation, story];
            c.members = people(&["Mia"]);
            c.due = due(now + 5 * HOUR, false);
            c.fields = BTreeMap::from([(len, FieldValue::Number(2.0))]);
        });
        s.card(doing, "Light pass \u{2014} forest", |c, now| {
            c.labels = vec![light];
            c.members = people(&["Rui", "Enio", "Mia"]);
            c.start = Some(Moment(now));
            c.due = due(now + 6 * DAY, false);
            c.fields = BTreeMap::from([(prio, FieldValue::Option(OptionId(2)))]);
        });
        for (i, t) in ["Fox walk cycle", "Rain test", "Title card", "Sound sync"]
            .into_iter()
            .enumerate()
        {
            s.card(review, t, |c, now| {
                if i < 2 {
                    c.due = due(now + 2 * DAY + (i as i64) * 2 * HOUR, false);
                    c.members = people(&["Mia"]);
                }
            });
        }
        s.card(done, "Rough the opening shot", |c, now| {
            c.start = Some(Moment(now - 12 * DAY));
            c.due = due(now - 9 * DAY, true);
        });
        s.card(done, "Lock the script", |c, now| {
            c.labels = vec![writing];
            c.members = people(&["Enio", "Rui"]);
            c.start = Some(Moment(now - 8 * DAY));
            c.due = due(now - 4 * DAY, true);
        });
        s.card(done, "Pick the aspect ratio", |c, _| c.labels = vec![writing, later]);
    })
}

/// Um quadro de `n` cartões em 8 listas, com etiquetas, membros e datas a cada poucos — para medir
/// a tabela, o calendário e o painel no tamanho que o dono pode ter.
pub fn big(set: &mut KanbanSet, n: usize, now: Moment) -> KanbanId {
    write(set, "Big board", now, |s| {
        let tag = s.label("Shot", Hue::Purple, Shade::Normal);
        let lists: Vec<ListId> = (0..8).map(|l| s.list(&format!("Sequence {}", l + 1), None, None)).collect();
        for i in 0..n {
            let list = lists[i % lists.len()];
            s.card(list, &format!("Shot {:05} \u{2014} layout, anim, light", i + 1), |c, now| {
                if i % 3 == 0 {
                    c.labels = vec![tag];
                }
                if i % 5 == 0 {
                    c.members = people(&["Enio"]);
                }
                if i % 2 == 0 {
                    let d = (i as i64 % 60) - 30;
                    c.start = Some(Moment(now + d * DAY));
                    c.due = due(now + (d + 3) * DAY, i % 7 == 0);
                }
            });
        }
    })
}
