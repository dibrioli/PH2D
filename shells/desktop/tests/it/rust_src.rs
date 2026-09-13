//! **O parser de FONTE partilhado pelos gates de texto do harness** — funções, corpos e contagens.
//!
//! Nasceu no `fn_loc_caps` (molde `ph2d-editor-core/tests/it/architecture_panel_loc_cap.rs`) e mudou-se
//! para cá quando o `frame_text` precisou da MESMA pergunta — *onde começa e acaba esta função?*. Dez
//! gates deste harness já tinham o seu `body_of`/`fn_body` escrito à mão; duas cópias de um parser
//! discordam no primeiro apóstrofo em prosa, e é isso que este existe para não repetir.
//!
//! ⚠️ **Consciente de comentários, strings (cruas ou não) e literais de char.** A versão ingénua do molde
//! contava mal uma função inteira por causa de um `doesn't` num comentário.

/// `(nome, LOC do corpo)` de cada função, do `{` ao `}` correspondente, saltando os módulos
/// `#[cfg(test)]` inteiros. Uma função aninhada conta dentro da de fora.
pub fn extract_fn_locs(src: &str) -> Vec<(String, usize)> {
    let stripped = strip_test_modules(src);
    let mut out = Vec::new();
    let mut i = 0;
    while i < stripped.len() {
        let Some((name, body_start)) = find_fn_opener(&stripped, i) else {
            break;
        };
        let Some(body_end) = find_matching_brace(&stripped, body_start) else {
            break;
        };
        out.push((name, stripped[body_start..=body_end].lines().count()));
        i = body_end + 1;
    }
    out
}

/// O CORPO (entre as chavetas, sem elas) da primeira `fn name` do texto — aninhada ou não.
pub fn fn_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let mut i = 0;
    while let Some((n, open)) = find_fn_opener(src, i) {
        let close = find_matching_brace(src, open)?;
        if n == name {
            return Some(&src[open + 1..close]);
        }
        i = open + 1;
    }
    None
}

/// Os NOMES de todas as funções do texto (aninhadas incluídas), pela ordem em que abrem.
pub fn fn_names(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some((n, open)) = find_fn_opener(src, i) {
        out.push(n);
        i = open + 1;
    }
    out
}

/// Do `{` em `open` ao `}` correspondente, ignorando chavetas em comentários, strings e chars.
pub fn find_matching_brace(src: &str, open: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => i = find_line_end(b, i),
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i = i + 2 + src[i + 2..].find("*/")? + 2;
            }
            b'r' if i + 1 < b.len() && matches!(b[i + 1], b'"' | b'#') => {
                match skip_raw_string(src, i) {
                    Some(next) => i = next,
                    None => i += 1,
                }
            }
            b'"' => i = skip_string(b, i)?,
            b'\'' => i = skip_char_or_lifetime(b, i),
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

fn skip_string(b: &[u8], from: usize) -> Option<usize> {
    let mut i = from + 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

fn skip_raw_string(src: &str, from: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut i = from + 1;
    let mut hashes = 0usize;
    while i < b.len() && b[i] == b'#' {
        hashes += 1;
        i += 1;
    }
    if i >= b.len() || b[i] != b'"' {
        return None;
    }
    i += 1;
    let mut terminator = String::with_capacity(hashes + 1);
    terminator.push('"');
    terminator.extend(std::iter::repeat_n('#', hashes));
    src[i..].find(&terminator).map(|r| i + r + terminator.len())
}

fn skip_char_or_lifetime(b: &[u8], from: usize) -> usize {
    let after_tick = from + 1;
    if after_tick < b.len() && b[after_tick] == b'\\' {
        let mut j = after_tick + 1;
        while j < b.len() && b[j] != b'\'' {
            j += 1;
        }
        return if j < b.len() { j + 1 } else { after_tick };
    }
    let mut j = after_tick;
    while j < b.len() && j < after_tick + 4 {
        if b[j] == b'\'' {
            return j + 1;
        }
        j += 1;
    }
    after_tick
}

fn find_fn_opener(src: &str, from: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        let line_start = i;
        let line_end = find_line_end(bytes, i);
        let line = &src[line_start..line_end];
        let trimmed = line.trim_start();
        if let Some(fn_kw_pos) = find_fn_keyword(trimmed) {
            let after_fn = trimmed[fn_kw_pos + 3..].trim_start();
            let name_end = after_fn
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(after_fn.len());
            if name_end == 0 {
                i = line_end + 1;
                continue;
            }
            let name = after_fn[..name_end].to_string();
            let scan_start = line_start
                + (line.len() - trimmed.len())
                + fn_kw_pos
                + 3
                + (trimmed[fn_kw_pos + 3..].len() - after_fn.len())
                + name_end;
            if let Some(b) = find_top_level_brace(bytes, scan_start.min(bytes.len())) {
                return Some((name, b));
            }
        }
        i = line_end + 1;
    }
    None
}

fn find_line_end(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// `fn ` no início da linha (depois de espaços e de palavras de visibilidade/qualificação).
fn find_fn_keyword(trimmed: &str) -> Option<usize> {
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        if &bytes[i..i + 3] == b"fn " {
            let prefix = trimmed[..i].trim();
            let permitted = prefix.is_empty()
                || prefix.starts_with("pub")
                || prefix
                    .split_whitespace()
                    .all(|w| matches!(w, "async" | "const" | "unsafe" | "extern"));
            if permitted {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// O `{` que abre o corpo, ao nível de topo de `(` `[` `<`; `None` se a assinatura acaba em `;`.
fn find_top_level_brace(bytes: &[u8], from: usize) -> Option<usize> {
    let (mut paren, mut bracket, mut angle) = (0i32, 0i32, 0i32);
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 && i > 0 && bytes[i - 1] != b'-' => angle -= 1,
            b'{' if paren == 0 && bracket == 0 && angle == 0 => return Some(i),
            b';' if paren == 0 && bracket == 0 && angle == 0 => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Tira os blocos `#[cfg(test)]` (o módulo inteiro) antes de medir.
fn strip_test_modules(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let Some(r) = src[i..].find("#[cfg(test)]") else {
            out.push_str(&src[i..]);
            break;
        };
        let attr = i + r;
        out.push_str(&src[i..attr]);
        let mut j = attr + "#[cfg(test)]".len();
        while j < bytes.len() && bytes[j] != b'{' && bytes[j] != b';' {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] == b';' {
            // `#[cfg(test)] mod x;` — um módulo noutro ficheiro; nada a saltar aqui.
            i = attr + "#[cfg(test)]".len();
            out.push_str("#[cfg(test)]");
            continue;
        }
        let Some(k) = find_matching_brace(src, j) else {
            break;
        };
        i = k + 1;
    }
    out
}

/// Os filhos que um ficheiro da shell declara por `#[path = "…"]`, recursivamente e concatenados — o texto que um gate
/// de CAMINHO deixa de ver quando o assunto do ficheiro se parte em filhos (`line/render-bodies`: o `snapshots.rs` e o
/// `sim_extract.rs`). ⚠️ Uma AUSÊNCIA lida só no pai fica VERDE sobre o código que se mudou para um filho, e uma
/// contagem exacta fica cega a uma segunda ocorrência escrita num deles (auditoria do fecho da linha).
pub fn path_children(path: &std::path::Path) -> String {
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let dir = path.parent().expect("um ficheiro da shell mora numa pasta");
    let mut out = String::new();
    for line in src.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("#[path = \"")
            && let Some(end) = rest.find("\"]")
        {
            out.push_str(&with_path_children(&dir.join(&rest[..end])));
            out.push('\n');
        }
    }
    out
}

/// O ficheiro e os filhos `#[path]` dele (ver [`path_children`]).
pub fn with_path_children(path: &std::path::Path) -> String {
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    format!("{src}\n{}", path_children(path))
}

/// **A lente dos filhos lê os filhos, e só eles:** o `snapshots.rs` parte-se em filhos `#[path]`, e o texto de cada um
/// tem de chegar à lente — senão uma ausência medida por ela fica verde sobre nada.
#[test]
fn the_path_lens_reads_the_children() {
    let pai = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/snapshots.rs");
    let filhos = path_children(&pai);
    for marca in ["fn sprite_info(", "fn identity(", "fn global_view("] {
        assert!(
            filhos.contains(marca),
            "a lente dos filhos não chega a `{marca}` — um filho `#[path]` ficou de fora"
        );
    }
    assert!(
        !filhos.contains("fn publish_hierarchy("),
        "o pai entrou na lente dos FILHOS"
    );
    assert!(
        with_path_children(&pai).contains("fn publish_hierarchy("),
        "a lente inteira perdeu o pai"
    );
}
