//! O estado da aba: a vista escolhida, o snapshot do quadro e os pedidos por aplicar (plano §3).

use crate::edit::InputMapEdit;
use crate::ids;
use crate::layout::Layout;
use crate::snapshot::Snapshot;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};

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
}

impl InputMapDoc {
    #[must_use]
    pub fn new(view: View) -> Self {
        Self {
            view,
            snap: Snapshot::default(),
            pending: Vec::new(),
            hits: HitIndex::default(),
            store: WidgetStore::with_capacity(0),
            layout: Layout::default(),
            hover: None,
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
        self.layout.area.contains(x, y)
    }

    /// O ponteiro passou: guarda o segmento para a dica. `true` = mudou (pede quadro).
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let h = self.hits.hit(x, y).and_then(View::of_id);
        let changed = h != self.hover;
        self.hover = h;
        changed
    }

    #[must_use]
    pub fn take_pending(&mut self) -> Vec<InputMapEdit> {
        std::mem::take(&mut self.pending)
    }
}
