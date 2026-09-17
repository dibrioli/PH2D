//! ⭐ **O QUE UM FICHEIRO DA SHELL DIZ, lido através da tabela de strings.**
//!
//! Desde 2026-09-16 os avisos da shell moram em `ph2d-i18n/src/shell*.rs` e o fonte guarda a CHAVE
//! (`ph2d_i18n::tr("shell.…")`). Os gates que liam a FRASE no fonte passaram a lê-la por aqui: a
//! chave tem de estar no código **e** o texto dela tem de dizer a frase — as duas metades, porque
//! uma chave certa com o texto errado é o defeito que a frase existia para impedir.

/// As chaves que o texto cita (`tr("…")`, `tr_with("…"`, `TextKey::new("…")`), com a posição.
pub(crate) fn keys_in(src: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    // ⚠️ O `rustfmt` parte `tr(\n    "…")` em linhas: entre o parêntese e a aspa pode haver espaço.
    for pat in ["tr(", "tr_with(", "TextKey::new("] {
        let mut from = 0;
        while let Some(i) = src[from..].find(pat) {
            let open = from + i + pat.len();
            let lead = src[open..].len() - src[open..].trim_start().len();
            from = open;
            if !src[open + lead..].starts_with('"') {
                continue;
            }
            let start = open + lead + 1;
            let Some(len) = src[start..].find('"') else {
                break;
            };
            out.push((from - pat.len(), &src[start..start + len]));
            from = start + len;
        }
    }
    out.sort_unstable();
    out
}

/// O fonte seguido do texto de cada chave que ele cita — um `contains(frase)` sobre isto responde
/// «este código mostra esta frase?».
pub(crate) fn with_texts(src: &str) -> String {
    let mut out = src.to_string();
    for (_, k) in keys_in(src) {
        out.push('\n');
        out.push_str(ph2d_i18n::tr(k));
    }
    out
}

/// A posição (no fonte) da primeira chave cujo texto contém `frase` — para os gates que medem uma
/// ORDEM (onde o aviso é dado), e não só a presença.
pub(crate) fn key_pos(src: &str, frase: &str) -> Option<usize> {
    keys_in(src)
        .into_iter()
        .find(|(_, k)| ph2d_i18n::tr(k).contains(frase))
        .map(|(i, _)| i)
}
