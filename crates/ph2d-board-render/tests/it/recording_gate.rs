//! ⭐ **O desenho do quadro passa TODO pelas portas que gravam** (W8, `docs/MiroClone/06_equipa_e_saida.md`
//! §0): o SVG e o PDF leem a gravação da cena (`ph2d_vector::DrawOp`); um traço desenhado pelo
//! `inner_mut()` do Vello sai no ecrã e no PNG e FALTA no vector — sem erro nenhum.

use std::path::Path;

fn sources(dir: &Path, out: &mut Vec<(String, String)>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if p.is_dir() {
            sources(&p, out);
        } else if name.ends_with(".rs") && !name.ends_with("_tests.rs") {
            out.push((p.display().to_string(), std::fs::read_to_string(&p).unwrap()));
        }
    }
}

#[test]
fn board_paint_never_bypasses_the_recording() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for c in ["ph2d-board-render/src", "ph2d-board-layout/src"] {
        sources(&root.join(c), &mut files);
    }
    assert!(files.len() > 10, "o censo varre os dois crates ({})", files.len());
    let mut bad = Vec::new();
    for (path, text) in &files {
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("inner_mut()") || code.contains(".inner_mut(") {
                bad.push(format!("{path}:{}", i + 1));
            }
        }
    }
    assert!(bad.is_empty(), "desenho fora da gravação: {bad:?}");
}
