//! Wave 2 PR 11.9 — HR-18 file-LOC cap gate, **active**.
//!
//! Every `.rs` file in `shells/desktop/src/` must stay under 600 LOC (SKILL §HR-18) — SALVO as
//! entradas NUMERADAS de [`FILE_OVERAGE_OK`], cada uma com o tecto MEDIDO, que só desce.
//!
//! ⛔⛔ **O marcador `// ph2d-loc-cap: <razão>` MORREU em 2026-09-12** (auditoria de arquitectura A3).
//! Ele isentava SEM número, e uma isenção sem número é uma licença: o `render_loop/mod.rs` cresceu de
//! 1 667 linhas (15/06) para 14 009 (12/09) debaixo dele, e o `input_dispatch.rs` de 1 284 para
//! 7 266. Hoje a excepção é uma linha desta tabela, e o gate tem as DUAS metades da catraca do
//! `the_shell_only_shrinks`: *cresceu acima do tecto* e *o tecto ficou para trás* (folga acima de
//! [`FOLGA_MAXIMA`]). Um ficheiro que ainda traga o marcador reprova: ele já não isenta nada, e
//! deixá-lo lá diria o contrário a quem abre o ficheiro.

use std::fs;
use std::path::{Path, PathBuf};

/// Hard cap from HR-18 §2.
const FILE_LOC_CAP: usize = 600;

/// Quanto um tecto pode ficar ACIMA do ficheiro antes de ser obsoleto — um corte maior do que isto
/// baixa o número no mesmo commit.
const FOLGA_MAXIMA: usize = 20;

/// A janela em que um marcador antigo ainda seria lido (e agora reprova).
const MARKER_WINDOW_LINES: usize = 20;

/// ⛔ **Os tectos NUMERADOS** — `(ficheiro relativo a src/, LOC medido, razão)`. Medidos na auditoria
/// de 2026-09-12, depois do `cargo fmt`; a razão é a que o marcador trazia. **Só descem.**
const FILE_OVERAGE_OK: &[(&str, usize, &str)] = &[
    (
        "app_state.rs",
        1551,
        "AppGfx + App are the shell's two top-level aggregates —",
    ),
    (
        "input_dispatch.rs",
        951,
        "Onda 2C multi-select dispatch + hit_map routing +",
    ),
    (
        "main.rs",
        1289,
        "crate-root module hub — the 80+ `mod` declarations are an",
    ),
    (
        "render_loop/fase_bus_drain.rs",
        2654,
        "fase do quadro (OBRA 2): o dreno do barramento, verbatim, e o struct dos pedidos que ele devolve; um braço do `match` não é um statement",
    ),
    (
        "render_loop/mod.rs",
        1604,
        "frame orchestrator — heavy phases already extracted to",
    ),
    (
        "render_loop/sim_extract.rs",
        1155,
        "per-sprite RenderInstance build accreted every W3 render",
    ),
    (
        "render_loop/snapshots.rs",
        1398,
        "accreted one producer per W3 Inspector section",
    ),
];

fn collect_rs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rel(p: &Path) -> String {
    p.strip_prefix(src_root())
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// `true` se a linha é o marcador antigo (`// ph2d-loc-cap: …` no início da linha).
fn is_retired_marker(line: &str) -> bool {
    line.trim_start()
        .strip_prefix("//")
        .is_some_and(|r| r.trim_start().starts_with("ph2d-loc-cap:"))
}

#[test]
fn shell_files_respect_hr18_loc_cap() {
    let files = collect_rs(&src_root());
    assert!(
        files.len() >= 300,
        "li só {} ficheiros em src/ — a varredura partiu-se",
        files.len()
    );
    let mut over = Vec::new();
    for path in files {
        let loc = fs::read_to_string(&path)
            .expect("read shell file")
            .lines()
            .count();
        if loc <= FILE_LOC_CAP {
            continue;
        }
        let r = rel(&path);
        let tecto = FILE_OVERAGE_OK
            .iter()
            .find(|(p, _, _)| *p == r)
            .map_or(FILE_LOC_CAP, |(_, n, _)| *n);
        if loc > tecto {
            over.push(format!("{r} — {loc} LOC (tecto {tecto})"));
        }
    }
    assert!(
        over.is_empty(),
        "HR-18 — ficheiros da shell acima do tecto:\n  {}\n\ncura: partir o ficheiro em módulos \
         irmãos por RESPONSABILIDADE. ⛔ Nunca subir o número de `FILE_OVERAGE_OK`: ele só desce.",
        over.join("\n  ")
    );
}

#[test]
fn overage_allowlist_has_no_stale_entries() {
    let mut stale = Vec::new();
    for (p, tecto, razao) in FILE_OVERAGE_OK {
        assert!(!razao.trim().is_empty(), "`{p}` sem razão");
        let loc = fs::read_to_string(src_root().join(p)).map_or(0, |t| t.lines().count());
        if loc <= FILE_LOC_CAP {
            stale.push(format!(
                "{p} (tecto {tecto}, agora {loc} ≤ {FILE_LOC_CAP}) — apague a entrada"
            ));
        } else if *tecto > loc + FOLGA_MAXIMA {
            stale.push(format!(
                "{p} (tecto {tecto}, agora {loc}) — o tecto ficou {} linhas para trás: baixe-o para {loc}",
                tecto - loc
            ));
        }
    }
    assert!(
        stale.is_empty(),
        "entradas de FILE_OVERAGE_OK obsoletas (a catraca só se mantém honesta se descer com o corte):\n  {}",
        stale.join("\n  ")
    );
}

#[test]
fn no_file_leans_on_the_retired_marker() {
    let mut com_marcador = Vec::new();
    for path in collect_rs(&src_root()) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if text
            .lines()
            .take(MARKER_WINDOW_LINES)
            .any(is_retired_marker)
        {
            com_marcador.push(rel(&path));
        }
    }
    assert!(
        com_marcador.is_empty(),
        "estes ficheiros ainda declaram `// ph2d-loc-cap:` — o marcador já não isenta nada (a excepção \
         é uma linha NUMERADA de `FILE_OVERAGE_OK`):\n  {}",
        com_marcador.join("\n  ")
    );
}

/// ⚠️ **E o leitor sabe dizer «não».**
#[test]
fn the_marker_reader_can_say_no() {
    assert!(is_retired_marker("// ph2d-loc-cap: razão"));
    assert!(is_retired_marker("    //ph2d-loc-cap:x"));
    assert!(!is_retired_marker(
        "//! o `// ph2d-loc-cap:` que aqui estava saiu"
    ));
    assert!(!is_retired_marker(
        "let x = 1; // ph2d-loc-cap: no fim da linha"
    ));
    assert!(!is_retired_marker(
        "// Tecto de LOC NUMERADO em `tests/it/file_loc_caps.rs` — razão"
    ));
}
