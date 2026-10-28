//! `RichDoc` → markdown. Escreve o que `parse` lê de volta igual: escapa o texto, tira os
//! espaços das pontas das linhas e larga a ênfase que o CommonMark não reconheceria.

use crate::inline_md::emit_inline;
use ph2d_richdoc::{Block, BlockKind, RichDoc};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Marker {
    Dash,
    Number,
}

/// Escreve markdown (CommonMark + tarefas + riscado). Vazio → `""`; senão acaba em `\n`.
pub fn render(doc: &RichDoc) -> String {
    let mut out = String::new();
    // Largura do marcador de cada item antepassado aberto.
    let mut widths: Vec<usize> = Vec::new();
    // Lista activa em cada nível: o marcador e o último número.
    let mut lists: Vec<(Marker, u64)> = Vec::new();
    // O bloco anterior: era item? estava vazio? com que indentação?
    let mut prev: Option<(bool, bool, usize)> = None;
    let blocks: Vec<&Block> = doc.blocks().iter().filter(|b| !skipped(b)).collect();
    let mut i = 0;
    while i < blocks.len() {
        let b = blocks[i];
        i += 1;
        let k = b.indent.min(widths.len());
        widths.truncate(k);
        let prefix = " ".repeat(widths.iter().sum());
        let is_item = b.kind.is_list_item();
        // Item vazio com um parágrafo filho logo a seguir: o markdown lê-o como o texto do item.
        let mut text = &b.inline;
        if is_item && blank(text) {
            if let Some(n) = blocks
                .get(i)
                .filter(|n| n.kind == BlockKind::Paragraph && n.indent == k + 1)
            {
                text = &n.inline;
                i += 1;
            }
        }
        match prev {
            // Um item vazio não interrompe o texto de cima (seria sublinhado de título).
            Some((_, false, pk)) if is_item && blank(text) && k > pk => out.push_str("\n\n"),
            Some((true, _, _)) if is_item => out.push('\n'),
            // Um item vazio só segura o filho na linha logo a seguir (sem linha em branco).
            Some((true, true, pk)) if k > pk => out.push('\n'),
            Some(_) => out.push_str("\n\n"),
            None => {}
        }
        prev = Some((is_item, blank(text), k));
        if is_item {
            lists.truncate(k + 1);
            let marker = if b.kind == BlockKind::Numbered {
                Marker::Number
            } else {
                Marker::Dash
            };
            let n = match lists.get(k) {
                Some(&(m, n)) if m == marker => n + 1,
                _ => 1,
            };
            lists.truncate(k);
            lists.resize(k, (Marker::Dash, 0));
            lists.push((marker, n));
            let head = match &b.kind {
                BlockKind::Numbered => format!("{n}."),
                BlockKind::Check(true) => "- [x]".to_owned(),
                BlockKind::Check(false) => "- [ ]".to_owned(),
                _ => "-".to_owned(),
            };
            let width = if marker == Marker::Number {
                head.len() + 1
            } else {
                2
            };
            widths.push(width);
            let cont = " ".repeat(widths.iter().sum());
            out.push_str(&prefix);
            out.push_str(&head);
            if !blank(text) {
                out.push(' ');
                emit_inline(&mut out, text, &cont, false);
            }
            continue;
        }
        lists.truncate(k);
        block(&mut out, b, &prefix);
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

/// Blocos que o markdown não consegue escrever (parágrafo/citação sem texto visível).
fn skipped(b: &Block) -> bool {
    matches!(b.kind, BlockKind::Paragraph | BlockKind::Quote) && blank(&b.inline)
}

fn blank(i: &ph2d_richdoc::Inline) -> bool {
    i.text().trim().is_empty()
}

fn block(out: &mut String, b: &Block, prefix: &str) {
    out.push_str(prefix);
    match &b.kind {
        BlockKind::Heading(l) => {
            out.push_str(&"#".repeat(usize::from((*l).clamp(1, 3))));
            if !b.inline.text().trim().is_empty() {
                out.push(' ');
                emit_inline(out, &b.inline, prefix, true);
            }
        }
        BlockKind::Quote => {
            out.push_str("> ");
            emit_inline(out, &b.inline, &format!("{prefix}> "), false);
        }
        BlockKind::Code(lang) => code_block(out, b.inline.text(), lang, prefix),
        BlockKind::Rule => out.push_str("---"),
        _ => emit_inline(out, &b.inline, prefix, false),
    }
}

fn code_block(out: &mut String, text: &str, lang: &str, prefix: &str) {
    let fence = "`".repeat(longest_run(text, '`').max(2) + 1);
    let lang: String = lang
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '`')
        .collect();
    out.push_str(&fence);
    out.push_str(&lang);
    for line in text.split('\n') {
        out.push('\n');
        if !line.is_empty() {
            out.push_str(prefix);
            out.push_str(line);
        }
    }
    if text.is_empty() {
        // Sem linhas: a cerca abre e fecha logo.
        out.truncate(out.len() - 1);
    }
    out.push('\n');
    out.push_str(prefix);
    out.push_str(&fence);
}

pub(crate) fn longest_run(text: &str, ch: char) -> usize {
    let (mut best, mut cur) = (0, 0);
    for c in text.chars() {
        cur = if c == ch { cur + 1 } else { 0 };
        best = best.max(cur);
    }
    best
}
