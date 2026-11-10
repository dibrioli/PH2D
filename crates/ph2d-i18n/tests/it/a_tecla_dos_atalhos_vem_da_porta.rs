//! ⭐ **Nenhum `Cmd+` cru**: a tabela escreve o acorde como `Ctrl+` e a porta `ph2d_i18n::atalho` diz `Cmd+` só no
//! macOS (dono 10/11: o app dizia «Cmd+Z» no Linux e no Windows). Fora da tabela, quem escreve um atalho passa pela
//! mesma porta. Lê a tabela com subpastas (o leitor de `a_tabela_inglesa_fala_ingles` só lê o 1.º nível) e o fonte
//! `.rs` de `crates/` e `shells/` fora de comentários.

use std::path::{Path, PathBuf};

fn rs_em(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n != "target") {
                rs_em(&p, out);
            }
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn raiz_do_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Os únicos que escrevem `Cmd+`: a porta (é o que ela devolve no macOS) e este censo.
const ISENTOS: &[&str] = &[
    "ph2d-i18n/src/atalho.rs",
    "ph2d-i18n/tests/it/a_tecla_dos_atalhos_vem_da_porta.rs",
];

#[test]
fn nenhuma_string_da_tabela_diz_cmd() {
    let mut fich = Vec::new();
    rs_em(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut fich,
    );
    let (mut com_ctrl, mut mas) = (0usize, Vec::new());
    for p in &fich {
        let src = std::fs::read_to_string(p).expect("lê a tabela");
        for par in ph2d_label_census::keys::declared_pairs_in(&src) {
            com_ctrl += usize::from(par.texto.contains(ph2d_i18n::atalho::NA_TABELA));
            if par.texto.contains("Cmd+") {
                mas.push(format!(
                    "{}:{} {} {:?}",
                    p.display(),
                    par.linha,
                    par.chave,
                    par.texto
                ));
            }
        }
    }
    // Controlo positivo: o leitor vê os acordes (10/11: 95 entradas com `Ctrl+`).
    assert!(
        com_ctrl >= 90,
        "o leitor viu só {com_ctrl} entradas com Ctrl+: partiu-se?"
    );
    assert!(
        mas.is_empty(),
        "escreva o acorde como Ctrl+ (a porta diz Cmd+ no macOS):\n  {}",
        mas.join("\n  ")
    );
}

#[test]
fn nenhum_literal_do_repo_diz_cmd() {
    let r = raiz_do_repo();
    let mut fich = Vec::new();
    rs_em(&r.join("crates"), &mut fich);
    rs_em(&r.join("shells"), &mut fich);
    assert!(
        fich.len() > 3_000,
        "o censo leu {} ficheiros: a raiz mudou?",
        fich.len()
    );
    let mas: Vec<String> = fich
        .iter()
        .filter(|p| {
            let p = p.to_string_lossy().replace('\\', "/");
            !ISENTOS.iter().any(|i| p.ends_with(i))
        })
        .flat_map(|p| {
            let src = std::fs::read_to_string(p).unwrap_or_default();
            let p = p.strip_prefix(&r).unwrap_or(p).display().to_string();
            src.lines()
                .enumerate()
                .filter(|(_, l)| {
                    let t = l.trim_start();
                    l.contains("Cmd+") && !t.starts_with("//") && !t.starts_with('*')
                })
                .map(|(i, l)| format!("{p}:{} {}", i + 1, l.trim()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        mas.is_empty(),
        "atalho escrito com Cmd+ fora da porta — escreva Ctrl+ e passe por ph2d_i18n::atalho::atalho:\n  {}",
        mas.join("\n  ")
    );
}
