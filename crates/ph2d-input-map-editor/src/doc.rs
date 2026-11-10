//! O estado da aba: a vista escolhida, o snapshot do quadro e os pedidos por aplicar (plano §3).

use crate::edit::InputMapEdit;
use crate::ids;
use crate::layout::Layout;
use crate::snapshot::Snapshot;
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::KeyInput;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;
use ph2d_input::Key;

use crate::census::Census;
use crate::view_list::state::ListState;

/// As três vistas do mesmo mapa. A 1.ª vez é o [`View::Keyboard`]; a última escolha vai às prefs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum View {
    List,
    #[default]
    Keyboard,
    TryIt,
}

impl View {
    pub const ALL: [Self; 3] = [Self::List, Self::Keyboard, Self::TryIt];

    /// O valor de `input_map_view=` nas prefs.
    #[must_use]
    pub const fn pref_key(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Keyboard => "keyboard",
            Self::TryIt => "try",
        }
    }

    #[must_use]
    pub fn from_pref_key(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.pref_key() == s.trim())
    }

    /// O segmento do seletor.
    #[must_use]
    pub const fn id(self) -> NodeId {
        match self {
            Self::List => ids::VIEW_LIST,
            Self::Keyboard => ids::VIEW_KEYBOARD,
            Self::TryIt => ids::VIEW_TRY,
        }
    }

    #[must_use]
    pub fn of_id(id: NodeId) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.id() == id)
    }

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::List => "input_map_tab.view.list",
            Self::Keyboard => "input_map_tab.view.keyboard",
            Self::TryIt => "input_map_tab.view.try",
        }
    }

    #[must_use]
    pub const fn tip_key(self) -> &'static str {
        match self {
            Self::List => "input_map_tab.view.list_tip",
            Self::Keyboard => "input_map_tab.view.keyboard_tip",
            Self::TryIt => "input_map_tab.view.try_tip",
        }
    }
}

/// Uma aba Input Map.
pub struct InputMapDoc {
    view: View,
    pub snap: Snapshot,
    /// O que os gestos pediram desde o último `frame`; a família drena-o.
    pub pending: Vec<InputMapEdit>,
    /// Os alvos da última pintura (os segmentos do seletor).
    pub(crate) hits: HitIndex,
    /// O cartão pinta por um `WidgetStore`; a aba não tem um, e este fica vazio.
    pub(crate) store: WidgetStore,
    pub(crate) layout: Layout,
    /// O segmento sob o ponteiro (a dica).
    pub(crate) hover: Option<View>,
    /// O estado da vista A.
    pub list: ListState,
    /// O que a última pintura pôs na tela (textos e campos numéricos).
    pub census: Census,
}

impl InputMapDoc {
    #[must_use]
    pub fn new(view: View) -> Self {
        Self {
            view,
            snap: Snapshot::default(),
            pending: Vec::new(),
            hits: HitIndex::default(),
            store: text_fields(),
            layout: Layout::default(),
            hover: None,
            list: ListState::default(),
            census: Census::default(),
        }
    }

    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    pub fn set_view(&mut self, view: View) {
        self.view = view;
    }

    /// O segmento sob o ponteiro.
    #[must_use]
    pub const fn hover(&self) -> Option<View> {
        self.hover
    }

    /// Onde está o segmento da vista `v` (da última pintura).
    #[must_use]
    pub fn segment_rect(&self, v: View) -> Option<ph2d_editor_core::zones::Rect> {
        self.hits.rect_for(v.id())
    }

    pub fn set_snapshot(&mut self, snap: Snapshot) {
        self.snap = snap;
    }

    /// Carregar em `(x, y)`: um segmento troca a vista. `true` = dentro da aba (tomado).
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        if let Some(v) = self.hits.hit(x, y).and_then(View::of_id) {
            self.view = v;
            return true;
        }
        if self.view == View::List && self.layout.body.contains(x, y) {
            return crate::view_list::press(self, x, y);
        }
        self.layout.area.contains(x, y)
    }

    /// O ponteiro passou: guarda o segmento para a dica. `true` = mudou (pede quadro).
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let h = self.hits.hit(x, y).and_then(View::of_id);
        let row = self.snap.map.actions().iter().map(|a| a.id).find(|a| {
            self.hits
                .rect_for(crate::ids::ListHit::Row(*a).id())
                .is_some_and(|r| r.contains(x, y))
        });
        let dragged = self.view == View::List && crate::view_list::drag(self, x);
        let changed = h != self.hover || row != self.list.hover_row || dragged;
        let target = self.hits.hit(x, y).filter(|_| h.is_none());
        let changed = changed || target != self.list.hover_target;
        self.hover = h;
        self.list.hover_row = row;
        self.list.hover_target = target;
        changed
    }

    /// O botão soltou: acaba o arrasto de um número.
    pub fn release(&mut self) -> bool {
        crate::view_list::release(self)
    }

    /// A roda: rola a lista.
    pub fn wheel(&mut self, dy: f32) -> bool {
        self.view == View::List && crate::view_list::wheel(self, dy)
    }

    /// Uma tecla para o campo focado da aba. `true` = tomada.
    pub fn key(&mut self, input: &KeyInput<'_>) -> bool {
        self.view == View::List && crate::view_list::key(self, input)
    }

    /// A tecla física (`DocumentSurface::raw_key`): a busca por tecla. `true` = tomada.
    pub fn raw_key(&mut self, key: Key, pressed: bool, writes_text: bool) -> bool {
        self.view == View::List && crate::view_list::raw_key(self, key, pressed, writes_text)
    }

    /// ⭐ Um arrasto de número está vivo: o undo do projecto espera o fim (um passo por arrasto).
    #[must_use]
    pub const fn gesture_in_progress(&self) -> bool {
        self.list.scrub.is_some()
    }

    /// O pedido à escuta partilhada (`Some(Some(a))` arma, `Some(None)` desarma). A família drena-o.
    pub fn take_listen_request(&mut self) -> Option<Option<ph2d_input::ActionId>> {
        self.list.listen_request.take()
    }

    /// O texto de um campo da aba (`ids::NEW_NAME`, `ids::SEARCH`).
    #[must_use]
    pub fn field_text(&self, id: NodeId) -> &str {
        self.store.text(id).unwrap_or("")
    }

    /// O campo focado.
    #[must_use]
    pub fn focus(&self) -> Option<NodeId> {
        self.store.focus_id()
    }

    /// Onde um alvo foi pintado (da última pintura).
    #[must_use]
    pub fn rect_of(&self, id: NodeId) -> Option<ph2d_editor_core::zones::Rect> {
        self.hits.rect_for(id)
    }

    #[must_use]
    pub fn take_pending(&mut self) -> Vec<InputMapEdit> {
        std::mem::take(&mut self.pending)
    }
}

/// O `WidgetStore` da aba: os dois campos de texto do cartão (nome novo e busca).
fn text_fields() -> WidgetStore {
    let mut s = WidgetStore::with_capacity(2);
    for id in [ids::NEW_NAME, ids::SEARCH] {
        s.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    s
}
