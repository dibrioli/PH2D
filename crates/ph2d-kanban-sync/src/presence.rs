//! **Quem está a ver** (ADR-0181 §6, no Kanban): que Kanban, que cartão aberto, e o cursor (opcional,
//! nas coordenadas do conteúdo do Kanban, sem a rolagem). Não é documento: sem carimbo, o último
//! ganha. A pintura (avatares no cabeçalho e no cartão) é da UI.

use std::collections::BTreeMap;

use ph2d_board_sync::{PeerId, Track};
use ph2d_kanban_model::{CardId, KanbanId};
use serde::{Deserialize, Serialize};

/// O que uma pessoa mostra de si no Kanban. `kanban` é o id da SESSÃO.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KanbanPresence {
    pub peer: PeerId,
    /// O Kanban que tem à frente (`None` = nenhum).
    pub kanban: Option<KanbanId>,
    /// O cartão aberto (o verso).
    pub card: Option<CardId>,
    pub cursor: Option<[f64; 2]>,
}

/// O que se vê dos outros.
#[derive(Clone, Debug, Default)]
pub struct Viewers {
    seen: BTreeMap<PeerId, (KanbanPresence, Track<2>)>,
}

impl Viewers {
    /// Uma presença que chegou em `now_ms`.
    pub fn heard(&mut self, now_ms: u64, p: KanbanPresence) {
        let slot = self
            .seen
            .entry(p.peer)
            .or_insert_with(|| (p.clone(), Track::default()));
        if let Some(c) = p.cursor {
            slot.1.push(now_ms, c);
        }
        slot.0 = p;
    }

    /// Saiu da sala.
    pub fn gone(&mut self, peer: PeerId) {
        self.seen.remove(&peer);
    }

    /// Quem está a ver o Kanban `kanban` (id da sessão) — os avatares do cabeçalho.
    #[must_use]
    pub fn on_kanban(&self, kanban: KanbanId) -> Vec<PeerId> {
        self.seen
            .values()
            .filter(|(p, _)| p.kanban == Some(kanban))
            .map(|(p, _)| p.peer)
            .collect()
    }

    /// Quem tem o cartão `card` aberto — os avatares do cartão.
    #[must_use]
    pub fn on_card(&self, kanban: KanbanId, card: CardId) -> Vec<PeerId> {
        self.seen
            .values()
            .filter(|(p, _)| p.kanban == Some(kanban) && p.card == Some(card))
            .map(|(p, _)| p.peer)
            .collect()
    }

    /// O cursor de `peer` a desenhar em `now_ms` (interpolado, como no Quadro).
    #[must_use]
    pub fn cursor(&self, peer: PeerId, now_ms: u64) -> Option<[f64; 2]> {
        let (p, t) = self.seen.get(&peer)?;
        p.cursor?;
        t.at(now_ms)
    }

    #[must_use]
    pub fn get(&self, peer: PeerId) -> Option<&KanbanPresence> {
        self.seen.get(&peer).map(|(p, _)| p)
    }
}

impl Viewers {
    /// Todas as presenças (para quem entra ver já onde estão os outros).
    pub fn all(&self) -> impl Iterator<Item = &KanbanPresence> {
        self.seen.values().map(|(p, _)| p)
    }
}
