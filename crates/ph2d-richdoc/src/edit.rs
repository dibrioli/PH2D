//! Operações puras de edição sobre `RichDoc`. Cada uma recebe a selecção e devolve o caret
//! (ou a selecção) que resulta.

use crate::doc::{Block, BlockKind, Caret, RichDoc, Selection};
use crate::inline::{Inline, Mark};

impl RichDoc {
    /// Apaga o conteúdo seleccionado (junta blocos se atravessar) e devolve o caret.
    pub fn delete_selection(&mut self, sel: Selection) -> Caret {
        let (a, b) = sel.ordered();
        let (a, b) = (self.clamp(a), self.clamp(b));
        if a == b {
            return a;
        }
        let blocks = self.blocks_mut();
        if a.block == b.block {
            blocks[a.block].inline.delete(a.byte..b.byte);
            return a;
        }
        let mut tail = blocks[b.block].inline.clone();
        let tail_kind = blocks[b.block].kind.clone();
        let tail = tail.split_off(b.byte);
        blocks.drain(a.block + 1..=b.block);
        let first = &mut blocks[a.block];
        first.inline.split_off(a.byte);
        if first.kind == BlockKind::Rule {
            first.kind = tail_kind;
        }
        append_into(first, tail);
        self.normalize();
        a
    }

    /// Insere `text` com exactamente `marks`; `\n` parte o bloco (num bloco de código é literal).
    pub fn insert_text(&mut self, sel: Selection, text: &str, marks: &[Mark]) -> Caret {
        let mut c = self.delete_selection(sel);
        for (i, line) in text.split('\n').enumerate() {
            if i > 0 {
                c = if self.blocks()[c.block].kind.is_code() {
                    self.insert_raw(c, "\n", &[])
                } else {
                    self.split_block(c)
                };
            }
            c = self.insert_raw(c, line, marks);
        }
        c
    }

    /// Quebra de linha DENTRO do bloco (Shift+Enter).
    pub fn insert_line_break(&mut self, sel: Selection) -> Caret {
        let c = self.delete_selection(sel);
        self.insert_raw(c, "\n", &[])
    }

    /// Insere `@nome` como menção, seguido de um espaço.
    pub fn insert_mention(&mut self, sel: Selection, name: &str) -> Caret {
        let c = self.delete_selection(sel);
        let text = format!("@{name}");
        let end = self.insert_raw(c, &text, &[Mark::Mention]);
        self.insert_raw(end, " ", &[])
    }

    fn insert_raw(&mut self, c: Caret, text: &str, marks: &[Mark]) -> Caret {
        if self.blocks()[c.block].kind == BlockKind::Rule {
            let at = c.block + 1;
            self.blocks_mut().insert(at, Block::paragraph(""));
            return self.insert_raw(Caret::new(at, 0), text, marks);
        }
        let b = &mut self.blocks_mut()[c.block];
        let marks: &[Mark] = if b.kind.is_code() { &[] } else { marks };
        b.inline.insert(c.byte, text, marks);
        Caret::new(c.block, c.byte + text.len())
    }

    /// Parte o bloco no caret: o resto vai para um bloco novo do mesmo tipo (título → parágrafo,
    /// checklist → por marcar).
    pub(crate) fn split_block(&mut self, c: Caret) -> Caret {
        let c = self.clamp(c);
        let blocks = self.blocks_mut();
        let cur = &mut blocks[c.block];
        if cur.kind == BlockKind::Heading(1)
            || cur.kind == BlockKind::Heading(2)
            || cur.kind == BlockKind::Heading(3)
        {
            if c.byte == 0 && !cur.inline.is_empty() {
                let indent = cur.indent;
                blocks.insert(c.block, Block::paragraph("").with_indent(indent));
                return Caret::new(c.block + 1, 0);
            }
        }
        let tail = cur.inline.split_off(c.byte);
        let kind = match &cur.kind {
            BlockKind::Heading(_) | BlockKind::Rule => BlockKind::Paragraph,
            BlockKind::Check(_) => BlockKind::Check(false),
            k => k.clone(),
        };
        let indent = cur.indent;
        blocks.insert(
            c.block + 1,
            Block {
                kind,
                indent,
                inline: tail,
            },
        );
        Caret::new(c.block + 1, 0)
    }

    /// Enter: no código insere `\n` (numa última linha vazia sai do bloco); num item/citação vazio
    /// desindenta ou sai da lista; senão parte o bloco.
    pub fn enter(&mut self, sel: Selection) -> Caret {
        let c = self.delete_selection(sel);
        let b = &self.blocks()[c.block];
        let at_end = c.byte == b.inline.len();
        if b.kind.is_code() {
            if at_end && b.inline.text().ends_with('\n') {
                let len = b.inline.len();
                let indent = b.indent;
                self.blocks_mut()[c.block].inline.delete(len - 1..len);
                self.blocks_mut()
                    .insert(c.block + 1, Block::paragraph("").with_indent(indent));
                return Caret::new(c.block + 1, 0);
            }
            return self.insert_raw(c, "\n", &[]);
        }
        if (b.kind.is_list_item() || b.kind == BlockKind::Quote) && b.inline.is_empty() {
            let blk = &mut self.blocks_mut()[c.block];
            if blk.kind.is_list_item() && blk.indent > 0 {
                blk.indent -= 1;
            } else {
                blk.kind = BlockKind::Paragraph;
            }
            self.fix_indents();
            return c;
        }
        let c = self.split_block(c);
        self.fix_indents();
        c
    }

    /// Backspace. No início de um bloco: item/título/citação → parágrafo; parágrafo indentado →
    /// desindenta; senão junta ao bloco anterior. Uma menção apaga-se inteira.
    pub fn delete_backward(&mut self, sel: Selection) -> Caret {
        if !sel.is_collapsed() {
            return self.delete_selection(sel);
        }
        let c = self.clamp(sel.head);
        let b = &self.blocks()[c.block];
        if b.kind == BlockKind::Rule {
            return self.remove_block(c.block);
        }
        if c.byte > 0 {
            let prev = self.move_left(c);
            let start = b
                .inline
                .mention_at(prev.byte)
                .map_or(prev.byte, |r| r.start);
            self.blocks_mut()[c.block].inline.delete(start..c.byte);
            return Caret::new(c.block, start);
        }
        let blk = &mut self.blocks_mut()[c.block];
        if blk.kind != BlockKind::Paragraph {
            blk.kind = BlockKind::Paragraph;
            self.fix_indents();
            return c;
        }
        if blk.indent > 0 {
            blk.indent -= 1;
            self.fix_indents();
            return c;
        }
        if c.block == 0 {
            return c;
        }
        if self.blocks()[c.block - 1].kind == BlockKind::Rule {
            self.blocks_mut().remove(c.block - 1);
            self.fix_indents();
            return Caret::new(c.block - 1, 0);
        }
        self.join_with_next(c.block - 1)
    }

    /// Delete. No fim de um bloco junta o seguinte (uma régua seguinte sai). Menção inteira.
    pub fn delete_forward(&mut self, sel: Selection) -> Caret {
        if !sel.is_collapsed() {
            return self.delete_selection(sel);
        }
        let c = self.clamp(sel.head);
        let b = &self.blocks()[c.block];
        if b.kind == BlockKind::Rule {
            return self.remove_block(c.block);
        }
        if c.byte < b.inline.len() {
            let next = self.move_right(c);
            let end = b.inline.mention_at(c.byte).map_or(next.byte, |r| r.end);
            let start = b.inline.mention_at(c.byte).map_or(c.byte, |r| r.start);
            self.blocks_mut()[c.block].inline.delete(start..end);
            return Caret::new(c.block, start);
        }
        if c.block + 1 >= self.blocks().len() {
            return c;
        }
        if self.blocks()[c.block + 1].kind == BlockKind::Rule {
            self.blocks_mut().remove(c.block + 1);
            self.fix_indents();
            return c;
        }
        self.join_with_next(c.block)
    }

    fn join_with_next(&mut self, i: usize) -> Caret {
        let next = self.blocks_mut().remove(i + 1);
        let at = self.blocks()[i].inline.len();
        append_into(&mut self.blocks_mut()[i], next.inline);
        self.fix_indents();
        Caret::new(i, at)
    }

    fn remove_block(&mut self, i: usize) -> Caret {
        self.blocks_mut().remove(i);
        self.normalize();
        if i < self.blocks().len() {
            Caret::new(i, 0)
        } else {
            self.end()
        }
    }

    /// Insere uma régua depois do bloco do caret (se este estiver vazio, transforma-o) e devolve
    /// o caret num parágrafo a seguir.
    pub fn insert_rule(&mut self, sel: Selection) -> Caret {
        let c = self.delete_selection(sel);
        let empty = self.blocks()[c.block].inline.is_empty();
        let at = if empty {
            let b = &mut self.blocks_mut()[c.block];
            b.kind = BlockKind::Rule;
            c.block
        } else {
            let indent = self.blocks()[c.block].indent;
            let rule = Block::new(BlockKind::Rule, Inline::default()).with_indent(indent);
            self.blocks_mut().insert(c.block + 1, rule);
            c.block + 1
        };
        let next_is_editable = self
            .blocks()
            .get(at + 1)
            .is_some_and(|b| b.kind != BlockKind::Rule);
        if !next_is_editable {
            let indent = self.blocks()[at].indent;
            self.blocks_mut()
                .insert(at + 1, Block::paragraph("").with_indent(indent));
        }
        self.normalize();
        Caret::new(at + 1, 0)
    }

    /// Muda o tipo dos blocos tocados pela selecção. `Rule` = `insert_rule`.
    pub fn set_block_kind(&mut self, sel: Selection, kind: BlockKind) -> Selection {
        if kind == BlockKind::Rule {
            return Selection::caret(self.insert_rule(sel));
        }
        let (a, b) = sel.ordered();
        let (a, b) = (self.clamp(a), self.clamp(b));
        for blk in &mut self.blocks_mut()[a.block..=b.block] {
            if blk.kind != BlockKind::Rule {
                blk.kind = kind.clone();
            }
        }
        self.normalize();
        sel
    }

    /// Tab: indenta os blocos tocados até onde o aninhamento deixa.
    pub fn indent(&mut self, sel: Selection) -> Selection {
        let (a, b) = sel.ordered();
        let (a, b) = (self.clamp(a), self.clamp(b));
        for i in a.block..=b.block {
            self.blocks_mut()[i].indent += 1;
            self.fix_indents();
        }
        sel
    }

    /// Shift+Tab: desindenta os blocos tocados (os filhos acompanham).
    pub fn outdent(&mut self, sel: Selection) -> Selection {
        let (a, b) = sel.ordered();
        let (a, b) = (self.clamp(a), self.clamp(b));
        for i in a.block..=b.block {
            let blk = &mut self.blocks_mut()[i];
            blk.indent = blk.indent.saturating_sub(1);
        }
        self.fix_indents();
        sel
    }

    /// Alterna uma marca na selecção: se TODO o texto seleccionado já a tem, tira-a; senão põe.
    /// Código e réguas ficam de fora. Devolve se mudou alguma coisa.
    pub fn toggle_mark(&mut self, sel: Selection, mark: &Mark) -> bool {
        let segs = self.segments(sel);
        if segs.is_empty() {
            return false;
        }
        let all = segs
            .iter()
            .all(|&(i, s, e)| self.blocks()[i].inline.covers(s..e, mark));
        for (i, s, e) in segs {
            let inline = &mut self.blocks_mut()[i].inline;
            if all {
                inline.remove_mark(s..e, mark);
            } else {
                inline.add_mark(s..e, mark.clone());
            }
        }
        true
    }

    /// A marca cobre toda a selecção (não colapsada)? É o estado do botão da barra.
    pub fn mark_active(&self, sel: Selection, mark: &Mark) -> bool {
        let segs = self.segments(sel);
        !segs.is_empty()
            && segs
                .iter()
                .all(|&(i, s, e)| self.blocks()[i].inline.covers(s..e, mark))
    }

    /// Troços `(bloco, início, fim)` não vazios da selecção, fora de código e réguas.
    fn segments(&self, sel: Selection) -> Vec<(usize, usize, usize)> {
        let (a, b) = sel.ordered();
        let (a, b) = (self.clamp(a), self.clamp(b));
        (a.block..=b.block)
            .filter_map(|i| {
                let blk = &self.blocks()[i];
                let s = if i == a.block { a.byte } else { 0 };
                let e = if i == b.block {
                    b.byte
                } else {
                    blk.inline.len()
                };
                (s < e && !blk.kind.is_code() && blk.kind != BlockKind::Rule).then_some((i, s, e))
            })
            .collect()
    }
}

fn append_into(dst: &mut Block, mut src: Inline) {
    if dst.kind.is_code() {
        src.clear_marks();
    }
    dst.inline.append(src);
}
