//! O estado de SESSÃO da facilitação dos quadros (MiroClone W7, `docs/MiroClone/05_facilitacao.md`) — o
//! cronómetro de cada quadro, as sessões de voto e os painéis abertos; nada disto vai para o ficheiro
//! (F1). Do lado dos DOCUMENTOS, como o `documents_nav.rs`; quem o pinta é `board_facilitate`.

use std::collections::BTreeMap;

use ph2d_board_edit::{Timer, VoteSession, VoteSetup};
use ph2d_board_model::BoardId;

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
}
