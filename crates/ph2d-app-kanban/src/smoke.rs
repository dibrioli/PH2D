//! ⭐ **As cenas de smoke do Kanban** — `PH2D_KANBAN_SMOKE=<n>` abre, no arranque, um Kanban montado
//! numa aba. Roteador PRÓPRIO (o do Quadro é o `PH2D_BOARD_SMOKE`); a próxima cena é a `CENAS + 1`.
//!
//! - **1** — o quadro da maquete aprovada (27/10): «Short film — production», cinco listas, a
//!   «Doing» azul e cheia (limite `3 / 3`), a «Review» recolhida, e cartões com todas as variantes
//!   da frente que a W2 pinta. ⚠️ É a maquete SEM o que chega depois: prazos, capas de imagem e
//!   campos personalizados (W3) e o cartão envelhecido (W6) — esses cartões aparecem sem eles.
//! - **2** — um quadro grande (8 listas × 250 cartões) para sentir o arrasto a 60 fps.

use crate::docs::KanbanDocs;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_kanban_model::{
    Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist, ChecklistId,
    Comment, CommentId, Cover, CoverFill, FracKey, Hue, KanbanId, KanbanOp, KanbanSet, Label,
    LabelColor, LabelId, Link, List, ListId, Member, Meta, Moment, Record, Shade,
};

/// As cenas que existem.
pub const CENAS: u8 = 2;
pub const ENV: &str = "PH2D_KANBAN_SMOKE";

/// O roteador que a família declara ao registo (o gate de colisão lê-o).
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: ENV,
    max_level: CENAS as u32,
}];

/// O nível pedido pela env (`None` = nenhum, ou fora de `1..=CENAS`).
#[must_use]
pub fn armed_level() -> Option<u8> {
    std::env::var(ENV)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=CENAS).contains(n))
}

/// Monta a cena pedida pela env (inerte sem ela). Chamar depois de instalar a família.
pub fn stage_armed_smokes(hero: &mut HeroScreen) {
    if let Some(n) = armed_level() {
        stage(hero, n);
    }
}

/// Monta a cena `n` no projecto aberto e abre-a numa aba.
pub fn stage(hero: &mut HeroScreen, n: u8) -> Option<KanbanId> {
    let shared = hero.documents.kanbans_shared();
    let id = {
        let mut set = shared.borrow_mut();
        match n {
            1 => short_film(&mut set),
            2 => big(&mut set),
            _ => return None,
        }
    };
    let docs = hero.documents.surfaces_mut().find_mut::<KanbanDocs>()?;
    docs.follow_project(shared);
    docs.open(id);
    let review = review_list(&docs.set.borrow(), id);
    if n == 1
        && let Some(review) = review
        && let Some(t) = docs.tab_mut(id.0)
    {
        t.view.collapsed.insert(review);
    }
    Some(id)
}

fn review_list(set: &KanbanSet, id: KanbanId) -> Option<ListId> {
    set.get(id)?
        .lists
        .values()
        .find(|l| l.title == "Review")
        .map(|l| l.id)
}

/// Quem escreve uma cena: ids em sequência e as chaves de ordem por ordem de chegada.
struct Scene {
    ops: Vec<KanbanOp>,
    next: u64,
    pos: Option<FracKey>,
}

impl Scene {
    fn new(after: u64) -> Self {
        Self {
            ops: Vec::new(),
            next: after,
            pos: None,
        }
    }

    fn id(&mut self) -> u64 {
        self.next += 1;
        self.next
    }

    fn pos(&mut self) -> FracKey {
        let p = FracKey::between(self.pos.as_ref(), None);
        self.pos = Some(p.clone());
        p
    }

    fn put(&mut self, r: Record) {
        self.ops.push(KanbanOp::put(r));
    }

    fn label(&mut self, name: &str, hue: Hue, shade: Shade) -> LabelId {
        let id = LabelId(self.id());
        self.put(Record::Label(Label {
            id,
            name: name.into(),
            color: Some(LabelColor { hue, shade }),
            meta: Meta::default(),
        }));
        id
    }

    fn list(&mut self, title: &str, color: Option<Hue>, limit: Option<u32>) -> ListId {
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
        }));
        id
    }

    fn card(&mut self, list: ListId, title: &str, f: impl FnOnce(&mut Card)) -> CardId {
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
        };
        f(&mut c);
        self.put(Record::Card(c));
        id
    }

    fn comments(&mut self, card: CardId, n: u32) {
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

    fn attachments(&mut self, card: CardId, n: u32) {
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

    fn checklist(&mut self, card: CardId, done: u32, all: u32) {
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

fn people(names: &[&str]) -> Vec<Member> {
    names.iter().map(|n| Member((*n).into())).collect()
}

fn write(set: &mut KanbanSet, name: &str, f: impl FnOnce(&mut Scene)) -> KanbanId {
    let id = set.create(name);
    let Some(k) = set.doc_mut(id) else {
        return id;
    };
    let mut s = Scene::new(k.mint() - 1);
    f(&mut s);
    ph2d_kanban_model::apply_batch(k, s.ops);
    id
}

/// A cena 1: o quadro da maquete.
fn short_film(set: &mut KanbanSet) -> KanbanId {
    write(set, "Short film \u{2014} production", |s| {
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
        });
        s.checklist(lock, 8, 8);
        s.card(done, "Pick the aspect ratio", |c| {
            c.labels = vec![writing, later]
        });
    })
}

/// A cena 2: um quadro grande.
fn big(set: &mut KanbanSet) -> KanbanId {
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
