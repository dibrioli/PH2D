//! O estado de SESSÃO da facilitação dos quadros (MiroClone W7, `docs/MiroClone/05_facilitacao.md`) — o
//! cronómetro de cada quadro, as sessões de voto e os painéis abertos; nada disto vai para o ficheiro
//! (F1). Do lado dos DOCUMENTOS, como o `documents_nav.rs`; quem o pinta é `board_facilitate`.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_edit::{Timer, VoteSession, VoteSetup};
use ph2d_board_model::{BoardId, ElementId};

/// O painel de votação de um quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VotePanel {
    /// A abrir uma sessão (V1).
    Setup(VoteSetup),
    /// A sessão aberta (V2–V3): o painel lê-a do editor.
    Session,
    /// O resultado (V4): o top 3, ou todos.
    Results { all: bool },
}

/// O estado de SESSÃO da facilitação.
#[derive(Default)]
pub(crate) struct Fac {
    /// O cronómetro de cada quadro com o cartão aberto (F1: trocar de aba não o pára).
    pub(crate) timers: BTreeMap<BoardId, Timer>,
    /// O painel de votação aberto em cada quadro.
    pub(crate) panels: BTreeMap<BoardId, VotePanel>,
    /// As sessões de voto dos quadros que NÃO são o activo (a do activo vive no editor).
    pub(crate) parked: BTreeMap<BoardId, VoteSession>,
    /// O quadro cuja sessão está no editor.
    pub(crate) in_editor: Option<BoardId>,
    /// V6 — o cronómetro destes quadros foi armado pela votação: no zero, ela acaba.
    pub(crate) vote_clock: BTreeMap<BoardId, bool>,
    /// A galeria dos carimbos e as opções de agrupar, abertas na barra de contexto.
    pub(crate) stamps_open: bool,
    pub(crate) cluster_open: bool,
    /// F6 — a campainha à espera de quem a toca (a shell, uma vez por quadro).
    pub(crate) chime: bool,
    /// W12 — a votação AO VIVO de cada quadro: a rodada que a sessão do editor segue, o meu
    /// «I'm done», e o último boletim enviado (um novo só sai quando muda).
    pub(crate) live_round: BTreeMap<BoardId, u32>,
    pub(crate) live_done: BTreeSet<BoardId>,
    pub(crate) live_sent: BTreeMap<BoardId, (BTreeMap<ElementId, u32>, bool)>,
    /// W12 — o painel do modo privado aberto em cada quadro, com as opções de ligar.
    pub(crate) private: BTreeMap<BoardId, PrivateSetup>,
    /// W12 — os quadros com o modo privado ligado (a virada abre o painel a todos, como o Miro).
    pub(crate) private_on: BTreeSet<BoardId>,
}

/// W12 — as opções de ligar o modo privado (as do diálogo do Miro).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PrivateSetup {
    pub(crate) anonymous: bool,
    pub(crate) anyone_can_end: bool,
}

impl Fac {
    /// Esquece tudo do quadro `id` (apagado). `true` = a sessão dele estava no editor (quem chama
    /// tira-a de lá).
    pub(crate) fn forget(&mut self, id: BoardId) -> bool {
        self.timers.remove(&id);
        self.panels.remove(&id);
        self.parked.remove(&id);
        self.vote_clock.remove(&id);
        self.live_round.remove(&id);
        self.live_done.remove(&id);
        self.live_sent.remove(&id);
        self.private.remove(&id);
        self.private_on.remove(&id);
        let in_editor = self.in_editor == Some(id);
        if in_editor {
            self.in_editor = None;
        }
        in_editor
    }
}
