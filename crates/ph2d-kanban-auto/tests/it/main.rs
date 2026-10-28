//! O Butler pela API pública: cenas montadas SÓ por ops (a porta real), gestos aplicados como a UI
//! os aplica (`Seen` → aplicar → `react` → aplicar o plano em nome do Butler).

mod agenda;
mod frase;
mod regras;
mod sonda;

use std::collections::BTreeMap;

use ph2d_kanban_auto::{Outcome, Seen, Spec, react};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{
    Actor, Card, CardId, CardRef, Due, History, Hue, KanbanId, KanbanOp, KanbanSet, Label,
    LabelColor, LabelId, List, ListId, Member, Meta, Moment, Place, Plan, Record, Shade, move_card,
};

pub(crate) const DAY: i64 = 24 * 60 * 60 * 1000;
pub(crate) const HOUR: i64 = 60 * 60 * 1000;
/// «Agora»: 2026-10-28 12:00 UTC (uma quarta-feira).
pub(crate) const NOW: Moment = Moment(1_793_188_800_000);

pub(crate) fn utc() -> Zone {
    Zone::utc()
}

pub(crate) fn enio() -> Member {
    Member("Enio".to_owned())
}

/// Um Kanban com To Do · Doing · Done e a etiqueta Urgent.
pub(crate) struct Scene {
    pub set: KanbanSet,
    pub k: KanbanId,
    pub todo: ListId,
    pub doing: ListId,
    pub done: ListId,
    pub urgent: LabelId,
}

impl Scene {
    pub(crate) fn new() -> Self {
        let mut set = KanbanSet::default();
        let k = set.create("Projeto");
        let mut s = Self {
            set,
            k,
            todo: ListId(0),
            doing: ListId(0),
            done: ListId(0),
            urgent: LabelId(0),
        };
        s.todo = s.list("To Do");
        s.doing = s.list("Doing");
        s.done = s.list("Done");
        let id = LabelId(s.set.get(k).unwrap().mint());
        let l = Label {
            id,
            name: "Urgent".to_owned(),
            color: Some(LabelColor {
                hue: Hue::Red,
                shade: Shade::Normal,
            }),
            meta: Meta::default(),
        };
        s.set.apply(k, vec![KanbanOp::put(Record::Label(l))]);
        s.urgent = id;
        s
    }

    pub(crate) fn list(&mut self, title: &str) -> ListId {
        let kd = self.set.get(self.k).unwrap();
        let id = ListId(kd.mint());
        let l = List {
            id,
            title: title.to_owned(),
            pos: kd.pos_after_lists(),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        };
        self.set.apply(self.k, vec![KanbanOp::put(Record::List(l))]);
        id
    }

    pub(crate) fn card(
        &mut self,
        list: ListId,
        title: &str,
        edit: impl FnOnce(&mut Card),
    ) -> CardId {
        let kd = self.set.get(self.k).unwrap();
        let id = CardId(kd.mint());
        let mut c = Card::new(id, list, kd.pos_after_cards(list), title, NOW);
        edit(&mut c);
        self.set.apply(self.k, vec![KanbanOp::put(Record::card(c))]);
        id
    }

    pub(crate) fn add(&mut self, spec: &Spec) -> ph2d_kanban_model::AutomationId {
        let k = self.set.get(self.k).unwrap();
        let op = spec.create(k, NOW);
        let Some(ph2d_kanban_model::RecordId::Automation(id)) = op.target() else {
            panic!("create põe uma automação");
        };
        self.set.apply(self.k, vec![op]);
        id
    }

    pub(crate) fn get(&self, c: CardId) -> Card {
        self.set.get(self.k).unwrap().cards[&c].clone()
    }

    /// O gesto da pessoa: aplica `plan` e devolve o que o Butler respondeu (sem o aplicar).
    pub(crate) fn gesture(&mut self, plan: Plan, by: &Member) -> Outcome {
        let seen = Seen::plan(&self.set, &plan);
        apply(&mut self.set, plan, by);
        react(&self.set, seen.events(&self.set, by), NOW, &utc())
    }

    pub(crate) fn edit(&mut self, c: CardId, by: &Member, f: impl FnOnce(&mut Card)) -> Outcome {
        let mut card = self.get(c);
        f(&mut card);
        let mut p = Plan::default();
        p.push(self.k, KanbanOp::put(Record::card(card)));
        self.gesture(p, by)
    }

    pub(crate) fn move_to(&mut self, c: CardId, list: ListId, by: &Member) -> Outcome {
        let src = CardRef {
            kanban: self.k,
            card: c,
        };
        let pos = self.set.get(self.k).unwrap().pos_after_cards(list);
        let to = Place {
            kanban: self.k,
            list,
            pos,
        };
        let plan = move_card(&self.set, src, &to).unwrap();
        self.gesture(plan, by)
    }

    /// Aplica o que o Butler respondeu, em nome dele.
    pub(crate) fn butler(&mut self, o: Outcome) {
        apply(&mut self.set, o.plan, &Member::butler());
    }
}

pub(crate) fn apply(set: &mut KanbanSet, plan: Plan, by: &Member) {
    let mut h: BTreeMap<KanbanId, History> = BTreeMap::new();
    let actor = Actor {
        by: by.clone(),
        at: NOW,
    };
    set.apply_plan(plan, &mut h, 1, &actor);
}

pub(crate) fn due(at: i64) -> Option<Due> {
    Some(Due {
        at: Moment(at),
        done: false,
        remind_before_min: None,
    })
}
