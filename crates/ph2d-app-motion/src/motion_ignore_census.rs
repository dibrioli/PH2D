//! O censo das CLASSES de `#[ignore]` (doc 123 §5): a razão de cada teste ignorado da crate abre
//! com `placa: ` (gate que precisa do adapter — entra no portão de fecho), `sonda: ` (imprime ou
//! mede, nunca reprova) ou `gerador: ` (escreve ficheiros). Nem o libtest nem o nextest filtram pela
//! razão; o `scripts/motion-gate-placa.sh` lê o prefixo nas fontes, e este censo garante que o lê
//! em TODAS.

use std::path::{Path, PathBuf};

const CLASSES: [&str; 3] = ["placa: ", "sonda: ", "gerador: "];

fn fontes(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            fontes(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn every_ignore_in_the_motion_crate_names_its_class() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut ficheiros = Vec::new();
    fontes(&raiz.join("src"), &mut ficheiros);
    fontes(&raiz.join("tests"), &mut ficheiros);
    ficheiros.sort();

    let mut contagem = [0usize; CLASSES.len()];
    let mut sem_classe = Vec::new();
    for f in &ficheiros {
        let texto = std::fs::read_to_string(f).expect("ler a fonte");
        for (i, linha) in texto.lines().enumerate() {
            let l = linha.trim_start();
            let atributo =
                l.starts_with("#[ignore") || (l.starts_with("#[cfg_attr") && l.contains("ignore"));
            if !atributo {
                continue;
            }
            let classe = l
                .strip_prefix("#[ignore = \"")
                .and_then(|r| CLASSES.iter().position(|c| r.starts_with(c)));
            match classe {
                Some(k) => contagem[k] += 1,
                None => sem_classe.push(format!(
                    "{}:{}: {l}",
                    f.strip_prefix(raiz).unwrap_or(f).display(),
                    i + 1
                )),
            }
        }
    }

    let total: usize = contagem.iter().sum();
    for (c, n) in CLASSES.iter().zip(contagem) {
        eprintln!("  {c:<10}{n}");
    }
    eprintln!("  total     {total} em {} ficheiros", ficheiros.len());
    assert!(
        sem_classe.is_empty(),
        "{} `#[ignore]` sem `placa: `/`sonda: `/`gerador: ` na razão (doc 123 §5):\n{}",
        sem_classe.len(),
        sem_classe.join("\n")
    );
    // Controlo: o censo leu a população real, não um diretório vazio.
    assert!(
        contagem.iter().all(|&n| n >= 1),
        "classe vazia: {contagem:?}"
    );
    assert!(
        total >= 250,
        "só {total} `#[ignore]` lidos — o censo não leu a crate"
    );
}
