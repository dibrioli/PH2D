//! Wave 10 / Etapa 5.2 — cap `ph2d-panel-*/src/**` files at 600 LOC
//! and individual functions at 200 LOC.
//!
//! Panels are widget orchestrators: each `paint*.rs` should be a
//! readable composition of section-painters + canonical widget
//! primitives, not a 600-LOC monolith. A panel file that grows past
//! 600 LOC is a candidate for splitting into sibling files
//! (`paint.rs` + `paint_sections.rs` + `paint_helpers.rs`).
//!
//! The 200-LOC/function cap exists for the same reason: a `paint()`
//! body over 200 lines reads as a mega-function and resists
//! per-section review.
//!
//! Excludes `tests/` siblings (test files are allowed to be longer)
//! and `ph2d-panel-registry-init` (codegen target, not a panel surface).

use std::fs;
use std::path::{Path, PathBuf};

const PANEL_FILE_LOC_CAP: usize = 600;
const PANEL_FN_LOC_CAP: usize = 200;

/// Per-file overage allowance — frozen technical debt. Each entry:
/// (relative path under `crates/`, allowed LOC, why). Driving every
/// entry to zero is the goal; new entries require Coord-A sign-off.
const FILE_OVERAGE_OK: &[(&str, usize, &str)] = &[
    // Enio 2026-05-26: paint_sections.rs cresceu com Dither Strength +
    // Dither Grain sliders (2 rows novas dentro de
    // paint_posterize_quantize_section). Split em paint_helpers.rs é
    // follow-up; mantém o cap visível enquanto isso.
    (
        "ph2d-panel-color-equalization/src/paint_sections.rs",
        660,
        "Enio 2026-05-26 dither strength+grain rows — split deferred",
    ),
    // Coord 2026-06-04 ship-prep: Painter W4 adjustment panels (Curves/Levels/
    // B&W/Selective Color/Gradient Map) grew this orchestrator. Per-adjustment
    // sibling split is a Painter-impl follow-up; frozen at the ship-canonical 829.
    (
        "ph2d-panel-painter-layers/src/paint_adjust.rs",
        829,
        "Painter W4 bespoke adjustment panels — per-adjustment split deferred (Painter impl follow-up)",
    ),
    // Deform Wave 1: the monolithic Click-dispatch match gained ONE predicate call
    // (`is_deform_click`) to forward the Deform panel's clicks. The file was already at the 600 cap;
    // splitting the giant dispatch match is a separate refactor. Frozen at 601.
    (
        "ph2d-panel-painter-layers/src/event.rs",
        601,
        "Deform Wave 1 added one is_deform_click() call to the at-cap dispatch match; match split deferred",
    ),
];

/// Per-function overage allowance. Each entry:
/// (relative file path, function name, allowed LOC, why).
const FN_OVERAGE_OK: &[(&str, &str, usize, &str)] = &[
    // ── RE-BASELINE 2026-07-10 (the "deliberate foundational pass" this gate
    // asked for). Until today the brace walker toggled a char-literal flag on
    // every `'`, so a prose apostrophe in a `//` comment ("doesn't") or a
    // lifetime tick (`&'a`) closed a function early and UNDER-counted it. Every
    // number below is now a real measurement:
    //   · 3 entries were deleted — their fns are, and were, under the cap
    //     (grid-snap populate = 126, inspector color_tint = 124,
    //      painter-layers paint_adjustment_params = 54).
    //   · 2 entries were lying LOW and are corrected UP to the truth
    //     (inspector apply_event_impl 353 → 477; paint_transform_section 212 → 281).
    //   · 8 fns were fully masked and appear here for the first time.
    // This is a correction of the MEASUREMENT, never a licence to grow: the
    // numbers may shrink, never rise, and the honest split (per-section helpers
    // threading `y: f32`) is now unblocked — it is paint/dispatch code with no
    // unit coverage, so each split lands with its own smoke, one panel at a time.
    // ──────────────────────────────────────────────────────────────────────────
    // ph2d-panel-color-equalization populate: 200→203 after `cargo fmt --all`
    // re-flowed long lines (solo Coord 2026-05-29). A data-spec populate; a
    // split into a number_specs helper is the same deferred follow-up.
    (
        "ph2d-panel-color-equalization/src/populate.rs",
        "populate",
        203,
        "fmt --all re-flow pushed it 200→203; data-spec populate, split deferred",
    ),
    // Wave 10 / Etapa 5.2: long paint orchestrators that grew with the panel's
    // feature set. Splitting into per-section helpers is a follow-up Etapa (one
    // panel at a time, with smoke validation).
    (
        "ph2d-panel-hierarchy/src/paint.rs",
        "paint_hierarchy_body",
        364,
        "Wave 10 paint orchestrator; ratcheted 384->364 quando a wave do hover lhe acrescentou UMA linha e o tecto foi pago por extracção em vez de tolerância: a caixa de renomear saiu para `paint_rename_input`. As tolerâncias encolhem, nunca crescem.",
    ),
    (
        "ph2d-panel-hierarchy/src/row.rs",
        "paint_hierarchy_row",
        267,
        "row painter (icons + twirl + rename + companions); 291 -> 281 em 2026-08-22 (a tabela de TOM dos selos saiu para `badge_tone`), 281 -> 267 em 2026-08-23 quando o realce de proveniencia lhe acrescentou linhas e o tecto foi pago por EXTRACCAO: o fundo da linha saiu para `paint_row_background`. As tolerancias encolhem, nunca crescem — e uma feature nova paga-as com um corte, nao com um numero maior.",
    ),
    // ⚠️ `ph2d-panel-hierarchy/src/event.rs::apply_event` ESTEVE aqui, tolerado a 216 — e a
    // entrada foi REMOVIDA em 2026-08-19, não subida para 219. O "Pack into Sheet" ia
    // acrescentar-lhe quatro linhas, e a tolerância do vizinho de cima diz, pela mão de quem a
    // pagou, *«as tolerâncias encolhem, nunca crescem»*. O bloco do menu de contexto saiu para
    // `try_context_menu_row`, a mãe caiu para dentro do teto, e a tolerância deixou de ter
    // objeto. *A cura de um teto estourado é o corte; subir o número é adiar com juros.*
    // The inspector is the worst offender and the reason the split was blocked:
    // the parser under-counted its dispatcher by 124 LOC.
    (
        "ph2d-panel-inspector/src/event.rs",
        "apply_event_impl",
        292,
        "was frozen at a mis-measured 353; truly 477, ratcheted to 452 when W3's two physics colour dots pushed it over and the colour-dot arm moved out to `section_color_click`; ratcheted again to 442 when W-Signal's text row pushed it to 470 and the two TEXT arms moved out to `section_text_changed`; ratcheted to 410 em 2026-08-20 quando o par de PRECISAO (plano `docs/Sprite_projeto/18` W5) o empurrou e os DOIS pares da seccao Render Source (Strategy + Format) sairam para `event_precision.rs` — irmao e nao funcao irma, porque o ficheiro tambem estava a rocar o teto de 600 e extrair no mesmo ficheiro curaria um teto e estouraria o outro. A catraca so' desce: um cluster de cada vez. Ratcheted a 399 em 2026-08-21 quando a linha `Emissive` (plano `docs/Sprite_projeto/18` W8) o levou a 433 e os DOIS sliders-com-chip da sprite (Opacidade + Emissive) sairam juntos para `event_sprite_value.rs` -- levar so' o novo devolveria o numero a 410 exactos, e ficar no mesmo sitio nao e' encolher. Ratcheted a 385 em 2026-08-21 (auditoria `docs/Sprite_projeto/20` §3): dar fan-out de BulkSelect a` caixa `Visible` do topo -- ela editava so' a primaria enquanto a §8 logo abaixo editava toda a seleccao -- exigiu ganhar antes a afordancia de divergencia, e o braco cresceu; ele saiu inteiro para `visibility_toggle`, funcao IRMA no mesmo ficheiro (que estava a 542/600, com folga, ao contrario do caso do par de PRECISAO). Ratcheted a 292 em 2026-08-21 quando a §5 9-SLICE (spec Sprite 03 §3.5) o levou a 389 e o cluster da REGIAO + ORIGEM (region enabled/filter-clip, RegionX/Y/W/H, Centered, OffsetX/Y -- sete bracos, uma lei so': despacho POR EIXO) saiu para `event_sprite_geometry.rs`. Levar so' as cinco linhas novas deixaria o numero onde estava, e ficar no mesmo sitio nao e' encolher. Sequence of independent first-match-wins `if let WidgetEvent::…` blocks",
    ),
    (
        "ph2d-panel-inspector/src/paint.rs",
        "paint_inspector",
        292,
        "§0-§14 section orchestrator; ratcheted 431->424 when §11 Physics Body forced the honest split (section frame + phase B moved to paint_frame.rs). §12 Physics Joint (W3) added ~22 LOC and paid for them by moving the whole physics FAMILY out too — `paint_frame::paint_physics_sections` — landing back on exactly 424. ⚠️ Then §14 Platform Player (W5) pushed it to 427 and this entry did EXACTLY what its own last sentence promised — *the next section splits again*: the note distribution (a self-contained block that looks at no section at all) moved out to `paint_frame::split_notes`, landing at 414. Allowances shrink; they never grow. ⚠️ Ratcheted a 387 em 2026-08-21 quando a §5 9-Slice o empurrou a 436: as QUATRO seccoes compartilhadas (§5 Slice, §7 Ordering, §9 Sampling, §10 Blend) sairam juntas para `paint_frame::paint_shared_sections`, como a familia da fisica antes delas. Andam juntas porque partilham a mesma porta -- qualquer entidade com Transform -- e porque os quatro slots de nota (6..9) ficam obviamente distintos lado a lado. Ratcheted a 380 em 2026-08-21 quando a §12 Sockets/Anchors o empurrou a 403: as TRES seccoes da SPRITE (§3 Render Source, §6 Color & Tint, §4 Sprite Sheet) sairam para `paint_frame_shared::paint_sprite_sections` -- andam juntas porque partilham a mesma porta, so' existem se houver sprite -- e as notas finais, que nao olham para seccao nenhuma, para `paint_frame::paint_trailing_notes`. Ratcheted a 348 em 2026-08-23 quando o seletor «Rides Parent Anchor» da §12 (ADR-0072 §2.6) o empurrou a 403: os TRES POPOVERS DIFERIDOS (§9 Sampling, §7 Sorting Layer, §12 Mount) sairam juntos para `paint_frame_shared::paint_deferred_popovers`. Andam juntos por uma LEI e nao por vizinhanca -- um popover aberto tem de sair da ordem em que a sua seccao foi pintada, senao a seccao seguinte desenha-lhe por cima -- e levar so' o novo devolveria o numero a 380 exactos, Ratcheted a 292 em 2026-08-23 pela §11 Animation, e em TRES passos: o par de seccoes com ESTADO DE PAINEL (§11 + §12, as unicas cuja pintura depende de que LINHA esta' aberta) saiu para `paint_frame_shared::paint_stateful_sections`; a leitura dos treze snapshots + o `any_section` viraram `paint_frame::LiveSnapshots::fetch`. ⚠️ Os dois juntos devolveram o numero a 348 EXACTOS -- a armadilha que esta propria nota descreve, e que so' se ve' MEDINDO --, entao saiu um TERCEIRO cluster: o cabecalho (titulo, subtitulo, fechar, divisor) para `paint_head.rs`, que nao e' orquestracao nenhuma. 292.",
    ),
    (
        "ph2d-panel-inspector/src/sections/transform.rs",
        "paint_transform_section",
        268,
        "was frozen at a mis-measured 212; truly 281 → 279 quando o botão de reset passou a ler o \
         par `(estado, t)` do store numa chamada em vez de três linhas → 277 → 268 quando a F4b \
         a fez cruzar e a geometria partilhada por TODA row (a largura de um chip e o empilhar da \
         seção) saiu para `chip_metrics`. Per-row split segue diferido (needs smoke)",
    ),
    // (A tolerancia do inspector `sync_sprite_fields` (202) esta' GONE: a linha `Emissive`
    // (plano `docs/Sprite_projeto/18` W8) empurrou-a para 223, e os DOIS sliders-com-chip da sprite
    // sairam para `sync_sprite_value.rs` -- 179 LOC, debaixo do teto de 200, tolerancia nenhuma
    // precisa. Que e' exactamente para isto que o teto serve, e o que o "split is mechanical" de
    // 2026-07-10 tinha adiado.)
    // (The painter-layers `apply_event_impl` allowance is GONE: adding the per-layer Impasto rows
    // pushed it over its 281, so the `ValueChanged` arm was extracted whole into `route_value_changed`
    // — 181 LOC, under the 200 cap, no allowance needed. Which is exactly what the cap is for.)
    (
        "ph2d-panel-painter-layers/src/paint.rs",
        "paint",
        267,
        "unmasked by the 2026-07-10 parser fix; layer-stack paint orchestrator, per-section split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-equalize-sizes/src/paint.rs",
        "paint_body_sections",
        237,
        "unmasked by the 2026-07-10 parser fix; ratcheted 249 -> 237 em 2026-08-20 quando a \
         correccao do layout do modo Fixed o empurrou para 253 e as duas linhas de ACCAO sairam \
         para o IRMAO `paint_actions.rs`. ⚠️ A primeira tentativa extraiu-as no MESMO ficheiro: \
         curou este tecto e empurrou o do FICHEIRO para 608 contra 600 — os dois tetos medem \
         grandezas diferentes, e o corte que cura ambos e' para o irmao. Split por seccao continua \
         diferido (needs smoke)",
    ),
    (
        "ph2d-panel-audio-mixer/src/paint.rs",
        "paint",
        212,
        "unmasked by the 2026-07-10 parser fix; a fileira de strips SAIU em 2026-08-15 (222 -> 212, \
         medido DEPOIS do rustfmt, que re-expande a chamada) \
         quando a fiacao do store lhe custou uma linha — as tolerancias encolhem, nunca crescem",
    ),
];

#[test]
fn panel_files_under_loc_cap() {
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let panel_dirs = collect_panel_dirs(&crates_root);
    let mut offenders: Vec<(String, usize)> = Vec::new();

    for panel_dir in &panel_dirs {
        let src = panel_dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let crate_name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        visit_files(&src, &mut |path| {
            let body = fs::read_to_string(path).expect("read panel file");
            let loc = body.lines().count();
            if loc > PANEL_FILE_LOC_CAP {
                let rel = path
                    .strip_prefix(panel_dir)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| path.display().to_string());
                let key = format!("{crate_name}/{rel}");
                let allowed = FILE_OVERAGE_OK
                    .iter()
                    .find(|(k, _, _)| *k == key)
                    .map(|(_, n, _)| *n)
                    .unwrap_or(PANEL_FILE_LOC_CAP);
                if loc > allowed {
                    offenders.push((key, loc));
                }
            }
        });
    }

    offenders.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    assert!(
        offenders.is_empty(),
        "panel-* files over {PANEL_FILE_LOC_CAP}-LOC cap:\n  {}\n\
         fix: split the panel paint orchestrator into sibling files \
         (`paint.rs` + `paint_sections.rs` + `paint_helpers.rs`), or \
         add an entry to FILE_OVERAGE_OK in this test with a 1-line \
         justification.",
        offenders
            .iter()
            .map(|(p, n)| format!("{p} ({n} LOC)"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn panel_functions_under_loc_cap() {
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let panel_dirs = collect_panel_dirs(&crates_root);
    let mut offenders: Vec<(String, String, usize)> = Vec::new();

    for panel_dir in &panel_dirs {
        let src = panel_dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let crate_name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        visit_files(&src, &mut |path| {
            let body = fs::read_to_string(path).expect("read panel file");
            let rel = path
                .strip_prefix(panel_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.display().to_string());
            let key = format!("{crate_name}/{rel}");
            for (fn_name, loc) in extract_fn_locs(&body) {
                if loc <= PANEL_FN_LOC_CAP {
                    continue;
                }
                let allowed = FN_OVERAGE_OK
                    .iter()
                    .find(|(k, f, _, _)| *k == key && *f == fn_name)
                    .map(|(_, _, n, _)| *n)
                    .unwrap_or(PANEL_FN_LOC_CAP);
                if loc > allowed {
                    offenders.push((key.clone(), fn_name, loc));
                }
            }
        });
    }

    offenders.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));
    assert!(
        offenders.is_empty(),
        "panel-* fn over {PANEL_FN_LOC_CAP}-LOC cap:\n  {}\n\
         fix: split the body into per-section helpers (each helper takes \
         the per-frame mutables + `y: f32` in and returns `y: f32` out), \
         or add an entry to FN_OVERAGE_OK with justification.",
        offenders
            .iter()
            .map(|(p, f, n)| format!("{p}::{f} ({n} LOC)"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Guard the allowance list itself. An entry whose function has been split
/// (or deleted) below the cap is dead weight: it silently re-permits the
/// overage if the function ever grows back. The 2026-07-10 re-baseline found
/// three such fossils — `grid-snap::populate` (really 126 LOC, frozen at 235),
/// `inspector::paint_color_tint_section` (124, frozen at 289) and
/// `painter-layers::paint_adjustment_params` (54, frozen at 227) — each one a
/// standing licence to triple in size unnoticed. Mirrors the same guard on
/// `architecture_workspace_file_loc_cap`.
#[test]
fn fn_overage_allowlist_has_no_stale_entries() {
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut measured: Vec<(String, String, usize)> = Vec::new();
    for panel_dir in collect_panel_dirs(&crates_root) {
        let src = panel_dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let crate_name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        visit_files(&src, &mut |path| {
            let body = fs::read_to_string(path).expect("read panel file");
            let rel = path
                .strip_prefix(&panel_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.display().to_string());
            let key = format!("{crate_name}/{rel}");
            for (fn_name, loc) in extract_fn_locs(&body) {
                measured.push((key.clone(), fn_name, loc));
            }
        });
    }

    let mut stale: Vec<String> = Vec::new();
    for (key, fn_name, allowed, _) in FN_OVERAGE_OK {
        match measured
            .iter()
            .find(|(k, f, _)| k == key && f == fn_name)
            .map(|(_, _, loc)| *loc)
        {
            None => stale.push(format!("{key}::{fn_name} — function no longer exists")),
            Some(loc) if loc <= PANEL_FN_LOC_CAP => stale.push(format!(
                "{key}::{fn_name} — now {loc} LOC, under the {PANEL_FN_LOC_CAP} cap"
            )),
            Some(loc) if loc < *allowed => stale.push(format!(
                "{key}::{fn_name} — now {loc} LOC, entry still frozen at {allowed}"
            )),
            Some(_) => {}
        }
    }

    assert!(
        stale.is_empty(),
        "FN_OVERAGE_OK entries that no longer describe reality:\n  {}\n\
         fix: delete the entry (fn is under the cap) or lower it to the \
         measured LOC. Allowances shrink; they never grow.",
        stale.join("\n  ")
    );
}

fn collect_panel_dirs(crates_root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let Ok(entries) = fs::read_dir(crates_root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with("ph2d-panel-") && name != "ph2d-panel-registry-init" {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn visit_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, cb);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        cb(&path);
    }
}

/// Extract `(fn_name, body_loc)` pairs from a Rust source. Body LOC
/// counts the lines between the `fn name(...) {` opener and the
/// matching `}`, inclusive, and skips `#[cfg(test)]` modules entirely.
///
/// The brace walk is **comment-aware** (see [`find_matching_brace`]).
/// It used to toggle a naive `in_char` flag on every `'`, so a prose
/// apostrophe inside a `//` comment ("doesn't", "sprite's") or a
/// lifetime tick (`&'a`) desynchronised the depth counter and closed
/// the function early — under-counting it. `apply_event_impl` in
/// `ph2d-panel-inspector` measured 353 that way and is really 477.
fn extract_fn_locs(src: &str) -> Vec<(String, usize)> {
    let stripped = strip_test_modules(src);
    let mut out: Vec<(String, usize)> = Vec::new();
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

/// Walk from the `{` at `open` to its matching `}`, ignoring braces that
/// live inside a comment, a string (raw or not) or a char literal.
/// Returns the byte index of the closing `}`.
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
            // `r"…"` / `r#"…"#` (and the `r` of `br"…"`, whose `b` is inert).
            b'r' if i + 1 < b.len() && matches!(b[i + 1], b'"' | b'#') => {
                match skip_raw_string(src, i) {
                    Some(next) => i = next,
                    // A raw *identifier* (`r#type`) — not a string.
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

/// From the opening `"`, return the index just past the closing one.
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

/// From the `r` of `r##"…"##`, return the index just past the terminator.
/// `None` when this `r` opens a raw identifier rather than a raw string.
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
    src[i..]
        .find(&terminator)
        .map(|rel| i + rel + terminator.len())
}

/// From a `'`, return the index just past a char literal (`'x'`, `'\n'`,
/// `b'{'`), or just past the tick alone when it opens a lifetime (`&'a`,
/// `'static`) — a lifetime has no closing quote to find.
fn skip_char_or_lifetime(b: &[u8], from: usize) -> usize {
    let after_tick = from + 1;
    if after_tick < b.len() && b[after_tick] == b'\\' {
        // Escaped: scan to the closing quote (`'\n'`, `'\''`, `'\u{1F}'`).
        let mut j = after_tick + 1;
        while j < b.len() && b[j] != b'\'' {
            j += 1;
        }
        return if j < b.len() { j + 1 } else { after_tick };
    }
    // Unescaped char literals are one scalar wide; a quote further out than
    // that means we are looking at a lifetime, not a literal.
    let mut j = after_tick;
    while j < b.len() && j < after_tick + 4 {
        if b[j] == b'\'' {
            return j + 1;
        }
        j += 1;
    }
    after_tick
}

/// Find next `fn <name>(...) {` (or `<vis> fn ... {`) starting at
/// `from`. Returns `(name, brace_idx)`. Skips fn-pointer declarations
/// and `fn` inside strings/comments by being naive-but-good-enough:
/// requires `\nfn ` or `\npub fn ` or `\npub(crate) fn ` etc. at line
/// start (after trim). Adequate for panel code where fns sit at column 0.
fn find_fn_opener(src: &str, from: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        let line_start = i;
        let line_end = find_line_end(bytes, i);
        let line = &src[line_start..line_end];
        let trimmed = line.trim_start();
        if let Some(fn_kw_pos) = find_fn_keyword(trimmed) {
            let after_fn = &trimmed[fn_kw_pos + 3..];
            // Extract name
            let after_fn = after_fn.trim_start();
            let name_end = after_fn
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(after_fn.len());
            if name_end == 0 {
                i = line_end + 1;
                continue;
            }
            let name = after_fn[..name_end].to_string();
            // Find the `{` that opens the body — may be on this line OR
            // on a later line (multi-line signature). Walk forward from
            // the position right after `fn <name>`.
            let scan_start = line_start
                + (line.len() - trimmed.len())
                + fn_kw_pos
                + 3
                + (after_fn.len() - after_fn.trim_start().len())
                + name_end;
            let scan_start = scan_start.min(bytes.len());
            let body_start = find_top_level_brace(bytes, scan_start);
            if let Some(b) = body_start {
                return Some((name, b));
            }
            i = line_end + 1;
            continue;
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

/// Find `fn ` keyword in a trimmed line — but only if it is preceded
/// (in the trimmed prefix) by whitespace/visibility keywords. Returns
/// byte offset of the `f` in `fn`.
fn find_fn_keyword(trimmed: &str) -> Option<usize> {
    // Accept: "fn ", "pub fn ", "pub(crate) fn ", "pub(super) fn ",
    // "async fn ", "const fn ", "unsafe fn ", combinations.
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        if &bytes[i..i + 3] == b"fn " {
            // Must be at start, or after whitespace + permitted keywords only.
            let prefix = &trimmed[..i];
            let prefix_trim = prefix.trim();
            let is_permitted = prefix_trim.is_empty()
                || matches!(
                    prefix_trim,
                    "pub"
                        | "pub(crate)"
                        | "pub(super)"
                        | "async"
                        | "const"
                        | "unsafe"
                        | "pub async"
                        | "pub const"
                        | "pub unsafe"
                        | "async unsafe"
                        | "const unsafe"
                        | "pub(crate) async"
                        | "pub(crate) const"
                        | "pub(crate) unsafe"
                )
                || prefix_trim.starts_with("pub(");
            if is_permitted {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Walk forward from `from` until we hit a top-level `{` (depth-aware
/// with respect to `(` `[` `<` `>` `]` `)`). Returns its byte index.
fn find_top_level_brace(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    // Skip naive about `<` `>` (generics) — count them as nesting too
    // so the `where` clause angle brackets don't trip us. False positives
    // possible on comparisons inside sig, but signatures don't usually
    // contain `>` / `<` outside generics.
    let mut angle = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 => angle -= 1,
            b'{' if paren == 0 && bracket == 0 && angle == 0 => return Some(i),
            b';' if paren == 0 && bracket == 0 && angle == 0 => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Strip `#[cfg(test)]` mod blocks brace-counting. Same shape as
/// `no_magic_numeric::cfg_test_byte_ranges` but returns the source
/// with those ranges replaced by empty space (so line numbers shift
/// — we only care about counting LOC per fn, not line numbers).
fn strip_test_modules(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let Some(rel) = src[i..].find("#[cfg(test)]") else {
            out.push_str(&src[i..]);
            break;
        };
        let attr_start = i + rel;
        out.push_str(&src[i..attr_start]);
        let mut j = attr_start + "#[cfg(test)]".len();
        while j < bytes.len() && bytes[j] != b'{' {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }
        // Comment-aware, like `extract_fn_locs` — a brace quoted inside a
        // test's string or comment must not end the module early.
        let Some(k) = find_matching_brace(src, j) else {
            break;
        };
        // Skip the test mod entirely.
        i = k + 1;
    }
    out
}
