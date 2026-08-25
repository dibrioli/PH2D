//! ⭐⭐⭐ **O INTERRUPTOR DO CAMINHO SOLDADO TEM UMA PORTA SÓ.**
//!
//! ⚠️ **O gate LÊ O FONTE**, e é de propósito: um segundo sítio a ler
//! `PH2D_GRIDMAP_WELD` compila, passa a suíte, e reintroduz o defeito exacto que esta
//! linha curou — o instrumento e o produto nasceram a ler a MESMA variável com sentidos
//! **opostos** (um como opt-in, o outro como opt-out).
//!
//! *Uma pergunta com duas respostas é a que envelhece.*

use std::path::Path;

/// Todos os `.rs` rastreados da workspace.
fn sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "*.rs"])
        .output()
        .expect("git ls-files");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|rel| {
            std::fs::read_to_string(root.join(rel))
                .ok()
                .map(|s| (rel.to_string(), s))
        })
        .collect()
}

#[test]
fn only_one_place_reads_the_welded_switch() {
    let files = sources();
    assert!(
        files.len() > 500,
        "a varredura tem de conter a workspace: {} ficheiros",
        files.len()
    );
    // ⚠️ **A régua é quem LÊ, não quem NOMEIA.** A primeira redacção deste gate
    // procurava a string, e reprovou sobre o doc-comment do produto — que *deve* nomear
    // o interruptor, é a porta pela qual o Enio o descobre. *Um gate que confunde
    // documentação com acoplamento pede que se apague a documentação.*
    let readers: Vec<&str> = files
        .iter()
        .filter(|(_, s)| s.contains("var(\"PH2D_GRIDMAP_WELD\")"))
        .map(|(p, _)| p.as_str())
        .collect();
    assert_eq!(
        readers,
        vec!["crates/ph2d-gridmap/src/weld_round.rs"],
        "quem NOMEIA `PH2D_GRIDMAP_WELD` tem de ser só a porta da crate; os outros \
         chamam `welded_enabled()`. Achado em: {readers:?}"
    );
    let callers = files
        .iter()
        .filter(|(p, s)| s.contains("welded_enabled") && p != "crates/ph2d-gridmap/src/lib.rs")
        .count();
    assert!(
        callers >= 3,
        "a fixtura tem de conter o fenómeno: a porta e os DOIS consumidores — achei {callers}"
    );
}
