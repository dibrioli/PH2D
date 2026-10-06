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

pub use ph2d_board_model::BoardSet;
use ph2d_board_model::{Board, BoardId, History};
use std::collections::BTreeMap;

/// Os quadros e a aba activa.
#[derive(Debug, Default)]
pub struct Documents {
    boards: BoardSet,
    /// `None` = a aba `Cena`.
    active: Option<BoardId>,
    /// Âncora do arrasto da vista do quadro (px de ecrã), enquanto ele dura.
    pub(crate) pan_from: Option<[f64; 2]>,
    /// O quadro cuja aba é, agora, o campo de renomear.
    pub(crate) renaming: Option<BoardId>,
    /// Uma aba de quadro com o dedo em cima — ver `document_tabs_menu::pointer`.
    pub(crate) tab_drag: Option<TabDrag>,
    /// O que vive enquanto se edita um quadro (editor, desfazer por quadro, texto moldado).
    pub(crate) live: BoardLive,
}

/// ⭐ **O estado de SESSÃO dos quadros** — nada disto vai para o ficheiro.
#[derive(Default)]
pub(crate) struct BoardLive {
    /// O editor do quadro activo (nasce no 1.º gesto: o estilo de nascença vem do tema).
    pub(crate) editor: Option<ph2d_board_edit::Editor>,
    /// O desfazer de CADA quadro (plano §1.4: `Ctrl+Z` num quadro nunca desfaz outro nem a cena).
    pub(crate) histories: BTreeMap<BoardId, History>,
    pub(crate) render_cache: ph2d_board_render::RenderCache,
    /// O último carregar no quadro `(instante ns, x, y)` — o duplo-clique.
    pub(crate) last_down: Option<(u128, [f32; 2])>,
    /// `Espaço` em baixo: arrastar move a vista (o idioma do Figma/Miro/Excalidraw).
    pub(crate) space: bool,
    /// A grelha de todas as formas, aberta pelo «mais formas» da barra curta.
    pub(crate) shapes_open: bool,
}

impl std::fmt::Debug for BoardLive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoardLive")
            .field("histories", &self.histories.len())
            .field("space", &self.space)
            .finish_non_exhaustive()
    }
}

/// O dedo desceu sobre a aba de `board` em `start` e está em `cursor` (px de ecrã).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TabDrag {
    pub board: BoardId,
    pub start: (f32, f32),
    pub cursor: (f32, f32),
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
    /// Sair de um quadro termina o texto que se escrevia nele e esquece a selecção.
    pub fn activate(&mut self, id: Option<BoardId>) {
        let id = id.filter(|id| self.boards.get(*id).is_some());
        if id != self.active {
            self.leave_board();
        }
        self.active = id;
        self.pan_from = None;
    }

    fn leave_board(&mut self) {
        let Some(prev) = self.active else {
            return;
        };
        let live = &mut self.live;
        if let (Some(ed), Some(board)) = (live.editor.as_mut(), self.boards.get_mut(prev)) {
            let h = live.histories.entry(prev).or_default();
            ed.commit_text(&mut board.doc, h);
            ed.cancel_gesture(&mut board.doc);
            ed.select(&board.doc, []);
        }
    }

    /// O quadro activo, o histórico dele e o resto do estado de sessão — juntos, para um gesto.
    pub(crate) fn active_parts(&mut self) -> Option<(&mut Board, &mut BoardLive)> {
        let id = self.active?;
        let board = self.boards.get_mut(id)?;
        Some((board, &mut self.live))
    }

    /// Cria um quadro vazio no fim das abas e torna-o activo.
    pub fn create(&mut self, name: String) -> BoardId {
        let id = self.boards.create(name);
        self.activate(Some(id));
        id
    }

    /// Troca todos os quadros (ao carregar um projecto) e volta à `Cena`.
    pub fn replace(&mut self, boards: BoardSet) {
        self.leave_board();
        self.boards = boards;
        self.renaming = None;
        self.tab_drag = None;
        self.live.histories.clear();
        self.active = None;
        self.pan_from = None;
    }

    /// Muda o nome. `false` se `id` não existe.
    pub fn rename(&mut self, id: BoardId, name: String) -> bool {
        self.boards.rename(id, name)
    }

    /// Cópia de `id` logo à direita dele, e a cópia fica activa.
    pub fn duplicate(&mut self, id: BoardId, name: String) -> Option<BoardId> {
        let copy = self.boards.duplicate(id, name)?;
        self.activate(Some(copy));
        Some(copy)
    }

    /// Leva a aba de `id` para a posição `to`. `false` se `id` não existe.
    pub fn move_tab(&mut self, id: BoardId, to: usize) -> bool {
        self.boards.move_tab(id, to)
    }

    /// Apaga o quadro. Se era o activo, a aba que fica no lugar dele abre (a da direita, senão a
    /// da esquerda, senão a `Cena`) — o idioma das abas de um navegador.
    pub fn remove(&mut self, id: BoardId) -> Option<Board> {
        let at = self.boards.boards().iter().position(|b| b.id == id)?;
        if self.active == Some(id) {
            self.leave_board();
        }
        let gone = self.boards.remove(id)?;
        self.live.histories.remove(&id);
        if self.renaming == Some(id) {
            self.renaming = None;
        }
        if self.active == Some(id) {
            let list = self.boards.boards();
            let next = list.get(at).or_else(|| list.last()).map(|b| b.id);
            self.activate(next);
        }
        Some(gone)
    }
}
