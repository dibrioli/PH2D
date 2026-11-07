//! W4 do Kanban ([ADR-0195] §3) — **um Kanban no quadro**: o elemento guarda só a REFERÊNCIA ao
//! documento do projecto ([`KanbanId`]) e o estado da vista DESTE elemento ([`KanbanBoardView`]). O
//! documento vive no `KanbanSet` do projecto: apagar o elemento nunca o apaga, e dois elementos (ou a
//! aba e um elemento) sobre o mesmo Kanban leem o MESMO registo.
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub use ph2d_kanban_model::{KanbanId, KanbanSet, ListId};
use serde::{Deserialize, Serialize};

use crate::{BoardDoc, Element, ElementId, ElementKind, FracKey, Style};

/// O que é SÓ desta vista do Kanban — nunca dado do documento. ⚠️ postcard é posicional: campo novo
/// sobe o [`crate::FORMAT_VERSION`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// Os Kanbans do projecto, partilhados com o editor, o desenho e a aba.
pub type KanbanSource = Rc<RefCell<KanbanSet>>;

thread_local! {
    static SOURCE: RefCell<Option<KanbanSource>> = const { RefCell::new(None) };
}

/// Os Kanbans do projecto que este fio lê (o editor instala o `Rc` do projecto a cada quadro).
pub fn install_kanbans(set: KanbanSource) {
    SOURCE.with(|s| *s.borrow_mut() = Some(set));
}

/// Os Kanbans instalados neste fio (`None` = nenhum).
#[must_use]
pub fn installed_kanbans() -> Option<KanbanSource> {
    SOURCE.with(|s| s.borrow().clone())
}

impl Element {
    /// ⭐ D17 — uma referência a um Kanban que o projecto NÃO tem (apagado, ou de um quadro vindo de
    /// fora) é INERTE: não se pinta, não se apanha, não se exporta — mas fica no documento, com a
    /// caixa e as chaves, e revive se o Kanban volta (o `Ctrl+Z` do apagar). Sem Kanbans instalados
    /// (ou com o registo emprestado) nada é inerte.
    #[must_use]
    pub fn kanban_missing(&self) -> bool {
        let Some(k) = self.kanban() else {
            return false;
        };
        SOURCE.with(|s| {
            s.borrow()
                .as_ref()
                .and_then(|set| set.try_borrow().ok().map(|set| set.get(k.kanban).is_none()))
                .unwrap_or(false)
        })
    }

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
