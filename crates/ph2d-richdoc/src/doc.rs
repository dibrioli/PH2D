//! O documento: blocos com tipo, nível de indentação e texto inline; caret e selecção.

use crate::inline::Inline;

/// Tipo de bloco. `Heading` vai de 1 a 3; `Code` leva a linguagem (pode ser vazia).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Paragraph,
    Heading(u8),
    Bullet,
    Numbered,
    Check(bool),
    Quote,
    Code(String),
    Rule,
}

impl BlockKind {
    /// Título com o nível preso a 1..=3.
    pub fn heading(level: u8) -> BlockKind {
        BlockKind::Heading(level.clamp(1, 3))
    }

    /// Item de lista (com marcador, numerado ou de checklist).
    pub fn is_list_item(&self) -> bool {
        matches!(
            self,
            BlockKind::Bullet | BlockKind::Numbered | BlockKind::Check(_)
        )
    }

    pub fn is_code(&self) -> bool {
        matches!(self, BlockKind::Code(_))
    }
}

/// Um bloco. `indent` = profundidade no aninhamento de listas: um item com `indent = k` é filho
/// do último item com `k - 1`; um bloco não-lista com `indent = k` é conteúdo desse item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub indent: usize,
    pub inline: Inline,
}

impl Block {
    pub fn new(kind: BlockKind, inline: Inline) -> Block {
        Block {
            kind,
            indent: 0,
            inline,
        }
    }

    pub fn paragraph(text: &str) -> Block {
        Block::new(BlockKind::Paragraph, Inline::plain(text))
    }

    pub fn with_indent(mut self, indent: usize) -> Block {
        self.indent = indent;
        self
    }
}

/// Posição: bloco e byte dentro do texto do bloco (fronteira de char).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caret {
    pub block: usize,
    pub byte: usize,
}

impl Caret {
    pub fn new(block: usize, byte: usize) -> Caret {
        Caret { block, byte }
    }
}

/// Selecção: `anchor` fica, `head` move-se. Colapsada = caret.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Selection {
    pub anchor: Caret,
    pub head: Caret,
}

impl Selection {
    pub fn caret(c: Caret) -> Selection {
        Selection { anchor: c, head: c }
    }

    pub fn new(anchor: Caret, head: Caret) -> Selection {
        Selection { anchor, head }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// `(início, fim)` por ordem do documento.
    pub fn ordered(&self) -> (Caret, Caret) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

/// Documento rico. Tem SEMPRE pelo menos um bloco; as indentações respeitam o aninhamento.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichDoc {
    blocks: Vec<Block>,
}

impl Default for RichDoc {
    fn default() -> Self {
        RichDoc::new()
    }
}

impl RichDoc {
    /// Um parágrafo vazio.
    pub fn new() -> RichDoc {
        RichDoc {
            blocks: vec![Block::paragraph("")],
        }
    }

    /// Normaliza: lista vazia vira um parágrafo; régua sem texto; indentações válidas.
    pub fn from_blocks(blocks: Vec<Block>) -> RichDoc {
        let mut d = RichDoc { blocks };
        d.normalize();
        d
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn block(&self, i: usize) -> Option<&Block> {
        self.blocks.get(i)
    }

    pub fn into_blocks(self) -> Vec<Block> {
        self.blocks
    }

    pub(crate) fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    /// O documento é só um parágrafo vazio?
    pub fn is_empty(&self) -> bool {
        self.blocks.len() == 1
            && self.blocks[0].kind == BlockKind::Paragraph
            && self.blocks[0].inline.is_empty()
    }

    /// Texto plano para pesquisa: um bloco por linha, réguas omitidas.
    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        for b in self.blocks.iter().filter(|b| b.kind != BlockKind::Rule) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(b.inline.text());
        }
        out
    }

    /// Prende o caret ao documento (bloco existente, byte em fronteira de char).
    pub fn clamp(&self, c: Caret) -> Caret {
        let block = c.block.min(self.blocks.len() - 1);
        let text = self.blocks[block].inline.text();
        let mut byte = c.byte.min(text.len());
        while !text.is_char_boundary(byte) {
            byte -= 1;
        }
        Caret { block, byte }
    }

    pub fn start(&self) -> Caret {
        Caret::new(0, 0)
    }

    pub fn end(&self) -> Caret {
        let b = self.blocks.len() - 1;
        Caret::new(b, self.blocks[b].inline.len())
    }

    /// Um caractere para trás (atravessa blocos).
    pub fn move_left(&self, c: Caret) -> Caret {
        let c = self.clamp(c);
        if c.byte > 0 {
            let t = self.blocks[c.block].inline.text();
            let p = t[..c.byte].char_indices().last().map_or(0, |(i, _)| i);
            Caret::new(c.block, p)
        } else if c.block > 0 {
            Caret::new(c.block - 1, self.blocks[c.block - 1].inline.len())
        } else {
            c
        }
    }

    /// Um caractere para a frente (atravessa blocos).
    pub fn move_right(&self, c: Caret) -> Caret {
        let c = self.clamp(c);
        let t = self.blocks[c.block].inline.text();
        if let Some(ch) = t[c.byte..].chars().next() {
            Caret::new(c.block, c.byte + ch.len_utf8())
        } else if c.block + 1 < self.blocks.len() {
            Caret::new(c.block + 1, 0)
        } else {
            c
        }
    }

    pub(crate) fn normalize(&mut self) {
        if self.blocks.is_empty() {
            self.blocks.push(Block::paragraph(""));
        }
        for b in &mut self.blocks {
            if let BlockKind::Heading(l) = b.kind {
                b.kind = BlockKind::heading(l);
            }
            if b.kind == BlockKind::Rule {
                b.inline = Inline::default();
            }
            if b.kind.is_code() {
                b.inline.clear_marks();
            }
        }
        self.fix_indents();
    }

    /// Cada bloco cabe no aninhamento do anterior: depois de um item `k` cabe até `k + 1`,
    /// depois de outro bloco `k` cabe até `k`.
    pub(crate) fn fix_indents(&mut self) {
        let mut max = 0;
        for b in &mut self.blocks {
            b.indent = b.indent.min(max);
            max = if b.kind.is_list_item() {
                b.indent + 1
            } else {
                b.indent
            };
        }
    }
}
