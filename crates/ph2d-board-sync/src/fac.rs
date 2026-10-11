//! **A facilitação ao vivo** ([ADR-0182]): o que TODOS vêem (o modo privado, as notas escondidas sem
//! autor, a votação sem as escolhas de ninguém) e os pedidos ao anfitrião, que decide.
//!
//! [ADR-0182]: ../../../docs/architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{ElementId, VoteSetup};
use serde::{Deserialize, Serialize};

use crate::PeerId;

/// «Turn off for all» → as notas revelam-se depois desta contagem (Miro PM: *«A 5-second countdown»*).
pub const REVEAL_COUNTDOWN_MS: u64 = 5_000;

/// O estado da facilitação de uma sala, como cada pessoa o recebe.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fac {
    pub private: Option<Private>,
    /// As notas com texto por revelar (de quem, não se diz).
    pub hidden: BTreeSet<ElementId>,
    pub vote: Option<VoteRoom>,
}

/// O modo privado ligado.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Private {
    /// Quem o ligou.
    pub by: PeerId,
    /// «Make names anonymous»: cursores e selecções sem nome nem cor, «seguir» desligado.
    pub anonymous: bool,
    /// «Allow anyone to turn off».
    pub anyone_can_end: bool,
    /// «Anyone can reveal their stickies» (só quem ligou o muda).
    pub anyone_can_reveal: bool,
    /// A contagem para revelar: o que faltava quando a mensagem saiu do anfitrião.
    pub reveal_in_ms: Option<u64>,
}

/// Uma votação entre pessoas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteRoom {
    /// A rodada (cada votação nova sobe-a): um boletim só conta na rodada dele.
    pub round: u32,
    /// Quem a iniciou (só essa pessoa a acaba à mão).
    pub by: PeerId,
    pub setup: VoteSetup,
    pub targets: BTreeSet<ElementId>,
    /// O que faltava no relógio quando a mensagem saiu do anfitrião (`None` = sem relógio).
    pub left_ms: Option<u64>,
    /// Quem já disse «I'm done» (o Miro mostra-o a todos).
    pub done: BTreeSet<PeerId>,
    /// Quantos votos cada pessoa usou — SÓ na cópia de quem iniciou (Miro VO); vazio nas outras.
    pub used: BTreeMap<PeerId, u32>,
    /// Acabou: os totais já estão no quadro.
    pub ended: bool,
}

/// O boletim de uma pessoa: vai inteiro ao anfitrião a cada mudança e substitui o anterior.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ballot {
    /// A rodada da votação em que se votou ([`VoteRoom::round`]).
    pub round: u32,
    pub votes: BTreeMap<ElementId, u32>,
    pub done: bool,
}

impl Ballot {
    #[must_use]
    pub fn used(&self) -> u32 {
        self.votes.values().sum()
    }
}

/// Um pedido ao anfitrião (o dele vale como o de qualquer um, pelas mesmas regras).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ask {
    PrivateOn {
        anonymous: bool,
        anyone_can_end: bool,
    },
    /// «Turn off for all»: arma a contagem e depois revela tudo.
    PrivateOff,
    /// Só quem ligou.
    AnyoneCanReveal(bool),
    /// «Reveal all my sticky notes».
    RevealMine,
    VoteStart {
        setup: VoteSetup,
        targets: BTreeSet<ElementId>,
    },
    /// «End for all» — só quem iniciou.
    VoteEnd,
}

/// Porque o anfitrião não fez o que se pediu (a frase que o artista lê).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// O modo privado só o desliga quem o ligou («Allow anyone to turn off» está desligado).
    OnlyStarterEndsPrivate,
    /// «Reveal all my sticky notes» não está ligado.
    RevealMineOff,
    /// Já há uma votação a decorrer.
    VoteRunning,
    /// Só quem iniciou a votação a acaba.
    OnlyStarterEndsVote,
    /// Nada a votar.
    NothingToVote,
}

#[cfg(test)]
#[path = "fac_tests.rs"]
mod tests;
