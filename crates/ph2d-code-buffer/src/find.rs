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

thread_local! {
    static SCANNED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Quantos bytes de texto a busca leu nesta thread desde o início: o trabalho dela, sem relógio
/// (o gate de custo por tecla conta-o).
#[must_use]
pub fn find_bytes_scanned() -> u64 {
    SCANNED.with(std::cell::Cell::get)
}

fn scanned(n: usize) {
    SCANNED.with(|c| c.set(c.get() + n as u64));
}

/// ⭐ **As ocorrências depois de `edits`** (ordem sequencial, como o `Buffer` as devolve) — sem
/// varrer o texto inteiro: cada troca suja uma janela à volta dela (o maior casamento possível de
/// cada lado, mais um carácter para a fronteira de palavra); as ocorrências fora das janelas andam
/// com o texto, e cada janela é re-procurada até a busca voltar a pisar o mesmo passo que a do zero
/// (o casamento guloso pode mudar de fase: `aa` em `aaaa`). O custo é o das janelas — não cresce
/// com o ficheiro — mais uma soma por ocorrência. Igual a [`find_all`] (gate de traço aleatório).
pub fn find_update(
    text: &Text,
    needle: &str,
    opts: FindOptions,
    matches: &mut Vec<std::ops::Range<usize>>,
    edits: &[crate::change::TextEdit],
) {
    if needle.is_empty() {
        matches.clear();
        return;
    }
    // O maior casamento em bytes (minúsculas podem mudar de largura) e um carácter de contexto.
    let span = 4 * needle.chars().count() + 4;
    let mut dirty: Vec<std::ops::Range<usize>> = Vec::new();
    for e in edits {
        let delta = e.new_end_byte as isize - e.old_end_byte as isize;
        let shift = |b: usize| (b as isize + delta) as usize;
        let lo = e.start_byte.saturating_sub(span);
        let hi = e.old_end_byte + span;
        // Uma ocorrência que toca a janela sai; a janela começa no início dela (ela não mudou).
        let mut start = lo;
        matches.retain(|m| {
            let touches = m.end > lo && m.start < hi;
            if touches {
                start = start.min(m.start);
            }
            !touches
        });
        for m in matches.iter_mut().filter(|m| m.start >= hi) {
            *m = shift(m.start)..shift(m.end);
        }
        for d in &mut dirty {
            *d = if d.start >= e.old_end_byte {
                shift(d.start)..shift(d.end)
            } else if d.end <= e.start_byte {
                d.clone()
            } else {
                d.start.min(e.start_byte)..shift(d.end.max(e.old_end_byte))
            };
        }
        dirty.push(start..e.new_end_byte + span);
    }
    dirty.sort_by_key(|d| d.start);
    let mut merged: Vec<std::ops::Range<usize>> = Vec::new();
    for d in dirty {
        match merged.last_mut() {
            Some(last) if d.start <= last.end => last.end = last.end.max(d.end),
            _ => merged.push(d),
        }
    }
    let kept = std::mem::take(matches);
    let mut k = 0;
    for win in merged {
        while k < kept.len() && kept[k].end <= win.start {
            matches.push(kept[k].clone());
            k += 1;
        }
        let from = matches.last().map_or(0, |m| m.end);
        let p = rescan(
            text,
            needle,
            opts,
            from.max(text.floor_char(win.start)),
            win.end,
            &kept[k..],
            matches,
        );
        while k < kept.len() && kept[k].start < p {
            k += 1;
        }
    }
    matches.extend_from_slice(&kept[k..]);
}

/// Procura de `p` em diante até passar `stop` num sítio que não está DENTRO de uma ocorrência
/// antiga (aí a busca do zero estava no mesmo passo); devolve onde parou.
fn rescan(
    text: &Text,
    needle: &str,
    opts: FindOptions,
    mut p: usize,
    stop: usize,
    old: &[std::ops::Range<usize>],
    out: &mut Vec<std::ops::Range<usize>>,
) -> usize {
    let len = text.len();
    let reach = 4 * needle.chars().count() + 4;
    // A janela de leitura: um carácter antes de `p` (fronteira de palavra) até `p + reach`.
    let mut base = text.floor_char(p.saturating_sub(4));
    let mut w = String::new();
    let mut w_end = base;
    while p < len {
        if p >= stop && !old.iter().take_while(|m| m.start < p).any(|m| m.end > p) {
            break;
        }
        if p + reach > w_end && w_end < len {
            base = text.floor_char(p.saturating_sub(4));
            w_end = text
                .floor_char((p + reach * 8).min(len))
                .max(text.ceil_char((p + reach).min(len)));
            w = text.slice(base..w_end);
            scanned(w.len());
        }
        let at = p - base;
        let end = if opts.case_sensitive {
            w[at..].starts_with(needle).then(|| at + needle.len())
        } else {
            match_fold(&w, at, needle)
        };
        let edge = |e: usize| {
            !opts.whole_word
                || (!w[..at].chars().next_back().is_some_and(is_word_char)
                    && !w[e..].chars().next().is_some_and(is_word_char))
        };
        match end {
            Some(e)
                if e + base <= w_end
                    && (e < w.len() || w_end == len || !opts.whole_word)
                    && edge(e) =>
            {
                out.push(p..base + e);
                p = base + e;
            }
            _ => p += w[at..].chars().next().map_or(1, char::len_utf8),
        }
    }
    p
}

/// Todas as ocorrências, sem sobreposição, da esquerda para a direita. Agulha vazia = nenhuma.
pub fn find_all(text: &Text, needle: &str, opts: FindOptions) -> Vec<std::ops::Range<usize>> {
    if needle.is_empty() {
        return Vec::new();
    }
    let hay = text.to_string();
    scanned(hay.len());
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
