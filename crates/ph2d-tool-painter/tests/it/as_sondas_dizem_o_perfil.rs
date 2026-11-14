//! B44.4 (missão 4K N10): uma sonda de relógio diz o PERFIL que mediu e recusa o `dev`; o cabeçalho manda
//! correr em `--profile smoke` (o `--release` só com `PH2D_RELEASE="mede velocidade"`, guarda R8); o
//! `mede_o_dab` no Digital lê os dabs do `band_diag` (o `wet_diag` dá 0 fora do Wet). Censo de texto.

use std::path::Path;

/// Ficheiros com WIP alheio aberto (outra linha da rodada): o cabeçalho cura-se quando ela fechar. Vazio desde o
/// F6k (o F1i fechou `measure_undo_cost.rs`).
const PENDENTES: &[&str] = &[];

fn varre(dir: &Path, raiz: &Path, maus: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            varre(&p, raiz, maus);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let rel = p
                .strip_prefix(raiz)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if PENDENTES.contains(&rel.as_str()) || rel.ends_with("as_sondas_dizem_o_perfil.rs") {
                continue;
            }
            for (i, l) in std::fs::read_to_string(&p).unwrap().lines().enumerate() {
                let t = l.trim_start();
                let comando = ["cargo test", "cargo run", "cargo build", "cargo bench"]
                    .iter()
                    .any(|c| t.contains(c));
                if t.starts_with("//")
                    && comando
                    && t.contains("--release")
                    && !t.contains("PH2D_RELEASE")
                {
                    maus.push(format!("{rel}:{} {}", i + 1, t));
                }
            }
        }
    }
}

#[test]
fn as_sondas_dizem_o_perfil_e_correm_em_smoke() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut maus = Vec::new();
    for d in ["src", "tests", "examples"] {
        varre(&raiz.join(d), raiz, &mut maus);
    }
    for e in std::fs::read_dir(raiz.join("examples")).unwrap().flatten() {
        let nome = e.file_name().to_string_lossy().into_owned();
        if nome.starts_with("mede_") {
            let s = std::fs::read_to_string(e.path()).unwrap();
            if !s.contains("perfil_da_sonda()") {
                maus.push(format!("examples/{nome}: não chama perfil_da_sonda()"));
            }
        }
    }
    let dab = std::fs::read_to_string(raiz.join("examples/mede_o_dab.rs")).unwrap();
    if !dab.contains("band_diag::take()") {
        maus.push("examples/mede_o_dab.rs: o Digital não lê os dabs do band_diag".into());
    }
    assert!(
        maus.is_empty(),
        "{} sondas sem o perfil certo:\n{}",
        maus.len(),
        maus.join("\n")
    );
}
