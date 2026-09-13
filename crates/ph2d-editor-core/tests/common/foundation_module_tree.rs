//! O LEITOR da árvore de módulos da `ph2d-editor-core` — a régua do gate
//! `architecture_the_foundation_modules_form_a_dag`, num ficheiro à parte pelo tecto de LOC.
//!
//! Dep-free (std only). Tudo o que ele lê, lê em CÓDIGO: comentários (e literais de string e de
//! carácter) em branco, com os offsets e as quebras de linha preservados.
//!
//! # As formas que ele vê
//!
//! - **A árvore**, a partir do `lib.rs`: `mod x;` com ou sem `#[path]` — os atributos podem estar
//!   separados do `mod` por doc-comments ou linhas em branco — e `mod x;` DENTRO de um `mod y { }`
//!   inline (o ficheiro fica em `…/y/x.rs`, e um `#[path]` de lá é relativo a `…/y/`).
//! - **As re-exportações da raiz, em TODA forma**: qualquer visibilidade (um `use` privado no
//!   `lib.rs` também é visível à crate inteira), atributos à frente, `crate::`/`self::` à frente,
//!   grupos aninhados, `as`, `{self as x}` — e as GLOBS de um módulo de topo, que o gate recusa.
//! - **As referências**: `crate::X`, cada item de `crate::{…}`, `super::…::X` e `super::…::{…}` que
//!   sobem à raiz (com a profundidade dos `mod` inline), e — num escopo com uma glob da raiz
//!   (`use crate::*`, ou um `use super::*` que sobe até lá) — o caminho SOLTO `widget::X`.
//! - **Os literais que enganam um leitor ingénuo**: `'\''` (o carácter a seguir à barra é do
//!   literal, nunca o fecho) e `br#"…"#`/`cr"…"` (o prefixo de bytes ou de C não esconde o `r`).
//!
//! ⚠️ As quatro últimas formas de cada lista foram achadas CEGAS pela auditoria de fecho da
//! A10 (2026-09-12) — nenhuma aparecia na árvore desse dia, e é por isso que cada uma tem sentinela.
//!
//! ⛔ **O que ele NÃO vê, nomeado:** um item re-exportado usado SOLTO (`Toast`, sem caminho) num
//! escopo com glob da raiz; um `#[path]` partido em várias linhas (`#[cfg_attr(\n…)]`); e um módulo
//! filho com o MESMO nome de um módulo de topo, num escopo com glob da raiz (lê-se como o de topo).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Um `r"…"`/`r#"…"#` abre em `i`? O `r` não pode fechar um identificador — excepto quando é o
/// prefixo de um literal de bytes ou de C (`br"…"`, `cr#"…"#`).
fn abre_raw(b: &[u8], i: usize) -> bool {
    let livre = |k: usize| k == 0 || !is_ident(b[k - 1]);
    livre(i) || (i >= 1 && matches!(b[i - 1], b'b' | b'c') && livre(i - 1))
}

/// Comentários (e, com `strings = true`, literais de string e de carácter) em branco, com os
/// offsets e as quebras de linha preservados.
pub(crate) fn blank(src: &str, strings: bool) -> String {
    let b = src.as_bytes();
    let n = b.len();
    let mut out = b.to_vec();
    let apaga = |out: &mut Vec<u8>, a: usize, e: usize| {
        for x in out.iter_mut().take(e.min(n)).skip(a) {
            if *x != b'\n' {
                *x = b' ';
            }
        }
    };
    let mut i = 0;
    while i < n {
        let c = b[i];
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let e = src[i..].find('\n').map_or(n, |x| i + x);
            apaga(&mut out, i, e);
            i = e;
        } else if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let (mut depth, mut j) = (1, i + 2);
            while j < n && depth > 0 {
                if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            apaga(&mut out, i, j);
            i = j;
        } else if c == b'r' && i + 1 < n && (b[i + 1] == b'"' || b[i + 1] == b'#') && abre_raw(b, i)
        {
            let mut j = i + 1;
            while j < n && b[j] == b'#' {
                j += 1;
            }
            if j < n && b[j] == b'"' {
                let fecho = format!("\"{}", "#".repeat(j - i - 1));
                let e = src[j + 1..]
                    .find(&fecho)
                    .map_or(n, |x| j + 1 + x + fecho.len());
                if strings {
                    apaga(&mut out, i, e);
                }
                i = e;
            } else {
                i += 1;
            }
        } else if c == b'"' {
            let mut j = i + 1;
            while j < n && b[j] != b'"' {
                if b[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            let e = (j + 1).min(n);
            if strings {
                apaga(&mut out, i, e);
            }
            i = e;
        } else if c == b'\'' {
            // Um literal de carácter (`'x'`, `'\n'`, `'\''`, `'é'`) — ou um tempo de vida (`'a`), que
            // não fecha. ⚠️ Numa escapa, o carácter a seguir à barra é do literal: procurar o fecho a
            // partir DELE fazia `'\''` fechar na aspa escapada, e a aspa real abria um literal falso.
            let e = if i + 1 < n && b[i + 1] == b'\\' {
                (i + 3 <= n && src.is_char_boundary(i + 3))
                    .then(|| src[i + 3..].find('\'').map(|x| i + 3 + x + 1))
                    .flatten()
            } else {
                src[i + 1..].chars().next().and_then(|ch| {
                    let fim = i + 1 + ch.len_utf8();
                    (fim < n && b[fim] == b'\'').then_some(fim + 1)
                })
            };
            match e {
                Some(e) => {
                    if strings {
                        apaga(&mut out, i, e);
                    }
                    i = e;
                }
                None => i += 1,
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("só bytes ASCII foram escritos")
}

pub(crate) fn ident_at(s: &str, i: usize) -> Option<&str> {
    let b = s.as_bytes();
    let e = (i..b.len()).find(|&k| !is_ident(b[k])).unwrap_or(b.len());
    (e > i && !b[i].is_ascii_digit()).then(|| &s[i..e])
}

/// `(abre, fecha, nome)` de cada `mod NOME {` inline — o que está dentro dele é um nível mais fundo.
pub(crate) fn inline_mods(cod: &str) -> Vec<(usize, usize, String)> {
    let b = cod.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = cod[i..].find("mod ") {
        let at = i + p;
        i = at + 4;
        if at > 0 && is_ident(b[at - 1]) {
            continue;
        }
        let Some(nome) = ident_at(cod, at + 4) else {
            continue;
        };
        let mut j = at + 4 + nome.len();
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < b.len() && b[j] == b'{' {
            let mut depth = 0usize;
            for (k, &ch) in b.iter().enumerate().skip(j) {
                if ch == b'{' {
                    depth += 1;
                } else if ch == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        out.push((j, k, nome.to_owned()));
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Onde começam os atributos do item cuja linha contém `at`: sobe pelas linhas que são `#[…]` — e
/// pelas que ficam VAZIAS sem os comentários (um doc-comment entre o `#[path]` e o `mod` não os
/// separa).
fn atributos_acima(sem_comentarios: &str, at: usize) -> usize {
    let mut ini = sem_comentarios[..at].rfind('\n').map_or(0, |x| x + 1);
    let mut inicio = ini;
    while ini > 0 {
        let prev = sem_comentarios[..ini - 1].rfind('\n').map_or(0, |x| x + 1);
        let linha = sem_comentarios[prev..ini - 1].trim();
        if linha.starts_with("#[") {
            inicio = prev;
        } else if !linha.is_empty() {
            break;
        }
        ini = prev;
    }
    inicio
}

/// `(ficheiro, caminho de módulo)` de todo ficheiro que a árvore alcança a partir do `lib.rs`.
pub(crate) fn arvore(src: &Path) -> Vec<(PathBuf, Vec<String>)> {
    let lib = src.join("lib.rs");
    let mut out = vec![(lib.clone(), Vec::new())];
    let mut fila = vec![(lib, Vec::<String>::new())];
    let mut vistos = BTreeSet::new();
    while let Some((f, caminho)) = fila.pop() {
        if !vistos.insert(f.clone()) {
            continue;
        }
        let texto = std::fs::read_to_string(&f).unwrap_or_default();
        let sem_comentarios = blank(&texto, false);
        let cod = blank(&texto, true);
        let dentro = inline_mods(&cod);
        let dir = f.parent().expect("dir").to_path_buf();
        let nome_f = f.file_name().and_then(|x| x.to_str()).unwrap_or("");
        let sob = if nome_f == "lib.rs" || nome_f == "mod.rs" {
            dir.clone()
        } else {
            dir.join(nome_f.trim_end_matches(".rs"))
        };
        let b = cod.as_bytes();
        let mut i = 0;
        while let Some(p) = cod[i..].find("mod ") {
            let at = i + p;
            i = at + 4;
            if at > 0 && is_ident(b[at - 1]) {
                continue;
            }
            let Some(nome) = ident_at(&cod, at + 4) else {
                continue;
            };
            let mut j = at + 4 + nome.len();
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= b.len() || b[j] != b';' {
                continue;
            }
            // Os `mod { }` inline que envolvem este, de fora para dentro: são directórios do caminho.
            let mut envolventes: Vec<&(usize, usize, String)> = dentro
                .iter()
                .filter(|(a, e, _)| at > *a && at < *e)
                .collect();
            envolventes.sort_by_key(|(a, _, _)| *a);
            let base = envolventes
                .iter()
                .fold(sob.clone(), |d, (_, _, m)| d.join(m));
            let attrs = &sem_comentarios[atributos_acima(&sem_comentarios, at)..at];
            let alvo = attrs.find("#[path").and_then(|k| {
                let r = &attrs[k..];
                let a = r.find('"')? + 1;
                let e = r[a..].find('"')? + a;
                // Fora de um `mod { }` inline o `#[path]` é relativo ao DIRECTÓRIO do ficheiro; dentro
                // de um, ao directório do módulo inline (a referência do Rust, «The path attribute»).
                Some(if envolventes.is_empty() {
                    dir.join(&r[a..e])
                } else {
                    base.join(&r[a..e])
                })
            });
            let cands = match alvo {
                Some(p) => vec![p],
                None => vec![
                    base.join(format!("{nome}.rs")),
                    base.join(nome).join("mod.rs"),
                ],
            };
            if let Some(c) = cands.into_iter().find(|c| c.is_file()) {
                let mut filho = caminho.clone();
                filho.extend(envolventes.iter().map(|(_, _, m)| m.clone()));
                filho.push(nome.to_owned());
                out.push((c.clone(), filho.clone()));
                fila.push((c, filho));
            }
        }
    }
    out
}

/// As folhas de uma árvore de `use` — `(caminho completo, alias)`: `a::{b, c::{d as e}}` dá
/// `[a, b]` e `[a, c, d] as e`.
fn folhas(arv: &str, prefixo: &[String], out: &mut Vec<(Vec<String>, Option<String>)>) {
    let s = arv.trim();
    if let Some(k) = s.find('{') {
        let mut depth = 0usize;
        let mut fim = s.len();
        for (q, ch) in s.bytes().enumerate().skip(k) {
            if ch == b'{' {
                depth += 1;
            } else if ch == b'}' {
                depth -= 1;
                if depth == 0 {
                    fim = q;
                    break;
                }
            }
        }
        let mut base = prefixo.to_vec();
        base.extend(
            s[..k]
                .split("::")
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_owned),
        );
        let corpo = &s[k + 1..fim];
        let (mut nivel, mut ini) = (0usize, 0);
        for (q, ch) in corpo.bytes().enumerate() {
            match ch {
                b'{' => nivel += 1,
                b'}' => nivel = nivel.saturating_sub(1),
                b',' if nivel == 0 => {
                    folhas(&corpo[ini..q], &base, out);
                    ini = q + 1;
                }
                _ => {}
            }
        }
        folhas(&corpo[ini..], &base, out);
    } else if !s.is_empty() {
        let partes: Vec<&str> = s.split_whitespace().collect();
        let (caminho, alias) = match partes.as_slice() {
            [c, "as", a] => (*c, Some((*a).to_owned())),
            _ => (s, None),
        };
        let mut segs = prefixo.to_vec();
        segs.extend(
            caminho
                .split("::")
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_owned),
        );
        out.push((segs, alias));
    }
}

/// O que o `lib.rs` põe na RAIZ vindo de um módulo de topo: `(nome → módulo, globs)`. Toda forma
/// de `use` a profundidade zero conta — qualquer visibilidade, com ou sem atributos à frente.
pub(crate) fn reexportacoes(
    lib: &str,
    tops: &BTreeSet<String>,
) -> (BTreeMap<String, String>, Vec<String>) {
    let cod = blank(lib, true);
    let b = cod.as_bytes();
    let mut prof = Vec::with_capacity(b.len());
    let mut d = 0usize;
    for &ch in b {
        prof.push(d);
        if ch == b'{' {
            d += 1;
        } else if ch == b'}' {
            d = d.saturating_sub(1);
        }
    }
    let mut nomes = BTreeMap::new();
    let mut globs = Vec::new();
    let mut i = 0;
    while let Some(p) = cod[i..].find("use ") {
        let at = i + p;
        i = at + 4;
        if (at > 0 && is_ident(b[at - 1])) || prof[at] != 0 {
            continue;
        }
        let Some(fim) = cod[at..].find(';') else {
            break;
        };
        let mut fs = Vec::new();
        folhas(&cod[at + 4..at + fim], &[], &mut fs);
        for (caminho, alias) in fs {
            let segs = match caminho.first().map(String::as_str) {
                Some("crate" | "self") => &caminho[1..],
                _ => &caminho[..],
            };
            let Some(topo) = segs.first().filter(|s| tops.contains(s.as_str())) else {
                continue;
            };
            match segs.last().map(String::as_str) {
                Some("*") => globs.push(topo.clone()),
                Some("self") => {
                    if let Some(a) = alias {
                        nomes.insert(a, topo.clone());
                    }
                }
                Some(ultimo) if segs.len() > 1 || alias.is_some() => {
                    nomes.insert(alias.unwrap_or_else(|| ultimo.to_owned()), topo.clone());
                }
                _ => {}
            }
        }
        i = at + fim;
    }
    (nomes, globs)
}

/// O primeiro segmento de cada item de topo de um grupo `{a::X, b, c::{d, e}}` que abre em `abre`.
fn grupo(cod: &str, abre: usize) -> Vec<String> {
    let b = cod.as_bytes();
    let mut out = Vec::new();
    let (mut nivel, mut ini) = (0usize, abre + 1);
    for (q, &ch) in b.iter().enumerate().skip(abre + 1) {
        match ch {
            b'{' => nivel += 1,
            b'}' if nivel > 0 => nivel -= 1,
            b',' | b'}' if nivel == 0 => {
                let bruto = &cod[ini..q];
                let off = ini + (bruto.len() - bruto.trim_start().len());
                if let Some(nome) = ident_at(cod, off) {
                    out.push(nome.to_owned());
                }
                if ch == b'}' {
                    break;
                }
                ini = q + 1;
            }
            _ => {}
        }
    }
    out
}

/// `(offset, nome)` de cada referência que chega à RAIZ num ficheiro cujo caminho de módulo é
/// `caminho`. O chamador filtra pelos módulos de topo e pelas re-exportações.
pub(crate) fn referencias(texto: &str, caminho: &[String]) -> Vec<(usize, String)> {
    let cod = blank(texto, true);
    let b = cod.as_bytes();
    let dentro = inline_mods(&cod);
    // O escopo de um offset: o `mod { }` inline mais interno que o contém (ou o ficheiro).
    let escopo = |at: usize| {
        dentro
            .iter()
            .enumerate()
            .filter(|(_, (a, e, _))| at > *a && at < *e)
            .max_by_key(|(_, (a, _, _))| *a)
            .map(|(k, _)| k)
    };
    let precede = |at: usize| at > 0 && (is_ident(b[at - 1]) || b[at - 1] == b':');
    // O que vem depois de um prefixo que chegou à raiz: um grupo, uma glob, ou um nome.
    let depois =
        |k: usize, at: usize, out: &mut Vec<(usize, String)>, globs: &mut Vec<Option<usize>>| {
            match b.get(k) {
                Some(b'{') => out.extend(grupo(&cod, k).into_iter().map(|n| (at, n))),
                Some(b'*') => globs.push(escopo(at)),
                _ => {
                    if let Some(nome) = ident_at(&cod, k) {
                        out.push((at, nome.to_owned()));
                    }
                }
            }
        };
    let mut out = Vec::new();
    let mut globs = Vec::new();
    let mut i = 0;
    while let Some(p) = cod[i..].find("crate::") {
        let at = i + p;
        i = at + 7;
        if !precede(at) {
            depois(at + 7, at, &mut out, &mut globs);
        }
    }
    let mut i = 0;
    while let Some(p) = cod[i..].find("super::") {
        let at = i + p;
        i = at + 7;
        if precede(at) {
            continue;
        }
        let mut n = 0;
        let mut k = at;
        while cod[k..].starts_with("super::") {
            n += 1;
            k += 7;
        }
        i = k;
        let profundidade =
            caminho.len() + dentro.iter().filter(|(a, e, _)| at > *a && at < *e).count();
        if n == profundidade {
            depois(k, at, &mut out, &mut globs);
        }
    }
    // Num escopo com uma glob da raiz, o caminho solto `widget::X` é a raiz a falar.
    if !globs.is_empty() {
        let mut i = 0;
        while let Some(p) = cod[i..].find("::") {
            let fim = i + p;
            i = fim + 2;
            let mut ini = fim;
            while ini > 0 && is_ident(b[ini - 1]) {
                ini -= 1;
            }
            if ini == fim || (ini > 0 && b[ini - 1] == b':') {
                continue;
            }
            let nome = &cod[ini..fim];
            if !matches!(nome, "crate" | "super" | "self" | "Self") && globs.contains(&escopo(ini))
            {
                out.push((ini, nome.to_owned()));
            }
        }
    }
    out
}
