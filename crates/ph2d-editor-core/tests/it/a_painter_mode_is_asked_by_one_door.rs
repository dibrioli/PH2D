//! ⛔⛔ **«O modo usa o Painter?» pergunta-se por UMA porta** — `ObjectMode::uses_the_painter`.
//!
//! A shell largava o Painter em todo modo que não fosse `== ObjectMode::Paint` (a fase das Image
//! Tools), e o Image ▸ Mask (04/10) morria no quadro seguinte: verde em todo gate, apanhado pela
//! FOTO da cena `PH2D_OBJECT_MODE_SMOKE=7`. A lei escrita para o 1.º irmão partiu o 2.º.

use std::fs;
use std::path::{Path, PathBuf};

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .to_path_buf()
}

/// Uma comparação À MÃO com o modo Paint (`== …ObjectMode::Paint` ou `!=`), numa linha.
fn compara_paint_a_mao(linha: &str) -> bool {
    let l = linha.trim_start();
    !l.starts_with("//")
        && ["== ", "!= "].iter().any(|op| {
            l.split(op).skip(1).any(|resto| {
                resto
                    .trim_start()
                    .trim_end_matches([')', ' ', '{'])
                    .ends_with("ObjectMode::Paint")
            })
        })
}

fn varre(dir: &Path, achados: &mut Vec<String>, ficheiros: &mut usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if p.is_dir() {
            if nome != "target" && nome != "tests" {
                varre(&p, achados, ficheiros);
            }
        } else if nome.ends_with(".rs") && !nome.ends_with("_tests.rs") && nome != "object_mode.rs"
        {
            let Ok(s) = fs::read_to_string(&p) else {
                continue;
            };
            *ficheiros += 1;
            for (i, l) in s.lines().enumerate() {
                if compara_paint_a_mao(l) {
                    achados.push(format!("{}:{}: {}", p.display(), i + 1, l.trim()));
                }
            }
        }
    }
}

#[test]
fn nobody_compares_the_mode_with_paint_by_hand() {
    // CONTROLO do filtro: as duas formas que existiram casam; a porta e um comentário não.
    assert!(compara_paint_a_mao(
        "            && hero.gizmo.mode.current() != ph2d_editor_core::object_mode::ObjectMode::Paint"
    ));
    assert!(compara_paint_a_mao(
        "                || self.gizmo.mode.current() == crate::object_mode::ObjectMode::Paint)"
    ));
    assert!(!compara_paint_a_mao(
        "        && !hero.gizmo.mode.current().uses_the_painter() {"
    ));
    assert!(!compara_paint_a_mao("    // ... != ObjectMode::Paint"));

    let (mut achados, mut ficheiros) = (Vec::new(), 0usize);
    for sub in ["shells/desktop/src", "crates/ph2d-editor-core/src"] {
        varre(&raiz().join(sub), &mut achados, &mut ficheiros);
    }
    assert!(
        ficheiros > 500,
        "controlo: a varredura achou só {ficheiros} ficheiros"
    );
    assert!(
        achados.is_empty(),
        "pergunte `ObjectMode::uses_the_painter()` — o Mask é o mesmo Painter:\n{}",
        achados.join("\n")
    );
}
