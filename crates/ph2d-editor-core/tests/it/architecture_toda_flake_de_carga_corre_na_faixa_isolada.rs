//! ⛔ **Toda flake de carga corre na faixa isolada do nextest** — a lista do doc e o filtro do
//! `.config/nextest.toml` são a MESMA lista escrita em dois sítios, e este gate os ata.
//!
//! Medido em 2026-10-02: a família vivia no `CLAUDE.md §5.0` e promover um membro era escrever o
//! nome LÁ; a faixa (`threads-required = 'num-cpus'`, `priority = -100`) só foi preenchida uma vez,
//! em 10/09. Resultado: **oito** membros promovidos depois disso nunca entraram na faixa e
//! continuaram a reprovar portões sob fan-out — cada um, uma corrida de portão a mais.
//! A lista viva é `docs/DevOps/FLAKES_DE_CARGA.md`.
//!
//! Três metades:
//! 1. todo membro da tabela está no filtro da faixa;
//! 2. todo nome do filtro está na tabela ou na lista «na faixa por outra razão» (senão a faixa
//!    serializa testes que ninguém sabe porque estão lá);
//! 3. todo nome ainda é um `fn` na árvore — o censo de obsolescência (um nome morto no filtro não
//!    casa nada e lê-se como «está protegido»).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ler(rel: &str) -> String {
    let p = raiz().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("ler {}: {e}", p.display()))
}

/// Os nomes da tabela de membros (`| \`nome\` | …`) e os da lista «na faixa por outra razão»
/// (`- \`nome\` — …`).
fn nomes_do_doc(doc: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let ident = |s: &str| -> Option<String> {
        let s = s.strip_prefix('`')?;
        let fim = s.find('`')?;
        let n = &s[..fim];
        n.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            .then(|| n.to_string())
    };
    let mut membros = BTreeSet::new();
    let mut outros = BTreeSet::new();
    for l in doc.lines() {
        if let Some(n) = l.strip_prefix("| ").and_then(ident) {
            membros.insert(n);
        } else if let Some(n) = l.strip_prefix("- ").and_then(ident) {
            outros.insert(n);
        }
    }
    (membros, outros)
}

/// Os `test(nome)` do filtro da faixa isolada — a linha `filter =` que precede
/// `threads-required = 'num-cpus'`.
fn nomes_da_faixa(toml: &str) -> BTreeSet<String> {
    let linhas: Vec<&str> = toml.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.trim() == "threads-required = 'num-cpus'")
        .expect("a faixa isolada (`threads-required = 'num-cpus'`) sumiu do nextest.toml");
    let filtro = linhas[..i]
        .iter()
        .rev()
        .find(|l| l.trim_start().starts_with("filter ="))
        .expect("a faixa isolada não tem `filter =`");
    let mut out = BTreeSet::new();
    let mut resto = *filtro;
    while let Some(p) = resto.find("test(") {
        let t = &resto[p + 5..];
        let fim = t.find(')').unwrap_or(t.len());
        let n = &t[..fim];
        if !n.starts_with('/') {
            out.insert(n.to_string());
        }
        resto = &t[fim..];
    }
    out
}

fn fns_da_arvore(dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            fns_da_arvore(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let Ok(s) = std::fs::read_to_string(&p) else {
                continue;
            };
            let mut resto = s.as_str();
            while let Some(i) = resto.find("fn ") {
                let t = &resto[i + 3..];
                let fim = t
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(t.len());
                out.insert(t[..fim].to_string());
                resto = &t[fim..];
            }
        }
    }
}

#[test]
fn architecture_toda_flake_de_carga_corre_na_faixa_isolada() {
    let (membros, outros) = nomes_do_doc(&ler("docs/DevOps/FLAKES_DE_CARGA.md"));
    let faixa = nomes_da_faixa(&ler(".config/nextest.toml"));
    // Pisos de população: uma régua que lesse zero passaria por vácuo nas três metades.
    assert!(
        membros.len() >= 25,
        "a tabela do doc deu só {} membros — o formato mudou",
        membros.len()
    );
    assert!(
        faixa.len() >= 25,
        "o filtro da faixa deu só {} nomes — o formato mudou",
        faixa.len()
    );

    let fora: Vec<_> = membros.difference(&faixa).collect();
    assert!(
        fora.is_empty(),
        "membros da família que NÃO correm na faixa isolada (vão reprovar portões sob carga):\n  {fora:?}\n\
         Cura: acrescentar `+ test(<nome>)` ao filtro da faixa em .config/nextest.toml."
    );
    let sem_dono: Vec<_> = faixa
        .iter()
        .filter(|n| !membros.contains(*n) && !outros.contains(*n))
        .collect();
    assert!(
        sem_dono.is_empty(),
        "nomes na faixa isolada que o doc não explica:\n  {sem_dono:?}\n\
         Cura: uma linha na tabela de docs/DevOps/FLAKES_DE_CARGA.md (ou na lista «na faixa por \
         outra razão»), ou tirá-los do filtro."
    );

    let mut fns = BTreeSet::new();
    for topo in ["crates", "shells"] {
        fns_da_arvore(&raiz().join(topo), &mut fns);
    }
    assert!(
        fns.len() > 10_000,
        "o censo de `fn` achou só {} — a varredura partiu",
        fns.len()
    );
    let mortos: Vec<_> = membros
        .union(&faixa)
        .filter(|n| !fns.contains(*n))
        .collect();
    assert!(
        mortos.is_empty(),
        "nomes que já não são `fn` nenhum na árvore (um filtro morto lê-se como «protegido»):\n  {mortos:?}"
    );
}

/// Controlo positivo do leitor do doc: as duas formas de linha são reconhecidas e o resto não.
#[test]
fn o_leitor_do_doc_separa_membros_de_outros() {
    let doc = "| teste | onde |\n|---|---|\n| `um_membro` | x |\n- `outro_motivo` — y\n- texto\n";
    let (m, o) = nomes_do_doc(doc);
    assert_eq!(
        m.into_iter().collect::<Vec<_>>(),
        vec!["um_membro".to_string()]
    );
    assert_eq!(
        o.into_iter().collect::<Vec<_>>(),
        vec!["outro_motivo".to_string()]
    );
}
