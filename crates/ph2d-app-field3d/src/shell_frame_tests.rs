//! **O QUADRO da shell, lido pelos gates que medem que ele CHAMA esta família** — o
//! `shells/desktop/src/render_loop/mod.rs` e as `render_loop/fase_*.rs` em que a `line/render-loop` o
//! partiu (13/09), numa só string e sem comentários.
//!
//! ⚠️ **Aponta para FORA de propósito** (HOWTO §2.6): o sujeito destes gates é a SHELL. Até 13/09
//! eles liam só o `mod.rs`; quando o laço virou índice + 125 fases, os dois reprovaram ALTO — e um gate
//! de AUSÊNCIA com a mesma lente teria ficado verde a ler um índice sem código. ⇒ uma porta só para
//! os dois (`export_tests`, `mode_tests`), com PISO de população: uma varredura que perdesse as fases
//! leria só o índice.
//!
//! ⛔ **Serve a agulhas de PRESENÇA.** Uma pergunta de ORDEM entre fases não se responde por
//! concatenação alfabética: essa é a régua `frame_text::render_frame` dos gates da shell, que segue as
//! chamadas na ordem do quadro.

use std::path::Path;

/// Medido: **125** fases em 2026-09-13.
const PISO_FASES: usize = 100;

/// O `mod.rs` e as fases, por nome, com as linhas de comentário tiradas.
pub(crate) fn frame_code() -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src/render_loop");
    let mut nomes: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("o laço existe em {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n == "mod.rs" || (n.starts_with("fase_") && n.ends_with(".rs")))
        .collect();
    nomes.sort();
    assert!(
        nomes.iter().any(|n| n == "mod.rs"),
        "o índice do quadro (`render_loop/mod.rs`) sumiu"
    );
    assert!(
        nomes.len() > PISO_FASES,
        "o quadro leu-se com {} ficheiros, abaixo do piso de {PISO_FASES} fases — a varredura perdeu-as",
        nomes.len()
    );
    nomes
        .iter()
        .map(|n| std::fs::read_to_string(dir.join(n)).expect("o ficheiro do quadro lê-se"))
        .flat_map(|src| {
            src.lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
