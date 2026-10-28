//! W4 do Kanban ([ADR-0195] §3) — **um Kanban no quadro**: o elemento guarda só a REFERÊNCIA ao
//! documento do projecto ([`KanbanId`]) e o estado da vista DESTE elemento ([`KanbanBoardView`]). O
//! documento vive no `KanbanSet` do projecto: apagar o elemento nunca o apaga, e dois elementos (ou a
//! aba e um elemento) sobre o mesmo Kanban leem o MESMO registo.
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

pub use ph2d_kanban_model::{KanbanId, ListId};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{BoardDoc, Element, ElementId, ElementKind, FracKey, Style};

/// O que é SÓ desta vista do Kanban — nunca dado do documento. ⚠️ postcard é posicional: campo novo
/// sobe o [`crate::FORMAT_VERSION`] e o leitor do degrau anterior ([`KanbanBoardViewV18`]).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KanbanBoardView {
    /// A rolagem horizontal do quadro, em px da vista.
    pub scroll_x: f64,
    /// O zoom interno (`1` = o tamanho da aba).
    pub zoom: f64,
    /// As listas recolhidas NESTA vista (o Trello: recolher é pessoal).
    pub collapsed: BTreeSet<ListId>,
    /// A rolagem vertical de cada lista, em px da vista (formato 19); ausente = `0`.
    pub scroll_y: BTreeMap<ListId, f64>,
}

impl Default for KanbanBoardView {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            zoom: 1.0,
            collapsed: BTreeSet::new(),
            scroll_y: BTreeMap::new(),
        }
    }
}

thread_local! {
    /// O formato que [`crate::BoardSet::from_bytes`] está a ler neste fio (o resto lê o de hoje).
    static READING: Cell<u32> = const { Cell::new(crate::FORMAT_VERSION) };
}

/// Corre `f` a ler bytes do formato `version` (a vista do Kanban escolhe a forma por ele).
pub(crate) fn reading<R>(version: u32, f: impl FnOnce() -> R) -> R {
    let before = READING.with(|r| r.replace(version));
    let out = f();
    READING.with(|r| r.set(before));
    out
}

/// A vista do formato 18: uma rolagem `[x, y]` (o `y` nunca se usou) e sem a rolagem das listas.
#[derive(Deserialize)]
struct KanbanBoardViewV18 {
    scroll: [f64; 2],
    zoom: f64,
    collapsed: BTreeSet<ListId>,
}

#[derive(Deserialize)]
struct KanbanBoardViewV19 {
    scroll_x: f64,
    zoom: f64,
    collapsed: BTreeSet<ListId>,
    scroll_y: BTreeMap<ListId, f64>,
}

impl<'de> Deserialize<'de> for KanbanBoardView {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if READING.with(Cell::get) <= 18 {
            let v = KanbanBoardViewV18::deserialize(d)?;
            return Ok(Self {
                scroll_x: v.scroll[0],
                zoom: v.zoom,
                collapsed: v.collapsed,
                scroll_y: BTreeMap::new(),
            });
        }
        let v = KanbanBoardViewV19::deserialize(d)?;
        Ok(Self {
            scroll_x: v.scroll_x,
            zoom: v.zoom,
            collapsed: v.collapsed,
            scroll_y: v.scroll_y,
        })
    }
}

impl KanbanBoardView {
    /// Recolhe ou abre a lista `list` NESTA vista.
    pub fn toggle_list(&mut self, list: ListId) {
        if !self.collapsed.remove(&list) {
            self.collapsed.insert(list);
        }
    }
}

/// Um Kanban no quadro: estica-se à caixa `x, y, w, h` do elemento.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardKanban {
    /// O documento do projecto que este elemento mostra.
    pub kanban: KanbanId,
    pub view: KanbanBoardView,
    /// A opacidade, os cantos redondos e a moldura, como num vídeo.
    pub style: Style,
}

impl BoardKanban {
    /// A referência a `kanban`, com a vista de nascença e cantos redondos (os do Trello).
    #[must_use]
    pub fn new(kanban: KanbanId, mut style: Style) -> Self {
        style.round = true;
        Self {
            kanban,
            view: KanbanBoardView::default(),
            style,
        }
    }
}

impl Element {
    /// Um Kanban na caixa `[x, y, w, h]` (a referência; o documento é de quem chama).
    #[must_use]
    pub fn new_kanban(id: ElementId, z: FracKey, k: BoardKanban, bx: [f64; 4]) -> Self {
        Self::bare(id, z, ElementKind::Kanban(Box::new(k)), bx)
    }

    /// O Kanban, se é um.
    #[must_use]
    pub fn kanban(&self) -> Option<&BoardKanban> {
        match &self.kind {
            ElementKind::Kanban(k) => Some(k),
            _ => None,
        }
    }

    pub fn kanban_mut(&mut self) -> Option<&mut BoardKanban> {
        match &mut self.kind {
            ElementKind::Kanban(k) => Some(k),
            _ => None,
        }
    }
}

impl BoardDoc {
    /// A vista do Kanban `id` para a mudar. ⚠️ Não é uma op: não se desfaz, não viaja ao vivo e não
    /// muda a [`BoardDoc::rev`] — é estado de VISTA (rolar uma lista não é editar o quadro).
    pub fn kanban_view_mut(&mut self, id: ElementId) -> Option<&mut KanbanBoardView> {
        match &mut self.elements.get_mut(&id).filter(|e| !e.deleted)?.kind {
            ElementKind::Kanban(k) => Some(&mut k.view),
            _ => None,
        }
    }

    /// Os elementos vivos que mostram o Kanban `kanban` (a aba pergunta «onde está no quadro?»).
    #[must_use]
    pub fn kanban_elements(&self, kanban: KanbanId) -> Vec<ElementId> {
        self.live_with_hidden()
            .filter(|e| e.kanban().is_some_and(|k| k.kanban == kanban))
            .map(|e| e.id)
            .collect()
    }
}

#[cfg(test)]
#[path = "kanban_tests.rs"]
mod tests;
