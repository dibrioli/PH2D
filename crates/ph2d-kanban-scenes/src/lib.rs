//! `ph2d-kanban-scenes` — as cenas de smoke do Kanban como modelo puro (o «agora» por argumento):
//! o «Short film — production» da maquete, a capa gerada e o escritor [`Scene`]. Quem as usa: o
//! `PH2D_KANBAN_SMOKE`, a cena 30 do Quadro e o `PH2D_CODE_SMOKE=10` do editor de scripts.

pub mod script;
mod sky;

pub use sky::sky_png;

use ph2d_kanban_model::{
    Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist, ChecklistId,
    Comment, CommentId, Cover, CoverFill, Due, FracKey, Hue, KanbanId, KanbanOp, KanbanSet, Label,
    LabelColor, LabelId, Link, List, ListId, Member, Meta, Moment, Record, Shade,
};

/// O título do Kanban da cena 1.
pub const SHORT_FILM: &str = "Short film \u{2014} production";

/// Quem escreve uma cena: ids em sequência e as chaves de ordem por ordem de chegada.
pub struct Scene {
    ops: Vec<KanbanOp>,
    next: u64,
    pos: Option<FracKey>,
}

impl Scene {
    /// Um escritor cujo primeiro id é `after + 1`.
    #[must_use]
    pub fn new(after: u64) -> Self {
        Self {
            ops: Vec::new(),
            next: after,
            pos: None,
        }
    }

    /// As operações escritas, por ordem.
    #[must_use]
    pub fn into_ops(self) -> Vec<KanbanOp> {
        self.ops
    }

    pub fn id(&mut self) -> u64 {
        self.next += 1;
        self.next
    }

    pub fn pos(&mut self) -> FracKey {
        let p = FracKey::between(self.pos.as_ref(), None);
        self.pos = Some(p.clone());
        p
    }

    pub fn put(&mut self, r: Record) {
        self.ops.push(KanbanOp::put(r));
    }

    pub fn label(&mut self, name: &str, hue: Hue, shade: Shade) -> LabelId {
        let id = LabelId(self.id());
        self.put(Record::Label(Label {
            id,
            name: name.into(),
            color: Some(LabelColor { hue, shade }),
            meta: Meta::default(),
        }));
        id
    }

    pub fn list(&mut self, title: &str, color: Option<Hue>, limit: Option<u32>) -> ListId {
        let id = ListId(self.id());
        let pos = self.pos();
        self.put(Record::List(List {
            id,
            title: title.into(),
            pos,
            color,
            card_limit: limit,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        }));
        id
    }

    pub fn card(&mut self, list: ListId, title: &str, f: impl FnOnce(&mut Card)) -> CardId {
        let id = CardId(self.id());
        let mut c = Card {
            id,
            list,
            pos: self.pos(),
            title: title.into(),
            description: String::new(),
            labels: Vec::new(),
            members: Vec::new(),
            start: None,
            due: None,
            cover: None,
            fields: std::collections::BTreeMap::new(),
            location: String::new(),
            votes: Vec::new(),
            created: Moment(0),
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
            origin: None,
            template: false,
        };
        f(&mut c);
        self.put(Record::Card(c.into()));
        id
    }

    pub fn comments(&mut self, card: CardId, n: u32) {
        for i in 0..n {
            let id = CommentId(self.id());
            self.put(Record::Comment(Comment {
                id,
                card,
                author: Member("Enio".into()),
                at: Moment(0),
                text: format!("Note {i}"),
                edited: None,
                reactions: Vec::new(),
                meta: Meta::default(),
            }));
        }
    }

    pub fn attachments(&mut self, card: CardId, n: u32) {
        for i in 0..n {
            let id = AttachmentId(self.id());
            let pos = self.pos();
            self.put(Record::Attachment(Attachment {
                id,
                card,
                pos,
                name: format!("ref-{i}.png"),
                link: Link::Url(format!("https://example.com/{i}")),
                at: Moment(0),
                meta: Meta::default(),
            }));
        }
    }

    pub fn file(&mut self, card: CardId, name: &str, f: ph2d_kanban_model::FileId) {
        let id = AttachmentId(self.id());
        let pos = self.pos();
        self.put(Record::Attachment(Attachment {
            id,
            card,
            pos,
            name: name.into(),
            link: Link::File(f),
            at: Moment(0),
            meta: Meta::default(),
        }));
    }

    pub fn checklist(&mut self, card: CardId, done: u32, all: u32) {
        let id = ChecklistId(self.id());
        let pos = self.pos();
        self.put(Record::Checklist(Checklist {
            id,
            card,
            pos,
            title: "Checklist".into(),
            hide_checked: false,
            meta: Meta::default(),
        }));
        for i in 0..all {
            let item = CheckItemId(self.id());
            let pos = self.pos();
            self.put(Record::CheckItem(CheckItem {
                id: item,
                checklist: id,
                pos,
                text: format!("Step {i}"),
                done: i < done,
                member: None,
                due: None,
                meta: Meta::default(),
            }));
        }
    }
}

#[must_use]
pub fn people(names: &[&str]) -> Vec<Member> {
    names.iter().map(|n| Member((*n).into())).collect()
}

/// Cria o Kanban `name` em `set` e escreve-o num só lote.
pub fn write(set: &mut KanbanSet, name: &str, f: impl FnOnce(&mut Scene)) -> KanbanId {
    let id = set.create(name);
    let Some(k) = set.doc_mut(id) else {
        return id;
    };
    let mut s = Scene::new(k.mint() - 1);
    f(&mut s);
    ph2d_kanban_model::apply_batch(k, s.into_ops());
    id
}

/// A lista «Review» do Kanban `id` (a que a cena 1 recolhe).
#[must_use]
pub fn review_list(set: &KanbanSet, id: KanbanId) -> Option<ListId> {
    set.get(id)?
        .lists
        .values()
        .find(|l| l.title == "Review")
        .map(|l| l.id)
}

/// ⭐ O Kanban da cena 1 em `set` e a sua «Review».
pub fn short_film_into(set: &mut KanbanSet, now: Moment) -> (KanbanId, Option<ListId>) {
    let id = short_film(set, now);
    (id, review_list(set, id))
}

/// A cena 1: o quadro da maquete aprovada. Os prazos contam-se a partir de `now`.
pub fn short_film(set: &mut KanbanSet, now: Moment) -> KanbanId {
    const HOUR: i64 = 3_600_000;
    let now = now.0;
    let (size, png) = sky_png();
    let sky = set.put_file(png, Some(size));
    write(set, SHORT_FILM, |s| {
        let writing = s.label("Writing", Hue::Green, Shade::Normal);
        let audio = s.label("Audio", Hue::Sky, Shade::Normal);
        let later = s.label("Later", Hue::Black, Shade::Subtle);
        let story = s.label("Story", Hue::Purple, Shade::Normal);
        let urgent = s.label("Urgent", Hue::Red, Shade::Bold);
        let character = s.label("Character", Hue::Pink, Shade::Normal);
        let look = s.label("Look", Hue::Orange, Shade::Subtle);
        let animation = s.label("Animation", Hue::Green, Shade::Bold);
        let light = s.label("Light", Hue::Lime, Shade::Normal);
        let backlog = s.list("Backlog", None, None);
        let todo = s.list("To Do", None, None);
        let doing = s.list("Doing", Some(Hue::Blue), Some(3));
        let review = s.list("Review", None, None);
        let done = s.list("Done", None, None);
        s.card(backlog, "Write the logline", |c| {
            c.labels = vec![writing];
            c.description = "One sentence.".into();
        });
        let refs = s.card(backlog, "Collect references for the forest", |c| {
            c.description = "Moodboard.".into()
        });
        s.attachments(refs, 6);
        s.comments(refs, 1);
        s.card(backlog, "Pick the music", |c| {
            c.labels = vec![audio];
            c.members = people(&["Mia"]);
        });
        s.card(backlog, "Voice casting", |c| c.labels = vec![audio, later]);
        let sb = s.card(todo, "Storyboard act 1", |c| {
            c.labels = vec![story, urgent];
            c.description = "Panels 1\u{2013}40.".into();
            c.members = people(&["Rui", "Ana"]);
        });
        s.comments(sb, 4);
        s.attachments(sb, 2);
        s.checklist(sb, 2, 6);
        let fox = s.card(todo, "Design the fox", |c| {
            c.labels = vec![character];
            c.members = people(&["Ana"]);
        });
        s.comments(fox, 3);
        s.attachments(fox, 2);
        let sheet = s.card(todo, "Fox concept sheet", |c| {
            c.labels = vec![character, look];
            c.members = people(&["Ana"]);
            c.cover = Some(Cover {
                fill: CoverFill::Image(sky),
                full: false,
            });
            c.due = Some(Due {
                at: Moment(now - 2 * 24 * HOUR),
                done: false,
                remind_before_min: None,
            });
        });
        s.file(sheet, "fox_sky.png", sky);
        let cs = s.card(todo, "Color script", |c| {
            c.labels = vec![look];
            c.cover = Some(Cover {
                fill: CoverFill::Color(Hue::Orange),
                full: false,
            });
        });
        s.checklist(cs, 0, 4);
        let shot = s.card(doing, "Animate shot 12", |c| {
            c.labels = vec![animation];
            c.members = people(&["Enio"]);
        });
        s.checklist(shot, 3, 5);
        s.card(doing, "Clean up shot 08", |c| {
            c.labels = vec![animation, story];
            c.members = people(&["Mia"]);
            c.due = Some(Due {
                at: Moment(now + 5 * HOUR),
                done: false,
                remind_before_min: Some(60),
            });
        });
        s.card(doing, "Light pass \u{2014} forest", |c| {
            c.labels = vec![light];
            c.votes = people(&["Rui", "Enio"]);
            c.members = people(&["Rui", "Enio", "Mia"]);
            c.location = "Studio B".into();
        });
        for t in ["Fox walk cycle", "Rain test", "Title card", "Sound sync"] {
            s.card(review, t, |_| {});
        }
        s.card(done, "Rough the opening shot", |c| {
            c.cover = Some(Cover {
                fill: CoverFill::Color(Hue::Sky),
                full: false,
            })
        });
        let lock = s.card(done, "Lock the script", |c| {
            c.labels = vec![writing];
            c.members = people(&["Enio", "Rui"]);
            c.due = Some(Due {
                at: Moment(now - 4 * 24 * HOUR),
                done: true,
                remind_before_min: None,
            });
        });
        s.checklist(lock, 8, 8);
        s.card(done, "Pick the aspect ratio", |c| {
            c.labels = vec![writing, later]
        });
    })
}

#[cfg(test)]
mod tests;
