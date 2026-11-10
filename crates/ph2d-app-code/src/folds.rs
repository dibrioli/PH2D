//! Os gestos de dobrar da aba (Ctrl+Shift+[ ]/Alt+F) sobre as dobras fechadas
//! ([`ph2d_code_assist::folds::ClosedFolds`], que seguem o texto pelas edições).

pub(crate) use ph2d_code_assist::folds::ClosedFolds;

impl crate::doc::CodeDoc {
    /// Ctrl+Shift+[ dobra a dobra mais funda (ainda aberta) que contém a linha do cursor;
    /// Ctrl+Shift+] abre a fechada cujo cabeçalho é a linha do cursor. O cursor sobe ao cabeçalho:
    /// escondido, o `reveal` da tecla abria-a logo.
    /// Alt+F: a linha do cursor é o cabeçalho de uma dobra fechada ⇒ abre-a; senão dobra.
    pub(crate) fn toggle_fold_at_cursor(&mut self) {
        self.syntax.fresh(self.buffer.text());
        self.derive();
        let t = self.buffer.text();
        let line = t.line_of_byte(self.buffer.selection().primary().head);
        let closed = self
            .syntax
            .fold_at(t, line)
            .is_some_and(|f| self.folds.is_closed(t.byte_of_line(f.start_line)));
        self.fold_at_cursor(!closed);
    }

    pub(crate) fn fold_at_cursor(&mut self, fold: bool) {
        self.syntax.fresh(self.buffer.text());
        self.derive();
        let t = self.buffer.text();
        let line = t.line_of_byte(self.buffer.selection().primary().head);
        if !fold {
            if let Some(f) = self.syntax.fold_at(t, line)
                && self.folds.is_closed(t.byte_of_line(f.start_line))
            {
                self.folds.toggle(f, t);
                self.folds.arm(&mut self.syntax);
            }
            return;
        }
        let inner = self
            .syntax
            .folds(t)
            .into_iter()
            .filter(|f| f.start_line <= line && line <= f.end_line)
            .filter(|f| !self.folds.is_closed(t.byte_of_line(f.start_line)))
            .max_by_key(|f| f.start_line);
        let Some(f) = inner else { return };
        self.folds.toggle(f, t);
        self.folds.arm(&mut self.syntax);
        if line != f.start_line {
            let b = t.byte_of_line(f.start_line);
            self.buffer
                .set_selection(ph2d_code_buffer::Selection::single(
                    ph2d_code_buffer::Range::cursor(b),
                ));
        }
    }
}
