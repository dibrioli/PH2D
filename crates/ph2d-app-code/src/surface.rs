//! `impl DocumentSurface for CodeDocs` — as portas do hero (abas, pintura, rato, teclado, fecho). Uma
//! aba de ajuda ([`crate::help`], o bit [`crate::help::HELP_TAG`] no id) vai para as portas dela; o
//! Ctrl+W e o Alt+← / Alt+→ são da FAMÍLIA (valem em qualquer aba dela).

use crate::docs::{A11Y_SALT, CodeDocs, blink_on};
use crate::help::page_of;
use ph2d_a11y::{Node, NodeId};
use ph2d_editor_core::doc_surface::{
    Compose, DocCommand, DocTab, DocumentSurface, Key, KeyInput, KeyOutcome, PointerInput,
    PointerLook,
};
use ph2d_editor_core::ime::ComposeKey;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::any::Any;

impl DocumentSurface for CodeDocs {
    fn tabs(&self) -> Vec<DocTab> {
        self.tab_ids()
            .into_iter()
            .filter_map(|id| match page_of(id) {
                Some(p) => Some(DocTab {
                    id,
                    title: ph2d_code_help::catalog()[p].title.clone(),
                    dirty: false,
                }),
                None => self.docs.iter().find(|d| d.script.0 == id).map(|d| DocTab {
                    id,
                    title: d.title.clone(),
                    dirty: d.buffer.is_dirty(),
                }),
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
        self.pump();
        self.area = area;
        let (list_r, doc_r) = self.split_area(area);
        let dirty: Vec<ScriptId> = self
            .buffers()
            .filter(|d| d.2.is_dirty())
            .map(|d| d.0)
            .collect();
        self.list
            .paint(list_r, id, &|s| dirty.contains(&s), scene, text, theme);
        if let Some(page) = page_of(id) {
            self.paint_help(page, doc_r, scene, text, theme);
            return;
        }
        let (now, steady) = self.clock;
        let typing = self.list.focus.is_none();
        let assist = self.intel.assist();
        if let Some((doc, store)) = self.split(id) {
            let lit = typing && blink_on(now, doc.last_input_ns, steady);
            doc.paint(doc_r, scene, text, theme, store, (lit, assist));
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
            self.pressed = true;
            return true;
        }
        if matches!(kind, PointerKind::Down) {
            self.list.focus = None;
            let _ = self.touched(id);
        }
        if let Some(page) = page_of(id) {
            return self.help_pointer(page, input, kind, button, (x, y));
        }
        // O largar só é da aba se o carregar foi (a lei do quadro): o de um clique no chrome não.
        let was_pressed = matches!(kind, PointerKind::Up) && std::mem::take(&mut self.pressed);
        let in_area = self.area.contains(x, y);
        let Some((doc, store)) = self.split(id) else {
            return false;
        };
        let taken = match (kind, button) {
            (PointerKind::Down, PointerButton::Primary) => {
                doc.press(area, input.text, input.mods, input.now_ns, x, y, store)
            }
            (PointerKind::Up, _) => doc.release() || was_pressed,
            _ => in_area,
        };
        if matches!(kind, PointerKind::Down) {
            self.pressed = taken;
        }
        taken
    }

    fn pointer_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool {
        if let Some(page) = page_of(id) {
            return self.help_pointer_move(page, input, x, y);
        }
        let (now, inside) = (self.clock.0, self.area.contains(x, y));
        let Some(d) = self.doc(id) else { return false };
        d.reach_mouse(inside, x, y);
        if d.drag_to(x, y) {
            return true;
        }
        d.hover_at(x, y, now);
        false
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        if !self.area.contains(x, y) {
            return false;
        }
        if self.list.view.wheel(x, y, dy) {
            return true;
        }
        if let Some(page) = page_of(id) {
            self.help_wheel(page, dy);
            return true;
        }
        if let Some((doc, store)) = self.split(id) {
            doc.wheel(x, y, dy, shift, store);
        }
        true
    }

    fn key(&mut self, id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        let KeyInput {
            key,
            mods,
            pressed,
            text,
            paste,
        } = input;
        if pressed && let Some(out) = self.family_key(id, key, mods) {
            return out;
        }
        let in_field = self.list.focus.is_some()
            || self
                .docs
                .iter()
                .find(|d| d.script.0 == id)
                .is_some_and(|d| d.find.open && d.find.focus.is_some());
        if let Some(out) = self.problems_key(id, key, mods, pressed) {
            return out;
        }
        if crate::keys::foreign_chord(key, mods)
            || (crate::keys::suggest_chord(key, mods) && in_field)
        {
            return KeyOutcome::default();
        }
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
        if let Some(page) = page_of(id) {
            if !pressed {
                return KeyOutcome {
                    consumed: true,
                    copy: None,
                };
            }
            return self.help_key(page, key, mods, text, paste.as_deref());
        }
        let out = match self.touched(id) {
            Some(d) => d.key(key, mods, pressed, text, paste.as_deref()),
            None => KeyOutcome::default(),
        };
        self.submit();
        out
    }

    fn compose(&mut self, id: u64, c: Compose<'_>) -> bool {
        if self.list.focus.is_some() {
            return false;
        }
        if let Some(page) = page_of(id) {
            return self.help_compose(page, c);
        }
        let Some(d) = self.touched(id) else {
            return false;
        };
        if d.find.open && d.find.focus.is_some() {
            return false;
        }
        match c {
            Compose::Preedit(s, _) => d.preedit = (!s.is_empty()).then(|| (s.to_owned(), false)),
            Compose::Commit(ch) => {
                d.preedit = None;
                d.type_text(ch.encode_utf8(&mut [0; 4]));
                d.reveal();
                self.submit();
            }
            Compose::DeadKey(ComposeKey::Dead(ch)) => d.preedit = Some((ch.to_string(), true)),
            // Só toma o que é dela: sem composição em curso, o Esc é do editor e o Shift da tecla.
            Compose::DeadKey(ComposeKey::Escape) => return d.preedit.take().is_some(),
            Compose::DeadKey(ComposeKey::Modifier) => {
                return d.preedit.as_ref().is_some_and(|p| p.1);
            }
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
        if let Some(page) = page_of(id) {
            self.close_help(page);
            return;
        }
        let _ = self.nav.remove_tab(id);
        let Some(i) = self.docs.iter().position(|d| d.script.0 == id) else {
            return;
        };
        let doc = self.docs.remove(i);
        if doc.buffer.is_dirty() {
            self.parked.insert(doc.script, doc.park());
        }
    }

    fn undo(&mut self, id: u64, redo: bool) -> bool {
        let did = self.touched(id).is_some_and(|d| {
            let did = d.undo(redo);
            d.reveal();
            did
        });
        self.submit();
        did
    }

    fn a11y(&self, id: u64) -> Vec<(NodeId, Node)> {
        if let Some(page) = page_of(id) {
            return self.help_a11y(page);
        }
        self.docs
            .iter()
            .find(|d| d.script.0 == id)
            .map(|d| d.a11y(NodeId(A11Y_SALT ^ (id << 32))))
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
        self.tab_commands(id)
    }

    fn places(&mut self, id: u64) -> Option<ph2d_editor_core::doc_surface::DocPlaces> {
        self.tab_places(id)
    }

    fn go_to(&mut self, id: u64, line: usize, col: usize) -> bool {
        self.tab_go_to(id, line, col)
    }

    fn pointer_look(&self, id: u64, x: f32, y: f32) -> Option<PointerLook> {
        if let Some(page) = page_of(id) {
            return self.help.open.get(&page)?.view.pointer_look(x, y);
        }
        self.tab_pointer_look(id, x, y)
    }

    fn command(&mut self, id: u64, cmd: ph2d_a11y::NodeId) -> bool {
        self.list.focus = None;
        if self.family_command(cmd) {
            return true;
        }
        let did = self.touched(id).is_some_and(|d| {
            let did = d.run(cmd);
            if did {
                d.reveal();
            }
            did
        });
        self.submit();
        did
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl CodeDocs {
    /// As teclas da FAMÍLIA, em qualquer aba dela: Ctrl+W fecha a aba activa; Alt+← / Alt+→ andam
    /// pelo histórico ([`crate::nav`]). `None` = não é uma delas.
    fn family_key(&mut self, id: u64, key: Key, mods: ph2d_host::Modifiers) -> Option<KeyOutcome> {
        let cmd = mods.ctrl || mods.meta;
        let taken = KeyOutcome {
            consumed: true,
            copy: None,
        };
        match key {
            Key::Char('w' | 'W') if cmd && !mods.shift && !mods.alt => {
                self.close_active(id);
                Some(taken)
            }
            Key::Left | Key::Right if mods.alt && !cmd && !mods.shift => {
                let _ = self.nav_step(id, key == Key::Left);
                Some(taken)
            }
            _ => None,
        }
    }
}
