//! ⭐ W23 B15/B14 — **o que o quadro tem À ESPERA da pintura** e o **aviso com um botão**: dados do
//! documento (o `BoardLive` guarda-os), pintados e clicados pelo ecrã (`screens::hero::board_alert`,
//! `board_send`). Moram aqui pela DAG da fundação: `documents` não lê `screens`.

use ph2d_board_model::BoardId;
use std::time::Instant;

/// A escolha do modal à espera da pintura: de que quadro, para qual (`None` = um novo), copiar ou mover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Send {
    pub(crate) from: BoardId,
    pub(crate) to: Option<BoardId>,
    pub(crate) keep: bool,
}

/// Quanto vive um aviso COM botão, em segundos: o tecto do Material para um *snackbar* com acção
/// (*«4–10 seconds»*) — dá tempo de decidir, não só de ler; um texto longo vive o que a leitura pede.
pub const LIFE_S: f32 = 10.0;

/// O que o botão faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Desfaz os dois passos de um «Move to board…» — só enquanto ambos são o topo do desfazer do
    /// quadro deles (`*_at` = a hora do passo, [`ph2d_board_model::History::undo_at`]).
    UndoMove {
        from: BoardId,
        from_at: u64,
        to: BoardId,
        to_at: u64,
    },
    /// W23 B14 — abre o fio de uma menção ou resposta (no quadro dele).
    ShowThread {
        board: BoardId,
        thread: ph2d_board_model::ThreadId,
    },
}

impl Action {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::UndoMove { .. } => "board.send.undo",
            Self::ShowThread { .. } => "board.comment.show",
        }
    }
}

/// Um aviso do quadro.
#[derive(Clone, Debug)]
pub(crate) struct Alert {
    pub(crate) text: String,
    pub(crate) action: Option<Action>,
    pub(crate) at: Instant,
}

impl Alert {
    pub(crate) fn new(text: String, action: Option<Action>) -> Self {
        Self {
            text,
            action,
            at: Instant::now(),
        }
    }

    pub(crate) fn life_s(&self) -> f32 {
        LIFE_S.max(crate::toast_reading::reading_ttl_s(&self.text))
    }
}
