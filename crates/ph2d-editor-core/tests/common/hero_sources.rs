//! ⛔⛔ **O ficheiro do hero que responde a uma pergunta MUDA — pergunte ao MÓDULO, não ao nome.**
//!
//! Quatro gates deste crate liam `include_str!("../src/screens/hero/paint.rs")`. Em 2026-08-30 o
//! tecto de LOC obrigou a cortar dali o bloco da geometria para um irmão (`frame_layout.rs`) —
//! *pure code motion*, produto intacto — e as duas espécies de gate reagiram de maneiras opostas:
//!
//! | o gate afirma | o que o corte lhe fez |
//! |---|---|
//! | **presença** (*«alguém chama isto»*) | reprovou **alto**, com uma acusação falsa |
//! | **ausência** (*«isto NÃO voltou»*) | ficou **verde e vazio** — a prova mudou-se para fora do alcance dele |
//!
//! ⚠️ **A segunda é a perigosa**: `the_side_columns_are_anchored` exigia que
//! `blender_picker_offset(…)` não estivesse no `paint.rs`, e depois do corte essa ausência passou
//! a ser de graça — o offset podia voltar no ficheiro ao lado com o gate a passar.
//!
//! ⇒ a pergunta certa é sobre o **módulo**: *alguém em `screens/hero` faz isto?* Um corte por
//! responsabilidade deixa de mexer com os gates, que é o que um corte por responsabilidade deve
//! fazer.
//!
//! Não é um alvo de teste: vive em `tests/common/`, que o cargo não compila como binário.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// A raiz do módulo do hero.
fn hero_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/screens/hero")
}

/// **Todo o código-fonte de `screens/hero`, recursivamente** — `(caminho, conteúdo)`.
///
/// ⚠️ Recursivo de propósito: o `chrome/`, o `topbar/` e os irmãos que uma wave futura cortar
/// entram sozinhos.
#[must_use]
pub fn hero_sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    walk(&hero_dir(), &mut out);
    assert!(
        !out.is_empty(),
        "o módulo do hero não tem fontes — o caminho mudou e este helper ficou cego"
    );
    out
}

fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(src) = std::fs::read_to_string(&path)
        {
            out.push((path.display().to_string(), src));
        }
    }
}

/// **Algum ficheiro do hero contém isto?** — devolve o primeiro `(caminho, conteúdo)` que contém.
#[must_use]
pub fn hero_file_containing(needle: &str) -> Option<(String, String)> {
    hero_sources().into_iter().find(|(_, s)| s.contains(needle))
}

/// **Nenhum ficheiro do hero contém isto** — a asserção de AUSÊNCIA, feita ao módulo inteiro.
pub fn assert_hero_never_contains(needle: &str, why: &str) {
    if let Some((path, _)) = hero_file_containing(needle) {
        panic!("`{needle}` vive em {path}: {why}");
    }
}
