//! ⭐ **`CodeDocs` — a família instalada no hero**: os documentos abertos (um por script, o id da aba
//! é o `ScriptId`), os que se fecharam com alterações por salvar, e o console de cada script.
//!
//! # A aba fechada com alterações por salvar
//!
//! ⛔ **Nunca se descarta texto em silêncio.** Fechar uma aba com ● guarda o documento INTEIRO (o
//! texto, o cursor, o histórico de desfazer) em [`CodeDocs::parked`]; o «Edit script» seguinte desse
//! script reabre-o tal como estava, ainda com ●. Só um projeto novo (o script deixou de existir ou
//! mudou de nome) o larga — o mesmo gesto que já deita fora o trabalho não gravado da cena.

use crate::console::Store;
use crate::doc::CodeDoc;
use ph2d_a11y::{Node, NodeId};
use ph2d_editor_core::doc_surface::{
    Compose, DocCommand, DocTab, DocumentSurface, Key, KeyOutcome, PointerInput,
};
use ph2d_editor_core::ime::ComposeKey;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_panel_code::{Badge, FindHit, Problem, ScriptListView, count_text};
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::any::Any;
use std::collections::BTreeMap;

/// O que um Ctrl+S pediu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveKind {
    Save,
    SaveAs,
}

/// O id do nó `Document` da aba `id` (os `TextRun` vêm a seguir, um por linha).
const A11Y_SALT: u64 = 0xC0DE_0000_0000_0000;

pub struct CodeDocs {
    pub(crate) docs: Vec<CodeDoc>,
    pub(crate) parked: BTreeMap<ScriptId, CodeDoc>,
    pub(crate) consoles: BTreeMap<ScriptId, Store>,
    pub(crate) focus: Option<u64>,
    /// A área da aba no último quadro (o rato pergunta por ela).
    pub(crate) area: Rect,
    /// E11: a lista de scripts do projeto, à esquerda.
    pub(crate) list: crate::list::ListState,
    /// O relógio do chrome (ns) e o «movimento reduzido» — o cursor pisca por eles.
    pub(crate) clock: (u128, bool),
}

/// O meio-período do piscar do cursor: o token `Slow` (o do piscar do F4, 1,25 Hz — abaixo dos 3 Hz
/// da WCAG 2.3.1). Aceso um período, apagado outro; uma tecla recomeça-o aceso.
pub(crate) fn blink_on(now_ns: u128, last_input_ns: u128, steady: bool) -> bool {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let half = (f64::from(ph2d_tokens::Duration::Slow.ms()) * 1e6) as u128;
    steady || (now_ns.saturating_sub(last_input_ns) / half.max(1)).is_multiple_of(2)
}

impl Default for CodeDocs {
    fn default() -> Self {
        Self {
            docs: Vec::new(),
            parked: BTreeMap::new(),
            consoles: BTreeMap::new(),
            focus: None,
            area: Rect::new(0.0, 0.0, 0.0, 0.0),
            list: crate::list::ListState::default(),
            clock: (0, false),
        }
    }
}

impl CodeDocs {
    pub(crate) fn doc(&mut self, id: u64) -> Option<&mut CodeDoc> {
        self.docs.iter_mut().find(|d| d.script.0 == id)
    }

    /// A aba `id` com uma tecla, um clique ou um comando agora: o cursor fica aceso.
    fn touched(&mut self, id: u64) -> Option<&mut CodeDoc> {
        let now = self.clock.0;
        let d = self.doc(id)?;
        d.last_input_ns = now;
        Some(d)
    }

    /// O relógio do chrome (`hero.store.clock_ns()`) e o «movimento reduzido» (cursor fixo).
    pub fn set_clock(&mut self, now_ns: u128, steady: bool) {
        self.clock = (now_ns, steady);
    }

    /// O cursor da aba `id` está aceso agora?
    #[must_use]
    pub fn caret_lit(&self, id: ScriptId) -> bool {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .is_some_and(|d| blink_on(self.clock.0, d.last_input_ns, self.clock.1))
    }

    /// As áreas da lista e do documento dentro da aba.
    fn split_area(&self, area: Rect) -> (Rect, Rect) {
        let w = ScriptListView::width(self.list.open).min(area.w);
        (
            Rect::new(area.x, area.y, w, area.h),
            Rect::new(area.x + w, area.y, (area.w - w).max(0.0), area.h),
        )
    }

    /// Abre (ou activa) o script `id`: um documento aberto ganha o foco; um estacionado volta como
    /// estava; senão nasce do texto da biblioteca.
    pub fn open(&mut self, id: ScriptId, name: &str, text: &str) {
        if !self.docs.iter().any(|d| d.script == id) {
            let doc = self
                .parked
                .remove(&id)
                .unwrap_or_else(|| CodeDoc::new(id, name, text));
            self.docs.push(doc);
        }
        self.focus = Some(id.0);
    }

    /// Os scripts com aba aberta.
    #[must_use]
    pub fn open_scripts(&self) -> Vec<ScriptId> {
        self.docs.iter().map(|d| d.script).collect()
    }

    /// O texto da aba do script `id` (aberta).
    #[must_use]
    pub fn text_of(&self, id: ScriptId) -> Option<String> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.buffer.text().to_string())
    }

    /// As linhas do console do script `id`, como a aba as mostra.
    #[must_use]
    pub fn console_texts(&self, id: ScriptId) -> Vec<String> {
        self.consoles.get(&id).map(Store::texts).unwrap_or_default()
    }

    #[must_use]
    pub fn console_open(&self, id: ScriptId) -> bool {
        self.consoles.get(&id).is_some_and(|s| s.open)
    }

    /// O cursor primário do script `id` (aberto): `(linha, byte)`, 0-based.
    #[must_use]
    pub fn cursor_of(&self, id: ScriptId) -> Option<(usize, usize)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let h = d.buffer.selection().primary().head;
        Some((d.buffer.text().line_of_byte(h), h))
    }

    /// As linhas que a vista do script `id` mostrou no último quadro.
    #[must_use]
    pub fn visible_lines(&self, id: ScriptId) -> Option<std::ops::Range<usize>> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.view.visible_lines())
    }

    /// A rolagem `(x, y)` da vista do script `id`, em píxeis.
    #[must_use]
    pub fn scroll_of(&self, id: ScriptId) -> Option<(f32, f32)> {
        self.docs.iter().find(|d| d.script == id).map(|d| d.view.scroll())
    }

    /// O documento estacionado de `id` (fechado com ●), se o há.
    #[must_use]
    pub fn is_parked(&self, id: ScriptId) -> bool {
        self.parked.contains_key(&id)
    }

    /// O selo da barra de estado do script `id` (aberto).
    #[must_use]
    pub fn badge_of(&self, id: ScriptId) -> Option<Badge> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.badge.clone())
    }

    /// Os problemas que a aba do script `id` mostra (E7).
    #[must_use]
    pub fn problems_of(&mut self, id: ScriptId) -> Vec<Problem> {
        self.doc(id.0).map(CodeDoc::problems).unwrap_or_default()
    }

    /// Há alterações por salvar na aba (aberta ou estacionada) do script `id`?
    #[must_use]
    pub fn is_dirty(&self, id: ScriptId) -> bool {
        self.docs
            .iter()
            .chain(self.parked.values())
            .any(|d| d.script == id && d.buffer.is_dirty())
    }

    /// Onde a barra de busca pintou `hit` no último quadro (os testes clicam aí).
    #[must_use]
    pub fn find_target(&self, id: ScriptId, hit: FindHit) -> Option<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.find_view.target_rect(hit))
    }

    /// O texto da contagem da busca que a barra mostra («3 of 12» / «No results»).
    #[must_use]
    pub fn find_count_text(&mut self, id: ScriptId) -> Option<String> {
        let d = self.doc(id.0)?;
        d.find.refresh(&d.buffer);
        count_text(d.find.count())
    }

    /// Onde o console pintou a linha `i` no último quadro.
    #[must_use]
    pub fn console_row_rect(&self, id: ScriptId, i: usize) -> Option<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.console_view.row_rect(i))
    }

    fn split(&mut self, id: u64) -> Option<(&mut CodeDoc, &mut Store)> {
        let doc = self.docs.iter_mut().find(|d| d.script.0 == id)?;
        let store = self.consoles.entry(doc.script).or_default();
        Some((doc, store))
    }
}

impl DocumentSurface for CodeDocs {
    fn tabs(&self) -> Vec<DocTab> {
        self.docs
            .iter()
            .map(|d| DocTab {
                id: d.script.0,
                title: d.title.clone(),
                dirty: d.buffer.is_dirty(),
            })
            .collect()
    }

    fn paint(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: &Theme,
    ) {
        self.area = area;
        let (list_r, doc_r) = self.split_area(area);
        let dirty: Vec<ScriptId> = self
            .docs
            .iter()
            .chain(self.parked.values())
            .filter(|d| d.buffer.is_dirty())
            .map(|d| d.script)
            .collect();
        self.list
            .paint(list_r, id, &|s| dirty.contains(&s), scene, text, theme);
        let (now, steady) = self.clock;
        let typing = self.list.focus.is_none();
        if let Some((doc, store)) = self.split(id) {
            let lit = typing && blink_on(now, doc.last_input_ns, steady);
            doc.paint(doc_r, scene, text, theme, store, lit);
        }
    }

    fn pointer(
        &mut self,
        id: u64,
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        let (_, area) = self.split_area(self.area);
        if matches!((kind, button), (PointerKind::Down, PointerButton::Primary))
            && let Some(h) = self.list.view.hit(input.text, x, y)
        {
            self.list.press(h, input.now_ns);
            return true;
        }
        if matches!(kind, PointerKind::Down) {
            self.list.focus = None;
            let _ = self.touched(id);
        }
        let Some((doc, store)) = self.split(id) else {
            return false;
        };
        match (kind, button) {
            (PointerKind::Down, PointerButton::Primary) => {
                doc.press(area, input.text, input.mods, input.now_ns, x, y, store)
            }
            (PointerKind::Up, _) => doc.release() || self.area.contains(x, y),
            _ => self.area.contains(x, y),
        }
    }

    fn pointer_move(&mut self, id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc(id).is_some_and(|d| d.drag_to(x, y))
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        if !self.area.contains(x, y) {
            return false;
        }
        if self.list.view.wheel(x, y, dy) {
            return true;
        }
        if let Some((doc, store)) = self.split(id) {
            doc.wheel(x, y, dy, shift, store);
        }
        true
    }

    fn key(
        &mut self,
        id: u64,
        _ts: &mut TextSystem,
        key: Key,
        mods: Modifiers,
        pressed: bool,
        text: Option<&str>,
        paste: Option<String>,
    ) -> KeyOutcome {
        if self.list.focus.is_some() {
            if !pressed {
                return KeyOutcome {
                    consumed: true,
                    copy: None,
                };
            }
            let k = crate::edit::FieldKey {
                key,
                mods,
                text,
                paste: paste.as_deref(),
            };
            let out = self.list.key(k);
            if out.consumed {
                return out;
            }
        }
        match self.touched(id) {
            Some(d) => d.key(key, mods, pressed, text, paste.as_deref()),
            None => KeyOutcome::default(),
        }
    }

    fn compose(&mut self, id: u64, c: Compose<'_>) -> bool {
        if self.list.focus.is_some() {
            return false;
        }
        let Some(d) = self.touched(id) else { return false };
        if d.find.open && d.find.focus.is_some() {
            return false;
        }
        match c {
            Compose::Preedit(s, _) => d.preedit = (!s.is_empty()).then(|| (s.to_owned(), false)),
            Compose::Commit(ch) => {
                d.preedit = None;
                d.type_text(ch.encode_utf8(&mut [0; 4]));
                d.reveal();
            }
            Compose::DeadKey(ComposeKey::Dead(ch)) => d.preedit = Some((ch.to_string(), true)),
            Compose::DeadKey(ComposeKey::Escape) => d.preedit = None,
            Compose::DeadKey(ComposeKey::Modifier) => {}
            // Outra tecla: a tecla morta desfaz-se e a tecla segue para o `key` (o texto composto
            // vem nela).
            Compose::DeadKey(ComposeKey::Other) => {
                if d.preedit.as_ref().is_some_and(|p| p.1) {
                    d.preedit = None;
                }
                return false;
            }
        }
        true
    }

    fn close(&mut self, id: u64) {
        let Some(i) = self.docs.iter().position(|d| d.script.0 == id) else {
            return;
        };
        let doc = self.docs.remove(i);
        if doc.buffer.is_dirty() {
            self.parked.insert(doc.script, doc);
        }
    }

    fn undo(&mut self, id: u64, redo: bool) -> bool {
        self.touched(id).is_some_and(|d| {
            let did = d.undo(redo);
            d.reveal();
            did
        })
    }

    fn a11y(&self, id: u64) -> Vec<(NodeId, Node)> {
        self.docs
            .iter()
            .find(|d| d.script.0 == id)
            .map(|d| d.view.a11y_nodes(&d.buffer, NodeId(A11Y_SALT ^ (id << 32))))
            .unwrap_or_default()
    }

    fn caret_area(&self, id: u64) -> Option<Rect> {
        let d = self.docs.iter().find(|d| d.script.0 == id)?;
        d.view
            .caret_rect(&d.buffer, d.buffer.selection().primary().head)
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    fn commands(&self, id: u64) -> Vec<DocCommand> {
        if self.docs.iter().any(|d| d.script.0 == id) {
            crate::commands::commands()
        } else {
            Vec::new()
        }
    }

    fn command(&mut self, id: u64, cmd: ph2d_a11y::NodeId) -> bool {
        self.list.focus = None;
        self.touched(id).is_some_and(|d| {
            let did = d.run(cmd);
            if did {
                d.reveal();
            }
            did
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
