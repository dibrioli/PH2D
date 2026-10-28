//! **A agenda do projecto** (v5, W9 — plano §8.1): os eventos próprios do Calendário e o que é
//! PESSOAL de cada um (a Inbox, os blocos do Planner, as notificações lidas).
//!
//! A agenda é um documento com a MESMA lei dos Kanbans (registos com [`Meta`], ops, desfazer por
//! campo, réplica), na chave reservada [`KanbanId::AGENDA`] — os Kanbans contam de 1. Não aparece em
//! [`crate::KanbanSet::iter`]: não é um quadro. Os eventos viajam numa sessão ao vivo; os registos
//! pessoais ([`Record::is_personal`]) nunca saem do projecto de quem os escreveu (o Trello: a Inbox é
//! só tua).

use ph2d_frac::FracKey;
use serde::{Deserialize, Serialize};

use crate::color::Hue;
use crate::kanban::Kanban;
use crate::ops::{Record, RecordId};
use crate::records::{BlockId, CardRef, EventId, InboxId, KanbanId, Member, Meta, Moment, SeenId};

impl KanbanId {
    /// A agenda do projecto (eventos, Inbox, Planner, lidas).
    pub const AGENDA: KanbanId = KanbanId(0);
}

/// A recorrência simples de um evento (a do Google Calendar sem regras: cada N fica para depois).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Repeat {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Um evento próprio do Calendário: reunião, marco, entrega ao cliente, feriado.
///
/// `end` é EXCLUSIVO. Dia inteiro: `start`/`end` são a meia-noite UTC dos dias civis (o `DATE` do
/// RFC 5545) — o mesmo dia em qualquer fuso.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub title: String,
    pub start: Moment,
    pub end: Moment,
    pub all_day: bool,
    pub repeat: Option<Repeat>,
    /// Sem cor = a cor da agenda (os tokens).
    pub color: Option<Hue>,
    pub people: Vec<Member>,
    pub notes: String,
    /// Lembrar N minutos antes do início.
    pub remind_before_min: Option<u32>,
    pub meta: Meta,
    /// v7 — a série acaba ANTES deste início (exclusivo; o «This and following» que a partiu).
    pub until: Option<Moment>,
    /// v7 — as ocorrências que deixaram a regra, pelo início ORIGINAL (o `EXDATE` do RFC 5545).
    pub exceptions: Vec<Exception>,
}

/// Uma ocorrência de uma série que deixou de seguir a regra (v7): apagada, ou substituída por um
/// evento avulso (o «This event» do Google Calendar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exception {
    /// O início que a regra dava à ocorrência.
    pub at: Moment,
    /// `None` = apagada; `Some` = o evento que a substitui.
    pub becomes: Option<EventId>,
}

/// Uma excepção casa a ocorrência a menos de meio dia dela: a hora de verão anda uma hora e a
/// série mais densa é diária.
const MATCH_MS: i64 = 12 * 3_600_000;

impl Event {
    /// Um evento de `start` a `end`, sem mais nada.
    #[must_use]
    pub fn new(id: EventId, title: &str, start: Moment, end: Moment, all_day: bool) -> Self {
        Self {
            id,
            title: title.to_owned(),
            start,
            end,
            all_day,
            repeat: None,
            color: None,
            people: Vec::new(),
            notes: String::new(),
            remind_before_min: None,
            meta: Meta::default(),
            until: None,
            exceptions: Vec::new(),
        }
    }

    /// A excepção da ocorrência que a regra põe em `at`.
    #[must_use]
    pub fn exception_at(&self, at: Moment) -> Option<&Exception> {
        self.exceptions
            .iter()
            .find(|x| (x.at.0 - at.0).abs() < MATCH_MS)
    }

    /// A regra põe uma ocorrência em `at`, mas ela não aparece (depois do fim ou com excepção).
    #[must_use]
    pub fn skips(&self, at: Moment) -> bool {
        self.until.is_some_and(|u| at >= u) || self.exception_at(at).is_some()
    }
}

/// Um item da Inbox: a captura rápida de uma pessoa. «Mover para um Kanban» faz dele um cartão.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: InboxId,
    pub owner: Member,
    pub text: String,
    pub created: Moment,
    pub pos: FracKey,
    pub meta: Meta,
}

/// O que ocupa um bloco do Planner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockOf {
    Inbox(InboxId),
    Card(CardRef),
}

/// Um bloco de tempo do Planner: o item ou cartão, o início e a duração.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub owner: Member,
    pub of: BlockOf,
    pub start: Moment,
    pub minutes: u32,
    pub meta: Meta,
}

/// As notificações que `owner` já leu: todas até `before` («Marcar todas como lidas») e as chaves
/// soltas de depois (`read`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seen {
    pub id: SeenId,
    pub owner: Member,
    pub before: Moment,
    pub read: Vec<u64>,
    pub meta: Meta,
}

impl Record {
    /// De UMA pessoa (Inbox, bloco, lidas): fica no projecto dela, nunca viaja.
    #[must_use]
    pub fn is_personal(&self) -> bool {
        self.id().is_personal()
    }
}

impl RecordId {
    /// [`Record::is_personal`] pela chave.
    #[must_use]
    pub fn is_personal(&self) -> bool {
        matches!(
            self,
            RecordId::Inbox(_) | RecordId::Block(_) | RecordId::Seen(_)
        )
    }
}

impl Kanban {
    /// O mesmo documento sem os registos pessoais (o que uma sessão ao vivo pode levar).
    #[must_use]
    pub fn without_personal(&self) -> Kanban {
        let mut k = self.clone();
        k.inbox.clear();
        k.blocks.clear();
        k.seen.clear();
        k
    }

    /// Traz os registos pessoais de `from` (a agenda que um retrato da sessão substituiu).
    pub fn keep_personal_of(&mut self, from: &Kanban) {
        self.inbox.clone_from(&from.inbox);
        self.blocks.clone_from(&from.blocks);
        self.seen.clone_from(&from.seen);
    }

    /// Os itens vivos da Inbox de `who`, pela ordem.
    #[must_use]
    pub fn inbox_of(&self, who: &Member) -> Vec<&InboxItem> {
        let mut v: Vec<&InboxItem> = self
            .inbox
            .values()
            .filter(|i| !i.meta.deleted && &i.owner == who)
            .collect();
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        v
    }

    /// A posição depois do último item da Inbox (de qualquer pessoa: as posições não colidem).
    #[must_use]
    pub fn pos_after_inbox(&self) -> FracKey {
        let last = self
            .inbox
            .values()
            .filter(|i| !i.meta.deleted)
            .map(|i| &i.pos)
            .max();
        FracKey::between(last, None)
    }

    /// O registo de lidas de `who` (vivo).
    #[must_use]
    pub fn seen_of(&self, who: &Member) -> Option<&Seen> {
        self.seen
            .values()
            .find(|s| !s.meta.deleted && &s.owner == who)
    }
}

#[cfg(test)]
#[path = "agenda_tests.rs"]
pub(crate) mod tests;
