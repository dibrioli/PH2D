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
    /// O ponteiro está sobre a faixa do Play (a dica dela, em todas as vistas).
    pub(crate) banner_hover: bool,
    /// O mapa como ficou depois do último `end_frame`: outro no quadro seguinte = trocado lá fora.
    applied: Option<ph2d_input::InputMap>,
    /// O estado da vista A.
    pub list: ListState,
    /// O estado da vista B.
    pub kb: crate::view_keyboard::state::KbState,
    /// O estado da vista C (o bench do Try it).
    pub tryit: crate::view_try::state::TryState,
    /// O que a última pintura pôs na tela (textos e campos numéricos).
    pub census: Census,
    /// O registo de controlos, o premido, o foco e os valores do movimento (`crate::interact`).
    pub(crate) ui: crate::interact::Ui,
    /// A cena da pintura (repintada quando o hover instantâneo de um alvo novo muda).
    pub(crate) scratch: ph2d_vector::VectorScene,
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
            banner_hover: false,
            applied: None,
            list: ListState::default(),
            kb: crate::view_keyboard::state::KbState::default(),
            tryit: crate::view_try::state::TryState::new(),
            census: Census::default(),
            ui: crate::interact::Ui::default(),
            scratch: ph2d_vector::VectorScene::default(),
        }
    }

    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    /// Trocar de vista desarma a escuta da aba: a próxima tecla na vista nova nunca vira ligação em silêncio.
    pub fn set_view(&mut self, view: View) {
        self.tryit.reread |= view == View::TryIt && self.view != view;
        if self.view != view {
            self.stop_listening();
        }
        self.view = view;
    }

    /// Pede à família que desarme a escuta da aba (a do Trigger, que troca, nunca chega ao snapshot).
    pub fn stop_listening(&mut self) {
        if self.snap.listening.is_some() || self.list.listen_request.is_some() {
            self.list.listen_request = Some(None);
        }
    }

    /// O mapa mudou fora da aba (load, undo/redo, o «Create this action» do Trigger): os `ActionId` são por
    /// mapa, então o número a meio, o arrasto, a escuta e os pedidos por aplicar caem; a acção aberta da B
    /// fica só se o mapa novo ainda a tem.
    pub fn map_swapped(&mut self, map: &ph2d_input::InputMap) {
        self.list.typing = None;
        self.list.scrub = None;
        self.list.listen_request = None;
        self.pending.clear();
        if self.kb.open.is_some_and(|a| map.get(a).is_none()) {
            self.kb.open = None;
        }
    }

    /// Antes dos pedidos do quadro: o mapa mudou fora da aba desde o último [`Self::end_frame`]?
    pub fn begin_frame(&mut self, map: &ph2d_input::InputMap) -> bool {
        let swapped = self.applied.as_ref().is_some_and(|m| m != map);
        if swapped {
            self.map_swapped(map);
        }
        swapped
    }

    /// Depois dos pedidos: guarda o mapa aplicado e arma/desarma a escuta pedida; sem `keep` (aba fechada, outra
    /// à frente, mapa trocado) a escuta da aba cai — a do Trigger, que troca, fica.
    pub fn end_frame(
        &mut self,
        map: &ph2d_input::InputMap,
        store: &mut WidgetStore,
        listen: Option<Option<ph2d_input::ActionId>>,
        keep: bool,
    ) {
        self.applied = Some(map.clone());
        match listen {
            Some(Some(id)) => store.listen_for_binding(id),
            Some(None) => store.stop_listening(),
            None => {}
        }
        if !keep && store.input_map_listening().is_some() && !store.input_map_listen_replaces() {
            store.stop_listening();
        }
    }

    /// O Play está a correr (o mapa da aba é provado ao vivo e desfeito no Stop).
    #[must_use]
    pub const fn play_running(&self) -> bool {
        self.snap.play_running
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
        self.tryit.reread |= snap.game_id != self.snap.game_id;
        self.snap = snap;
        crate::view_try::reread_player_file(self);
    }

    /// O quadro do bench do Try it: com o jogo parado, as teclas da aba (+ o comando ao vivo); com o Play a
    /// correr, o estado do jogo (`live`). Nunca escreve no mapa nem no `live`.
    pub fn tick_try(&mut self, live: &ph2d_input::InputState) {
        crate::view_try::tick(self, live);
    }

    /// O ficheiro do jogador deste jogo neste computador (o `ForgetPlayerKeys` apaga-o); `game_id` 0 não tem.
    #[must_use]
    pub fn player_file(&self) -> Option<std::path::PathBuf> {
        ph2d_player_controls::player_file_of(self.tryit.base.as_deref(), self.snap.game_id)
    }

    /// Carregar em `(x, y)`: um controlo do registo captura (dispara no Up dentro); o resto vai à vista. `true` =
    /// dentro da aba (tomado).
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        if let Some(took) = self.press_control(x, y) {
            return took;
        }
        self.press_legacy(x, y)
    }

    /// O carregar de antes do registo: o alvo dispara no Down.
    pub(crate) fn press_legacy(&mut self, x: f32, y: f32) -> bool {
        if let Some(v) = self.hits.hit(x, y).and_then(View::of_id) {
            self.set_view(v);
            return true;
        }
        if self.view == View::TryIt && self.layout.body.contains(x, y) {
            return crate::view_try::press(self, x, y);
        }
        if self.view == View::List && self.layout.body.contains(x, y) {
            return crate::view_list::press(self, x, y);
        }
        if self.view == View::Keyboard && self.layout.body.contains(x, y) {
            return crate::view_keyboard::press(self, x, y);
        }
        self.layout.area.contains(x, y)
    }

    /// O ponteiro passou: guarda o segmento para a dica. `true` = mudou (pede quadro).
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let was = (
            self.hot_control(),
            self.ui.pressed.map(|p| self.is_pressed(p)),
        );
        self.ui.pointer = Some((x, y));
        let moved = was
            != (
                self.hot_control(),
                self.ui.pressed.map(|p| self.is_pressed(p)),
            );
        let banner = self.layout.banner.is_some_and(|r| r.contains(x, y));
        let banner_changed = banner != self.banner_hover;
        self.banner_hover = banner;
        self.motion_in_view(x, y) || banner_changed || moved
    }

    fn motion_in_view(&mut self, x: f32, y: f32) -> bool {
        if self.view == View::Keyboard {
            let h = self.hits.hit(x, y).and_then(View::of_id);
            let changed = h != self.hover;
            self.hover = h;
            return crate::view_keyboard::motion(self, x, y) || changed;
        }
        let h = self.hits.hit(x, y).and_then(View::of_id);
        let row = self.snap.map.actions().iter().map(|a| a.id).find(|a| {
            self.hits
                .rect_for(crate::ids::ListHit::Row(*a).id())
                .is_some_and(|r| r.contains(x, y))
        });
        let dragged = crate::view_list::drag(self, x)
            | crate::view_list::drag_thumb(self, y)
            | crate::view_list::drag_text(self, x)
            | crate::view_try::drag_thumb(self, y)
            | crate::view_keyboard::hscroll::drag(self, x);
        let changed = h != self.hover || row != self.list.hover_row || dragged;
        let target = self.hits.hit(x, y).filter(|_| h.is_none());
        let changed = changed || target != self.list.hover_target;
        self.hover = h;
        self.list.hover_row = row;
        self.list.hover_target = target;
        changed
    }

    /// O botão soltou em `(x, y)`: o controlo capturado dispara se o Up é DENTRO dele; acaba o arrasto de um número.
    /// O Up no último ponto do ponteiro.
    pub fn release(&mut self) -> bool {
        let (x, y) = self.ui.pointer.unwrap_or((f32::NAN, f32::NAN));
        self.release_at(x, y)
    }

    pub fn release_at(&mut self, x: f32, y: f32) -> bool {
        let fired = self.release_control(x, y).unwrap_or(false);
        self.kb.thumb_drag = None;
        self.kb.hdrag = None;
        let thumb = self.tryit.thumb_drag.take().is_some();
        crate::view_list::release(self) || fired || thumb
    }

    /// A roda: rola a lista.
    pub fn wheel(&mut self, dy: f32) -> bool {
        self.wheel_shift(dy, false)
    }

    /// A roda com o Shift (ou os dois dedos de lado): rola o teclado na horizontal quando ele não cabe.
    pub fn wheel_shift(&mut self, dy: f32, shift: bool) -> bool {
        match self.view {
            View::List => crate::view_list::wheel(self, dy),
            View::TryIt if shift => crate::view_keyboard::hscroll::wheel(self, dy),
            View::TryIt => crate::view_try::wheel(self, dy),
            View::Keyboard => crate::view_keyboard::wheel(self, dy, shift),
        }
    }

    /// O deslize de lado do trackpad em `(x, y)`: só rola o teclado com o ponteiro sobre ele.
    pub fn wheel_sideways(&mut self, x: f32, y: f32, dx: f32) -> bool {
        crate::view_keyboard::hscroll::wheel_at(self, (x, y), dx)
    }

    /// Uma tecla para o campo focado da aba. `true` = tomada.
    pub fn key(&mut self, input: &KeyInput<'_>) -> bool {
        // A escuta e o diálogo ANTES do foco: o Esc cancela-os; à escuta, o Tab é uma tecla.
        if crate::view_try::before_focus_text(self, input) {
            return true;
        }
        if self.snap.listening.is_some() {
            return false;
        }
        // O número aberto da lista é dono do Tab, do Enter e do Esc (aplica e segue, aplica, devolve).
        let typing = matches!(self.view, View::List | View::Keyboard) && self.list.typing.is_some();
        let took = if typing {
            crate::view_list::key(self, input)
        } else {
            self.focus_text_key(input)
                || match self.view {
                    View::List => crate::view_list::key(self, input),
                    View::Keyboard => crate::view_keyboard::key(self, input),
                    View::TryIt => crate::view_try::key(self, input),
                }
        };
        if took {
            // O caret acende a cada tecla e o piscar recomeça dali (o do SO).
            self.restart_caret();
        }
        took
    }

    /// A tecla física (`DocumentSurface::raw_key`): a busca por tecla. `true` = tomada.
    pub fn raw_key(&mut self, key: Key, pressed: bool, writes_text: bool) -> bool {
        self.track_shift(key, pressed);
        // A escuta (C) e o diálogo da receita ANTES do foco; à escuta da A/B a tecla é da escuta partilhada.
        if crate::view_try::before_focus(self, key, pressed) {
            return true;
        }
        if self.snap.listening.is_some() {
            return false;
        }
        let editing =
            matches!(self.view, View::List | View::Keyboard) && self.list.typing.is_some();
        if !editing && self.focus_raw_key(key, pressed) {
            return true;
        }
        match self.view {
            View::List => crate::view_list::raw_key(self, key, pressed, writes_text),
            View::Keyboard => crate::view_keyboard::raw_key(self, key, pressed, writes_text),
            View::TryIt => crate::view_try::raw_key(self, key, pressed),
        }
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

    /// O caret e a âncora da selecção de um campo de texto, e se o caret está aceso nesta fase do piscar.
    #[must_use]
    pub fn field_caret(&self, id: NodeId) -> Option<(usize, Option<usize>, bool)> {
        match self.store.get(id)? {
            ph2d_editor_core::interaction::InteractiveState::TextInput {
                caret,
                selection_anchor,
                ..
            } => Some((*caret, *selection_anchor, self.store.caret_visible(id))),
            _ => None,
        }
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

    /// O cartão da vista da última pintura (tudo o que a vista pinta mora nele).
    #[must_use]
    pub const fn body(&self) -> ph2d_editor_core::zones::Rect {
        self.layout.body
    }

    #[must_use]
    pub fn take_pending(&mut self) -> Vec<InputMapEdit> {
        std::mem::take(&mut self.pending)
    }
}

/// O `WidgetStore` da aba: os dois campos de texto do cartão (nome novo e busca).
fn text_fields() -> WidgetStore {
    let mut s = WidgetStore::with_capacity(4);
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
    for id in [ids::ADD, crate::view_keyboard::state::KbHit::Remove.id()] {
        s.register(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    s
}

#[cfg(test)]
#[path = "doc_tests.rs"]
mod tests;
