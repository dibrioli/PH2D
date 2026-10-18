//! A leitura de uma mensagem crua: espaços, prefixo de posição, aspas e tipos enormes.

/// Os espaços seguidos (e as quebras de linha do `1000`) viram um só.
pub(crate) fn normalize(raw: &str) -> String {
    let one = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    strip_position(&one).to_string()
}

/// `bob.luau:12: attempt…` → `attempt…` (a posição de outro chunk, como o motor a dá).
fn strip_position(m: &str) -> &str {
    let head = &m[..m.len().min(120)];
    for (i, _) in head.match_indices(':') {
        let rest = &m[i + 1..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && rest[digits..].starts_with(": ") {
            return &rest[digits + 2..];
        }
    }
    m
}

/// O texto depois de `pat`.
pub(crate) fn after<'a>(m: &'a str, pat: &str) -> Option<&'a str> {
    m.find(pat).map(|i| &m[i + pat.len()..])
}

/// O texto entre `'…'` logo a seguir a `pat` (`Key 'x'` → `x`).
pub(crate) fn quoted_after<'a>(m: &'a str, pat: &str) -> Option<&'a str> {
    let rest = after(m, pat)?.strip_prefix('\'')?;
    rest.find('\'').map(|e| &rest[..e])
}

/// O primeiro `'…'` da mensagem.
pub(crate) fn first_quoted(m: &str) -> Option<&str> {
    let s = m.find('\'')? + 1;
    m[s..].find('\'').map(|e| &m[s..s + e])
}

/// Os algarismos logo a seguir a `pat`.
pub(crate) fn number_after(m: &str, pat: &str) -> Option<String> {
    let rest = after(m, pat)?;
    let n: String = rest.chars().take_while(char::is_ascii_digit).collect();
    (!n.is_empty()).then_some(n)
}

/// A palavra logo a seguir a `pat` (até espaço, `)` ou `,`).
pub(crate) fn word_after(m: &str, pat: &str) -> Option<String> {
    let rest = after(m, pat)?;
    let w: String = rest
        .chars()
        .take_while(|c| !matches!(c, ' ' | ')' | ','))
        .collect();
    (!w.is_empty()).then_some(w)
}

/// O token depois de `got ` (`'='` → `=`; `<eof>` → `None`).
pub(crate) fn got(m: &str) -> Option<&str> {
    quoted_after(m, "got ")
}

/// Um tipo legível numa linha: a tabela inteira do `ph2d` (567 caracteres, `*TRUNCATED*`) não cabe.
pub(crate) fn short_type(t: &str) -> String {
    let t = t.trim_end_matches("*TRUNCATED*").trim();
    if t.starts_with('{') {
        return ph2d_i18n::tr("code.explain.type_table").into();
    }
    if t.contains("->") {
        return ph2d_i18n::tr("code.explain.type_function").into();
    }
    if t.chars().count() > 32 {
        return t.chars().take(31).chain(['…']).collect();
    }
    t.to_string()
}

/// O tipo que o mlua converte: os números da máquina são todos «number», `String` é «string».
pub(crate) fn lua_type(t: &str) -> String {
    match t {
        "String" => "string".into(),
        "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64"
        | "usize" => "number".into(),
        t => t.into(),
    }
}

/// O fecho do que abre: `)` ← `(`.
pub(crate) fn opener(close: &str) -> &'static str {
    match close {
        ")" => "(",
        "]" => "[",
        _ => "{",
    }
}
