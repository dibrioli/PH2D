//! Ler o FONTE dos chamadores para o censo dos avisos (`toast_reading_census`): os ficheiros, os
//! argumentos, os literais, as chamadas, as consts e os corpos das funções. Saiu do censo pelo tecto
//! de LOC (doc 132).

use ph2d_i18n::{Idioma, tr_em};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn fontes(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        panic!("o censo não leu {}", dir.display());
    };
    for e in rd.flatten() {
        let p = e.path();
        let nome = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if !["target", "tests", "benches", "examples", "node_modules"].contains(&nome.as_str())
            {
                fontes(&p, out);
            }
        } else if nome.ends_with(".rs")
            && !["test", "probe", "diag", "sonda", "census"]
                .iter()
                .any(|t| nome.contains(t))
        {
            out.push(p);
        }
    }
}

/// O ficheiro sem os módulos de teste em linha.
pub(super) fn sem_testes(s: &str) -> &str {
    let mut corte = s.len();
    let mut de = 0;
    while let Some(i) = s[de..].find("#[cfg(test)]") {
        let i = de + i;
        let resto = s[i + 12..].trim_start();
        let linha = resto.lines().next().unwrap_or("");
        if (linha.starts_with("mod ") || linha.contains(" mod ")) && linha.contains('{') {
            corte = i;
            break;
        }
        de = i + 12;
    }
    &s[..corte]
}

/// O argumento entre os parênteses que abrem em `abre` (o índice do `(`).
pub(super) fn argumento(s: &str, abre: usize) -> &str {
    let b = s.as_bytes();
    let (mut fundo, mut i) = (0i32, abre);
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'(' => fundo += 1,
            b')' => {
                fundo -= 1;
                if fundo == 0 {
                    return &s[abre + 1..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    &s[abre + 1..]
}

/// Os literais de texto de um pedaço de fonte.
pub(super) fn literais(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            let mut lit: Vec<u8> = Vec::new();
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' && i + 1 < b.len() {
                    i += 1;
                    match b[i] {
                        // A continuação de linha come a quebra e o recuo seguinte.
                        b'\n' => {
                            while i + 1 < b.len() && b[i + 1].is_ascii_whitespace() {
                                i += 1;
                            }
                        }
                        b'n' => lit.push(b'\n'),
                        b'u' => {
                            let fim = s[i..].find('}').map_or(i, |f| i + f);
                            let hex = s.get(i + 2..fim).unwrap_or("");
                            if let Some(c) =
                                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                            {
                                lit.extend(c.to_string().bytes());
                            }
                            i = fim;
                        }
                        c => lit.push(c),
                    }
                } else {
                    lit.push(b[i]);
                }
                i += 1;
            }
            out.push(String::from_utf8_lossy(&lit).into_owned());
        }
        i += 1;
    }
    out
}

pub(super) fn e_chave(l: &str) -> bool {
    !l.is_empty() && !l.contains(' ') && tr_em(Idioma::Ingles, l) != l
}

/// Os nomes chamados num pedaço de fonte (`nome(` ou `.nome(`), sem os de sempre.
pub(super) fn chamadas(s: &str) -> Vec<String> {
    const COMUNS: &[&str] = &[
        "tr_with",
        "to_string",
        "format",
        "push",
        "warning",
        "success",
        "error",
        "clone",
        "into",
        "as_str",
        "unwrap_or",
        "unwrap_or_default",
        "unwrap_or_else",
        "map_or",
        "map_or_else",
        "is_some",
        "is_none",
        "as_mut",
        "as_ref",
        "iter",
        "collect",
        "Toast",
        "Some",
    ];
    let b = s.as_bytes();
    let mut out = Vec::new();
    for (i, _) in s.match_indices('(') {
        let mut j = i;
        while j > 0 && (b[j - 1].is_ascii_alphanumeric() || b[j - 1] == b'_') {
            j -= 1;
        }
        let nome = &s[j..i];
        if nome.len() >= 5 && !COMUNS.contains(&nome) && !out.iter().any(|o| o == nome) {
            out.push(nome.to_string());
        }
    }
    out
}

/// O nome da função que começa em `s` (`fn nome…`).
pub(super) fn nome_da_fn(s: &str) -> Option<&str> {
    let r = s.strip_prefix("fn ")?;
    let fim = r.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    (fim > 0).then(|| &r[..fim])
}

/// Os identificadores em MAIÚSCULAS (constantes) de um pedaço de fonte.
pub(super) fn maiusculas(s: &str) -> Vec<String> {
    s.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| {
            w.len() >= 3
                && w.chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
                && w.chars().any(|c| c.is_ascii_uppercase())
        })
        .map(str::to_string)
        .collect()
}

/// A definição (até ao `;`) de cada `const NOME`, por nome.
pub(super) fn definicoes_de_consts(textos: &[(PathBuf, String)]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, todo) in textos {
        let s = sem_testes(todo);
        for (i, _) in s.match_indices("const ") {
            let r = &s[i + 6..];
            let fim = r
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(r.len());
            let Some(pv) = r.find(';') else { continue };
            out.entry(r[..fim].to_string())
                .or_default()
                .push(r[..pv].to_string());
        }
    }
    out
}

/// O corpo (com chavetas equilibradas) de cada `fn nome` dos fontes, por nome.
pub(super) fn corpos_das_funcoes(textos: &[(PathBuf, String)]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, todo) in textos {
        let s = sem_testes(todo);
        for (i, _) in s.match_indices("fn ") {
            let resto = &s[i + 3..];
            let fim = resto
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(resto.len());
            let nome = &resto[..fim];
            let Some(a) = resto.find('{') else { continue };
            if resto[..a].contains(';') {
                continue;
            }
            let corpo = chavetas(&resto[a..]);
            out.entry(nome.to_string())
                .or_default()
                .push(corpo.to_string());
        }
    }
    out
}

pub(super) fn chavetas(s: &str) -> &str {
    let b = s.as_bytes();
    let (mut fundo, mut i) = (0i32, 0);
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'{' => fundo += 1,
            b'}' => {
                fundo -= 1;
                if fundo == 0 {
                    return &s[..=i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    s
}
