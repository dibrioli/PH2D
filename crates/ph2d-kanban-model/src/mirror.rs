//! **Cartões espelho** (v3, o «Mirror» do Trello de 2025): o MESMO cartão mostrado noutra lista,
//! deste ou de outro Kanban. O espelho é só um sítio ([`Mirror`]: lista, posição, original); tudo o
//! que se vê e edita é o original, por isso editar dos dois lados é a mesma escrita.
//!
//! Um espelho cujo original já não está vivo e à vista (apagado, arquivado) não aparece em
//! [`KanbanSet::entries_in`]; [`remove_card`] arquiva/apaga o original E tira todos os espelhos dele.

use crate::kanban::Kanban;
use crate::ops::{KanbanOp, Record, RecordId};
use crate::plan::{Place, Plan};
use crate::records::{Card, CardId, CardRef, KanbanId, ListId, Meta, Mirror, MirrorId};
use crate::set::KanbanSet;
use ph2d_frac::FracKey;

/// Uma linha de uma lista: um cartão dela ou um espelho.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Entry {
    Card(CardId),
    Mirror { id: MirrorId, of: CardRef },
}

impl Entry {
    /// O cartão que esta linha mostra (o original, se é um espelho).
    #[must_use]
    pub fn shows(self, here: KanbanId) -> CardRef {
        match self {
            Entry::Card(card) => CardRef { kanban: here, card },
            Entry::Mirror { of, .. } => of,
        }
    }

    /// O registo que guarda a posição desta linha.
    #[must_use]
    pub fn record(self) -> RecordId {
        match self {
            Entry::Card(c) => RecordId::Card(c),
            Entry::Mirror { id, .. } => RecordId::Mirror(id),
        }
    }
}

impl Kanban {
    /// A posição da linha `e` neste Kanban.
    #[must_use]
    pub fn entry_pos(&self, e: Entry) -> Option<&FracKey> {
        match e {
            Entry::Card(c) => self.cards.get(&c).map(|c| &c.pos),
            Entry::Mirror { id, .. } => self.mirrors.get(&id).map(|m| &m.pos),
        }
    }

    /// O registo da linha `e` com outra lista e posição (o `Put` de um arrasto).
    #[must_use]
    pub fn entry_moved(&self, e: Entry, list: ListId, pos: FracKey) -> Option<KanbanOp> {
        Some(KanbanOp::put(match e {
            Entry::Card(c) => Record::Card(Card {
                list,
                pos,
                ..self.cards.get(&c)?.clone()
            }),
            Entry::Mirror { id, .. } => Record::Mirror(Mirror {
                list,
                pos,
                ..self.mirrors.get(&id)?.clone()
            }),
        }))
    }
}

impl KanbanSet {
    /// O cartão `r`, vivo (arquivado incluído).
    #[must_use]
    pub fn card(&self, r: CardRef) -> Option<&Card> {
        self.get(r.kanban)?
            .cards
            .get(&r.card)
            .filter(|c| !c.meta.deleted)
    }

    /// As linhas da lista `list` do Kanban `kanban`: os cartões e os espelhos cujo original está à
    /// vista, pela posição (empate: o cartão antes do espelho, depois o id).
    #[must_use]
    pub fn entries_in(&self, kanban: KanbanId, list: ListId) -> Vec<Entry> {
        let Some(k) = self.get(kanban) else {
            return Vec::new();
        };
        let mut v: Vec<(&FracKey, Entry)> = k
            .cards_in(list)
            .into_iter()
            .map(|c| (&c.pos, Entry::Card(c.id)))
            .collect();
        v.extend(
            k.mirrors
                .values()
                .filter(|m| m.list == list && !m.meta.deleted)
                .filter(|m| self.card(m.of).is_some_and(|c| !c.archived))
                .map(|m| (&m.pos, Entry::Mirror { id: m.id, of: m.of })),
        );
        v.sort();
        v.into_iter().map(|(_, e)| e).collect()
    }

    /// Os espelhos vivos do cartão `of`, em todos os Kanbans.
    #[must_use]
    pub fn mirrors_of(&self, of: CardRef) -> Vec<(KanbanId, MirrorId)> {
        self.iter()
            .flat_map(|k| {
                k.mirrors
                    .values()
                    .filter(move |m| m.of == of && !m.meta.deleted)
                    .map(move |m| (k.id, m.id))
            })
            .collect()
    }

    /// O sítio para pôr uma linha no índice `index` da lista (o «Posição» do diálogo de mover do
    /// Trello; além do fim = no fim). `skip` = a linha que se está a mover, que não conta.
    #[must_use]
    pub fn place_at(
        &self,
        kanban: KanbanId,
        list: ListId,
        index: usize,
        skip: Option<Entry>,
    ) -> Option<Place> {
        let k = self.get(kanban)?;
        k.lists.get(&list).filter(|l| !l.meta.deleted)?;
        let rows: Vec<&FracKey> = self
            .entries_in(kanban, list)
            .into_iter()
            .filter(|e| Some(*e) != skip)
            .filter_map(|e| k.entry_pos(e))
            .collect();
        let lo = index.checked_sub(1).and_then(|i| rows.get(i).copied());
        let hi = rows.get(index).copied();
        let pos = match hi {
            Some(_) => FracKey::between(lo, hi),
            None => k.pos_after_cards(list),
        };
        Some(Place { kanban, list, pos })
    }
}

/// O gesto «Espelhar» o cartão `of` em `at`. `None` = o original não está vivo, a lista não existe,
/// ou o cartão já aparece nessa lista (ele próprio ou um espelho).
#[must_use]
pub fn mirror_card(set: &KanbanSet, of: CardRef, at: &Place) -> Option<Plan> {
    let original = set.card(of).filter(|c| !c.archived)?;
    let k = set.get(at.kanban)?;
    k.lists.get(&at.list).filter(|l| !l.meta.deleted)?;
    let here = of.kanban == at.kanban && original.list == at.list;
    let twice = k
        .mirrors
        .values()
        .any(|m| m.of == of && m.list == at.list && !m.meta.deleted);
    if here || twice {
        return None;
    }
    let mut plan = Plan::default();
    plan.push(
        at.kanban,
        KanbanOp::put(Record::Mirror(Mirror {
            id: MirrorId(k.mint()),
            list: at.list,
            pos: at.pos.clone(),
            of,
            meta: Meta::default(),
        })),
    );
    Some(plan)
}

/// Arquivar (`archive`) ou apagar o cartão `r` — e tirar todos os espelhos dele, em todos os
/// Kanbans (o Trello: o espelho é o cartão; sem cartão não há espelho). `None` = não está vivo.
#[must_use]
pub fn remove_card(set: &KanbanSet, r: CardRef, archive: bool) -> Option<Plan> {
    let c = set.card(r)?;
    let mut plan = Plan::default();
    plan.push(
        r.kanban,
        if archive {
            KanbanOp::put(Record::Card(Card {
                archived: true,
                ..c.clone()
            }))
        } else {
            KanbanOp::Delete(RecordId::Card(r.card))
        },
    );
    for (k, m) in set.mirrors_of(r) {
        plan.push(k, KanbanOp::Delete(RecordId::Mirror(m)));
    }
    Some(plan)
}

#[cfg(test)]
#[path = "mirror_tests.rs"]
mod tests;
