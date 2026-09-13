//! ⭐⭐ **TODA crate da workspace herda os lints da workspace — e o `unsafe` é PROIBIDO nelas.**
//!
//! Auditoria de arquitectura 2026-09-12, achado A2
//! (`docs/archive/integracao-jornadas/AUDITORIA_ARQUITETURA_2026-09-12.md`).
//!
//! # O defeito que isto fecha, medido
//!
//! A regra *«`#![forbid(unsafe_code)]` na primeira linha do `lib.rs`»* vivia na DIRETRIZ e na
//! memória de quem criava a crate — e nenhum gate a lia. Por mês de nascimento, com / sem o
//! atributo: **maio 82/0 · junho 3/1 · julho 152/6 · agosto 51/13 · setembro 18/23**. Entre as 23
//! de setembro estavam **7 famílias da W2 (277 644 L)** que MORAVAM na shell, que tem o `forbid`:
//! o código perdeu a garantia ao atravessar a fronteira, e nasceram nelas dois
//! `unsafe { std::env::set_var(..) }` que dentro da shell não compilariam. *Um atributo de crate
//! não viaja com o código* — a mesma lei da feature (HOWTO §2).
//!
//! # A cura é UMA fonte, e ela cobre o que o atributo nunca cobriu
//!
//! `[workspace.lints.rust] unsafe_code = "forbid"` na raiz e `[lints] workspace = true` em cada
//! membro. O lint da workspace chega a **todos os alvos** — `tests/`, `benches/`, `examples/`,
//! `build.rs` —, onde o `#![forbid]` do `lib.rs` nunca chegou: a medição achou `unsafe` em QUATRO
//! alvos fora de `src/` (uma sonda, um teste da shell, um exemplo e um bench).
//!
//! ⚠️ **O cargo recusa `workspace = true` com overrides na mesma tabela** (medido na 1.98:
//! *«cannot override `workspace.lints` in `lints`»*), logo uma excepção **não herda** e declara os
//! seus — e é por isso que ela tem de estar NOMEADA aqui, com o motivo e os módulos.
//!
//! # As metades
//!
//! 1. a raiz proíbe (sentinela: sem ela, herdar não proíbe nada e todo o resto fica verde);
//! 2. todo membro herda, salvo [`EXCECOES`], e toda excepção ainda é membro (obsolescência);
//! 3. toda excepção **não** herda, declara `unsafe_code = "deny"`, e o `#![allow(unsafe_code)]`
//!    mora **exactamente** nos módulos que a lista nomeia (confinamento);
//! 4. o leitor de manifesto sabe dizer «não» (controlos sintéticos) e o censo tem piso.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// As crates que existem para ISOLAR uma ABI C — o `unsafe` é a razão de existirem.
/// `(nome, módulos que autorizam unsafe por escrito, motivo)`.
const EXCECOES: &[(&str, &[&str], &str)] = &[
    (
        "ph2d-audio-opus",
        &["src/decoder.rs", "src/encoder.rs"],
        "o único codificador Opus sem `libopus` de sistema nos três SO é o `unsafe-libopus` (a \
         libopus transpilada por c2rust), que expõe a ABI C crua — ADR-0116",
    ),
    (
        "ph2d-imageio-avif",
        &["src/decode.rs", "src/encode.rs"],
        "FFI da `libavif` (`libavif-sys`): decode e encode por ponteiro",
    ),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn rel(base: &Path, p: &Path) -> String {
    p.strip_prefix(base)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// O corpo de uma tabela TOML (`[x]`) até ao cabeçalho seguinte — `[lints]` pára em `[lints.rust]`.
fn table(toml: &str, header: &str) -> String {
    let mut dentro = false;
    let mut out = String::new();
    for l in toml.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            dentro = t == header;
            continue;
        }
        if dentro {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

/// `chave = valor` numa tabela, com o comentário de fim de linha arrancado.
fn key_is(corpo: &str, chave: &str, valor: &str) -> bool {
    corpo.lines().any(|l| {
        let sem = l.split('#').next().unwrap_or("");
        let Some((k, v)) = sem.split_once('=') else {
            return false;
        };
        k.trim() == chave && v.trim() == valor
    })
}

fn herda(toml: &str) -> bool {
    key_is(&table(toml, "[lints]"), "workspace", "true")
}

fn package_name(toml: &str) -> Option<String> {
    table(toml, "[package]").lines().find_map(|l| {
        let (k, v) = l.split('#').next()?.split_once('=')?;
        (k.trim() == "name").then(|| v.trim().trim_matches('"').to_owned())
    })
}

fn toml_string_array(corpo_da_tabela: &str, chave: &str) -> Vec<String> {
    let linhas: Vec<&str> = corpo_da_tabela.lines().collect();
    let Some(i) = linhas.iter().position(|l| {
        l.trim_start()
            .strip_prefix(chave)
            .is_some_and(|r| r.trim_start().starts_with('='))
    }) else {
        return Vec::new();
    };
    let mut buf = String::new();
    for l in &linhas[i..] {
        let sem_comentario = l.split('#').next().unwrap_or("");
        buf.push_str(sem_comentario);
        buf.push('\n');
        if sem_comentario.contains(']') {
            break;
        }
    }
    let corpo = buf.split_once('[').map_or("", |(_, r)| r);
    let corpo = corpo.split(']').next().unwrap_or("");
    corpo
        .split(',')
        .filter_map(|s| {
            let s = s.trim().trim_matches(|c| c == '"' || c == '\'');
            (!s.is_empty()).then(|| s.to_owned())
        })
        .collect()
}

/// Os directórios dos membros: `<dir>/*` expandido, caminhos explícitos, `exclude` retirado.
fn members(root: &Path) -> Vec<PathBuf> {
    let toml = fs::read_to_string(root.join("Cargo.toml")).expect("o Cargo.toml da raiz");
    let ws = table(&toml, "[workspace]");
    let padroes = toml_string_array(&ws, "members");
    assert!(
        padroes.iter().any(|p| p == "crates/*"),
        "controlo do parser: `[workspace] members` não mostrou `crates/*` — leu {padroes:?}"
    );
    let excluidos: BTreeSet<PathBuf> = toml_string_array(&ws, "exclude")
        .iter()
        .filter_map(|e| fs::canonicalize(root.join(e)).ok())
        .collect();
    let mut out = BTreeSet::new();
    for p in &padroes {
        if let Some(dir) = p.strip_suffix("/*") {
            let rd = fs::read_dir(root.join(dir)).unwrap_or_else(|e| panic!("`{dir}`: {e}"));
            for e in rd.flatten() {
                let d = e.path();
                if d.join("Cargo.toml").is_file()
                    && let Ok(c) = fs::canonicalize(&d)
                {
                    out.insert(c);
                }
            }
        } else if let Ok(c) = fs::canonicalize(root.join(p)) {
            out.insert(c);
        }
    }
    out.retain(|d| !excluidos.contains(d));
    out.into_iter().collect()
}

fn rs_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, cb);
        } else if p.extension().is_some_and(|x| x == "rs") {
            cb(&p);
        }
    }
}

#[test]
fn every_member_inherits_the_workspace_lints() {
    let root = root();
    let raiz = fs::read_to_string(root.join("Cargo.toml")).expect("o Cargo.toml da raiz");
    assert!(
        key_is(
            &table(&raiz, "[workspace.lints.rust]"),
            "unsafe_code",
            "\"forbid\""
        ),
        "a raiz deixou de declarar `[workspace.lints.rust] unsafe_code = \"forbid\"` — herdar passa \
         a não proibir NADA, e a metade abaixo ficaria verde sobre o vácuo"
    );

    let membros = members(&root);
    assert!(
        membros.len() >= 300,
        "li só {} membros — o leitor de `[workspace] members` partiu-se e este gate ficaria verde \
         por vácuo",
        membros.len()
    );

    let excecoes: BTreeSet<&str> = EXCECOES.iter().map(|(n, _, _)| *n).collect();
    let mut vistos = BTreeSet::new();
    let mut sem_heranca = Vec::new();
    for d in &membros {
        let toml = fs::read_to_string(d.join("Cargo.toml")).expect("o manifesto do membro");
        let nome = package_name(&toml)
            .unwrap_or_else(|| panic!("{}: sem `[package] name`", rel(&root, d)));
        vistos.insert(nome.clone());
        if !excecoes.contains(nome.as_str()) && !herda(&toml) {
            sem_heranca.push(format!("{nome}  ({})", rel(&root, d)));
        }
    }
    assert!(
        sem_heranca.is_empty(),
        "estas crates NÃO herdam os lints da workspace — o `unsafe` fica PERMITIDO nelas, em todo \
         alvo:\n  {}\n\ncura: no fim do `Cargo.toml` delas,\n    [lints]\n    workspace = true\n\
         (se a crate EXISTE para isolar uma ABI C, ela entra em `EXCECOES` neste ficheiro, com o \
         motivo e os módulos)",
        sem_heranca.join("\n  ")
    );
    for (n, _, _) in EXCECOES {
        assert!(
            vistos.contains(*n),
            "`{n}` está em `EXCECOES` e já não é membro da workspace — apague a linha"
        );
    }
}

#[test]
fn every_exception_denies_unsafe_and_confines_it_to_the_modules_it_names() {
    let root = root();
    for (nome, modulos, motivo) in EXCECOES {
        assert!(
            !motivo.trim().is_empty(),
            "`{nome}`: uma excepção sem motivo"
        );
        let dir = root.join("crates").join(nome);
        let toml = fs::read_to_string(dir.join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("`{nome}`: {e} — apague-a de `EXCECOES`"));
        assert!(
            !herda(&toml),
            "`{nome}` passou a herdar os lints da workspace — deixou de ser excepção: apague a linha \
             de `EXCECOES` (e os `#![allow(unsafe_code)]` dela deixam de compilar, que é o ponto)"
        );
        assert!(
            key_is(&table(&toml, "[lints.rust]"), "unsafe_code", "\"deny\""),
            "`{nome}` não herda e não declara `[lints.rust] unsafe_code = \"deny\"` — o `unsafe` \
             fica solto na crate inteira"
        );
        let mut autorizam = BTreeSet::new();
        rs_files(&dir.join("src"), &mut |f| {
            let t = fs::read_to_string(f).expect("um .rs da excepção");
            if t.lines().any(|l| l.trim() == "#![allow(unsafe_code)]") {
                autorizam.insert(rel(&dir, f));
            }
        });
        let esperados: BTreeSet<String> = modulos.iter().map(|m| (*m).to_owned()).collect();
        assert_eq!(
            autorizam, esperados,
            "`{nome}`: os módulos que autorizam `unsafe` por escrito têm de ser EXACTAMENTE os que \
             `EXCECOES` nomeia"
        );
    }
}

/// ⚠️ **E o leitor sabe dizer «não».** Sem isto, um `herda` que devolvesse sempre `true` faria o
/// gate de cima passar sobre qualquer manifesto.
#[test]
fn the_manifest_reader_can_say_no() {
    assert!(herda(
        "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n"
    ));
    assert!(herda("[lints]\nworkspace=true # comentário\n"));
    assert!(!herda("[package]\nname = \"a\"\n"), "sem a tabela");
    assert!(!herda("[lints]\nworkspace = false\n"), "falso");
    assert!(
        !herda("[lints.rust]\nworkspace = true\n"),
        "a chave noutra tabela não conta"
    );
    assert!(
        !herda("[package.metadata]\nworkspace = true\n[lints]\n"),
        "a chave antes da tabela não conta"
    );
    assert!(
        !herda("# [lints]\n# workspace = true\n"),
        "comentado não conta"
    );
    assert!(key_is(
        &table("[lints.rust]\nunsafe_code = \"deny\"\n", "[lints.rust]"),
        "unsafe_code",
        "\"deny\""
    ));
    assert!(!key_is(
        &table("[lints]\nunsafe_code = \"deny\"\n", "[lints.rust]"),
        "unsafe_code",
        "\"deny\""
    ));
    assert_eq!(
        package_name("[package]\nname = \"ph2d-x\"\nversion.workspace = true\n").as_deref(),
        Some("ph2d-x")
    );
    assert_eq!(
        toml_string_array(
            "members = [\n  \"crates/*\", # x\n  \"tools/*\",\n]\n",
            "members"
        ),
        vec!["crates/*".to_owned(), "tools/*".to_owned()]
    );
}
