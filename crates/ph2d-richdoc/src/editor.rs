//! `Editor`: documento + selecção + as marcas «pendentes» de escrita (o botão Negrito com o
//! caret colapsado) e os atalhos markdown. É o que o widget do editor conduz.

use crate::doc::{BlockKind, Caret, RichDoc, Selection};
use crate::inline::Mark;
use crate::shortcuts::Shortcut;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Editor {
    doc: RichDoc,
    sel: Selection,
    /// Marcas que o próximo caractere escrito leva; `None` = as herdadas do texto.
    typing: Option<Vec<Mark>>,
}

impl Editor {
    pub fn new(doc: RichDoc) -> Editor {
        let sel = Selection::caret(doc.end());
        Editor {
            doc,
            sel,
            typing: None,
        }
    }

    pub fn doc(&self) -> &RichDoc {
        &self.doc
    }

    pub fn into_doc(self) -> RichDoc {
        self.doc
    }

    pub fn selection(&self) -> Selection {
        self.sel
    }

    pub fn set_selection(&mut self, sel: Selection) {
        let sel = Selection::new(self.doc.clamp(sel.anchor), self.doc.clamp(sel.head));
        if sel != self.sel {
            self.typing = None;
        }
        self.sel = sel;
    }

    fn put(&mut self, c: Caret) {
        self.sel = Selection::caret(c);
        self.typing = None;
    }

    /// Marcas activas: as da selecção inteira, ou as que a escrita levaria no caret.
    pub fn active_marks(&self) -> Vec<Mark> {
        if let Some(t) = &self.typing {
            return t.clone();
        }
        if self.sel.is_collapsed() {
            let c = self.sel.head;
            return self.doc.blocks()[c.block].inline.marks_for_typing(c.byte);
        }
        [
            Mark::Bold,
            Mark::Italic,
            Mark::Strike,
            Mark::Code,
            Mark::Mention,
        ]
        .into_iter()
        .filter(|m| self.doc.mark_active(self.sel, m))
        .collect()
    }

    /// Escreve (teclado): marcas pendentes + atalhos markdown.
    pub fn type_text(&mut self, text: &str) -> Option<Shortcut> {
        let marks = self.active_marks();
        let pending = self.typing.take();
        let c = self.doc.insert_text(self.sel, text, &marks);
        self.sel = Selection::caret(c);
        let last = text.chars().last()?;
        let hit = match last {
            ' ' => self.doc.block_shortcut(c),
            '-' => self.doc.rule_shortcut(c),
            '*' | '~' | '`' => self.doc.inline_shortcut(c),
            _ => None,
        };
        match hit {
            Some((c2, Shortcut::Mark(m))) => {
                self.sel = Selection::caret(c2);
                let inline = &self.doc.blocks()[c2.block].inline;
                let mut keep = inline.marks_for_typing(c2.byte);
                keep.retain(|k| !k.same_kind(&m));
                self.typing = Some(keep);
                Some(Shortcut::Mark(m))
            }
            Some((c2, s)) => {
                self.put(c2);
                Some(s)
            }
            // As marcas pendentes duram enquanto se escreve seguido (o código não é inclusivo).
            None => {
                self.typing = pending;
                None
            }
        }
    }

    /// Cola texto sem atalhos (as linhas viram blocos).
    pub fn paste(&mut self, text: &str) {
        let marks = self.active_marks();
        let c = self.doc.insert_text(self.sel, text, &marks);
        self.put(c);
    }

    pub fn backspace(&mut self) {
        let c = self.doc.delete_backward(self.sel);
        self.put(c);
    }

    pub fn delete(&mut self) {
        let c = self.doc.delete_forward(self.sel);
        self.put(c);
    }

    pub fn enter(&mut self) -> Option<Shortcut> {
        if self.sel.is_collapsed()
            && let Some((c, s)) = self.doc.enter_shortcut(self.sel.head)
        {
            self.put(c);
            return Some(s);
        }
        let c = self.doc.enter(self.sel);
        self.put(c);
        None
    }

    pub fn line_break(&mut self) {
        let c = self.doc.insert_line_break(self.sel);
        self.put(c);
    }

    /// Botão de marca: com selecção aplica/tira; com caret colapsado muda as marcas pendentes.
    pub fn toggle_mark(&mut self, mark: Mark) {
        if self.sel.is_collapsed() {
            let mut t = self.active_marks();
            if let Some(i) = t.iter().position(|m| m.same_kind(&mark)) {
                t.remove(i);
            } else {
                t.push(mark);
                t.sort();
            }
            self.typing = Some(t);
        } else {
            self.doc.toggle_mark(self.sel, &mark);
        }
    }

    pub fn set_block_kind(&mut self, kind: BlockKind) {
        let sel = self.doc.set_block_kind(self.sel, kind);
        self.set_selection(sel);
    }

    pub fn indent(&mut self) {
        self.sel = self.doc.indent(self.sel);
    }

    pub fn outdent(&mut self) {
        self.sel = self.doc.outdent(self.sel);
    }

    pub fn insert_mention(&mut self, name: &str) {
        let c = self.doc.insert_mention(self.sel, name);
        self.put(c);
    }

    /// Alterna o visto do item de checklist no bloco `i`.
    pub fn toggle_check(&mut self, i: usize) {
        if let Some(b) = self.doc.blocks().get(i)
            && let BlockKind::Check(v) = b.kind
        {
            let s = Selection::caret(Caret::new(i, 0));
            self.doc.set_block_kind(s, BlockKind::Check(!v));
        }
    }
}
