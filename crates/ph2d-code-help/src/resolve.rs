//! ⭐ **A porta ÚNICA: que página e que membro um sítio do script nomeia.** Chamada pela mão do Ctrl
//! (`reach`), pela chegada do Ctrl+clique/F12 (`goto`) e pelo link dentro da página — três
//! consumidores, uma resposta.
//!
//! Só depois de o `goto` saber que o nome NÃO está declarado neste script (a definição manda: um
//! `local print` vai à declaração). O balão do `luau-lsp` decide primeiro onde é exacto e a árvore
//! depois — medido em `docs/CodeEditor/ferramentas/lspprobe_ajuda.py` (`ajuda_bruto.json`):
//! `m.cos` só o balão o sabe (`function math.cos(`), `s:upper()` só pelo tipo do 1.º argumento,
//! `table.insert`/`debug.traceback` vêm sem nome, `math.pi` é só `number`.

use crate::luau::GLOBALS;
use crate::ours::TYPES;
use crate::{Anchor, anchor};

/// O que se sabe do sítio.
#[derive(Clone, Copy, Debug, Default)]
pub struct Query<'a> {
    /// O identificador sob o cursor.
    pub word: &'a str,
    /// O identificador logo antes do `.`/`:` que o precede (`math` em `math.sin`); `None` se não há.
    pub qualifier: Option<&'a str>,
    /// O separador é `:` (método).
    pub colon: bool,
    /// O balão do `luau-lsp` para o sítio (Markdown cru), se já chegou.
    pub hover: Option<&'a str>,
}

/// A página e o membro, ou `None` se o sítio não tem página.
#[must_use]
pub fn resolve(q: &Query<'_>) -> Option<Anchor> {
    let line = q.hover.and_then(first_code_line);
    if let Some(a) = line.and_then(|l| from_hover(l, q.word)) {
        return Some(a);
    }
    match q.qualifier {
        Some(lib) if !q.colon => anchor(lib, Some(q.word)),
        Some(_) => None,
        None => anchor(GLOBALS, Some(q.word))
            .or_else(|| anchor(q.word, None))
            .or_else(|| anchor(TYPES, Some(q.word))),
    }
}

/// A 1.ª linha de código do balão (dentro da cerca ```` ``` ````, ou a 1.ª se não há cerca).
fn first_code_line(md: &str) -> Option<&str> {
    let mut lines = md.lines().map(str::trim).filter(|l| !l.is_empty());
    let first = lines.next()?;
    if first.starts_with("```") {
        lines.next()
    } else {
        Some(first)
    }
}

/// `function A.B(` · `function x:B(T` (o tipo do 1.º argumento é a página) · `type T =`.
fn from_hover(line: &str, word: &str) -> Option<Anchor> {
    if let Some(t) = line.strip_prefix("type ") {
        let name = t
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .next()?;
        return (name == word).then(|| anchor(TYPES, Some(name))).flatten();
    }
    // o receptor pode ser um literal: `function ('x'):upper(string): string`
    let f = line.strip_prefix("function ")?;
    for colon in [true, false] {
        let needle = format!("{}{word}", if colon { ':' } else { '.' });
        let Some(i) = f.find(&needle) else { continue };
        let rest = &f[i + needle.len()..];
        if !rest.starts_with(['(', '<']) {
            continue;
        }
        if !colon {
            return anchor(&f[..i], Some(word));
        }
        let args = &rest[rest.find('(')? + 1..];
        let first = args.split([',', ')']).next()?.trim();
        let ty = first.rsplit(':').next()?.trim().trim_end_matches('?');
        return anchor(ty, Some(word));
    }
    None
}
