//! ⭐ **Os DOCUMENTOS do projecto além da cena** — hoje, os quadros (MiroClone), e qual deles a
//! barra de cima tem activo.
//!
//! > *«Os quadros não devem aparecer na hierarquia mas devem aparecer como abas na barra superior
//! > do APP.»* — o dono, 2026-10-05 (`docs/MiroClone/02_plano.md` §1.1).
//!
//! ⚠️ **Aqui e não na shell**, pela razão do [`crate::screens::HeroScreen::input_map`]: as abas são
//! pintadas por `paint_hero_screen`, que só recebe o `HeroScreen`. É dado de PROJECTO: a shell lê-o
//! ao gravar ([`Documents::boards`]) e escreve-o ao carregar
//! ([`crate::screens::hero::document_tabs::load`]).

use ph2d_board_model::{Board, BoardId, BoardSet};

/// Os quadros e a aba activa.
#[derive(Debug, Default)]
pub struct Documents {
    boards: BoardSet,
    /// `None` = a aba `Cena`.
    active: Option<BoardId>,
    /// Mudou algo que se grava desde a última vez que a shell perguntou.
    dirty: bool,
    /// Âncora do arrasto da vista do quadro (px de ecrã), enquanto ele dura.
    pub(crate) pan_from: Option<[f64; 2]>,
}

impl Documents {
    /// Todos os quadros, na ordem das abas — o que a shell grava.
    #[must_use]
    pub fn boards(&self) -> &BoardSet {
        &self.boards
    }

    /// O quadro da aba activa, ou `None` se a aba activa é a `Cena`.
    #[must_use]
    pub fn active(&self) -> Option<BoardId> {
        self.active
    }

    #[must_use]
    pub fn active_board(&self) -> Option<&Board> {
        self.active.and_then(|id| self.boards.get(id))
    }

    pub fn active_board_mut(&mut self) -> Option<&mut Board> {
        self.active.and_then(|id| self.boards.get_mut(id))
    }

    /// Torna activa a aba de `id` (`None` = a `Cena`). Um id que não existe cai na `Cena`.
    pub fn activate(&mut self, id: Option<BoardId>) {
        self.active = id.filter(|id| self.boards.get(*id).is_some());
        self.pan_from = None;
    }

    /// Cria um quadro vazio no fim das abas e torna-o activo.
    pub fn create(&mut self, name: String) -> BoardId {
        let id = self.boards.create(name);
        self.activate(Some(id));
        self.dirty = true;
        id
    }

    /// Troca todos os quadros (ao carregar um projecto). Volta à `Cena`; não suja o projecto.
    pub fn replace(&mut self, boards: BoardSet) {
        self.boards = boards;
        self.activate(None);
        self.dirty = false;
    }

    /// `true` uma vez por mudança gravável — a shell acende o «modificado» do título.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }
}
