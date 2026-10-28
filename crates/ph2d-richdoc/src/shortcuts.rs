//! Atalhos markdown ao escrever: `# `, `- `, `1. `, `[] `, `> `, ```` ``` ````, `---`, `**x**`,
//! `*x*`, `~~x~~`, `` `x` ``. Chamados depois de inserir o último caractere escrito.

use crate::doc::{BlockKind, Caret, RichDoc};
use crate::inline::Mark;

/// O atalho que disparou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shortcut {
    Block(BlockKind),
    Rule,
    Mark(Mark),
}

/// Tipo de bloco pedido pelo prefixo escrito antes de um espaço.
fn block_prefix(prefix: &str, current: &BlockKind) -> Option<BlockKind> {
    let from_paragraph = *current == BlockKind::Paragraph;
    let kind = match prefix {
        "#" => BlockKind::heading(1),
        "##" => BlockKind::heading(2),
        "###" => BlockKind::heading(3),
        "-" | "*" | "+" => BlockKind::Bullet,
        ">" => BlockKind::Quote,
        "[]" | "[ ]" => return list_or_para(current).then_some(BlockKind::Check(false)),
        "[x]" | "[X]" => return list_or_para(current).then_some(BlockKind::Check(true)),
        p if p.starts_with("```") && !p[3..].contains('`') => BlockKind::Code(p[3..].to_owned()),
        p if p.len() > 1
            && p.ends_with('.')
            && p[..p.len() - 1].bytes().all(|b| b.is_ascii_digit()) =>
        {
            BlockKind::Numbered
        }
        _ => return None,
    };
    from_paragraph.then_some(kind)
}

fn list_or_para(k: &BlockKind) -> bool {
    matches!(k, BlockKind::Paragraph | BlockKind::Bullet)
}

impl RichDoc {
    /// Atalhos de BLOCO depois de escrever um espaço em `c` (o caret logo após o espaço).
    pub(crate) fn block_shortcut(&mut self, c: Caret) -> Option<(Caret, Shortcut)> {
        let b = &self.blocks()[c.block];
        let prefix = b.inline.text().get(..c.byte.checked_sub(1)?)?;
        let kind = block_prefix(prefix, &b.kind)?;
        let blk = &mut self.blocks_mut()[c.block];
        blk.inline.delete(0..c.byte);
        blk.kind = kind.clone();
        self.normalize();
        Some((Caret::new(c.block, 0), Shortcut::Block(kind)))
    }

    /// `---` escrito num parágrafo vira régua.
    pub(crate) fn rule_shortcut(&mut self, c: Caret) -> Option<(Caret, Shortcut)> {
        let b = &self.blocks()[c.block];
        if b.kind != BlockKind::Paragraph || b.inline.text() != "---" || c.byte != 3 {
            return None;
        }
        self.blocks_mut()[c.block].inline.delete(0..3);
        Some((
            self.insert_rule(crate::Selection::caret(Caret::new(c.block, 0))),
            Shortcut::Rule,
        ))
    }

    /// Enter sobre um parágrafo que é só ```` ```lang ```` ou `---`.
    pub(crate) fn enter_shortcut(&mut self, c: Caret) -> Option<(Caret, Shortcut)> {
        let b = &self.blocks()[c.block];
        if b.kind != BlockKind::Paragraph || c.byte != b.inline.len() {
            return None;
        }
        let t = b.inline.text().to_owned();
        if t == "---" {
            return self.rule_shortcut(c);
        }
        let lang = t.strip_prefix("```")?;
        if lang.contains('`') || lang.contains(char::is_whitespace) {
            return None;
        }
        let kind = BlockKind::Code(lang.to_owned());
        let blk = &mut self.blocks_mut()[c.block];
        blk.inline.delete(0..t.len());
        blk.kind = kind.clone();
        self.normalize();
        Some((Caret::new(c.block, 0), Shortcut::Block(kind)))
    }

    /// Atalhos INLINE depois de escrever `*`, `~` ou `` ` `` (o caret logo após).
    pub(crate) fn inline_shortcut(&mut self, c: Caret) -> Option<(Caret, Shortcut)> {
        let b = &self.blocks()[c.block];
        if b.kind.is_code() || b.kind == BlockKind::Rule {
            return None;
        }
        let text = b.inline.text();
        let before = &text[..c.byte];
        let candidates: [(&str, Mark); 4] = [
            ("**", Mark::Bold),
            ("~~", Mark::Strike),
            ("*", Mark::Italic),
            ("`", Mark::Code),
        ];
        for (delim, mark) in candidates {
            if !before.ends_with(delim) {
                continue;
            }
            let close = c.byte - delim.len();
            let Some(open) = find_open(text, before, close, delim, &b.inline, &mark) else {
                continue;
            };
            let inline = &mut self.blocks_mut()[c.block].inline;
            inline.delete(close..c.byte);
            inline.delete(open..open + delim.len());
            let end = close - delim.len();
            inline.add_mark(open..end, mark.clone());
            return Some((Caret::new(c.block, end), Shortcut::Mark(mark)));
        }
        None
    }
}

/// Procura o delimitador de abertura que casa com o que fecha em `close`.
fn find_open(
    text: &str,
    before: &str,
    close: usize,
    delim: &str,
    inline: &crate::Inline,
    mark: &Mark,
) -> Option<usize> {
    let single = delim.len() == 1;
    // Um `*` simples colado a outro `*` é metade de um `**`.
    if single && delim == "*" && text[..close].ends_with('*') {
        return None;
    }
    let open = before[..close].rfind(delim)?;
    let content = &text[open + delim.len()..close];
    let edge_ok = !content.is_empty()
        && (*mark == Mark::Code
            || !(content.starts_with(char::is_whitespace)
                || content.ends_with(char::is_whitespace)));
    if !edge_ok || content.contains(delim) {
        return None;
    }
    if single && delim == "*" && open > 0 && text[..open].ends_with('*') {
        return None;
    }
    if inline.covers(open..open + delim.len(), &Mark::Code) {
        return None;
    }
    Some(open)
}
