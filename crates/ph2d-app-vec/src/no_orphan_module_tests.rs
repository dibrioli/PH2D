//! ⛔⛔ **NENHUM FICHEIRO DESTA CRATE FICA SEM `mod`** — o gate de um defeito que é MUDO por natureza.
//!
//! # O que aconteceu, e porque nada o apanhou
//!
//! Na W2 Fase D o `bool_reach_tests.rs` mudou-se da shell para cá. Na shell ele era um
//! `#[cfg(test)] mod` de TOPO, declarado no `main.rs`; a declaração foi apagada de lá e **nenhuma
//! foi escrita aqui**. Um ficheiro `.rs` que nenhum `mod` declara **não é compilado** — ele fica no
//! disco, no git, no diff, e fora do build.
//!
//! ⇒ os **três** gates que ele contém evaporaram-se, e:
//!
//! | instrumento | o que ele disse |
//! |---|---|
//! | `cargo check -p ph2d-app-vec --all-targets` | **verde** |
//! | `cargo check -p ph2d-host-desktop --all-targets` | **verde** |
//! | `cargo clippy` | **verde** |
//! | a suíte da crate | **verde** — com três testes a menos, e ninguém conta os que faltam |
//! | `cargo test --test it` | **verde** |
//!
//! *Não há nada para verificar num ficheiro que não entra no build.* O que o acusou foi o `ONLY-A`
//! da prova de `nextest-list-diff` — é literalmente para isto que aquele número existe.
//!
//! # ⚠️ Porque este gate vive aqui e não na shell
//!
//! Ele mede a **árvore desta crate**, e o sujeito é o `src/` dela. É a mesma forma do censo de
//! órfãos que o `CLAUDE.md` §5.0 descreve (*«o ÓRFÃO e o DUPLICADO são as duas metades do MESMO
//! audit, os dois mudos»*), aplicada ao sítio onde o movimento acontece.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Todo nome de módulo que esta crate declara — por `mod x;` **e** por `#[path = "x.rs"]`.
///
/// ⚠️ **As DUAS formas, e a segunda é a que importa aqui**: a convenção desta crate é
/// `#[cfg(test)] #[path = "X_tests.rs"] mod tests;`, logo um censo que só lesse `mod x;` acusaria
/// metade dos ficheiros de teste como órfãos e seria desligado no primeiro dia. *Um censo que
/// acusa o legítimo morre por descrédito, não por estar errado.*
fn declarados(dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for e in fs::read_dir(dir)
        .expect("o src/ desta crate existe")
        .flatten()
    {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        for l in src.lines() {
            let t = l.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if let Some(r) = t.strip_prefix("#[path = \"") {
                if let Some(n) = r.split('"').next() {
                    out.insert(n.to_string());
                }
            }
            // `mod x;` · `pub mod x;` · `pub(crate) mod x;` — nunca `mod x {`.
            if let Some(r) = t.split("mod ").nth(1) {
                if let Some(n) = r.strip_suffix(';') {
                    if !n.is_empty()
                        && n.chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    {
                        out.insert(format!("{n}.rs"));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn every_file_in_this_crate_is_declared_by_some_mod() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let decl = declarados(&dir);
    let mut ficheiros = 0_usize;
    let mut orfaos: Vec<String> = Vec::new();
    for e in fs::read_dir(&dir).expect("src/").flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let nome = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if nome == "lib.rs" {
            continue;
        }
        ficheiros += 1;
        if !decl.contains(&nome) {
            orfaos.push(nome);
        }
    }
    // ⛔ **O piso de população** (HOWTO §2.7): sem ele, uma varredura que lesse zero ficheiros daria
    //    `orfaos.is_empty()` trivialmente verdadeiro — o censo que fica verde a medir NADA.
    assert!(
        ficheiros >= 100,
        "este censo varreu {ficheiros} ficheiros e esperava >= 100 — perdeu o sujeito (a crate \
         mudou de sítio, ou a extensão deixou de ser `.rs`)"
    );
    assert!(
        orfaos.is_empty(),
        "estes ficheiros estao no disco e NENHUM `mod` os declara, logo NAO SAO COMPILADOS:\n  \
         {}\n\nSe sao codigo, declare-os; se sao testes, declare-os com `#[cfg(test)] #[path = \
         \"...\"] mod ...;` no modulo cuja lei eles exercitam. ⛔ Apagar nao e' a cura por omissao: \
         um ficheiro orfao pode ser trabalho que se perdeu numa mudanca de arvore, e foi \
         exactamente isso em 2026-09-12.",
        orfaos.join("\n  ")
    );
}

/// **Controle positivo do instrumento:** ele reconhece as DUAS formas de declaração.
///
/// ⛔ Sem isto, um `declarados()` que lesse só `mod x;` devolveria um conjunto pequeno, o censo
/// acima acusaria dezenas de ficheiros legítimos, e alguém o apagaria — deixando o defeito real
/// sem instrumento outra vez.
#[test]
fn the_scanner_sees_both_ways_a_module_is_declared() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let decl = declarados(&dir);
    assert!(
        decl.contains("bool_reach_tests.rs"),
        "o scanner nao ve' uma declaracao por `#[path]` — e essa e' a forma que esta crate usa para \
         quase todos os irmaos de teste"
    );
    assert!(
        decl.contains("snap.rs"),
        "o scanner nao ve' uma declaracao por `pub mod x;` — a forma do `lib.rs`"
    );
}
