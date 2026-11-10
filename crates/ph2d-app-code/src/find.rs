//! ⭐ **E9 — buscar/substituir e ir para linha**, sobre o núcleo puro `ph2d_code_buffer::find`. Os
//! campos são `Buffer`s de uma linha; substituir tudo é UM passo de undo (o núcleo garante-o). O
//! estado da barra vive em [`ph2d_panel_code::find_state`].

use crate::doc::CodeDoc;
use crate::edit::{self, FieldKey};
use ph2d_code_buffer::{Range, Selection, goto_line, replace_all, replace_one};
use ph2d_panel_code::FindField;
pub(crate) use ph2d_panel_code::find_state::{FindMode as Mode, FindState};

impl CodeDoc {
    fn select_match(&mut self, i: usize) {
        let m = self.find.matches[i].clone();
        self.buffer
            .set_selection(Selection::single(Range::new(m.start, m.end)));
        self.buffer.seal_history();
        self.find.refresh(&self.buffer);
        self.reveal();
    }

    /// A ocorrência seguinte (`back` = a anterior), dando a volta.
    pub fn find_step(&mut self, back: bool) {
        self.find.refresh(&self.buffer);
        let n = self.find.matches.len();
        let Some(cur) = self.find.current else { return };
        let on = self.buffer.selection().primary().span() == self.find.matches[cur];
        let i = match (back, on) {
            (false, true) => (cur + 1) % n,
            (false, false) => cur,
            (true, _) => (cur + n - 1) % n,
        };
        self.select_match(i);
    }

    /// Substitui a ocorrência seleccionada e passa à seguinte (sem uma seleccionada, só a selecciona).
    pub fn replace_current(&mut self) {
        self.find.refresh(&self.buffer);
        let Some(cur) = self.find.current else { return };
        let m = self.find.matches[cur].clone();
        if self.buffer.selection().primary().span() != m {
            self.select_match(cur);
            return;
        }
        let with = self.find.replace.text().to_string();
        let edits = replace_one(&mut self.buffer, m, &with);
        self.edited(&edits, true);
        self.find.refresh(&self.buffer);
        if self.find.current.is_some() {
            self.find_step(false);
        }
    }

    /// Substitui todas — UM passo de undo.
    pub fn replace_everything(&mut self) {
        self.find.refresh(&self.buffer);
        let with = self.find.replace.text().to_string();
        let matches = self.find.matches.clone();
        let edits = replace_all(&mut self.buffer, &matches, &with);
        self.edited(&edits, true);
        self.find.refresh(&self.buffer);
        self.reveal();
    }

    /// Enter no «Go to line»: o cursor na linha, à vista; a barra fecha.
    pub fn goto_commit(&mut self) {
        let Ok(n) = self.find.goto.text().to_string().trim().parse::<usize>() else {
            return;
        };
        goto_line(&mut self.buffer, n);
        self.find.close();
        self.reveal();
    }

    /// Uma tecla com um campo da barra focado.
    pub fn find_key(&mut self, f: FindField, k: FieldKey<'_>) -> Option<String> {
        let out = edit::field_key(self.find.field_mut(f), k);
        if out.escape {
            self.find.close();
        } else if out.tab && self.find.mode == Mode::Replace {
            self.find.focus = Some(if f == FindField::Query {
                FindField::Replace
            } else {
                FindField::Query
            });
        } else if out.enter {
            match f {
                FindField::GoTo => self.goto_commit(),
                FindField::Query => self.find_step(k.mods.shift),
                FindField::Replace if k.mods.ctrl || k.mods.meta => self.replace_everything(),
                FindField::Replace => self.replace_current(),
            }
        } else if out.changed && f == FindField::Query {
            // Escrever na busca salta para a primeira ocorrência a partir do cursor.
            self.find.refresh(&self.buffer);
            if let Some(i) = self.find.current {
                let m = self.find.matches[i].clone();
                self.view.ensure_visible(&self.buffer, m.start);
            }
        }
        out.copy
    }
}
