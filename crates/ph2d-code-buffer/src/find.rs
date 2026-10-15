//! Buscar/substituir no ficheiro e ir para linha (Onda 1 do CodeEditor, E9) — o núcleo puro;
//! a barra de busca e a prévia da contagem são do painel.

use crate::buffer::{Buffer, Edits};
use crate::change::{Change, ChangeSet};
use crate::history::EditKind;
use crate::motion::{first_non_space, is_word_char};
use crate::selection::{Range, Selection};
use crate::text::Text;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FindOptions {
    pub case_sensitive: bool,
    /// Só casa onde nem o carácter antes nem o depois são de palavra (`vida` ≠ `vida_max`).
    pub whole_word: bool,
}

/// Fim (em bytes de `hay`) do casamento de `needle` que começa em `at`, sem distinguir maiúsculas
/// (minúscula carácter a carácter, então `ÁGUA` casa `água` mesmo com larguras UTF-8 diferentes).
fn match_fold(hay: &str, at: usize, needle: &str) -> Option<usize> {
    let mut h = hay[at..].char_indices();
    let mut end = at;
    for n in needle.chars() {
        let (i, c) = h.next()?;
        if !c.to_lowercase().eq(n.to_lowercase()) {
            return None;
        }
        end = at + i + c.len_utf8();
    }
    Some(end)
}

/// Todas as ocorrências, sem sobreposição, da esquerda para a direita. Agulha vazia = nenhuma.
pub fn find_all(text: &Text, needle: &str, opts: FindOptions) -> Vec<std::ops::Range<usize>> {
    if needle.is_empty() {
        return Vec::new();
    }
    let hay = text.to_string();
    let word_edge = |r: &std::ops::Range<usize>| {
        !opts.whole_word
            || (!hay[..r.start].chars().next_back().is_some_and(is_word_char)
                && !hay[r.end..].chars().next().is_some_and(is_word_char))
    };
    let mut out = Vec::new();
    let mut at = 0;
    while at < hay.len() {
        let found = if opts.case_sensitive {
            hay[at..]
                .find(needle)
                .map(|i| (at + i, at + i + needle.len()))
        } else {
            hay[at..]
                .char_indices()
                .find_map(|(i, _)| match_fold(&hay, at + i, needle).map(|e| (at + i, e)))
        };
        let Some((s, e)) = found else { break };
        if word_edge(&(s..e)) {
            out.push(s..e);
            at = e;
        } else {
            at = s + hay[s..].chars().next().map_or(1, char::len_utf8);
        }
    }
    out
}

/// Troca TODAS as ocorrências por `with` num só passo de undo. Faixas que se sobrepõem (lista
/// velha) não trocam nada.
pub fn replace_all(buf: &mut Buffer, matches: &[std::ops::Range<usize>], with: &str) -> Edits {
    let changes = matches
        .iter()
        .map(|r| Change {
            range: r.clone(),
            insert: with.to_string(),
        })
        .collect();
    match ChangeSet::new(changes) {
        Some(cs) => buf.apply(cs, None, EditKind::Other),
        None => Vec::new(),
    }
}

/// Troca UMA ocorrência; o cursor fica no fim do texto novo.
pub fn replace_one(buf: &mut Buffer, m: std::ops::Range<usize>, with: &str) -> Edits {
    let end = m.start + with.len();
    let cs = ChangeSet::single(m, with);
    buf.apply(
        cs,
        Some(Selection::single(Range::cursor(end))),
        EditKind::Other,
    )
}

/// Ctrl+G: o cursor no 1.º não-espaço da linha `line_1based` (fora da faixa encosta no limite).
/// Devolve o byte do cursor.
pub fn goto_line(buf: &mut Buffer, line_1based: usize) -> usize {
    let t = buf.text();
    let line = line_1based.clamp(1, t.line_count()) - 1;
    let at = first_non_space(t, line);
    buf.set_selection(Selection::single(Range::cursor(at)));
    buf.seal_history();
    at
}
