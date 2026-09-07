//! ⭐⭐⭐ **O RECUO de um filho vem de UMA porta — o app tinha QUATRO respostas.**
//!
//! A pergunta é uma só: *quanto se desloca para a direita a linha de um filho?* Censada em
//! 2026-09-07, ela tinha uma resposta por superfície:
//!
//! | superfície | escrevia | passo |
//! |---|---|---|
//! | Hierarquia | `Spacing::Xl` | **16** |
//! | `variant_editor` | `INDENT_PX = 16.0` à mão | **16** |
//! | Painter Layers | `LAYER_INDENT_STEP = 14.0` à mão | **14** |
//! | Catálogo do Asset Browser | `Spacing::Md` | **8** |
//! | `tree_view` (a GALERIA) | `Spacing::Lg` | **12** ✅ |
//!
//! ⭐⭐ **A galeria já tinha a resposta do modelo, e nenhuma superfície do produto a copiou.** O
//! `tree_view` é a peça de referência do cromo e só é pintado na bancada — *uma referência que
//! ninguém chama não ensina; ela só regista que a resposta certa já era conhecida.*
//!
//! A lei é o `Tree.item_margin` do Godot Modern (MIT), `MAX(3 · increased_margin, 12)` = **12 px**,
//! e vive em [`ph2d_tokens::list_indent_px`]. ⚠️ O **piso** dela tem recurso e o recurso é a
//! coluna da seta ([`ph2d_tokens::tree_chevron_col_px`]): dois níveis põem as suas setas a um
//! passo de distância, logo um passo mais estreito que a seta faz a do filho entrar por cima da
//! do pai.
//!
//! # A régua, e por que ela distingue `*` de `*`
//!
//! Um passo por nível lê-se no fonte como uma **multiplicação por uma profundidade**. A varredura
//! ingénua (*a linha fala de `depth` e tem um `*`*) acusa dois inocentes, e os dois pelo mesmo
//! motivo: o `*` deles é uma **desreferência** (`*rect`, `(*v as f32)`), não um produto.
//!
//! ⇒ a régua exige o `*` **binário**, que o `rustfmt` escreve sempre com espaço dos dois lados e
//! que uma desreferência nunca tem (o `*` de um deref cola no nome). *Um censo que parseia o
//! fonte tem de saber todas as formas do que lê* — esta linha pagou essa lição sete vezes.

use std::fs;
use std::path::{Path, PathBuf};

/// As superfícies que HOJE recuam um filho. ⚠️ É a metade de OBSOLESCÊNCIA do censo: se uma delas
/// deixar de recuar, esta lista tem de encolher — senão o censo mede uma população que já não
/// existe e passa a verde por vazio.
const STEPS_BY_A_LEVEL: &[&str] = &[
    "crates/ph2d-editor-core/src/widget/tree_view.rs",
    "crates/ph2d-editor-core/src/widget/variant_editor.rs",
    "crates/ph2d-panel-hierarchy/src/paint.rs",
    "crates/ph2d-panel-painter-layers/src/paint_rows.rs",
    "crates/ph2d-panel-asset-browser/src/paint_catalog.rs",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                walk(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn ui_sources() -> Vec<(String, String)> {
    let root = repo_root();
    let mut files = Vec::new();
    walk(&root.join("crates/ph2d-editor-core/src"), &mut files);
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut files);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut files);
    files.sort();
    let mut out = Vec::new();
    for p in files {
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        if let Ok(raw) = fs::read_to_string(&p) {
            out.push((rel, without_tests(&raw)));
        }
    }
    out
}

fn without_tests(src: &str) -> String {
    match src.find("#[cfg(test)]") {
        Some(at) => src[..at].to_string(),
        None => src.to_string(),
    }
}

/// Esta linha desloca por NÍVEL? — uma profundidade multiplicada por alguma coisa.
///
/// ⚠️ **O `*` tem de ser binário.** O `rustfmt` normaliza um produto para ` * `; o `*` de uma
/// desreferência cola no nome (`*rect`, `*v`). Sem esta distinção o censo acusa duas linhas que
/// não fazem conta nenhuma — e uma delas é a cor de um cartão.
fn steps_by_a_level(line: &str) -> bool {
    let s = line.trim_start();
    if s.starts_with("//") {
        return false;
    }
    line.contains(" * ") && mentions_depth(line)
}

/// `depth` como PALAVRA (`depth`, `.depth`, `row.depth`) — nunca como pedaço de `depth_bias`.
fn mentions_depth(line: &str) -> bool {
    let b = line.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = line[from..].find("depth") {
        let at = from + rel;
        let end = at + "depth".len();
        let before_ok = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
        let after_ok = end >= b.len() || !(b[end].is_ascii_alphanumeric() || b[end] == b'_');
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// ⭐⭐⭐ **Toda superfície que recua um filho tira o passo da porta.**
#[test]
fn every_surface_that_steps_by_a_level_takes_the_step_from_the_door() {
    let mut strays = Vec::new();
    for (rel, src) in ui_sources() {
        if !src.lines().any(steps_by_a_level) {
            continue;
        }
        if !src.contains("list_indent_px") {
            strays.push(rel);
        }
    }
    assert!(
        strays.is_empty(),
        "estas superficies recuam um filho sem passar pela porta `ph2d_tokens::list_indent_px()` \
         — e um passo escolhido no sitio da pintura e' exactamente como o app chegou a QUATRO \
         respostas (16 / 16 / 14 / 8) para uma pergunta so':\n  {}",
        strays.join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — a lista declarada tem de continuar a descrever o código.
#[test]
fn the_declared_surfaces_still_step_by_a_level() {
    let sources = ui_sources();
    for declared in STEPS_BY_A_LEVEL {
        let found = sources
            .iter()
            .find(|(rel, _)| rel == declared)
            .unwrap_or_else(|| panic!("`{declared}` ja' nao existe: a lista do censo esta' velha"));
        assert!(
            found.1.lines().any(steps_by_a_level),
            "`{declared}` ja' nao recua um filho — tire-o da lista, senao o censo passa a medir \
             uma populacao que nao existe"
        );
    }
}

/// ⛔ **E nenhuma superfície declara um SEGUNDO passo.**
///
/// É a metade que impede a quinta resposta de nascer: as duas que existiam eram `const`
/// escritas à mão, cada uma certa sozinha, e nenhum teste podia vê-las.
#[test]
fn no_surface_declares_an_indent_constant_of_its_own() {
    let mut declared = Vec::new();
    for (rel, src) in ui_sources() {
        for (n, line) in src.lines().enumerate() {
            let s = line.trim_start();
            if s.starts_with("//") {
                continue;
            }
            let upper = s.to_ascii_uppercase();
            let names_an_indent = upper.contains("INDENT");
            let is_a_declaration = s.starts_with("const ") || s.starts_with("pub const ");
            if names_an_indent && is_a_declaration && !s.contains("list_indent_px") {
                declared.push(format!("{rel}:{}: {s}", n + 1));
            }
        }
    }
    assert!(
        declared.is_empty(),
        "uma constante de recuo nasceu fora da porta:\n  {}",
        declared.join("\n  ")
    );
}

/// ⛔⛔ **A GEOMETRIA que os dois ficheiros da hierarquia partilham vem de UMA declaração cada.**
///
/// A linha desenha a seta (`row.rs`) e o desenhador do parentesco desenha o fio que sai de baixo
/// dela (`paint.rs`) — e as **duas medidas que os dois têm de partilhar** estavam escritas nos
/// dois ficheiros, cada uma com metade de um comentário a mandar sincronizar à mão. ⚠️ Um par
/// sincronizado por comentário não é uma lei: é duas leis que hoje concordam, e a que derivasse
/// tirava o fio de baixo da seta — um report que este painel já pagou (Enio, 2026-05-26).
///
/// ⚠️⚠️ **Este teste nasceu de uma MUTAÇÃO QUE SOBREVIVEU**: repor `Spacing::Lg` numa das duas
/// cópias não acordava nada. É a 5.ª vez que esta linha escreve a porta certa e não a gateia.
#[test]
fn the_two_hierarchy_files_read_their_shared_geometry_from_one_declaration() {
    const SHARED: &[(&str, &str)] = &[
        ("chev_w", "tree_chevron_col_px"),
        ("chev_col_w", "tree_chevron_col_px"),
        ("pad", "row_inset_px"),
        ("row_inner_pad", "row_inset_px"),
    ];
    const FILES: &[&str] = &[
        "crates/ph2d-panel-hierarchy/src/row.rs",
        "crates/ph2d-panel-hierarchy/src/paint.rs",
    ];
    let sources = ui_sources();
    let mut seen = 0usize;
    for rel in FILES {
        let src = &sources
            .iter()
            .find(|(r, _)| r == rel)
            .unwrap_or_else(|| panic!("`{rel}` ja' nao existe: a lista esta' velha"))
            .1;
        for line in src.lines() {
            let s = line.trim_start();
            for (name, door) in SHARED {
                if s.starts_with(&format!("let {name} = ")) {
                    seen += 1;
                    assert!(
                        s.contains(door),
                        "`{rel}` volta a escolher o proprio `{name}`: `{s}` — a medida tem de vir \
                         de `{door}()`, senao o fio de parentesco sai de baixo da seta"
                    );
                }
            }
        }
    }
    // A metade justa: se as declaracoes desaparecerem, o teste acima fica VACUO.
    assert_eq!(
        seen,
        SHARED.len(),
        "esperava {} declaracoes partilhadas na hierarquia e achei {seen}: a lista esta' velha",
        SHARED.len()
    );
}

/// ⭐ **O piso do recuo é a coluna da seta**, e as duas medidas vivem em portas.
#[test]
fn the_step_is_never_narrower_than_the_arrow_column() {
    assert!(
        ph2d_tokens::list_indent_px() >= ph2d_tokens::tree_chevron_col_px(),
        "o passo ({}) ficou menor que a coluna da seta ({}): a seta de um filho entra por cima \
         da do pai",
        ph2d_tokens::list_indent_px(),
        ph2d_tokens::tree_chevron_col_px()
    );
}
