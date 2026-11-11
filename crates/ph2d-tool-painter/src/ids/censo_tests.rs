//! B50.2 (AU-AU03-3): todo `NodeId` declarado em `ids/` é nomeado em algum outro sítio da workspace
//! (`crates/` + `shells/`, ids incluídos: um array de `ids/` conta). Id sem emissor nem leitor é órfão.

use std::path::{Path, PathBuf};

fn fontes(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(ls) = std::fs::read_dir(dir) else {
        return;
    };
    for e in ls.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n != "target") {
                fontes(&p, out);
            }
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Quantas vezes o identificador `id` aparece em `texto` como token inteiro.
fn ocorrencias(texto: &str, id: &str) -> usize {
    let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    texto
        .match_indices(id)
        .filter(|(i, _)| {
            let antes = texto[..*i].chars().next_back();
            let depois = texto[i + id.len()..].chars().next();
            !antes.is_some_and(ident) && !depois.is_some_and(ident)
        })
        .count()
}

#[test]
fn nenhum_id_do_painter_fica_sem_referencia() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut ficheiros = Vec::new();
    fontes(&raiz.join("crates"), &mut ficheiros);
    fontes(&raiz.join("shells"), &mut ficheiros);
    let textos: Vec<String> = ficheiros
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    let mut ids_dir = Vec::new();
    fontes(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ids"),
        &mut ids_dir,
    );
    let mut ids = Vec::new();
    for p in &ids_dir {
        let t = std::fs::read_to_string(p).expect("ficheiro de ids");
        ids.extend(t.lines().filter_map(|l| {
            let r = l.trim_start().strip_prefix("pub const ")?;
            let (nome, tipo) = r.split_once(':')?;
            tipo.trim_start()
                .starts_with("NodeId")
                .then(|| nome.trim().to_owned())
        }));
    }
    assert!(ids.len() > 400, "controlo: só {} ids lidos", ids.len());
    let orfaos: Vec<&String> = ids
        .iter()
        .filter(|id| textos.iter().map(|t| ocorrencias(t, id)).sum::<usize>() < 2)
        .collect();
    assert!(orfaos.is_empty(), "id sem nenhuma referência: {orfaos:?}");
}
