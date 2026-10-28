//! **Os ids novos** de um Kanban — a porta ÚNICA ([`Kanban::mint`], [`Kanban::mint_n`]). Fora de
//! sessão, o contador local; numa sessão ao vivo ([`Kanban::set_session`], o molde do
//! `BoardDoc::set_session` do Quadro, ADR-0181 §5), sorteados com o [`SESSION_ID_BIT`]: dois colegas
//! a criar ao mesmo tempo nunca fabricam o mesmo id. O sorteio não é conteúdo nem vai para o ficheiro.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::kanban::Kanban;
use crate::records::{
    ActivityId, AttachmentId, AutomationId, CardId, CheckItemId, ChecklistId, CommentId, FieldId,
    LabelId, ListId, MirrorId,
};

/// O bit alto dos ids sorteados numa sessão: nunca colidem com os do contador local (que os
/// ficheiros usam), e a `Put` deles não sobe o contador.
pub const SESSION_ID_BIT: u64 = 1 << 63;

/// O passo do splitmix64 (Steele, Lea & Flood, 2014) — uma bijecção: contadores distintos dão
/// números distintos.
#[must_use]
pub fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// O sorteio de uma sessão (atómico: sorteia com `&self`). `None` = fora de sessão. O `PartialEq`
/// ignora-o; a CÓPIA sorteia noutra sequência (uma cópia de trabalho e o original nunca dão o mesmo).
#[derive(Debug, Default)]
pub(crate) struct Lottery(Option<AtomicU64>);

impl Lottery {
    fn new(seed: Option<u64>) -> Self {
        Self(seed.map(AtomicU64::new))
    }

    fn draw(&self) -> Option<u64> {
        let r = self.0.as_ref()?;
        Some(splitmix64(r.fetch_add(0x9e37_79b9_7f4a_7c15, Relaxed)))
    }
}

impl Clone for Lottery {
    fn clone(&self) -> Self {
        Self::new(self.draw().map(splitmix64))
    }
}

impl PartialEq for Lottery {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Uma fonte de ids seguidos para UM gesto (nenhum `Put` no meio): o contador sobe por ela; o
/// sorteio nunca devolve um id já usado nem um já dado.
pub(crate) struct Minter<'a> {
    k: &'a Kanban,
    next: u64,
    given: BTreeSet<u64>,
}

impl Minter<'_> {
    pub(crate) fn id(&mut self) -> u64 {
        if self.k.in_session() {
            loop {
                let id = self.k.draw();
                if self.given.insert(id) {
                    return id;
                }
            }
        }
        self.next += 1;
        self.next
    }

    /// Salta `n` ids do contador (já dados por um [`Kanban::mint`] do mesmo gesto). Em sessão, o
    /// sorteio não os repete.
    pub(crate) fn skip(&mut self, n: u64) {
        self.next += n;
    }
}

impl Kanban {
    /// Entra (`Some(semente)`, do sistema) ou sai (`None`) de uma sessão ao vivo.
    pub fn set_session(&mut self, seed: Option<u64>) {
        self.session = Lottery::new(seed);
    }

    /// Está numa sessão ao vivo?
    #[must_use]
    pub fn in_session(&self) -> bool {
        self.session.0.is_some()
    }

    /// Um sorteado com o bit alto que nenhum registo deste Kanban usa (só em sessão).
    fn draw(&self) -> u64 {
        loop {
            let id = self.session.draw().unwrap_or_default() | SESSION_ID_BIT;
            if !self.uses(id) {
                return id;
            }
        }
    }

    /// Algum registo (de qualquer tabela: os ids são de UM espaço) tem o id `id`?
    #[must_use]
    pub fn uses(&self, id: u64) -> bool {
        self.lists.contains_key(&ListId(id))
            || self.cards.contains_key(&CardId(id))
            || self.labels.contains_key(&LabelId(id))
            || self.checklists.contains_key(&ChecklistId(id))
            || self.items.contains_key(&CheckItemId(id))
            || self.comments.contains_key(&CommentId(id))
            || self.attachments.contains_key(&AttachmentId(id))
            || self.fields.contains_key(&FieldId(id))
            || self.activities.contains_key(&ActivityId(id))
            || self.mirrors.contains_key(&MirrorId(id))
            || self.automations.contains_key(&AutomationId(id))
            || self.events.contains_key(&crate::records::EventId(id))
            || self.inbox.contains_key(&crate::records::InboxId(id))
            || self.blocks.contains_key(&crate::records::BlockId(id))
            || self.seen.contains_key(&crate::records::SeenId(id))
            || self
                .project_links
                .contains_key(&crate::records::ProjectLinkId(id))
    }

    /// Um id novo para um registo deste Kanban. Não muda o documento. Fora de sessão, o contador:
    /// dois ids pedidos sem `Put` no meio são o MESMO — peça-os todos com [`Self::mint_n`]. Em
    /// sessão, sorteado ([`Self::set_session`]).
    #[must_use]
    pub fn mint(&self) -> u64 {
        if self.in_session() {
            return self.draw();
        }
        self.next_id + 1
    }

    /// `n` ids novos e distintos, para um gesto que cria vários registos.
    #[must_use]
    pub fn mint_n(&self, n: usize) -> Vec<u64> {
        let mut m = self.minter();
        (0..n).map(|_| m.id()).collect()
    }

    pub(crate) fn minter(&self) -> Minter<'_> {
        Minter {
            k: self,
            next: self.next_id,
            given: BTreeSet::new(),
        }
    }
}
