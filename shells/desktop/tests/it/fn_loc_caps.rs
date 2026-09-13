//! **Tecto por FUNÇÃO da shell** — a catraca NUMERADA da OBRA 2 da `line/render-loop` (2026-09-12).
//!
//! O `file_loc_caps` mede FICHEIROS, e um ficheiro pode caber no tecto dele com uma função só de
//! 13 566 linhas dentro — era o `run_render_frame`, o quadro inteiro. A OBRA 2 parte-o em FASES,
//! chamadas pela mesma ordem, e este gate é o que impede o corte de voltar a colar: toda função da
//! shell cabe em [`FN_LOC_CAP`], SALVO as entradas NUMERADAS de [`FN_OVERAGE_OK`], cada uma com o
//! tamanho MEDIDO, que só desce.
//!
//! Molde: `ph2d-editor-core/tests/it/architecture_panel_loc_cap.rs` (os painéis), com o mesmo parser
//! consciente de comentários, strings e literais de char — a versão ingénua dele contava mal uma
//! função inteira por causa de um apóstrofo em prosa.
//!
//! As DUAS metades da catraca:
//! - **cresceu**: uma função acima do tecto dela ⇒ o corte por RESPONSABILIDADE, nunca o número;
//! - **o tecto ficou para trás**: a função já não existe, já cabe no cap, ou mede menos do que a
//!   entrada diz ⇒ a entrada desce (ou sai) no MESMO commit. Folga ZERO: a medida é do parser, e
//!   um corte que a mude muda-a por uma razão.
//!
//! ⚠️ **Piso de população**: uma varredura partida acharia zero funções, e zero ofensores lê-se como
//! aprovado. O gate tem de ver centenas de funções e, pelo nome, o próprio `run_render_frame`.

use std::fs;
use std::path::{Path, PathBuf};

/// O tecto de uma função da shell — o mesmo dos painéis.
const FN_LOC_CAP: usize = 200;

/// As funções que passam do tecto: `(ficheiro relativo a src/, função, LOC medido, razão)`.
/// **Só descem.** Medidas pelo parser deste gate no nascimento dele (2026-09-12).
const NASCEU: &str = "medido no nascimento do gate (2026-09-12); nenhuma linha o partiu ainda";
const FN_OVERAGE_OK: &[(&str, &str, usize, &str)] = &[
    ("blend_smoke.rs", "blend_smoke", 205, NASCEU),
    ("build_smoke.rs", "build_smoke", 416, NASCEU),
    ("build_smoke_router.rs", "route", 441, NASCEU),
    ("envelope_smoke.rs", "frame", 247, NASCEU),
    ("hero_intents/hierarchy.rs", "drain_reparent", 297, NASCEU),
    (
        "hero_intents/image_edit/bgremoval.rs",
        "drain_bgremoval",
        237,
        NASCEU,
    ),
    (
        "hero_intents/image_edit/equalize_sizes.rs",
        "drain_equalize_sizes",
        204,
        NASCEU,
    ),
    (
        "hero_intents/sprite_merge.rs",
        "drain_merge_sprites",
        439,
        NASCEU,
    ),
    ("init.rs", "build_initial_state", 499, NASCEU),
    (
        "input_dispatch.rs",
        "on_cursor_moved",
        375,
        "o `input_dispatch.rs` fica FORA da `line/render-loop` (a OBRA 2 parte só o quadro)",
    ),
    (
        "input_dispatch.rs",
        "on_mouse_input",
        3102,
        "o `input_dispatch.rs` fica FORA da `line/render-loop` (a OBRA 2 parte só o quadro)",
    ),
    (
        "input_dispatch/gizmo_drag.rs",
        "advance_gizmo_drag",
        708,
        NASCEU,
    ),
    ("input_dispatch/keyboard.rs", "key_input", 544, NASCEU),
    ("input_handlers.rs", "handle_editor_key", 358, NASCEU),
    ("layout_live.rs", "lay_out", 206, NASCEU),
    ("main.rs", "new", 263, NASCEU),
    ("project_load.rs", "project_load_from", 484, NASCEU),
    ("render_loop/autokey_pass.rs", "apply_samples", 317, NASCEU),
    ("render_loop/bgremoval_preview.rs", "dispatch", 333, NASCEU),
    ("render_loop/hierarchy.rs", "dispatch", 414, NASCEU),
    ("render_loop/image_edit.rs", "dispatch", 483, NASCEU),
    ("render_loop/inspector_commits.rs", "dispatch", 383, NASCEU),
    (
        "render_loop/mod.rs",
        "run_render_frame",
        13566,
        "o QUADRO inteiro numa função; a OBRA 2 da `line/render-loop` parte-o em fases chamadas pela \
         mesma ordem, e cada fase que sai baixa este número no mesmo commit",
    ),
    ("render_loop/present.rs", "run_present_phase", 484, NASCEU),
    (
        "render_loop/push_look_probe.rs",
        "probe_push_render_and_look",
        314,
        NASCEU,
    ),
    ("render_loop/sim_extract.rs", "run", 491, NASCEU),
    ("render_loop/snapshots.rs", "publish", 1068, NASCEU),
];

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn collect_rs(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(collect_rs(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn rel(p: &Path) -> String {
    p.strip_prefix(src_root())
        .map(|r| r.to_string_lossy().into_owned())
        .unwrap_or_else(|_| p.display().to_string())
}

/// `true` para um ficheiro que é SÓ teste — o irmão `*_tests.rs` que um `#[cfg(test)] mod` puxa por
/// `#[path]`. O molde dos painéis também não mede testes: um `#[test]` longo é uma tabela de casos,
/// não uma função de produto a partir.
fn is_test_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "tests.rs" || n.ends_with("_tests.rs"))
}

/// `(ficheiro, função, LOC)` de toda função de PRODUTO da shell (fora dos `#[cfg(test)]` e dos
/// ficheiros de teste).
fn measured() -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    for path in collect_rs(&src_root()) {
        if is_test_file(&path) {
            continue;
        }
        let Ok(body) = fs::read_to_string(&path) else {
            continue;
        };
        let r = rel(&path);
        for (name, loc) in extract_fn_locs(&body) {
            out.push((r.clone(), name, loc));
        }
    }
    // O piso é SANIDADE, não catraca: medido 1 865 funções de produto em 2026-09-12. Uma varredura
    // partida lê zero ou poucas dezenas; uma W2 inteira a sair da shell não chega a tirar um quinto.
    assert!(
        out.len() > 1_500,
        "o parser achou só {} funções na shell — a varredura partiu-se, e zero ofensores leria-se \
         como aprovado",
        out.len()
    );
    assert!(
        out.iter()
            .any(|(f, n, _)| f == "render_loop/mod.rs" && n == "run_render_frame"),
        "o parser não vê o `run_render_frame` — ou ele mudou de casa (mova este piso com ele), ou \
         o parser partiu-se"
    );
    out
}

/// **Cresceu?**
#[test]
fn shell_functions_respect_their_ceiling() {
    let mut over = Vec::new();
    for (file, name, loc) in measured() {
        if loc <= FN_LOC_CAP {
            continue;
        }
        let tecto = FN_OVERAGE_OK
            .iter()
            .find(|(f, n, _, _)| *f == file && *n == name)
            .map_or(FN_LOC_CAP, |(_, _, t, _)| *t);
        if loc > tecto {
            over.push(format!("{file}::{name} — {loc} LOC (tecto {tecto})"));
        }
    }
    over.sort();
    assert!(
        over.is_empty(),
        "funções da shell acima do tecto:\n  {}\n\ncura: partir por RESPONSABILIDADE (uma fase, um \
         assunto). ⛔ Nunca subir o número de `FN_OVERAGE_OK`: ele só desce.",
        over.join("\n  ")
    );
}

/// **O tecto ficou para trás?**
#[test]
fn fn_overage_allowlist_has_no_stale_entries() {
    let all = measured();
    let mut stale = Vec::new();
    for (file, name, tecto, razao) in FN_OVERAGE_OK {
        assert!(!razao.trim().is_empty(), "`{file}::{name}` sem razão");
        match all
            .iter()
            .find(|(f, n, _)| f == file && n == name)
            .map(|(_, _, loc)| *loc)
        {
            None => stale.push(format!("{file}::{name} — a função já não existe")),
            Some(loc) if loc <= FN_LOC_CAP => stale.push(format!(
                "{file}::{name} — tem {loc} LOC, já cabe no cap de {FN_LOC_CAP}: apague a entrada"
            )),
            Some(loc) if loc < *tecto => stale.push(format!(
                "{file}::{name} — tem {loc} LOC e a entrada diz {tecto}: baixe-a para {loc}"
            )),
            Some(_) => {}
        }
    }
    assert!(
        stale.is_empty(),
        "entradas de FN_OVERAGE_OK que já não descrevem nada:\n  {}",
        stale.join("\n  ")
    );
}

/// `(nome, LOC do corpo)` de cada função, do `{` ao `}` correspondente, saltando os módulos
/// `#[cfg(test)]` inteiros. Uma função aninhada conta dentro da de fora.
fn extract_fn_locs(src: &str) -> Vec<(String, usize)> {
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

/// Do `{` em `open` ao `}` correspondente, ignorando chavetas em comentários, strings (cruas ou
/// não) e literais de char.
fn find_matching_brace(src: &str, open: usize) -> Option<usize> {
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
