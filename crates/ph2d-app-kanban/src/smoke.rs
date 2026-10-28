//! ⭐ **As cenas de smoke do Kanban** — `PH2D_KANBAN_SMOKE=<n>` abre, no arranque, um Kanban montado
//! numa aba. Roteador PRÓPRIO (o do Quadro é o `PH2D_BOARD_SMOKE`); a próxima cena é a `CENAS + 1`.
//!
//! - **1** — o quadro da maquete aprovada (27/10): «Short film — production», cinco listas, a
//!   «Doing» azul e cheia (limite `3 / 3`), a «Review» recolhida, e cartões com todas as variantes
//!   da frente que a W2 pinta. ⚠️ É a maquete SEM o que chega depois: prazos, capas de imagem e
//!   campos personalizados (W3) e o cartão envelhecido (W6) — esses cartões aparecem sem eles.
//! - **2** — um quadro grande (8 listas × 250 cartões) para sentir o arrasto a 60 fps.
//! - **3** — o quadro da cena 1 com a «To Do» NA MÃO, a meio do caminho entre a «Doing» e a
//!   «Review»: inclinada, com a sombra de voo e os cartões dentro; as outras abriram-lhe o espaço.
//!   Mexer o rato leva-a; um clique larga-a ali.
//! - **4** — o quadro da cena 1 com o VERSO de «Animate shot 12» aberto (a maquete `Back.dc.html`):
//!   capa, membros, etiquetas, prazo atrasado, descrição rica, campos, a checklist «Animation pass»
//!   com responsável e data, um comentário com menção e reacção, e o histórico.

use crate::docs::KanbanDocs;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_kanban_model::{
    Activity, ActivityId, Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId,
    Checklist, ChecklistId, Comment, CommentId, Cover, CoverFill, Due, FieldDef, FieldId,
    FieldKind, FieldOption, FieldValue, FracKey, Happened, Hue, KanbanId, KanbanOp, KanbanSet,
    Label, LabelColor, LabelId, Link, List, ListId, Member, Meta, Moment, OptionId, Reaction,
    Record, Shade, Subject,
};

/// As cenas que existem.
pub const CENAS: u8 = 4;
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
            1 | 3 | 4 => short_film(&mut set),
            2 => big(&mut set),
            _ => return None,
        }
    };
    let docs = hero.documents.surfaces_mut().find_mut::<KanbanDocs>()?;
    docs.follow_project(shared);
    docs.open(id);
    let review = review_list(&docs.set.borrow(), id);
    if matches!(n, 1 | 3 | 4)
        && let Some(review) = review
        && let Some(t) = docs.tab_mut(id.0)
    {
        t.view.collapsed.insert(review);
    }
    let todo = docs
        .set
        .borrow()
        .get(id)
        .and_then(|k| k.lists.values().find(|l| l.title == "To Do").map(|l| l.id));
    if n == 3
        && let Some(todo) = todo
        && let Some(t) = docs.tab_mut(id.0)
    {
        // Um passo e meio de lista para a direita, um pouco levantada.
        let step = crate::metrics::LIST_W + crate::metrics::LIST_GAP;
        t.smoke_hold = Some((todo, (1.55 * step, 2.0 * crate::metrics::LIST_HEAD_BOTTOM)));
    }
    if n == 4 {
        let shot = rich_back(&mut docs.set.borrow_mut(), id);
        if let Some(c) = shot {
            docs.open_back(id.0, c);
        }
    }
    Some(id)
}

/// SMOKE 3: põe na mão a lista pedida, assim que a disposição a tem (o arrasto como o do rato), presa
/// à origem das listas a cada quadro (a janela que maximiza move-a) até o rato a mexer.
pub(crate) fn hold(tab: &mut crate::tab::Tab) {
    let Some((list, (dx, dy))) = tab.smoke_hold else {
        return;
    };
    let Some(r) = tab.layout.list(list).map(|lb| lb.rect) else {
        return;
    };
    let b = tab.layout.board;
    let corner = (
        b.x + crate::metrics::BOARD_PAD + dx,
        b.y + crate::metrics::BOARD_PAD + dy,
    );
    if let Some(d) = tab.drag.as_mut() {
        d.at = (corner.0 + d.grab.0, corner.1 + d.grab.1);
        return;
    }
    let grab = (r.w / 2.0, crate::metrics::LINE_H);
    let to_list = tab.layout.lists.iter().position(|l| l.id == list);
    tab.drag = Some(crate::tab::Drag {
        lift: crate::tab::Lift::List(list),
        at: (corner.0 + grab.0, corner.1 + grab.1),
        grab,
        size: (r.w, r.h),
        to_card: None,
        to_list: to_list.unwrap_or(0),
    });
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
            watchers: Vec::new(),
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
            watchers: Vec::new(),
            origin: None,
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

/// A cena 4: «Animate shot 12» com tudo o que o verso mostra (a maquete `Back.dc.html`). As horas
/// contam-se a partir de agora (o prazo fica atrasado, o comentário «2 hours ago»).
fn rich_back(set: &mut KanbanSet, id: KanbanId) -> Option<CardId> {
    const HOUR: i64 = 3_600_000;
    let now = crate::actions::now().0;
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
    s.put(Record::Card(c));
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
    ph2d_kanban_model::apply_batch(k, s.ops);
    Some(card.id)
}
