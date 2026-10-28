//! **As ligações do cartão ao projecto** (v6, W11 — os «Power-Ups» do PH2D): o cartão que abre o
//! objecto da cena, o script, a macro, o trecho da timeline ou o Quadro. Um registo PRÓPRIO
//! ([`ProjectLink`]), não um anexo: o verso dá-lhes a secção do botão «Scene object» da maquete, e o
//! `Link` dos anexos fica fechado para quem o casa por inteiro.
//!
//! A chave é durável (o `StableId` do objecto, o `ScriptId`, o nome da macro na biblioteca), nunca os
//! bits de uma entidade; o `name` é o de RESERVA — o último nome visto — e é o que se mostra quando o
//! alvo deixou de existir (a ligação fica «quebrada», nunca some).

use ph2d_frac::FracKey;
use serde::{Deserialize, Serialize};

use crate::kanban::Kanban;
use crate::records::{CardId, CardRef, Meta, Moment, ProjectLinkId};
use crate::set::KanbanSet;

/// Para onde uma ligação aponta. Os `u64` são as chaves duráveis de cada família (o modelo é puro:
/// não depende delas).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Target {
    /// Um objecto da cena pelo `StableId` (um objecto Macro também é um objecto da cena).
    Object(u64),
    /// Um script do projecto pelo `ScriptId` (ADR-0188).
    Script(u64),
    /// Uma macro da biblioteca pelo nome (`<nome>.ph2dmacro`, ADR-0186: a biblioteca não tem id).
    Macro(String),
    /// Um trecho da timeline, em quadros `[from, to]`: do clip `strip` (`StripId`) do objecto `object`
    /// (`StableId`), ou da timeline da cena (`object = None`).
    Span {
        object: Option<u64>,
        strip: Option<u64>,
        from: i64,
        to: i64,
    },
    /// Um Quadro (`BoardId`) e, se há, um elemento dele (`ElementId`).
    Board { board: u64, element: Option<u64> },
}

impl Target {
    /// Esta ligação diz respeito a `query`? O objecto responde também pelos trechos dele; o Quadro
    /// inteiro, por cada elemento dele. O resto, igualdade.
    #[must_use]
    pub fn concerns(&self, query: &Target) -> bool {
        match (query, self) {
            (Target::Object(q), Target::Span { object, .. }) => *object == Some(*q),
            (
                Target::Board {
                    board: q,
                    element: None,
                },
                Target::Board { board, .. },
            ) => q == board,
            _ => self == query,
        }
    }
}

/// Uma ligação de um cartão a uma coisa do projecto.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectLink {
    pub id: ProjectLinkId,
    pub card: CardId,
    pub pos: FracKey,
    pub target: Target,
    /// O nome de reserva: o último nome visto do alvo.
    pub name: String,
    pub at: Moment,
    pub meta: Meta,
}

impl ProjectLink {
    #[must_use]
    pub fn new(
        id: ProjectLinkId,
        card: CardId,
        pos: FracKey,
        target: Target,
        name: &str,
        at: Moment,
    ) -> Self {
        Self {
            id,
            card,
            pos,
            target,
            name: name.to_owned(),
            at,
            meta: Meta::default(),
        }
    }
}

impl Kanban {
    /// As ligações vivas do cartão `card`, pela ordem.
    #[must_use]
    pub fn links_of(&self, card: CardId) -> Vec<&ProjectLink> {
        let mut v: Vec<&ProjectLink> = self
            .project_links
            .values()
            .filter(|l| l.card == card && !l.meta.deleted)
            .collect();
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        v
    }

    /// A posição depois da última ligação do cartão `card`.
    #[must_use]
    pub fn pos_after_links(&self, card: CardId) -> FracKey {
        let last = self
            .project_links
            .values()
            .filter(|l| l.card == card && !l.meta.deleted)
            .map(|l| &l.pos)
            .max();
        FracKey::between(last, None)
    }
}

impl KanbanSet {
    /// ⭐ O sentido inverso: os cartões VIVOS (arquivados contam) de todos os Kanbans que referem
    /// `target` ([`Target::concerns`]), sem repetição e por ordem.
    #[must_use]
    pub fn cards_linking(&self, target: &Target) -> Vec<CardRef> {
        let mut out: Vec<CardRef> = self
            .iter()
            .flat_map(|k| {
                k.project_links
                    .values()
                    .filter(|l| !l.meta.deleted && l.target.concerns(target))
                    .filter(|l| k.cards.get(&l.card).is_some_and(|c| !c.meta.deleted))
                    .map(|l| CardRef {
                        kanban: k.id,
                        card: l.card,
                    })
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[cfg(test)]
#[path = "links_tests.rs"]
pub(crate) mod tests;
