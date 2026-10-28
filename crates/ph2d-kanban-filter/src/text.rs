//! **Texto comparável**: minúsculas Unicode, sem acentos (NFD sem marcas combinantes), espaços
//! colapsados. «Ação», «ACAO» e «acao» são a mesma agulha.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// A forma comparável de `s` (estável: `normalize(normalize(s)) == normalize(s)`).
#[must_use]
pub fn normalize(s: &str) -> String {
    let folded: String = s
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parte uma consulta em pedaços: `"frase entre aspas"` fica inteira, o resto parte-se nos
/// espaços. Um `-` ou um `operador:` colado às aspas fica no pedaço (`-"a b"`, `list:"To Do"`).
pub(crate) fn tokens(q: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in q.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                cur.push(c);
            }
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Tira as aspas de um pedaço.
pub(crate) fn unquote(s: &str) -> &str {
    s.strip_prefix('"')
        .map(|s| s.strip_suffix('"').unwrap_or(s))
        .unwrap_or(s)
}

/// As agulhas de uma palavra-chave do filtro, já normalizadas (o Trello: palavras soltas casam
/// QUALQUER uma; entre aspas, a frase exacta).
pub(crate) fn needles(q: &str) -> Vec<String> {
    tokens(q)
        .iter()
        .map(|t| normalize(unquote(t)))
        .filter(|t| !t.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_folds_case_accents_and_spaces() {
        assert_eq!(normalize("  Ação   RÁPIDA "), "acao rapida");
        assert_eq!(normalize("Ñandú ÇÉ"), "nandu ce");
        assert_eq!(normalize("İstanbul ß"), normalize(&normalize("İstanbul ß")));
    }

    #[test]
    fn needles_keep_a_quoted_phrase_whole() {
        assert_eq!(
            needles(r#"shot "Cena Final" luz"#),
            ["shot", "cena final", "luz"]
        );
        assert_eq!(
            tokens(r#"-"a b" list:"To Do""#),
            [r#"-"a b""#, r#"list:"To Do""#]
        );
    }
}
