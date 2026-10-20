//! O rato e o teclado numa aba de ajuda (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.6): ↑↓ PgUp PgDn
//! Home End rolam; Tab/Shift+Tab andam pelos links e o Enter segue; Ctrl+F abre a busca (Enter/F3 a
//! seguinte, Shift+Enter a anterior, Esc fecha); arrastar seleciona e o Ctrl+C copia; o clique num
//! link segue-o (na mesma página só rola — não entra no histórico); o «Copy» copia o exemplo.

use crate::docs::CodeDocs;
use crate::edit::{FieldKey, field_key};
use crate::help::{HelpTab, tab_id};
use crate::help_doc::Target;
use crate::nav::Place;
use ph2d_editor_core::doc_surface::{Key, KeyOutcome, PointerInput};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_panel_code::{FindHit, HelpHit};

impl HelpTab {
    /// A ocorrência seguinte (`back` = a anterior) da busca, à vista.
    fn find_step(&mut self, back: bool) {
        let n = self.find.matches.len();
        let Some(cur) = self.find.current.filter(|_| n > 0) else {
            return;
        };
        let i = if back {
            (cur + n - 1) % n
        } else {
            (cur + 1) % n
        };
        self.find.current = Some(i);
        self.view.reveal(self.find.matches[i].start);
    }

    /// A busca mudou: a 1.ª ocorrência à vista.
    fn find_follow(&mut self) {
        self.find.refresh(self.doc.key, self.view.plain());
        if let Some(i) = self.find.current {
            self.view.reveal(self.find.matches[i].start);
        }
    }

    fn open_find(&mut self) {
        self.find.open = true;
        self.find.focus = true;
        self.find.query.select_all();
    }

    /// Tab / Shift+Tab: o link seguinte (anterior) pela ordem da página, à vista.
    fn focus_step(&mut self, back: bool) {
        let links = self.view.links_in_order();
        if links.is_empty() {
            return;
        }
        let at = self.focus.and_then(|f| links.iter().position(|l| *l == f));
        let n = links.len();
        let i = match (at, back) {
            (None, false) => 0,
            (None, true) => n - 1,
            (Some(i), false) => (i + 1) % n,
            (Some(i), true) => (i + n - 1) % n,
        };
        self.focus = Some(links[i]);
        if let Some(b) = self.view.link_byte(links[i]) {
            self.view.reveal(b);
        }
    }

    /// Uma tecla com o campo da busca focado. Devolve o texto copiado.
    fn find_key(&mut self, k: FieldKey<'_>) -> Option<String> {
        let cmd = k.mods.ctrl || k.mods.meta;
        match k.key {
            Key::Char('f') if cmd => {
                self.find.query.select_all();
                return None;
            }
            Key::F(3) => {
                self.find_step(k.mods.shift);
                return None;
            }
            _ => {}
        }
        let out = field_key(&mut self.find.query, k);
        if out.escape {
            self.find.close();
        } else if out.enter {
            self.find_step(k.mods.shift);
        } else if out.changed {
            self.find_follow();
        }
        out.copy
    }

    /// Uma tecla na página (fora do campo da busca). `Some(n)` = seguir o link `n`.
    fn page_key(&mut self, key: Key, mods: Modifiers, out: &mut KeyOutcome) -> Option<u32> {
        let cmd = mods.ctrl || mods.meta;
        let line = self.view.metrics().line_h;
        match key {
            Key::Char('f') if cmd => self.open_find(),
            Key::Char('c') if cmd => {
                out.copy = self
                    .selection
                    .clone()
                    .filter(|r| !r.is_empty())
                    .map(|r| self.view.text(r));
            }
            Key::Char('a') if cmd => self.selection = Some(0..self.view.plain().len()),
            Key::F(3) => self.find_step(mods.shift),
            Key::Up => self.view.scroll_by(-line),
            Key::Down => self.view.scroll_by(line),
            Key::PageUp => self.view.page(false),
            Key::PageDown => self.view.page(true),
            Key::Home => self.view.home(),
            Key::End => self.view.end(),
            Key::Tab => self.focus_step(mods.shift),
            Key::Enter => return self.focus,
            Key::Escape if self.find.open => self.find.close(),
            Key::Escape => {
                self.selection = None;
                self.focus = None;
            }
            _ => {}
        }
        None
    }
}

impl CodeDocs {
    /// Segue o número `n` da página `page`: um link para a MESMA página só rola (sem histórico); para
    /// outra, abre-a e empilha; o botão de um exemplo copia-o.
    pub(crate) fn help_follow(&mut self, page: usize, n: u32) {
        let Some(t) = self.help.open.get_mut(&page) else {
            return;
        };
        match t.targets.get(n as usize).copied() {
            Some(Target::Anchor(a)) if crate::help::page_index(a.page) == Some(page) => t.show(a),
            Some(Target::Anchor(a)) => {
                let from = Place::Help {
                    page,
                    scroll: t.view.scroll(),
                };
                self.open_help(a, Some(from));
            }
            Some(Target::Example(_)) => self.copy_out = t.view.example_text(n),
            None => {}
        }
    }

    /// Uma tecla premida na aba de ajuda `page`.
    pub(crate) fn help_key(
        &mut self,
        page: usize,
        key: Key,
        mods: Modifiers,
        text: Option<&str>,
        paste: Option<&str>,
    ) -> KeyOutcome {
        let mut out = KeyOutcome {
            consumed: true,
            copy: None,
        };
        let Some(t) = self.help.open.get_mut(&page) else {
            return KeyOutcome::default();
        };
        if t.find.open && t.find.focus {
            out.copy = t.find_key(FieldKey {
                key,
                mods,
                text,
                paste,
            });
            return out;
        }
        if let Some(n) = t.page_key(key, mods, &mut out) {
            self.help_follow(page, n);
        }
        out
    }

    /// Carregar/largar na aba de ajuda `page`. `true` = tomado.
    pub(crate) fn help_pointer(
        &mut self,
        page: usize,
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        (x, y): (f32, f32),
    ) -> bool {
        let in_area = self.area.contains(x, y);
        let Some(t) = self.help.open.get_mut(&page) else {
            return false;
        };
        match (kind, button) {
            (PointerKind::Down, PointerButton::Primary) if in_area => {
                self.pressed = true;
                let hit = t.view.hit(input.text, x, y);
                if !matches!(hit, Some(HelpHit::Find(_))) {
                    t.find.focus = false;
                }
                match hit {
                    Some(HelpHit::Back) => {
                        let _ = self.nav_step(tab_id(page), true);
                    }
                    Some(HelpHit::Forward) => {
                        let _ = self.nav_step(tab_id(page), false);
                    }
                    Some(HelpHit::Section(i)) => t.view.scroll_to_section(i),
                    Some(HelpHit::Link(n) | HelpHit::Copy(n)) => {
                        t.focus = None;
                        self.help_follow(page, n);
                    }
                    Some(HelpHit::Text(b)) => {
                        t.selection = None;
                        t.focus = None;
                        t.drag = Some(b);
                    }
                    Some(HelpHit::Find(h)) => find_click(t, h),
                    None => {}
                }
                true
            }
            (PointerKind::Up, _) => {
                t.drag = None;
                std::mem::take(&mut self.pressed)
            }
            _ => in_area,
        }
    }

    /// O rato sem botão (ou a arrastar) sobre a aba de ajuda `page`.
    pub(crate) fn help_pointer_move(
        &mut self,
        page: usize,
        input: PointerInput<'_>,
        x: f32,
        y: f32,
    ) -> bool {
        let Some(t) = self.help.open.get_mut(&page) else {
            return false;
        };
        let hit = t.view.hit(input.text, x, y);
        if let Some(from) = t.drag {
            if let Some(HelpHit::Text(b)) = hit {
                t.selection = (b != from).then(|| from.min(b)..from.max(b));
            }
            return true;
        }
        t.hover = hit.filter(|h| !matches!(h, HelpHit::Text(_)));
        false
    }

    /// O carácter composto (IME, tecla morta) entra no campo da busca, se ele tem o teclado.
    pub(crate) fn help_compose(
        &mut self,
        page: usize,
        c: ph2d_editor_core::doc_surface::Compose<'_>,
    ) -> bool {
        let Some(t) = self
            .help
            .open
            .get_mut(&page)
            .filter(|t| t.find.open && t.find.focus)
        else {
            return false;
        };
        match c {
            ph2d_editor_core::doc_surface::Compose::Commit(ch) => {
                t.find.query.insert(ch.encode_utf8(&mut [0; 4]));
                t.find_follow();
                true
            }
            _ => false,
        }
    }

    /// A roda sobre a aba de ajuda `page`.
    pub(crate) fn help_wheel(&mut self, page: usize, dy: f32) {
        if let Some(t) = self.help.open.get_mut(&page) {
            t.view.scroll_by(-dy);
        }
    }
}

/// Um clique na barra da busca da página.
fn find_click(t: &mut HelpTab, h: FindHit) {
    match h {
        FindHit::Field(_, b) => {
            t.find.focus = true;
            t.find
                .query
                .set_selection(ph2d_code_buffer::Selection::single(
                    ph2d_code_buffer::Range::cursor(b),
                ));
        }
        FindHit::Next => t.find_step(false),
        FindHit::Prev => t.find_step(true),
        FindHit::Close => t.find.close(),
        FindHit::Case => {
            t.find.case = !t.find.case;
            t.find_follow();
        }
        FindHit::Word => {
            t.find.word = !t.find.word;
            t.find_follow();
        }
        FindHit::ReplaceOne | FindHit::ReplaceAll | FindHit::Bar => t.find.focus = true,
    }
}
