//! `ph2d.kanban` pela API pública: cenas montadas por ops, scripts corridos como o host os corre
//! (`KanbanScripts::run` / `react`), o desfazer pelo `undo_gesture` da UI.

mod api;
mod escrever;
mod eventos;

use std::collections::BTreeMap;

use ph2d_kanban_auto::{Seen, Spec};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{
    Actor, Card, CardId, CardRef, CheckItem, CheckItemId, Checklist, ChecklistId, History, Hue,
    KanbanId, KanbanOp, KanbanSet, Label, LabelColor, LabelId, List, ListId, Member, Meta, Moment,
    Place, Record, Shade, move_card,
};
use ph2d_kanban_script::{Ctx, KanbanScripts, Report};

/// Listas, cartões (título, lista, arquivado, etiquetas) e comentários.
type Photo = (Vec<String>, Vec<(String, ListId, bool, usize)>, usize);

/// «Agora»: 2026-10-28 12:00 UTC.
pub(crate) const NOW: Moment = Moment(1_793_188_800_000);

pub(crate) fn enio() -> Member {
    Member("Enio".to_owned())
}

/// Um Kanban «Projeto» com To Do · Doing · Done, a etiqueta Urgent e os históricos da UI.
pub(crate) struct Scene {
    pub set: KanbanSet,
    pub histories: BTreeMap<KanbanId, History>,
    pub scripts: KanbanScripts,
    pub zone: Zone,
    pub k: KanbanId,
    pub todo: ListId,
    pub doing: ListId,
    pub done: ListId,
    pub tick: u64,
}

impl Scene {
    pub(crate) fn new() -> Self {
        let mut set = KanbanSet::default();
        let k = set.create("Projeto");
        let mut s = Self {
            set,
            histories: BTreeMap::new(),
            scripts: KanbanScripts::new(),
            zone: Zone::utc(),
            k,
            todo: ListId(0),
            doing: ListId(0),
            done: ListId(0),
            tick: 0,
        };
        s.todo = s.list("To Do");
        s.doing = s.list("Doing");
        s.done = s.list("Done");
        s.label("Urgent");
        s
    }

    pub(crate) fn list(&mut self, title: &str) -> ListId {
        let kd = self.set.get(self.k).unwrap();
        let l = List {
            id: ListId(kd.mint()),
            title: title.to_owned(),
            pos: kd.pos_after_lists(),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        };
        let id = l.id;
        self.set.apply(self.k, vec![KanbanOp::put(Record::List(l))]);
        id
    }

    pub(crate) fn label(&mut self, name: &str) -> LabelId {
        let id = LabelId(self.set.get(self.k).unwrap().mint());
        let l = Label {
            id,
            name: name.to_owned(),
            color: Some(LabelColor {
                hue: Hue::Red,
                shade: Shade::Normal,
            }),
            meta: Meta::default(),
        };
        self.set
            .apply(self.k, vec![KanbanOp::put(Record::Label(l))]);
        id
    }

    pub(crate) fn card(
        &mut self,
        list: ListId,
        title: &str,
        edit: impl FnOnce(&mut Card),
    ) -> CardId {
        let kd = self.set.get(self.k).unwrap();
        let mut c = Card::new(
            CardId(kd.mint()),
            list,
            kd.pos_after_cards(list),
            title,
            NOW,
        );
        edit(&mut c);
        let id = c.id;
        self.set.apply(self.k, vec![KanbanOp::put(Record::card(c))]);
        id
    }

    /// Uma checklist com os itens `items` no cartão `card`.
    pub(crate) fn checklist(&mut self, card: CardId, items: &[&str]) {
        let kd = self.set.get(self.k).unwrap();
        let ids = kd.mint_n(1 + items.len());
        let cl = ChecklistId(ids[0]);
        let mut ops = vec![KanbanOp::put(Record::Checklist(Checklist {
            id: cl,
            card,
            pos: ph2d_kanban_model::FracKey::between(None, None),
            title: "Steps".into(),
            hide_checked: false,
            meta: Meta::default(),
        }))];
        let mut pos = None;
        for (t, id) in items.iter().zip(&ids[1..]) {
            let here = ph2d_kanban_model::FracKey::between(pos.as_ref(), None);
            ops.push(KanbanOp::put(Record::CheckItem(CheckItem {
                id: CheckItemId(*id),
                checklist: cl,
                pos: here.clone(),
                text: (*t).to_owned(),
                done: false,
                member: None,
                due: None,
                meta: Meta::default(),
            })));
            pos = Some(here);
        }
        self.set.apply(self.k, ops);
    }

    pub(crate) fn add(&mut self, spec: &Spec) {
        let op = spec
            .create(self.set.get(self.k).unwrap(), NOW)
            .expect("automação válida");
        self.set.apply(self.k, vec![op]);
    }

    pub(crate) fn run(&mut self, name: &str, source: &str) -> Report {
        self.tick += 1;
        let cx = Ctx {
            now: NOW,
            zone: &self.zone,
            me: enio(),
            at: self.tick,
        };
        self.scripts
            .run(name, source, &mut self.set, &mut self.histories, &cx)
    }

    /// O gesto da pessoa (mover para `list`), com o histórico da UI, e depois os handlers.
    pub(crate) fn person_moves(&mut self, c: CardId, list: ListId) -> Report {
        self.tick += 1;
        let src = CardRef {
            kanban: self.k,
            card: c,
        };
        let pos = self.set.get(self.k).unwrap().pos_after_cards(list);
        let plan = move_card(
            &self.set,
            src,
            &Place {
                kanban: self.k,
                list,
                pos,
            },
        )
        .unwrap();
        let seen = Seen::plan(&self.set, &plan);
        let actor = Actor {
            by: enio(),
            at: NOW,
        };
        self.set
            .apply_plan(plan, &mut self.histories, self.tick, &actor);
        let events = seen.events(&self.set, &enio());
        let cx = Ctx {
            now: NOW,
            zone: &self.zone,
            me: enio(),
            at: self.tick,
        };
        self.scripts
            .react(&mut self.set, &mut self.histories, events, &cx)
    }

    pub(crate) fn undo(&mut self) -> bool {
        self.tick += 1;
        let actor = Actor {
            by: enio(),
            at: NOW,
        };
        self.set
            .undo_gesture(&mut self.histories, self.tick, &actor)
    }

    /// O cartão vivo com este título.
    pub(crate) fn by_title(&self, title: &str) -> Option<Card> {
        self.set
            .get(self.k)
            .unwrap()
            .cards
            .values()
            .find(|c| !c.meta.deleted && c.title == title)
            .cloned()
    }

    /// O que se vê: listas e cartões vivos (título, lista, arquivado, etiquetas) e comentários.
    pub(crate) fn photo(&self) -> Photo {
        let k = self.set.get(self.k).unwrap();
        let lists = k.lists_in_order().iter().map(|l| l.title.clone()).collect();
        let mut cards: Vec<_> = k
            .cards
            .values()
            .filter(|c| !c.meta.deleted)
            .map(|c| (c.title.clone(), c.list, c.archived, c.labels.len()))
            .collect();
        cards.sort();
        let comments = k.comments.values().filter(|c| !c.meta.deleted).count();
        (lists, cards, comments)
    }
}

pub(crate) fn ok(r: &Report) {
    assert!(r.errors.is_empty(), "erros: {:?}", r.errors);
}
