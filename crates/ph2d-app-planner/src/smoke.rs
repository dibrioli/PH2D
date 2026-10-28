//! **A cena de smoke** `PH2D_PLANNER_SMOKE=1`: a cena 1 do Kanban («Short film — production») e,
//! por cima, um projecto rico para o Calendário: entregas de hoje a daqui a duas semanas, um item de
//! checklist com data, eventos próprios (a reunião semanal, um marco de dia inteiro, a entrega ao
//! cliente com lembrete), a Inbox com três capturas, um bloco no Planner e uma menção a mim.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_kanban_model::{
    Block, BlockId, BlockOf, CardRef, Comment, CommentId, Due, Event, EventId, Hue, InboxId,
    InboxItem, KanbanId, KanbanOp, KanbanSet, Member, Meta, Moment, Record, Repeat,
};

use crate::docs::{Mode, PlannerDocs};

/// A env que arma a cena.
pub const ENV: &str = "PH2D_PLANNER_SMOKE";
/// As cenas que há.
pub const CENAS: u8 = 1;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: ENV,
    max_level: CENAS as u32,
}];

const HOUR: i64 = 3_600_000;
const DAY: i64 = 24 * HOUR;

/// O nível pedido pela env.
#[must_use]
pub fn armed_level() -> Option<u8> {
    let n: u8 = std::env::var(ENV).ok()?.trim().parse().ok()?;
    (1..=CENAS).contains(&n).then_some(n)
}

/// Quem sou na cena (o mesmo «eu» do quadro).
fn me_of(hero: &HeroScreen) -> Member {
    let a = hero.documents.author().trim();
    if a.is_empty() {
        Member(ph2d_i18n::tr("kanban.back.me").to_owned())
    } else {
        Member(a.to_owned())
    }
}

/// As datas e as pessoas que a cena 1 do Kanban não tem.
fn dress_cards(set: &mut KanbanSet, k: KanbanId, me: &Member, now: i64) {
    let Some(kb) = set.get(k) else { return };
    let plan: Vec<(&str, i64, Option<u32>)> = vec![
        ("Storyboard act 1", 1, Some(60)),
        ("Design the fox", 3, None),
        ("Color script", 6, None),
        ("Animate shot 12", 9, None),
        ("Voice casting", 13, None),
        ("Write the logline", 0, Some(15)),
    ];
    let mut ops = Vec::new();
    for (title, days, remind) in plan {
        let Some(c) = kb.cards.values().find(|c| c.title == title) else {
            continue;
        };
        let mut n = c.clone();
        n.due = Some(Due {
            at: Moment(now + days * DAY + 2 * HOUR),
            done: false,
            remind_before_min: remind,
        });
        if days >= 6 {
            n.start = Some(Moment(now + (days - 2) * DAY));
        }
        if !n.members.contains(me) {
            n.members.push(me.clone());
        }
        ops.push(KanbanOp::put(Record::Card(n)));
    }
    // Um item de checklist com data (o primeiro do storyboard).
    if let Some(it) = kb.items.values().find(|i| !i.meta.deleted) {
        let mut n = it.clone();
        n.due = Some(Moment(now + 2 * DAY));
        n.member = Some(me.clone());
        ops.push(KanbanOp::put(Record::CheckItem(n)));
    }
    // Uma menção a mim, de uma colega.
    if let Some(c) = kb.cards.values().find(|c| c.title == "Design the fox") {
        let ids = kb.mint_n(1);
        ops.push(KanbanOp::put(Record::Comment(Comment {
            id: CommentId(ids[0]),
            card: c.id,
            author: Member("Ana".into()),
            at: Moment(now - HOUR),
            text: format!("@{} can you check the ears before Friday?", me.0),
            edited: None,
            reactions: Vec::new(),
            meta: Meta::default(),
        })));
    }
    // Duas cartas sem data para a coluna do Planner.
    for title in ["Pick the music", "Rain test"] {
        if let Some(c) = kb.cards.values().find(|c| c.title == title) {
            let mut n = c.clone();
            n.members.push(me.clone());
            ops.push(KanbanOp::put(Record::Card(n)));
        }
    }
    set.apply(k, ops);
}

fn dress_agenda(set: &mut KanbanSet, k: KanbanId, me: &Member, now: i64) {
    let a = set.agenda();
    let ids = a.mint_n(8);
    let day0 = now - now.rem_euclid(DAY);
    let mut standup = Event::new(EventId(ids[0]), "Team review", Moment(day0 + DAY + 10 * HOUR), Moment(day0 + DAY + 11 * HOUR), false);
    standup.repeat = Some(Repeat::Weekly);
    standup.people = vec![me.clone(), Member("Ana".into()), Member("Rui".into())];
    standup.remind_before_min = Some(15);
    let mut milestone = Event::new(EventId(ids[1]), "Animatic locked", Moment(day0 + 4 * DAY), Moment(day0 + 5 * DAY), true);
    milestone.color = Some(Hue::Green);
    let mut client = Event::new(EventId(ids[2]), "Delivery to the client", Moment(now + 30 * 60_000), Moment(now + 90 * 60_000), false);
    client.color = Some(Hue::Red);
    client.remind_before_min = Some(60);
    client.notes = "Export the cut and the stills.".into();
    let mut ops = vec![
        KanbanOp::put(Record::Event(standup)),
        KanbanOp::put(Record::Event(milestone)),
        KanbanOp::put(Record::Event(client)),
    ];
    let mut pos = a.pos_after_inbox();
    for (i, text) in ["Call the sound studio", "Buy a new pen nib", "Idea: rain in the last shot"].into_iter().enumerate() {
        ops.push(KanbanOp::put(Record::Inbox(InboxItem {
            id: InboxId(ids[3 + i]),
            owner: me.clone(),
            text: text.into(),
            created: Moment(now - HOUR),
            pos: pos.clone(),
            meta: Meta::default(),
        })));
        pos = ph2d_kanban_model::FracKey::between(Some(&pos), None);
    }
    let card = set
        .get(k)
        .and_then(|kb| kb.cards.values().find(|c| c.title == "Rain test").map(|c| c.id));
    if let Some(card) = card {
        ops.push(KanbanOp::put(Record::Block(Block {
            id: BlockId(ids[6]),
            owner: me.clone(),
            of: BlockOf::Card(CardRef { kanban: k, card }),
            start: Moment(day0 + DAY + 14 * HOUR),
            minutes: 90,
            meta: Meta::default(),
        })));
    }
    set.apply(KanbanId::AGENDA, ops);
}

/// Monta a cena `n` e abre a aba no Calendário. `None` = cena desconhecida ou família em falta.
pub fn stage(hero: &mut HeroScreen, n: u8) -> Option<KanbanId> {
    if n != 1 {
        return None;
    }
    let k = ph2d_app_kanban::smoke::stage(hero, 1)?;
    let me = me_of(hero);
    let now = crate::wall_clock().0;
    {
        let shared = hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        dress_cards(&mut set, k, &me, now);
        dress_agenda(&mut set, k, &me, now);
    }
    let shared = hero.documents.kanbans_shared();
    let docs = hero.documents.surfaces_mut().find_mut::<PlannerDocs>()?;
    docs.follow_project(shared);
    docs.open(Mode::Calendar);
    Some(k)
}

/// A cena armada pela env (a shell chama-o ao instalar a família).
pub fn stage_armed_smokes(hero: &mut HeroScreen) {
    if let Some(n) = armed_level() {
        stage(hero, n);
    }
}
