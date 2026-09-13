//! **A OUTRA METADE: a chave existe dos DOIS lados.**
//!
//! ⭐⭐⭐ Uma chave com erro de escrita PINTA O IDENTIFICADOR CRU na tela — e vaza a string: o
//! `ph2d_i18n::tr` de uma chave desconhecida faz `leak_key` (devolve o próprio identificador e faz
//! `Box::leak`), e num painel repintado a cada quadro isso é um vazamento POR QUADRO.
//!
//! ⭐ E o censo é dos dois lados porque os dois erros existem e a cura de cada um é oposta: uma chave
//! **usada e não declarada** pinta o identificador; uma **declarada e não usada** é uma órfã — e uma
//! órfã é onde alguém escreve, um dia, uma frase sobre um controlo que já não existe.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::source;

/// ⚠️ **Uma chave tem FORMA, e o censo tem de a conhecer.** A 1.ª redacção do gate aceitava qualquer
/// coisa sem espaços, e a primeira corrida acusou o PRÓPRIO ficheiro do gate: a mensagem de erro
/// dizia `"chrome.…"`, e o censo leu-a como uma chave em uso.
pub fn looks_like_a_key(k: &str, prefix: &str) -> bool {
    k.len() > prefix.len()
        && k.starts_with(prefix)
        // ⚠️ **Uma chave acaba num NOME, nunca num ponto** (2026-09-13): um gate que escreve o
        //    PREFIXO de uma secção (`"panel.inspector.player."`, para separar as duas metades de um
        //    vocabulário) era lido como uma chave em uso, e o censo acusava o próprio gate.
        && !k.ends_with('.')
        && k.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

fn code_of(p: &Path) -> Option<String> {
    fs::read_to_string(p)
        .ok()
        .map(|s| source::strip_comments(&s).into_iter().collect())
}

/// Todas as chaves `<prefix>…` **usadas** na árvore do repo, com o primeiro ficheiro onde aparecem.
///
/// ⚠️ As tabelas declaram; elas não usam — `tables` ficam de fora, senão o censo lê a própria fonte
/// como consumidor e os dois lados concordam sempre: um espelho não acusa. ⚠️ São VÁRIAS porque um
/// vocabulário pode estar partido por secção (o Inspector: a §14 num irmão, pelo tecto de LOC) — e
/// uma metade lida como consumidora daria por usadas todas as chaves que ela declara.
pub fn keys_used(repo: &Path, prefix: &str, tables: &[&str]) -> BTreeMap<String, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if p.is_dir() {
                if matches!(name, "target" | ".git" | "docs" | "Worktrees") {
                    continue;
                }
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let tables: Vec<PathBuf> = tables.iter().map(|t| repo.join(t)).collect();
    let mut files = Vec::new();
    walk(repo, &mut files);
    files.sort();
    let needle = format!("\"{prefix}");
    let mut out = BTreeMap::new();
    for p in files {
        if tables.contains(&p) {
            continue;
        }
        let Some(code) = code_of(&p) else {
            continue;
        };
        let mut i = 0usize;
        while let Some(k) = code[i..].find(&needle) {
            let start = i + k + 1;
            let Some(end) = code[start..].find('"') else {
                break;
            };
            let key = &code[start..start + end];
            if looks_like_a_key(key, prefix) {
                out.entry(key.to_string()).or_insert_with(|| {
                    p.strip_prefix(repo)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/")
                });
            }
            i = start + end;
        }
    }
    out
}

/// Todas as chaves `<prefix>…` **declaradas** nas tabelas `tables` — só o LADO ESQUERDO de um
/// braço (`"chave" =>`) conta, e as metades de um vocabulário partido somam-se.
pub fn keys_declared(repo: &Path, tables: &[&str], prefix: &str) -> BTreeSet<String> {
    let needle = format!("\"{prefix}");
    let mut out = BTreeSet::new();
    for table_rel in tables {
        let p = repo.join(table_rel);
        let code = code_of(&p).unwrap_or_else(|| panic!("a tabela {p:?} não se lê"));
        let mut i = 0usize;
        while let Some(k) = code[i..].find(&needle) {
            let start = i + k + 1;
            let Some(end) = code[start..].find('"') else {
                break;
            };
            let key = &code[start..start + end];
            if code[start + end + 1..].trim_start().starts_with("=>") {
                out.insert(key.to_string());
            }
            i = start + end;
        }
    }
    out
}
