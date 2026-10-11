//! **A votação** (W7, `docs/MiroClone/05_facilitacao.md` §2 V1–V7): a SESSÃO (votos por pessoa, «um
//! voto por objecto», os votos da pessoa ao ecrã, «I'm done» passa à seguinte) vive no editor; o
//! RESULTADO — o total de cada objecto — vai para o documento (`Element::votes`) num passo de desfazer.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History};
pub use ph2d_board_model::{VOTES_DEFAULT, VOTES_MAX, VoteSetup};

use crate::{Editor, Notice};

/// Uma sessão de voto aberta.
#[derive(Clone, Debug, PartialEq)]
pub struct VoteSession {
    setup: VoteSetup,
    targets: BTreeSet<ElementId>,
    /// Os votos de quem já disse «I'm done».
    totals: BTreeMap<ElementId, u32>,
    /// Os votos da pessoa ao ecrã.
    mine: BTreeMap<ElementId, u32>,
    voters: u32,
}

/// V1 — vota-se em formas e notas (o filtro por tipo do Miro); não em setas, traços nem frames, nem
/// num nó escondido por um ramo recolhido.
#[must_use]
pub fn votable(el: &Element) -> bool {
    el.shape().is_some() && !el.is_hidden()
}

impl VoteSession {
    /// V1 — os votáveis da selecção (com o que vive dentro dos frames dela), ou do quadro inteiro.
    /// `None` = nada a votar.
    #[must_use]
    pub fn new(doc: &BoardDoc, scope: &BTreeSet<ElementId>, setup: VoteSetup) -> Option<Self> {
        let ids: Vec<ElementId> = scope.iter().copied().collect();
        let targets: BTreeSet<ElementId> = if scope.is_empty() {
            doc.live().filter(|e| votable(e)).map(|e| e.id).collect()
        } else {
            crate::organize::with_descendants(doc, &ids)
                .into_iter()
                .filter(|id| doc.get(*id).is_some_and(votable))
                .collect()
        };
        let setup = VoteSetup {
            votes: setup.votes.clamp(1, VOTES_MAX),
            ..setup
        };
        (!targets.is_empty()).then(|| Self {
            setup,
            targets,
            totals: BTreeMap::new(),
            mine: BTreeMap::new(),
            voters: 0,
        })
    }

    #[must_use]
    pub fn setup(&self) -> VoteSetup {
        self.setup
    }

    #[must_use]
    pub fn targets(&self) -> &BTreeSet<ElementId> {
        &self.targets
    }

    /// V3 — os votos que a pessoa ao ecrã ainda tem.
    #[must_use]
    pub fn left(&self) -> u32 {
        self.setup.votes - self.mine.values().sum::<u32>()
    }

    /// V2 — os votos da pessoa ao ecrã em `id` (os dos outros não se vêem até ao fim).
    #[must_use]
    pub fn mine(&self, id: ElementId) -> u32 {
        self.mine.get(&id).copied().unwrap_or(0)
    }

    /// V3 — quantas pessoas já disseram «I'm done».
    #[must_use]
    pub fn voters(&self) -> u32 {
        self.voters
    }

    /// V2 — um voto em `id`. `Err` = a recusa que o artista lê.
    pub fn cast(&mut self, id: ElementId) -> Result<(), Notice> {
        if !self.targets.contains(&id) {
            return Err(Notice::NothingToVote);
        }
        if self.left() == 0 {
            return Err(Notice::NoVotesLeft);
        }
        if self.setup.one_per_object && self.mine(id) > 0 {
            return Err(Notice::OneVotePerObject);
        }
        *self.mine.entry(id).or_default() += 1;
        Ok(())
    }

    /// V2 — tira um voto seu de `id`. `false` = não tinha.
    pub fn retract(&mut self, id: ElementId) -> bool {
        let Some(n) = self.mine.get_mut(&id) else {
            return false;
        };
        *n -= 1;
        if *n == 0 {
            self.mine.remove(&id);
        }
        true
    }

    /// V3 — «I'm done»: os votos desta pessoa somam-se (e escondem-se), a seguinte começa do zero.
    pub fn next_voter(&mut self) {
        for (id, n) in std::mem::take(&mut self.mine) {
            *self.totals.entry(id).or_default() += n;
        }
        self.voters += 1;
    }

    /// V2 — os alvos apagados saem (e os votos postos neles voltam a quem os pôs: «N votes left» não
    /// conta votos num objecto que já não existe — auditoria W7). `true` = mudou.
    fn prune(&mut self, doc: &BoardDoc) -> bool {
        let gone: Vec<ElementId> = self
            .targets
            .iter()
            .copied()
            .filter(|id| doc.get(*id).is_none())
            .collect();
        for id in &gone {
            self.targets.remove(id);
            self.mine.remove(id);
            self.totals.remove(id);
        }
        !gone.is_empty()
    }

    /// V4 — os totais, com os da pessoa ao ecrã.
    fn finish(mut self) -> BTreeMap<ElementId, u32> {
        if !self.mine.is_empty() {
            self.next_voter();
        }
        self.totals
    }
}

/// V4 — o resultado no quadro: `(id, total)` dos que têm votos, do maior para o menor (empate: a
/// ordem de leitura — de cima, da esquerda).
#[must_use]
pub fn results(doc: &BoardDoc) -> Vec<(ElementId, u32)> {
    let mut v: Vec<&Element> = doc.live().filter(|e| e.votes() > 0).collect();
    v.sort_by(|a, b| {
        b.votes()
            .cmp(&a.votes())
            .then(a.y.total_cmp(&b.y))
            .then(a.x.total_cmp(&b.x))
            .then(a.id.cmp(&b.id))
    });
    v.into_iter().map(|e| (e.id, e.votes())).collect()
}

/// V5 — o menor e o maior total no quadro (o tamanho dos selos). `None` = nenhum resultado.
#[must_use]
pub fn vote_range(doc: &BoardDoc) -> Option<[u32; 2]> {
    doc.live()
        .map(Element::votes)
        .filter(|n| *n > 0)
        .fold(None, |r, n| match r {
            None => Some([n, n]),
            Some([lo, hi]) => Some([lo.min(n), hi.max(n)]),
        })
}

impl Editor {
    /// V1 — abre uma sessão sobre a selecção (ou o quadro inteiro). `false` = nada a votar (o aviso).
    pub fn start_vote(&mut self, doc: &BoardDoc, setup: VoteSetup) -> bool {
        match VoteSession::new(doc, &self.selection, setup) {
            Some(s) => {
                self.vote = Some(s);
                self.selection.clear();
                true
            }
            None => {
                self.notice = Some(Notice::NothingToVote);
                false
            }
        }
    }

    /// A sessão aberta, se há.
    #[must_use]
    pub fn vote(&self) -> Option<&VoteSession> {
        self.vote.as_ref()
    }

    /// Tira a sessão do editor (a aba muda: ela fica à espera do quadro dela).
    pub fn take_vote(&mut self) -> Option<VoteSession> {
        self.vote.take()
    }

    /// Devolve ao editor a sessão do quadro que voltou a ser o activo.
    pub fn put_vote(&mut self, s: Option<VoteSession>) {
        self.vote = s;
    }

    /// V2 — larga da sessão os alvos que foram apagados (chamado antes de cada desenho).
    pub fn prune_vote(&mut self, doc: &BoardDoc) -> bool {
        self.vote.as_mut().is_some_and(|s| s.prune(doc))
    }

    /// V3 — «I'm done». `false` = não há sessão.
    pub fn next_voter(&mut self) -> bool {
        self.vote.as_mut().map(VoteSession::next_voter).is_some()
    }

    /// V4 — «End for all»: os totais vão para o documento num passo — o resultado anterior sai do
    /// quadro inteiro. `false` = não havia sessão.
    pub fn end_vote(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let Some(s) = self.vote.take() else {
            return false;
        };
        let totals = s.finish();
        let ops: Vec<BoardOp> = doc
            .live_with_hidden()
            .filter_map(|e| {
                let want = totals.get(&e.id).copied().unwrap_or(0);
                (e.votes() != want).then(|| {
                    let mut e = e.clone();
                    e.set_votes(want);
                    BoardOp::Put(e)
                })
            })
            .collect();
        history.apply(doc, ops);
        true
    }

    /// V7 — «Clear results»: tira o resultado do quadro (um passo). `false` = não havia.
    pub fn clear_votes(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let ops: Vec<BoardOp> = doc
            .live_with_hidden()
            .filter(|e| e.votes() > 0)
            .map(|e| {
                let mut e = e.clone();
                e.set_votes(0);
                BoardOp::Put(e)
            })
            .collect();
        let any = !ops.is_empty();
        history.apply(doc, ops);
        any
    }

    /// V2 — um clique no `+` (ou no `−`) de `id`. A recusa fica no aviso.
    pub(crate) fn vote_click(&mut self, id: ElementId, minus: bool) {
        let Some(s) = self.vote.as_mut() else {
            return;
        };
        if minus {
            s.retract(id);
        } else if let Err(n) = s.cast(id) {
            self.notice = Some(n);
        }
    }
}
