//! **O registo de controlos da aba** (plano `docs/InputMap/01_plano.md` §13.3): `doc.control(c)` regista o alvo
//! de clique E o controlo — pintou ⟹ clicável, focável, com nó de acessibilidade e cursor. O premido é nativo
//! (o Down captura, o Up DENTRO dispara); o hover é o D32 do dono (no quadro, sem fade); só o polegar desvanece.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::motion::{Role, UiMotion};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use crate::doc::InputMapDoc;

/// O que um controlo é — dá o papel accesskit, o cursor e se o gesto começa no Down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlRole {
    Button,
    /// Um segmento de um seletor (List · Keyboard · Try it): o `Tab` do accesskit.
    Segment,
    Check,
    ListItem,
    Key,
    Chip,
    Link,
    TextField,
    NumberField,
    /// Uma alça que se arrasta (Dead/Press): o gesto começa no Down.
    DragHandle,
    /// O polegar de uma barra: o gesto começa no Down; o hover dele desvanece (`Role::Fade`).
    ScrollThumb,
    /// Um aviso que não se clica (a faixa do Play): só dica e leitor de ecrã.
    Note,
}

impl ControlRole {
    #[must_use]
    pub const fn a11y(self) -> ph2d_a11y::Role {
        use ph2d_a11y::Role as R;
        match self {
            Self::Button | Self::Key | Self::Chip => R::Button,
            Self::Segment => R::Tab,
            Self::Check => R::CheckBox,
            Self::ListItem => R::ListItem,
            Self::Link => R::Link,
            Self::TextField => R::TextInput,
            Self::NumberField => R::SpinButton,
            Self::DragHandle => R::Slider,
            Self::ScrollThumb => R::ScrollBar,
            Self::Note => R::Status,
        }
    }

    /// O cursor de omissão.
    #[must_use]
    pub const fn look(self) -> PointerLook {
        match self {
            Self::TextField => PointerLook::Text,
            Self::NumberField | Self::DragHandle => PointerLook::ResizeEw,
            Self::ScrollThumb | Self::Note => PointerLook::Arrow,
            _ => PointerLook::Hand,
        }
    }

    /// O gesto começa no Down (arrasto), não no Up.
    #[must_use]
    pub const fn drags(self) -> bool {
        matches!(self, Self::DragHandle | Self::ScrollThumb)
    }

    /// Um clique faz alguma coisa.
    #[must_use]
    pub const fn clickable(self) -> bool {
        !matches!(self, Self::Note)
    }
}

/// Os nós de acessibilidade da aba (`DocumentSurface::a11y`).
pub type A11yNodes = Vec<(NodeId, ph2d_a11y::Node)>;

/// Um controlo pintado.
#[derive(Clone, Debug, PartialEq)]
pub struct Control {
    pub id: NodeId,
    pub rect: Rect,
    pub role: ControlRole,
    /// O nome lido pelo leitor de ecrã (já traduzido).
    pub label: String,
    pub look: PointerLook,
    /// Desligado: a chave i18n do porquê (a dica mostra-a; o clique não dispara).
    pub disabled_why: Option<&'static str>,
    pub focusable: bool,
}

impl Control {
    #[must_use]
    pub fn new(id: NodeId, rect: Rect, role: ControlRole, label: impl Into<String>) -> Self {
        Self {
            id,
            rect,
            role,
            label: label.into(),
            look: role.look(),
            disabled_why: None,
            focusable: role.clickable(),
        }
    }

    #[must_use]
    pub fn disabled(mut self, why: Option<&'static str>) -> Self {
        self.disabled_why = why;
        self
    }

    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.disabled_why.is_none()
    }
}

/// O estado de interacção da aba (o premido, o foco, os valores do `UiMotion`).
#[derive(Debug, Default)]
pub struct Ui {
    /// Os controlos da última pintura, pela ordem dela (a do Tab).
    pub(crate) controls: Vec<Control>,
    pub(crate) pointer: Option<(f32, f32)>,
    /// O controlo capturado no Down.
    pub(crate) pressed: Option<NodeId>,
    pub(crate) focus: Option<NodeId>,
    /// Shift premido (a porta física não traz modificadores).
    pub(crate) shift: bool,
    /// O valor do desvanecer de cada polegar (`Role::Fade`).
    pub(crate) fade: BTreeMap<NodeId, f32>,
    pub(crate) fading: bool,
    pub(crate) elapsed: f64,
    pub(crate) reduced: bool,
    pub(crate) decorates: bool,
    /// Pulsos e carets pedidos pela última pintura (pedem quadro).
    pub(crate) pulses: BTreeSet<NodeId>,
    pub(crate) carets: BTreeSet<NodeId>,
}

/// Um ciclo do pulso da escuta (s) — o de um indicador «à espera» de SO (macOS ≈ 1–1,5 s).
const PULSE_PERIOD_S: f64 = 1.2;
/// O meio período do caret (s): o de omissão do Windows/GTK (530 ms).
const CARET_HALF_S: f64 = 0.53;

/// A tecla do vocabulário do app (`ph2d_input::Key::named_in_script`) é uma destas?
fn is_named(key: ph2d_input::Key, names: &[&str]) -> bool {
    names
        .iter()
        .filter_map(|n| ph2d_input::Key::named_in_script(n))
        .flatten()
        .any(|k| k == key)
}

/// O que uma tecla faz ao foco.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusKey {
    Next,
    Prev,
    Activate,
    Drop,
}

impl InputMapDoc {
    /// Regista um controlo pintado: o alvo de clique, o premido, o foco, o nó a11y e o cursor.
    pub fn control(&mut self, c: Control) {
        if self.hits.rect_for(c.id) != Some(c.rect) {
            self.hits.register(c.id, c.rect);
        }
        if let Some(i) = self.ui.controls.iter().position(|o| o.id == c.id) {
            self.ui.controls.remove(i);
        }
        self.ui.controls.push(c);
    }

    fn control_of(&self, id: NodeId) -> Option<&Control> {
        self.ui.controls.iter().find(|c| c.id == id)
    }

    /// O controlo sob `(x, y)` (o recorte da pintura vale).
    #[must_use]
    pub fn control_at(&self, x: f32, y: f32) -> Option<&Control> {
        self.hits.hit(x, y).and_then(|id| self.control_of(id))
    }

    /// O ponteiro está sobre `id` (e nada outro está capturado).
    #[must_use]
    pub fn is_hovered(&self, id: NodeId) -> bool {
        self.hot() == Some(id)
    }

    /// O controlo quente (um alvo velho, sem registo, não conta).
    pub(crate) fn hot_control(&self) -> Option<NodeId> {
        self.hot().filter(|id| self.control_of(*id).is_some())
    }

    /// Os nós de acessibilidade da aba: o documento e um por controlo.
    #[must_use]
    pub fn a11y(&self) -> Vec<(NodeId, ph2d_a11y::Node)> {
        let title = ph2d_i18n::tr("input_map_tab.title");
        self.a11y_nodes(crate::ids::DOCUMENT, title, self.layout.area)
    }

    /// O id quente: o capturado só enquanto o ponteiro está nele; sem captura, o de cima.
    fn hot(&self) -> Option<NodeId> {
        let (x, y) = self.ui.pointer?;
        match self.ui.pressed {
            Some(p) => self
                .hits
                .rect_for(p)
                .filter(|r| r.contains(x, y))
                .map(|_| p),
            None => self.hits.hit(x, y),
        }
    }

    /// Premido: capturado no Down e o ponteiro ainda dentro (arrastar para fora apaga o premido).
    #[must_use]
    pub fn is_pressed(&self, id: NodeId) -> bool {
        self.ui.pressed == Some(id) && self.hot() == Some(id)
    }

    /// Quanto do hover a pintura lê em `id` (o `hover_live` do store da aba): `0` frio, `1` pairado.
    #[must_use]
    pub fn hover_live(&self, id: NodeId) -> f32 {
        self.store.hover_live(id)
    }

    /// O controlo com o foco do teclado (o anel).
    #[must_use]
    pub fn is_focused(&self, id: NodeId) -> bool {
        self.ui.focus == Some(id)
    }

    #[must_use]
    pub const fn focused(&self) -> Option<NodeId> {
        self.ui.focus
    }

    /// O cursor em `(x, y)`: o do controlo (seta se desligado); `None` fora de todo controlo.
    #[must_use]
    pub fn look_at(&self, x: f32, y: f32) -> Option<PointerLook> {
        let c = self.control_at(x, y)?;
        Some(if c.enabled() {
            c.look
        } else {
            PointerLook::Arrow
        })
    }

    /// A chave i18n do porquê de um controlo desligado sob o ponteiro (a dica).
    #[must_use]
    pub fn disabled_why_here(&self) -> Option<(NodeId, &'static str)> {
        let (x, y) = self.ui.pointer?;
        let c = self.control_at(x, y)?;
        c.disabled_why.map(|w| (c.id, w))
    }

    /// Os nós de acessibilidade: o documento (`doc_id`, com `label` e `area`) e um por controlo, pela ordem do Tab.
    #[must_use]
    pub fn a11y_nodes(
        &self,
        doc_id: NodeId,
        label: &str,
        area: Rect,
    ) -> Vec<(NodeId, ph2d_a11y::Node)> {
        let b = |r: Rect| {
            (
                f64::from(r.x),
                f64::from(r.y),
                f64::from(r.w),
                f64::from(r.h),
            )
        };
        let (x, y, w, h) = b(area);
        let root = ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Document)
            .label(label)
            .bounds(x, y, w, h)
            .children(self.ui.controls.iter().map(|c| c.id))
            .build();
        let mut out = vec![(doc_id, root)];
        for c in &self.ui.controls {
            let (x, y, w, h) = b(c.rect);
            let mut n = ph2d_a11y::NodeBuilder::new(c.role.a11y())
                .label(c.label.clone())
                .bounds(x, y, w, h)
                .focusable(c.focusable && c.enabled());
            if c.role.clickable() && c.enabled() {
                n = n.action(ph2d_a11y::Action::Click);
            }
            let mut node = n.build();
            if !c.enabled() {
                node.set_disabled();
            }
            let sub = crate::view_keyboard::state::KbHit::Sub(self.kb.sub).id();
            if c.role == ControlRole::Segment
                && (View::of_id(c.id) == Some(self.view()) || c.id == sub)
            {
                node.set_selected(true);
            }
            out.push((c.id, node));
        }
        out
    }

    /// O pulso de `id` (`0..=1`, da fase do relógio do substrato); `None` = estático (Discreto ou reduced motion:
    /// o `Role::Decoration` está ausente).
    pub fn pulse(&mut self, id: NodeId) -> Option<f32> {
        if !self.ui.decorates {
            return None;
        }
        self.ui.pulses.insert(id);
        let phase = (self.ui.elapsed / PULSE_PERIOD_S).fract();
        Some((0.5 - 0.5 * (phase * std::f64::consts::TAU).cos()) as f32)
    }

    /// O caret de `id` está aceso agora? Fixo (sempre aceso) com reduced motion.
    pub fn caret_on(&mut self, id: NodeId) -> bool {
        if self.ui.reduced {
            return true;
        }
        self.ui.carets.insert(id);
        (self.ui.elapsed / CARET_HALF_S).floor() % 2.0 < 1.0
    }

    /// O desvanecer de um polegar (`ControlRole::ScrollThumb`): `0` frio, `1` quente.
    #[must_use]
    pub fn thumb_fade(&self, id: NodeId) -> f32 {
        self.ui.fade.get(&id).copied().unwrap_or(0.0)
    }

    /// Pede quadro: um polegar a desvanecer, um pulso ou um caret vivo.
    #[must_use]
    pub fn wants_frame(&self) -> bool {
        self.ui.fading || !self.ui.pulses.is_empty() || !self.ui.carets.is_empty()
    }

    /// **O quadro do movimento** (a família chama-o com o `hero.motion`): o hover de cada alvo pintado, instantâneo
    /// (`Role::Number`, D32); o do polegar desvanece (`Role::Fade`, partida fria). E o relógio para pulsos/carets.
    pub fn tick_motion(&mut self, motion: &mut UiMotion) {
        self.ui.elapsed = motion.elapsed();
        self.ui.reduced = motion.reduced_motion();
        self.ui.decorates = motion.decorates();
        let hot = self.hot();
        let ids: Vec<NodeId> = self.hits.desde(0).iter().map(|(id, _)| *id).collect();
        let mut fading = false;
        for id in ids {
            let t = if hot == Some(id) { 1.0 } else { 0.0 };
            let thumb = self
                .control_of(id)
                .is_some_and(|c| c.role == ControlRole::ScrollThumb);
            let v = if thumb {
                if motion.get(id).is_none() {
                    motion.animate(id, 0.0, Role::Fade);
                }
                let v = motion.animate(id, t, Role::Fade);
                fading |= (v - t).abs() > f32::EPSILON;
                self.ui.fade.insert(id, v);
                v
            } else {
                motion.animate(id, t, Role::Number)
            };
            self.store.set_hover_live(id, v);
        }
        self.ui.fading = fading;
    }

    /// Publica o hover INSTANTÂNEO com os rects desta pintura (o `Role::Number` é a identidade): `true` = algum
    /// valor mudou (um id visto pela 1.ª vez lia `SETTLED` = pairado) e a pintura repete-se.
    pub(crate) fn settle_hover(&mut self) -> bool {
        let hot = self.hot();
        let mut changed = false;
        let pairs: Vec<(NodeId, bool)> = self
            .hits
            .desde(0)
            .iter()
            .map(|(id, _)| (*id, self.ui.fade.contains_key(id)))
            .collect();
        for (id, fades) in pairs {
            let v = if fades {
                self.thumb_fade(id)
            } else if hot == Some(id) {
                1.0
            } else {
                0.0
            };
            if (self.store.hover_live(id) - v).abs() > f32::EPSILON {
                self.store.set_hover_live(id, v);
                changed = true;
            }
        }
        changed
    }

    /// O Down: um controlo (não de arrasto) captura e NÃO dispara. `None` = não é um controlo assim.
    pub(crate) fn press_control(&mut self, x: f32, y: f32) -> Option<bool> {
        self.ui.pointer = Some((x, y));
        let c = self.control_at(x, y)?;
        // O campo de texto foca no Down e o caret cai onde se carregou (o arrasto selecciona), como no SO.
        if c.role == ControlRole::TextField && c.enabled() && self.view() == View::List {
            let id = c.id;
            self.ui.focus = None;
            self.ui.pressed = None;
            crate::view_list::press_text(self, id, x);
            return Some(true);
        }
        if c.role.drags() {
            return None;
        }
        let (id, live) = (c.id, c.enabled() && c.role.clickable());
        self.ui.focus = None;
        self.ui.pressed = live.then_some(id);
        Some(true)
    }

    /// O Up: o capturado dispara só DENTRO; fora cancela. `None` = nada capturado.
    pub(crate) fn release_control(&mut self, x: f32, y: f32) -> Option<bool> {
        self.ui.pointer = Some((x, y));
        let id = self.ui.pressed.take()?;
        if self.hits.rect_for(id).is_some_and(|r| r.contains(x, y)) {
            self.activate(id);
        }
        Some(true)
    }

    /// Activa um controlo como um clique: o seletor troca a vista; os outros passam pelo `press` da vista no centro.
    pub(crate) fn activate(&mut self, id: NodeId) {
        let Some(c) = self.control_of(id) else {
            return;
        };
        if !c.enabled() || !c.role.clickable() {
            return;
        }
        if let Some(v) = View::of_id(id) {
            self.set_view(v);
            return;
        }
        if self.view() == View::List {
            crate::view_list::act(self, id);
            return;
        }
        // Os números Dead/Press da B são os da A (`field.rs`): abrem pelo id.
        if c.role == ControlRole::NumberField {
            crate::view_list::open_number(self, id);
            return;
        }
        let r = c.rect;
        self.press_legacy(r.x + r.w * 0.5, r.y + r.h * 0.5);
    }

    /// Tab / Shift+Tab / Enter / Space / Esc para o foco. `true` = tomada.
    pub(crate) fn focus_key(&mut self, k: FocusKey) -> bool {
        let order: Vec<NodeId> = self
            .ui
            .controls
            .iter()
            .filter(|c| c.focusable && c.enabled())
            .map(|c| c.id)
            .collect();
        let field = self.store.focus_id();
        let from = self.ui.focus.or(field);
        match k {
            FocusKey::Activate => match self.ui.focus.filter(|_| field.is_none()) {
                Some(id) => {
                    self.activate(id);
                    true
                }
                None => false,
            },
            FocusKey::Drop => self.ui.focus.take().is_some() && field.is_none(),
            FocusKey::Next | FocusKey::Prev if order.is_empty() => false,
            FocusKey::Next | FocusKey::Prev => {
                let n = order.len();
                let at = from.and_then(|f| order.iter().position(|o| *o == f));
                let i = match (at, k) {
                    (Some(i), FocusKey::Next) => (i + 1) % n,
                    (Some(i), _) => (i + n - 1) % n,
                    (None, FocusKey::Next) => 0,
                    (None, _) => n - 1,
                };
                self.land_focus(order[i]);
                true
            }
        }
    }

    /// O foco chega a `id`: um campo de texto foca o campo (o anel é o da caixa do campo); um número da lista
    /// abre o editor (`field.rs`); o resto ganha o anel da aba.
    pub(crate) fn land_focus(&mut self, id: NodeId) {
        let role = self.control_of(id).map(|c| c.role);
        self.ui.focus = None;
        self.store.set_focus(None);
        match role {
            Some(ControlRole::TextField) => self.store.set_focus(Some(id)),
            Some(ControlRole::NumberField) if matches!(self.view(), View::List | View::Keyboard) => {
                crate::view_list::open_number(self, id);
            }
            _ => self.ui.focus = Some(id),
        }
    }

    /// A tecla física para o foco (Tab, Shift, Enter, Space, Esc), pelos nomes do vocabulário do app
    /// (`Key::named_in_script`). `true` = tomada.
    pub(crate) fn focus_raw_key(&mut self, key: ph2d_input::Key, pressed: bool) -> bool {
        if is_named(key, &["left_shift", "right_shift"]) {
            self.ui.shift = pressed;
            return false;
        }
        if !pressed {
            return false;
        }
        let k = if is_named(key, &["tab"]) {
            if self.ui.shift {
                FocusKey::Prev
            } else {
                FocusKey::Next
            }
        } else if is_named(key, &["escape"]) {
            FocusKey::Drop
        } else if is_named(key, &["space"]) || key.is_enter_like() {
            FocusKey::Activate
        } else {
            return false;
        };
        self.focus_key(k)
    }

    /// A tecla de texto para o foco. `true` = tomada.
    pub(crate) fn focus_text_key(
        &mut self,
        input: &ph2d_editor_core::doc_surface::KeyInput<'_>,
    ) -> bool {
        use ph2d_editor_core::doc_surface::Key as K;
        if !input.pressed {
            return false;
        }
        let k = match input.key {
            K::Tab if input.mods.shift => FocusKey::Prev,
            K::Tab => FocusKey::Next,
            K::Escape => FocusKey::Drop,
            K::Enter | K::Space | K::Char(' ') => FocusKey::Activate,
            _ => return false,
        };
        self.focus_key(k)
    }

    /// Começo de uma pintura: o registo esvazia (a pintura enche-o de novo).
    pub(crate) fn begin_paint(&mut self) {
        self.ui.controls.clear();
        self.ui.pulses.clear();
        self.ui.carets.clear();
    }
}

use crate::doc::View;

/// **O anel de foco da aba** (um só): `ColorToken::FocusRing`, no traço do anel de foco dos botões.
pub fn paint_focus_ring(
    scene: &mut VectorScene,
    ink: ph2d_editor_core::widget::card_ink::CardInk,
    rect: Rect,
    radius: f32,
) {
    ph2d_editor_core::paint::stroke_rounded_rect(
        scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        ink.color(ColorToken::FocusRing),
    );
}
