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

use crate::rust_src::extract_fn_locs;

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
    (
        "render_loop/fase_audio_panels.rs",
        "fase_audio_panels",
        373,
        "fase do quadro (OBRA 2): o mixer e o editor de áudio, verbatim; assunto da família `audio` \
         que fica na shell porque as três features que o guardam são da shell",
    ),
    ("render_loop/hierarchy.rs", "dispatch", 414, NASCEU),
    ("render_loop/image_edit.rs", "dispatch", 483, NASCEU),
    ("render_loop/inspector_commits.rs", "dispatch", 383, NASCEU),
    (
        "render_loop/mod.rs",
        "run_render_frame",
        12181,
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
