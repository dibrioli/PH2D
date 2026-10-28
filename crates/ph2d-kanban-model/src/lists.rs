//! **O menu da lista** (v3): copiar, mover (para outro Kanban), mover todos os cartões, arquivar
//! todos os cartões — e o Kanban feito a partir de outro (o modelo de quadro do Trello).

use ph2d_frac::FracKey;

use crate::kanban::Kanban;
use crate::mirror::Entry;
use crate::ops::{KanbanOp, Record};
use crate::plan::{ListRef, Plan};
use crate::records::{Card, CardId, KanbanId, List, ListId, Meta, Mirror, MirrorId, Moment};
use crate::set::KanbanSet;
use crate::transfer::{Copier, CopyKeep, How, cross_move};

fn live_list(k: &Kanban, list: ListId) -> Option<&List> {
    k.lists.get(&list).filter(|l| !l.meta.deleted)
}

/// As linhas vivas de `list` (cartões arquivados incluídos se `archived`), pela posição.
fn rows(k: &Kanban, list: ListId, archived: bool) -> (Vec<&Card>, Vec<&Mirror>) {
    let mut cards: Vec<&Card> = k
        .cards
        .values()
        .filter(|c| c.list == list && !c.meta.deleted && (archived || !c.archived))
        .collect();
    cards.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    let mut mirrors: Vec<&Mirror> = k
        .mirrors
        .values()
        .filter(|m| m.list == list && !m.meta.deleted)
        .collect();
    mirrors.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    (cards, mirrors)
}

/// «Copiar lista»: uma lista nova `title` no Kanban `to`, em `pos`, com cópias dos cartões à vista
/// (tudo mantido) e dos espelhos.
#[must_use]
pub fn copy_list(
    set: &KanbanSet,
    src: ListRef,
    to: KanbanId,
    pos: FracKey,
    title: &str,
    created: Moment,
) -> Option<Plan> {
    let (sk, dk) = (set.get(src.kanban)?, set.get(to)?);
    let l = live_list(sk, src.list)?;
    let mut cp = Copier::new(sk, dk);
    let id = ListId(cp.id());
    cp.ops.push(KanbanOp::put(Record::List(List {
        id,
        title: title.to_owned(),
        pos,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
        ..l.clone()
    })));
    let (cards, mirrors) = rows(sk, src.list, false);
    let how = How {
        keep: CopyKeep::default(),
        title: None,
        created: Some(created),
        template: None,
        whole: false,
    };
    for c in cards {
        cp.card(c, id, c.pos.clone(), &how);
    }
    for m in mirrors {
        let mid = MirrorId(cp.id());
        cp.ops.push(KanbanOp::put(Record::Mirror(Mirror {
            id: mid,
            list: id,
            meta: Meta::default(),
            ..m.clone()
        })));
    }
    let mut plan = Plan::default();
    plan.extend(to, cp.ops);
    Some(plan)
}

/// «Mover lista» para o Kanban `to`, em `pos` — com todos os cartões (arquivados incluídos) e
/// espelhos. No mesmo Kanban é só a posição.
#[must_use]
pub fn move_list(set: &KanbanSet, src: ListRef, to: KanbanId, pos: FracKey) -> Option<Plan> {
    let (sk, dk) = (set.get(src.kanban)?, set.get(to)?);
    let l = live_list(sk, src.list)?;
    let mut plan = Plan::default();
    if src.kanban == to {
        plan.push(to, KanbanOp::put(Record::List(List { pos, ..l.clone() })));
        return Some(plan);
    }
    let id = ListId(dk.mint());
    let head = vec![KanbanOp::put(Record::List(List {
        id,
        pos,
        meta: Meta::default(),
        ..l.clone()
    }))];
    let (cards, mirrors) = rows(sk, src.list, true);
    let cards: Vec<_> = cards.iter().map(|c| (c.id, id, c.pos.clone())).collect();
    let mirrors: Vec<_> = mirrors.iter().map(|m| (m.id, id, m.pos.clone())).collect();
    cross_move(set, sk, dk, &cards, &mirrors, &mut plan, head, 1);
    plan.push(
        src.kanban,
        KanbanOp::Delete(crate::RecordId::List(src.list)),
    );
    Some(plan)
}

/// «Mover todos os cartões desta lista» para o fim da lista `to` (cartões à vista e espelhos, pela
/// ordem). `None` = é a mesma lista, ou uma das duas não existe.
#[must_use]
pub fn move_all_cards(set: &KanbanSet, src: ListRef, to: ListRef) -> Option<Plan> {
    if src == to {
        return None;
    }
    let (sk, dk) = (set.get(src.kanban)?, set.get(to.kanban)?);
    live_list(sk, src.list)?;
    live_list(dk, to.list)?;
    let mut entries: Vec<(FracKey, Entry)> = {
        let (cards, mirrors) = rows(sk, src.list, false);
        let c = cards.iter().map(|c| (c.pos.clone(), Entry::Card(c.id)));
        let m = mirrors.iter().map(|m| {
            let e = Entry::Mirror { id: m.id, of: m.of };
            (m.pos.clone(), e)
        });
        c.chain(m).collect()
    };
    entries.sort();
    let mut pos = dk.pos_after_cards(to.list);
    let mut cards: Vec<(CardId, ListId, FracKey)> = Vec::new();
    let mut mirrors: Vec<(MirrorId, ListId, FracKey)> = Vec::new();
    let mut plan = Plan::default();
    for (_, e) in entries {
        if src.kanban == to.kanban {
            plan.extend(to.kanban, sk.entry_moved(e, to.list, pos.clone()));
        } else {
            match e {
                Entry::Card(c) => cards.push((c, to.list, pos.clone())),
                Entry::Mirror { id, .. } => mirrors.push((id, to.list, pos.clone())),
            }
        }
        pos = FracKey::between(Some(&pos), None);
    }
    if src.kanban != to.kanban {
        cross_move(set, sk, dk, &cards, &mirrors, &mut plan, Vec::new(), 0);
    }
    Some(plan)
}

/// «Arquivar todos os cartões desta lista» (um gesto).
#[must_use]
pub fn archive_all_cards(k: &Kanban, list: ListId) -> Vec<KanbanOp> {
    k.cards_in(list)
        .into_iter()
        .map(|c| {
            KanbanOp::put(Record::card(Card {
                archived: true,
                ..c.clone()
            }))
        })
        .collect()
}

impl KanbanSet {
    /// Um Kanban NOVO `name` a partir de `src` (o «Criar quadro a partir do modelo» / «Copiar
    /// quadro» do Trello): fundo, envelhecimento, etiquetas, campos e listas à vista; com `keep_cards`
    /// os cartões à vista (sem comentários, votos nem seguidores), senão só os modelos de cartão.
    /// Nasce sem história nem espelhos. `None` = `src` não existe.
    pub fn create_from(&mut self, src: KanbanId, name: &str, keep_cards: bool) -> Option<KanbanId> {
        let s = self.get(src)?.clone();
        let id = self.create(name);
        let k = self.doc_mut(id)?;
        let bare = Meta::default();
        let mut h = k.header.clone();
        (h.background, h.aging) = (s.header.background, s.header.aging);
        let mut ops = vec![KanbanOp::Header(h)];
        ops.extend(s.labels.values().filter(|l| !l.meta.deleted).map(|l| {
            KanbanOp::put(Record::Label(crate::Label {
                meta: bare,
                ..l.clone()
            }))
        }));
        ops.extend(s.fields.values().filter(|f| !f.meta.deleted).map(|f| {
            KanbanOp::put(Record::Field(crate::FieldDef {
                meta: bare,
                ..f.clone()
            }))
        }));
        for l in s.lists_in_order() {
            ops.push(KanbanOp::put(Record::List(List {
                meta: bare,
                watchers: Vec::new(),
                ..l.clone()
            })));
        }
        crate::apply_batch(k, ops);
        // As mesmas chaves: o Kanban nasceu vazio, e assim etiquetas, campos e listas casam.
        let keep = CopyKeep {
            comments: false,
            ..CopyKeep::default()
        };
        let snapshot = k.clone();
        let mut cp = Copier::new(&s, &snapshot);
        for l in s.lists_in_order() {
            for c in s.cards_in(l.id) {
                if keep_cards || c.template {
                    let how = How {
                        keep,
                        title: None,
                        created: None,
                        template: None,
                        whole: false,
                    };
                    cp.card(c, l.id, c.pos.clone(), &how);
                }
            }
        }
        let ops = std::mem::take(&mut cp.ops);
        let k = self.doc_mut(id)?;
        crate::apply_batch(k, ops);
        Some(id)
    }
}
