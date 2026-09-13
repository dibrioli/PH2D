//! **O LEITOR DE FONTE** — um sítio só para as formas de um literal em Rust.
//!
//! ⚠️ Cada forma aqui já custou um censo errado neste repo: o literal de CARÁCTER com aspas dentro
//! (`find('"')` lido como string a abrir — o censo de 10/09 leu `438` onde havia `418`), o tempo
//! de vida que abre com `'` e não fecha, a string CRUA com `#`, e o comentário que parece código.

use std::ops::Range;

pub(crate) fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn starts_with_at(code: &[char], i: usize, pat: &str) -> bool {
    pat.chars()
        .enumerate()
        .all(|(k, p)| code.get(i + k) == Some(&p))
}

/// Se um literal de string COMEÇA em `i` — com os prefixos `b`, `c`, `r`, `br`, `cr` e os `#` de
/// uma string crua —, o índice logo a seguir a ele (o fim do ficheiro, se não fechar).
pub(crate) fn string_end(b: &[char], i: usize) -> Option<usize> {
    let n = b.len();
    let mut j = i;
    if b[j] != '"' {
        // Um prefixo só é prefixo no início de um token: `r#type` é um identificador cru, e a
        // guarda do `#` abaixo recusa-o.
        if i > 0 && is_ident(b[i - 1]) {
            return None;
        }
        if b[j] == 'b' || b[j] == 'c' {
            j += 1;
        }
        if j < n && b[j] == 'r' {
            j += 1;
        }
        if j == i {
            return None;
        }
    }
    let raw = j > i && b[j - 1] == 'r';
    let mut hashes = 0usize;
    if raw {
        while j < n && b[j] == '#' {
            hashes += 1;
            j += 1;
        }
    }
    if j >= n || b[j] != '"' {
        return None;
    }
    j += 1;
    if raw {
        while j < n {
            if b[j] == '"' && (0..hashes).all(|k| j + 1 + k < n && b[j + 1 + k] == '#') {
                return Some(j + 1 + hashes);
            }
            j += 1;
        }
        return Some(n);
    }
    let mut esc = false;
    while j < n {
        let c = b[j];
        if esc {
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == '"' {
            return Some(j + 1);
        }
        j += 1;
    }
    Some(n)
}

/// Se um literal de CARÁCTER começa em `i`, o índice logo a seguir. ⚠️ Um `'` também abre um tempo
/// de vida (`&'a str`), que não fecha: o discriminador é haver um `'` a fechar ao alcance.
fn char_literal_end(b: &[char], i: usize) -> Option<usize> {
    let n = b.len();
    if i + 1 < n && b[i + 1] == '\\' {
        // `'\u{00b7}'` tem dez caracteres.
        (i + 2..(i + 11).min(n))
            .find(|&j| b[j] == '\'')
            .map(|j| j + 1)
    } else if i + 2 < n && b[i + 2] == '\'' {
        Some(i + 3)
    } else {
        None
    }
}

/// O fonte com comentários (e literais de carácter) trocados por espaços — as quebras de linha
/// ficam, logo as linhas continuam a contar certo — e o interior das strings INTACTO.
pub(crate) fn strip_comments(src: &str) -> Vec<char> {
    let b: Vec<char> = src.chars().collect();
    let n = b.len();
    let mut out = Vec::with_capacity(n);
    let mut i = 0usize;
    while i < n {
        if let Some(end) = string_end(&b, i) {
            out.extend_from_slice(&b[i..end]);
            i = end;
            continue;
        }
        let c = b[i];
        if c == '\''
            && let Some(end) = char_literal_end(&b, i)
        {
            out.extend(std::iter::repeat_n(' ', end - i));
            i = end;
            continue;
        }
        if c == '/' && i + 1 < n && b[i + 1] == '/' {
            while i < n && b[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < n && b[i + 1] == '*' {
            let mut depth = 0usize;
            while i < n {
                if b[i] == '/' && i + 1 < n && b[i + 1] == '*' {
                    depth += 1;
                    out.extend([' ', ' ']);
                    i += 2;
                } else if b[i] == '*' && i + 1 < n && b[i + 1] == '/' {
                    depth -= 1;
                    out.extend([' ', ' ']);
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(if b[i] == '\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Um literal de string achado no código.
pub(crate) struct Lit {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) content: Range<usize>,
    /// `b"…"` / `c"…"` — bytes, não texto.
    pub(crate) bytes: bool,
}

pub(crate) fn literals(code: &[char]) -> Vec<Lit> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < code.len() {
        if let Some(end) = string_end(code, i) {
            let q = (i..end).find(|&k| code[k] == '"').unwrap_or(i);
            let prefix = &code[i..q];
            let hashes = prefix.iter().filter(|&&c| c == '#').count();
            let bytes = prefix.iter().any(|&c| c == 'b' || c == 'c');
            let content_end = end.saturating_sub(1 + hashes).max(q + 1);
            out.push(Lit {
                start: i,
                end,
                content: q + 1..content_end,
                bytes,
            });
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// `true` em toda posição que está DENTRO de um literal de string (aspas incluídas).
pub(crate) fn string_mask(code: &[char]) -> Vec<bool> {
    let mut mask = vec![false; code.len()];
    for l in literals(code) {
        for m in &mut mask[l.start..l.end] {
            *m = true;
        }
    }
    mask
}

fn matching(code: &[char], mask: &[bool], open: usize) -> Option<usize> {
    let close = match code[open] {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    let o = code[open];
    let mut depth = 0usize;
    for k in open..code.len() {
        if mask[k] {
            continue;
        }
        if code[k] == o {
            depth += 1;
        } else if code[k] == close {
            depth -= 1;
            if depth == 0 {
                return Some(k);
            }
        }
    }
    None
}

/// O fim do ITEM que um atributo em `attr` gateia: o `;` dele, ou o `}` do corpo dele.
fn item_end(code: &[char], mask: &[bool], attr: usize) -> Option<usize> {
    let mut j = attr;
    loop {
        while j < code.len() && code[j].is_whitespace() {
            j += 1;
        }
        if j + 1 < code.len() && code[j] == '#' && code[j + 1] == '[' {
            j = matching(code, mask, j + 1)? + 1;
        } else {
            break;
        }
    }
    let mut depth = 0usize;
    for k in j..code.len() {
        if mask[k] {
            continue;
        }
        match code[k] {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '{' if depth == 0 => return matching(code, mask, k).map(|e| e + 1),
            ';' if depth == 0 => return Some(k + 1),
            _ => {}
        }
    }
    None
}

/// Apaga todo item sob `#[cfg(test)]` / `#[cfg(all(test, …))]` escrito DENTRO do ficheiro — o
/// `mod tests { … }` inline, um `fn` só de teste, um `use` só de teste. ⛔ `cfg(any(test, …))` NÃO
/// entra: compila no produto com a feature ligada. (O ficheiro INTEIRO de teste é a pergunta do
/// [`crate::cfg_test`], feita ao pai.)
pub(crate) fn blank_cfg_test_items(code: &mut [char]) {
    let mask = string_mask(code);
    let mut i = 0usize;
    while i < code.len() {
        if !mask[i]
            && (starts_with_at(code, i, "#[cfg(test)]")
                || starts_with_at(code, i, "#[cfg(all(test"))
            && let Some(end) = item_end(code, &mask, i)
        {
            for ch in &mut code[i..end] {
                if *ch != '\n' {
                    *ch = ' ';
                }
            }
            i = end;
            continue;
        }
        i += 1;
    }
}

/// Onde um literal está: o argumento de que chamada, de que macro, ou de um atributo.
pub(crate) enum Ctx {
    Macro(String),
    Call(String),
    Attr,
    Other,
}

fn name_before(code: &[char], at: usize) -> Option<String> {
    let mut j = at;
    while j > 0 && code[j - 1].is_whitespace() {
        j -= 1;
    }
    let end = j;
    while j > 0 && is_ident(code[j - 1]) {
        j -= 1;
    }
    (j < end).then(|| code[j..end].iter().collect())
}

/// O parêntese (ou colchete) que ENVOLVE a posição `at`, e o nome que o abre.
pub(crate) fn enclosing(code: &[char], mask: &[bool], at: usize) -> Ctx {
    let mut depth = 0usize;
    let mut i = at;
    while i > 0 {
        i -= 1;
        if mask[i] {
            continue;
        }
        match code[i] {
            ')' | ']' | '}' => depth += 1,
            '(' | '[' | '{' if depth > 0 => depth -= 1,
            '(' => {
                let mut j = i;
                while j > 0 && code[j - 1].is_whitespace() {
                    j -= 1;
                }
                if j > 0 && code[j - 1] == '!' {
                    return name_before(code, j - 1).map_or(Ctx::Other, Ctx::Macro);
                }
                return name_before(code, j).map_or(Ctx::Other, Ctx::Call);
            }
            '[' => {
                let mut j = i;
                while j > 0 && code[j - 1].is_whitespace() {
                    j -= 1;
                }
                if j > 0 && code[j - 1] == '!' {
                    if j > 1 && code[j - 2] == '#' {
                        return Ctx::Attr;
                    }
                    return name_before(code, j - 1).map_or(Ctx::Other, Ctx::Macro);
                }
                return if j > 0 && code[j - 1] == '#' {
                    Ctx::Attr
                } else {
                    Ctx::Other
                };
            }
            '{' => return Ctx::Other,
            _ => {}
        }
    }
    Ctx::Other
}

/// O literal em `start..end` é um PADRÃO (`"a" =>`, `"a" | "b" =>`) ou o lado de uma COMPARAÇÃO
/// (`x == "a"`)? Nenhum dos dois pinta: um compara com texto que veio de outro sítio.
///
/// ⚠️ **O `|` que PRECEDE não decide sozinho** — `.map(|_| "Label")` é um fecho a DEVOLVER um
/// rótulo. Só a leitura para a frente (há um `=>` depois das alternativas?) separa os dois.
pub(crate) fn is_pattern_or_comparison(code: &[char], start: usize, end: usize) -> bool {
    let skip_ws = |mut j: usize| {
        while j < code.len() && code[j].is_whitespace() {
            j += 1;
        }
        j
    };
    let mut j = skip_ws(end);
    loop {
        if starts_with_at(code, j, "=>")
            || starts_with_at(code, j, "==")
            || starts_with_at(code, j, "!=")
        {
            return true;
        }
        if j < code.len() && code[j] == '|' && !starts_with_at(code, j, "||") {
            let k = skip_ws(j + 1);
            if let Some(e) = (k < code.len()).then(|| string_end(code, k)).flatten() {
                j = skip_ws(e);
                continue;
            }
        }
        break;
    }
    let mut k = start;
    while k > 0 && code[k - 1].is_whitespace() {
        k -= 1;
    }
    k >= 2 && matches!((code[k - 2], code[k - 1]), ('=', '=') | ('!', '='))
}
