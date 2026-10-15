//! O rato na aba: clicar põe o cursor, arrastar selecciona, duplo-clique selecciona a palavra,
//! Alt+clique junta um cursor, Shift+clique estende; a calha dobra/selecciona a linha; a barra de
//! busca e o console respondem aos seus alvos.

use crate::console::Store;
use crate::doc::CodeDoc;
use ph2d_code_buffer::{Range, Selection, word_at};
use ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_panel_code::{ConsoleHit, FindHit, Hit};
use ph2d_text::TextSystem;

impl CodeDoc {
    /// Carregar com o botão principal em `(x, y)` da aba (`area` = a da aba). `true` = tomado.
    #[allow(clippy::too_many_arguments)]
    pub fn press(
        &mut self,
        area: Rect,
        ts: &mut TextSystem,
        mods: Modifiers,
        now_ns: u128,
        x: f32,
        y: f32,
        console: &mut Store,
    ) -> bool {
        if !area.contains(x, y) {
            return false;
        }
        if self.find.open
            && let Some(h) = self.find_view.hit(ts, x, y)
        {
            self.find_click(h);
            return true;
        }
        if let Some(h) = self.console_view.hit(x, y) {
            match h {
                ConsoleHit::Toggle => console.open = !console.open,
                ConsoleHit::Clear => console.clear(),
                ConsoleHit::Row(i) => {
                    if let Some(line) = console.line_of(i) {
                        ph2d_code_buffer::goto_line(&mut self.buffer, line as usize);
                        self.find.focus = None;
                        self.reveal();
                    }
                }
                ConsoleHit::Body => {}
            }
            return true;
        }
        self.find.focus = None;
        match self.view.hit(&self.buffer, x, y) {
            Hit::Text(b) => self.click_text(b, mods, now_ns),
            Hit::Fold(line) => self.toggle_fold(line),
            Hit::Gutter(line) => {
                let t = self.buffer.text();
                let s = t.byte_of_line(line);
                let e = if line + 1 < t.line_count() {
                    t.byte_of_line(line + 1)
                } else {
                    t.len()
                };
                self.buffer
                    .set_selection(Selection::single(Range::new(s, e)));
                self.buffer.seal_history();
            }
            Hit::Status | Hit::Outside => {}
        }
        true
    }

    fn click_text(&mut self, b: usize, mods: Modifiers, now_ns: u128) {
        let double = self
            .last_click
            .is_some_and(|(t, at)| at == b && now_ns.saturating_sub(t) <= DOUBLE_CLICK_WINDOW_NS);
        self.last_click = Some((now_ns, b));
        if mods.alt {
            self.buffer.toggle_cursor(b);
        } else if double {
            let w = word_at(self.buffer.text(), b).unwrap_or(b..b);
            self.buffer
                .set_selection(Selection::single(Range::new(w.start, w.end)));
            self.last_click = None;
        } else if mods.shift {
            let p = self.buffer.selection().primary();
            self.buffer
                .set_selection(Selection::single(Range::new(p.anchor, b)));
            self.drag = Some(p.anchor);
        } else {
            self.buffer
                .set_selection(Selection::single(Range::cursor(b)));
            self.drag = Some(b);
        }
        self.buffer.seal_history();
    }

    /// O rato mexeu com o botão em baixo: a seleção vai do início do arrasto ao byte sob o rato.
    pub fn drag_to(&mut self, x: f32, y: f32) -> bool {
        let Some(anchor) = self.drag else {
            return false;
        };
        let tr = self.view.text_rect();
        // Fora da zona do texto o arrasto continua, preso à borda.
        let cx = x.clamp(tr.x, (tr.x + tr.w - 1.0).max(tr.x));
        let cy = y.clamp(tr.y, (tr.y + tr.h - 1.0).max(tr.y));
        if let Hit::Text(b) = self.view.hit(&self.buffer, cx, cy) {
            self.buffer
                .set_selection(Selection::single(Range::new(anchor, b)));
            self.view.ensure_visible(&self.buffer, b);
        }
        true
    }

    pub fn release(&mut self) -> bool {
        self.drag.take().is_some()
    }

    fn find_click(&mut self, h: FindHit) {
        match h {
            FindHit::Field(f, byte) => {
                self.find.focus = Some(f);
                let buf = self.find.field_mut(f);
                buf.set_selection(Selection::single(Range::cursor(byte)));
            }
            FindHit::Prev => self.find_step(true),
            FindHit::Next => self.find_step(false),
            FindHit::Case => self.find.case = !self.find.case,
            FindHit::Word => self.find.word = !self.find.word,
            FindHit::ReplaceOne => self.replace_current(),
            FindHit::ReplaceAll => self.replace_everything(),
            FindHit::Close => self.find.close(),
            FindHit::Bar => {}
        }
    }

    /// A roda: sobre o console rola-o, senão rola o texto.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32, console: &Store) {
        if self.console_view.area().contains(x, y) {
            self.console_view.scroll_by(dy, console.len());
        } else {
            self.view.scroll_by(-dy);
        }
    }
}
