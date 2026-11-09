//! O USO de cada nome nos scripts do projeto (fora de strings e comentários): a última chave da
//! ordem da lista de completação.

use std::collections::BTreeMap;

/// Quantas vezes cada nome aparece nos scripts do projeto (fora de strings e comentários), e cada
/// `a.b`/`a:b` também (a chave de um membro de `ph2d`).
#[must_use]
pub fn usage<'a>(texts: impl IntoIterator<Item = &'a str>) -> BTreeMap<String, u32> {
    let mut out = BTreeMap::new();
    for s in texts {
        count_words(s, &mut out);
    }
    out
}

/// A chave de uso de uma proposta.
pub(super) fn usage_key(qualifier: Option<&str>, label: &str) -> String {
    qualifier.map_or_else(|| label.to_owned(), |q| format!("{q}.{label}"))
}

/// O nível de um `[[`/`[==[` longo que começa em `b[i]` (`None` = não é um).
fn long_bracket(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'[') {
        return None;
    }
    let eq = b[i + 1..].iter().take_while(|&&c| c == b'=').count();
    (b.get(i + 1 + eq) == Some(&b'[')).then_some(eq)
}

/// O fim (exclusivo) do longo de nível `eq` cujo conteúdo começa em `from`.
fn long_end(b: &[u8], from: usize, eq: usize) -> usize {
    let mut close = vec![b']'];
    close.extend(std::iter::repeat_n(b'=', eq));
    close.push(b']');
    b[from..]
        .windows(close.len())
        .position(|w| w == close.as_slice())
        .map_or(b.len(), |p| from + p + close.len())
}

fn count_words(s: &str, out: &mut BTreeMap<String, u32>) {
    let b = s.as_bytes();
    let mut i = 0;
    // O último nome e se veio um `.`/`:` depois dele.
    let mut prev: Option<(usize, usize)> = None;
    let mut member = false;
    while i < b.len() {
        let c = b[i];
        if c == b'-' && b.get(i + 1) == Some(&b'-') {
            i = match long_bracket(b, i + 2) {
                Some(eq) => long_end(b, i + 4 + eq, eq),
                None => b[i..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .map_or(b.len(), |p| i + p),
            };
            prev = None;
            continue;
        }
        if let Some(eq) = long_bracket(b, i) {
            i = long_end(b, i + 2 + eq, eq);
            prev = None;
            continue;
        }
        if matches!(c, b'"' | b'\'' | b'`') {
            i += 1;
            while i < b.len() && b[i] != c && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
            prev = None;
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let st = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let w = &s[st..i];
            *out.entry(w.to_owned()).or_insert(0) += 1;
            if member && let Some((ps, pe)) = prev {
                *out.entry(format!("{}.{w}", &s[ps..pe])).or_insert(0) += 1;
            }
            prev = Some((st, i));
            member = false;
            continue;
        }
        if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            prev = None;
            continue;
        }
        if (c == b'.' || c == b':') && prev.is_some() && !member && b.get(i + 1) != Some(&b'.') {
            member = true;
        } else if !c.is_ascii_whitespace() {
            prev = None;
            member = false;
        }
        i += 1;
    }
}
