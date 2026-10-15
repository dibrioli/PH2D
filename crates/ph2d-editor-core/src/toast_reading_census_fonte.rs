//! Ler o FONTE dos chamadores para o censo dos avisos (`toast_reading_census`): os ficheiros, os
//! argumentos, os literais, as chamadas, as consts e os corpos das funções. Saiu do censo pelo tecto
//! de LOC (doc 132).

use ph2d_i18n::{Idioma, tr_em};
use std::collections::{BTreeMap, BTreeSet};
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

/// Os corpos de cada `fn nome`, com o ficheiro de onde vêm.
pub(super) type Corpos = BTreeMap<String, Vec<(PathBuf, String)>>;

/// O corpo (com chavetas equilibradas) de cada `fn nome` dos fontes, por nome.
pub(super) fn corpos_das_funcoes(textos: &[(PathBuf, String)]) -> Corpos {
    let mut out: Corpos = BTreeMap::new();
    for (p, todo) in textos {
        let s = sem_testes(todo);
        for (i, _) in s.match_indices("fn ") {
            let resto = &s[i + 3..];
            let fim = resto
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(resto.len());
            let nome = &resto[..fim];
            let Some(a) = resto.find('{') else { continue };
            // Um `;` fora de `[]`/`()` é uma declaração sem corpo; o de `[f32; 2]` não é.
            let mut fundo = 0i32;
            let declaracao = resto[..a].bytes().any(|c| {
                match c {
                    b'[' | b'(' => fundo += 1,
                    b']' | b')' => fundo -= 1,
                    _ => {}
                }
                c == b';' && fundo == 0
            });
            if declaracao {
                continue;
            }
            let corpo = chavetas(&resto[a..]);
            out.entry(nome.to_string())
                .or_default()
                .push((p.clone(), corpo.to_string()));
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

/// O índice depois do `"` que fecha o texto que abre em `i`.
fn fim_do_texto(b: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < b.len() && b[i] != b'"' {
        if b[i] == b'\\' {
            i += 1;
        }
        i += 1;
    }
    i + 1
}

fn e_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

const PALAVRAS: &[&str] = &[
    "let", "mut", "ref", "in", "for", "if", "else", "match", "as", "move", "return", "true",
    "false", "self", "crate", "super", "while", "loop", "fn", "impl", "use", "pub", "dyn", "_",
];

/// As variáveis de um pedaço de fonte: identificadores em minúsculas que não são chamada, campo
/// (`a.b`), caminho (`a::b`), macro nem palavra da língua.
fn variaveis(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            i = fim_do_texto(b, i);
        } else if b[i].is_ascii_lowercase() || b[i] == b'_' {
            let ini = i;
            while i < b.len() && e_ident(b[i]) {
                i += 1;
            }
            let antes = ini.checked_sub(1).map(|j| b[j]);
            let w = &s[ini..i];
            let resto = &s[i..];
            if !antes.is_some_and(|c| e_ident(c) || c == b'.' || c == b':' || c == b'\'')
                && !(resto.starts_with('(') || resto.starts_with("::") || resto.starts_with('!'))
                && !PALAVRAS.contains(&w)
                && !out.iter().any(|o| o == w)
            {
                out.push(w.to_string());
            }
        } else {
            i += 1;
        }
    }
    out
}

/// A variável de que uma expressão PARTE (`lines.into_iter()` → `lines`), se parte de uma.
fn receptor(e: &str) -> Option<String> {
    let t = e
        .trim_start_matches(['&', '*', ' '])
        .trim_start_matches("mut ");
    let fim = t.bytes().position(|c| !e_ident(c)).unwrap_or(t.len());
    let (w, resto) = t.split_at(fim);
    let livre = resto.is_empty() || resto.starts_with(['.', '?', ' ']);
    (w.starts_with(|c: char| c.is_ascii_lowercase()) && livre && !PALAVRAS.contains(&w))
        .then(|| w.to_string())
}

/// Do `let`/`for` que acaba em `de`, onde acaba o padrão e onde começa a expressão.
fn separa(s: &str, de: usize, e_for: bool) -> Option<(usize, usize)> {
    let b = s.as_bytes();
    let (mut fundo, mut j) = (0i32, de);
    while j < b.len() {
        match b[j] {
            b'"' => {
                j = fim_do_texto(b, j);
                continue;
            }
            b'(' | b'[' | b'{' => fundo += 1,
            b')' | b']' | b'}' => {
                fundo -= 1;
                if fundo < 0 {
                    return None;
                }
            }
            b';' if fundo == 0 => return None,
            b' ' if fundo == 0 && e_for && s[j..].starts_with(" in ") => return Some((j, j + 4)),
            b'=' if fundo == 0
                && !e_for
                && !matches!(b.get(j + 1), Some(b'=' | b'>'))
                && !matches!(b[j - 1], b'=' | b'!' | b'<' | b'>') =>
            {
                return Some((j, j + 1));
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// A expressão que começa em `de`, até ao `;` ou à `{` do bloco que ela abre.
fn expressao(s: &str, de: usize) -> &str {
    let b = s.as_bytes();
    let (mut fundo, mut j) = (0i32, de);
    while j < b.len() {
        match b[j] {
            b'"' => {
                j = fim_do_texto(b, j);
                continue;
            }
            b'(' | b'[' => fundo += 1,
            b'{' if fundo > 0 => fundo += 1,
            b')' | b']' | b'}' if fundo > 0 => fundo -= 1,
            b';' | b'{' | b'}' | b')' | b']' => break,
            _ => {}
        }
        j += 1;
    }
    s[de..j.min(b.len())].trim()
}

/// Cada `let`/`for` de um corpo, por ordem: o padrão e a expressão que o liga.
fn ligacoes_do_corpo(s: &str) -> Vec<(&str, &str)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            i = fim_do_texto(b, i);
            continue;
        }
        if s[i..].starts_with("//") {
            i += s[i..].find('\n').unwrap_or(s.len() - i);
            continue;
        }
        let livre = i == 0 || !e_ident(b[i - 1]);
        let e_for = s[i..].starts_with("for ");
        if livre
            && (e_for || s[i..].starts_with("let "))
            && let Some((p, e)) = separa(s, i + 4, e_for)
        {
            out.push((&s[i + 4..p], expressao(s, e)));
        }
        i += 1;
    }
    out
}

/// ⭐ De onde vem o texto de um aviso cujo argumento é uma VARIÁVEL (`Toast::warning(msg)`): cada
/// variável segue até ao último `let`/`for`/`if let` do corpo que a liga, e a variável de que essa
/// expressão parte segue também (`line` ← `lines.into_iter()` ← `spawn_if_enabled(..)`).
pub(super) fn ligacoes<'a>(corpo: &'a str, arg: &str) -> Vec<&'a str> {
    let todas = ligacoes_do_corpo(corpo);
    let mut pendentes = variaveis(arg);
    let mut vistas = BTreeSet::new();
    let mut out = Vec::new();
    while let Some(v) = pendentes.pop() {
        if !vistas.insert(v.clone()) {
            continue;
        }
        if let Some(&(_, e)) = todas.iter().rev().find(|(p, _)| variaveis(p).contains(&v)) {
            out.push(e);
            pendentes.extend(receptor(e));
        }
    }
    out
}

/// As chamadas a funções LIVRES (não os métodos `.x(`), com o módulo que as qualifica:
/// `view_request::resolve(` → `(Some("view_request"), "resolve")`.
fn chamadas_livres(s: &str) -> Vec<(Option<&str>, &str)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    for (i, _) in s.match_indices('(') {
        let mut j = i;
        while j > 0 && e_ident(b[j - 1]) {
            j -= 1;
        }
        let nome = &s[j..i];
        if nome.is_empty() || !nome.starts_with(|c: char| c.is_ascii_lowercase()) {
            continue;
        }
        let modulo = if s[..j].ends_with("::") {
            let fim = j - 2;
            let mut k = fim;
            while k > 0 && e_ident(b[k - 1]) {
                k -= 1;
            }
            Some(&s[k..fim])
        } else if j > 0 && b[j - 1] == b'.' {
            continue;
        } else {
            None
        };
        out.push((modulo, nome));
    }
    out
}

/// O módulo de um ficheiro de fonte (`dispatch.rs` → `dispatch`; `x/mod.rs` → `x`).
fn modulo_de(p: &Path) -> &str {
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem == "mod" || stem == "lib" {
        p.parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .unwrap_or("")
    } else {
        stem
    }
}

/// O corpo que uma chamada alcança: pelo módulo quando ela o nomeia, senão no mesmo ficheiro,
/// senão o único com esse nome. Um homónimo ambíguo não se segue (`resolve` tem 258 corpos).
fn alcanca<'c>(
    corpos: &'c Corpos,
    onde: &Path,
    modulo: Option<&str>,
    nome: &str,
) -> Vec<&'c (PathBuf, String)> {
    let Some(todos) = corpos.get(nome) else {
        return Vec::new();
    };
    match modulo {
        Some(m) if !["crate", "self", "super", "Self"].contains(&m) => {
            todos.iter().filter(|(p, _)| modulo_de(p) == m).collect()
        }
        _ => {
            let aqui: Vec<_> = todos.iter().filter(|(p, _)| p == onde).collect();
            if aqui.is_empty() && todos.len() == 1 {
                todos.iter().collect()
            } else {
                aqui
            }
        }
    }
}

/// ⭐ Os corpos que uma expressão chama, e os que ESSES chamam, até `saltos` — com o salto em
/// que cada um foi alcançado.
pub(super) fn seguidos<'a>(
    corpos: &'a Corpos,
    de: &'a Path,
    expr: &'a str,
    saltos: usize,
) -> Vec<(usize, &'a str)> {
    let mut out = Vec::new();
    let mut vistos = BTreeSet::new();
    let mut fila = vec![(0, de, expr)];
    while let Some((n, onde, texto)) = fila.pop() {
        if n >= saltos {
            continue;
        }
        for (modulo, nome) in chamadas_livres(texto) {
            for (p, c) in alcanca(corpos, onde, modulo, nome) {
                if vistos.insert(c.as_ptr() as usize) {
                    out.push((n + 1, c.as_str()));
                    fila.push((n + 1, p.as_path(), c.as_str()));
                }
            }
        }
    }
    out
}

/// A prosa de um corpo que pode chegar a um aviso: os literais com espaço que não são chave,
/// fora do que só o terminal lê (`eprintln!`, `panic!`, `.expect(`…).
pub(super) fn prosa(corpo: &str) -> Vec<String> {
    const TERMINAL: &[&str] = &[
        "eprintln!(",
        "println!(",
        "panic!(",
        "unreachable!(",
        "assert!(",
        "assert_eq!(",
        "debug_assert!(",
        "debug_assert_eq!(",
        "warn!(",
        "info!(",
        "debug!(",
        "trace!(",
        ".expect(",
    ];
    let mut s = corpo.to_string();
    for t in TERMINAL {
        while let Some(i) = s.find(t) {
            let fim = i + t.len() - 1 + argumento(&s, i + t.len() - 1).len() + 2;
            s.replace_range(i..fim.min(s.len()), "");
        }
    }
    literais(&s)
        .into_iter()
        .filter(|l| l.contains(' ') && l.chars().any(char::is_alphabetic) && !e_chave(l))
        .collect()
}
