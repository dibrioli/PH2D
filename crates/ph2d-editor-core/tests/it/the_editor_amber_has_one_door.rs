//! ⭐ **A âmbar do REALCE do editor tem UMA porta** — [`ph2d_editor_core::editor_highlight`].
//!
//! Até 2026-09-13 o literal `[1.0, 0.72, 0.2, …]` vivia em QUATRO constantes de DUAS famílias (o realce e
//! o hover do Flip, a trajectória e a tangente do Motion), unidas por um comentário a dizer «a mesma
//! âmbar». Este gate lê o CÓDIGO da workspace (sem comentários) e reprova o literal fora da porta; o piso
//! de chamadas impede que ele fique verde por a porta ter deixado de ser usada.

use std::path::{Path, PathBuf};

/// Medido: 7 5xx ficheiros `.rs` em `crates/`, `shells/` e `tools/` (2026-09-13).
const PISO_FICHEIROS: usize = 7_000;
/// Medido: 4 chamadas (Flip 2, Motion 2) em 2026-09-13 — contadas FORA deste ficheiro.
const PISO_CHAMADAS: usize = 4;
const PORTA: &str = "crates/ph2d-editor-core/src/editor_highlight.rs";
/// Este ficheiro fica fora das DUAS contagens: ele escreve o matiz como DADOS da metade justa e nomeia
/// `editor_highlight::amber(` três vezes nas próprias mensagens. ⚠️ A 1.ª redacção acusava-se a si mesma e
/// lia 7 chamadas onde havia 4 — o piso ficava verde com UM chamador real.
const ESTE_GATE: &str = "crates/ph2d-editor-core/tests/it/the_editor_amber_has_one_door.rs";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .to_path_buf()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// O texto sem comentários de linha — a prosa que EXPLICA a porta cita o matiz.
fn codigo(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// O matiz da âmbar escrito à mão, com qualquer espaçamento — e não um `0.2x` que só começa igual.
fn tem_o_matiz(cod: &str) -> bool {
    let compacto: String = cod.chars().filter(|c| !c.is_whitespace()).collect();
    compacto.match_indices("1.0,0.72,0.2").any(|(i, m)| {
        !compacto[i + m.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    })
}

#[test]
fn the_editor_amber_has_one_door() {
    let r = root();
    let mut ficheiros = Vec::new();
    for d in ["crates", "shells", "tools"] {
        walk(&r.join(d), &mut ficheiros);
    }
    assert!(
        ficheiros.len() >= PISO_FICHEIROS,
        "o varrimento leu {} ficheiros e esperava >= {PISO_FICHEIROS} — perdeu um directório",
        ficheiros.len()
    );
    let mut fora = Vec::new();
    let mut chamadas = 0;
    for f in &ficheiros {
        let rel = f
            .strip_prefix(&r)
            .expect("dentro da raiz")
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        if rel == ESTE_GATE {
            continue;
        }
        let cod = codigo(&src);
        chamadas += cod.matches("editor_highlight::amber(").count();
        if rel != PORTA && tem_o_matiz(&cod) {
            fora.push(rel);
        }
    }
    assert!(
        chamadas >= PISO_CHAMADAS,
        "só {chamadas} chamadas a `editor_highlight::amber(` (piso {PISO_CHAMADAS}) — a porta deixou de ser usada?"
    );
    assert!(
        fora.is_empty(),
        "a âmbar do realce escrita à mão fora da porta — chame `ph2d_editor_core::editor_highlight::amber(alfa)`:\n  {}",
        fora.join("\n  ")
    );
}

/// A metade justa da régua: ela vê o matiz partido em linhas e não confunde um vizinho que só começa igual.
#[test]
fn the_amber_reader_can_say_no() {
    assert!(tem_o_matiz(
        "const X: [f32; 4] = [1.0, 0.72,\n    0.2, 0.5];"
    ));
    assert!(!tem_o_matiz("const Y: [f32; 4] = [1.0, 0.72, 0.25, 1.0];"));
    assert!(!tem_o_matiz(&codigo("// [1.0, 0.72, 0.2] numa nota")));
}
