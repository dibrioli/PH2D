//! O rato na aba: clicar põe o cursor, arrastar selecciona, duplo-clique selecciona a palavra,
//! Alt+clique junta um cursor, Shift+clique estende, Ctrl+clique vai para a definição (I6); a calha dobra/selecciona a linha; a barra de
//! busca e a faixa de baixo respondem aos seus alvos; o contador da barra abre a lista de problemas.

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
        if self.rename.is_some() {
            if self.view.rename_contains(x, y) {
                return true;
            }
            self.rename = None;
        }
        if let Some(i) = self.view.hint_button_at(x, y) {
            self.apply_fix(i);
            return true;
        }
        if self.view.hint_hover_contains(x, y) {
            // O balão lê-se e clica-se; o texto por baixo dele não recebe o clique.
            return true;
        }
        self.hints.hover = None;
        self.goto = None;
        let acting = self.actions.is_some();
        let listing = self.refs.is_some();
        if self.find.open
            && let Some(h) = self.find_view.hit(ts, x, y)
        {
            self.find_click(h);
            return true;
        }
        if let Some(h) = self.console_view.press(x, y) {
            match h {
                ConsoleHit::Toggle => console.open = !console.open,
                ConsoleHit::Tab(t) => {
                    console.open = true;
                    console.tab = t;
                }
                ConsoleHit::Problem(k) => self.problem_pick(k),
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
        if let Some(by) = self.view.list_bar_press(x, y) {
            self.list_scroll(by);
            return true;
        }
        match self.view.completion_hit(x, y) {
            Some(ph2d_panel_code::CompletionHit::Row(i)) if acting => {
                self.apply_action(i);
                return true;
            }
            Some(ph2d_panel_code::CompletionHit::Row(i)) if listing => {
                self.refs_go(i);
                return true;
            }
            Some(ph2d_panel_code::CompletionHit::Row(i)) => {
                self.accept_completion(i);
                return true;
            }
            Some(ph2d_panel_code::CompletionHit::Body) => return true,
            None => {
                self.complete.close();
                self.actions = None;
                self.refs = None;
            }
        }
        if self.debug_press(x, y) || self.view.bar_press(x, y) {
            return true;
        }
        self.find.focus = None;
        match self.view.hit(&self.buffer, x, y) {
            Hit::Text(b) => self.click_text(b, mods, now_ns),
            Hit::Fold(line) => self.toggle_fold(line),
            Hit::Mark(line) => self.toggle_breakpoint(line),
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
            Hit::Status if self.view.counter_contains(x, y) => console.toggle_problems(),
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
            if mods.ctrl || mods.meta {
                self.go_to_definition(b);
            }
        }
        self.buffer.seal_history();
    }

    /// O rato mexeu com o botão em baixo: a seleção vai do início do arrasto ao byte sob o rato.
    pub fn drag_to(&mut self, x: f32, y: f32) -> bool {
        let list = self.view.list_bar_drag(y).map(|by| self.list_scroll(by));
        let bars = self.view.bar_drag_to(y) || self.console_view.drag_to(y);
        if self.debug.view.drag_to(x, y) || bars || list.is_some() {
            return true;
        }
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
        let resized = self.debug.view.release() | self.view.bar_release();
        let resized = resized | self.console_view.release() | self.view.list_bar_release();
        self.drag.take().is_some() || resized
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

    /// Rolar a lista aberta (referências, acções ou completação) `rows` linhas.
    fn list_scroll(&mut self, rows: isize) {
        if self.refs.is_some() {
            self.refs_scroll(rows);
        } else if self.actions.is_some() {
            self.actions_scroll(rows);
        } else {
            let (text, head) = (self.buffer.text(), self.buffer.selection().primary().head);
            let shown = ph2d_panel_code::COMPLETION_ROWS;
            self.complete.scroll_shown(text, head, rows, shown);
        }
    }

    /// A roda: sobre o console rola-o, senão rola o texto — na horizontal com o Shift.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32, shift: bool, console: &Store) {
        if self.wheel_inner(x, y, dy, console) {
            return;
        }
        if shift {
            self.view.scroll_x_by(-dy);
        } else {
            self.view.scroll_by(-dy);
        }
    }

    /// O deslizar de lado do trackpad: o Shift+roda sobre o texto; nas zonas que só rolam na vertical, nada.
    pub fn wheel_sideways(&mut self, x: f32, y: f32, dx: f32, console: &Store) -> bool {
        let text = !self.wheel_inner(x, y, 0.0, console);
        if text {
            self.view.scroll_x_by(-dx);
        }
        text
    }

    /// As zonas que só rolam na vertical (o depurador, as listas, o console). `true` = era de uma delas.
    fn wheel_inner(&mut self, x: f32, y: f32, dy: f32, console: &Store) -> bool {
        self.hints.hover = None;
        let rows = ph2d_panel_code::wheel_rows(dy, self.view.metrics().line_h);
        let over_list = self.view.completion_hit(x, y).is_some();
        if self.debug.view.wheel(x, y, dy) {
            // O quadro do depurador rola por dentro.
        } else if over_list {
            self.list_scroll(rows);
        } else if self.console_view.area().contains(x, y) {
            self.console_view.scroll_by(dy, console.len());
        } else {
            return false;
        }
        true
    }
}
