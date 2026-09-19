//! ⭐ **«É este ficheiro um módulo de teste?» — sobre uma árvore de verdade no disco.**
//!
//! O helper responde lendo os PAIS, então o controlo tem de ter pais: monta uma crate mínima num
//! directório temporário com as quatro formas que existem neste repo — o irmão `_tests.rs` por
//! `#[path]`, o `tests.rs` por nome, a árvore `tests/mod.rs` e a NETA que ela declara — e o vizinho de
//! produção que nenhuma delas pode arrastar.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ph2d_label_census::cfg_test::is_declared_under_cfg_test;

struct Tree(PathBuf);

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tree(files: &[(&str, &str)]) -> Tree {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let root = std::env::temp_dir().join(format!(
        "ph2d-label-census-cfg-test-{}-{nanos}",
        std::process::id()
    ));
    for (rel, body) in files {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().expect("um ficheiro tem pai")).expect("cria o directório");
        fs::write(&p, body).expect("escreve o ficheiro");
    }
    Tree(root)
}

fn is_test(t: &Tree, rel: &str) -> bool {
    is_declared_under_cfg_test(&t.0.join(rel))
}

#[test]
fn every_shape_of_a_test_module_is_test_and_the_product_neighbour_is_not() {
    let t = tree(&[
        (
            "src/lib.rs",
            "mod paint;\nmod dispatch;\n#[cfg(test)]\nmod tests;\n",
        ),
        (
            "src/paint.rs",
            "#[cfg(test)]\n#[path = \"paint_tests.rs\"]\nmod paint_tests;\nmod helper;\n#[cfg(test)]\nmod commented_tests; // o seam do card\nmod commented_prod; // produto\n",
        ),
        ("src/paint_tests.rs", "fn t() {}\n"),
        ("src/helper.rs", "fn h() {}\n"),
        ("src/paint/commented_tests.rs", "fn ct() {}\n"),
        ("src/paint/commented_prod.rs", "fn cp() {}\n"),
        ("src/tests.rs", "mod inner;\n"),
        ("src/tests/inner.rs", "fn i() {}\n"),
        (
            "src/dispatch/mod.rs",
            "mod keys;\n#[cfg(test)]\nmod tests;\n",
        ),
        ("src/dispatch/keys.rs", "fn k() {}\n"),
        ("src/dispatch/tests/mod.rs", "mod caret;\nmod clipboard;\n"),
        ("src/dispatch/tests/caret.rs", "fn c() {}\n"),
        ("src/dispatch/tests/clipboard.rs", "fn p() {}\n"),
    ]);
    // o irmão por `#[path]`, o `tests.rs` e a neta dele
    assert!(is_test(&t, "src/paint_tests.rs"));
    assert!(is_test(&t, "src/tests.rs"));
    assert!(is_test(&t, "src/tests/inner.rs"));
    // ⛔ a árvore `tests/mod.rs` e as netas — a forma que esta função não via
    assert!(is_test(&t, "src/dispatch/tests/mod.rs"));
    assert!(is_test(&t, "src/dispatch/tests/caret.rs"));
    assert!(is_test(&t, "src/dispatch/tests/clipboard.rs"));
    // e o PRODUTO continua produto: o `dispatch/mod.rs`, o vizinho e o `helper` do pai com teste
    assert!(!is_test(&t, "src/dispatch/mod.rs"));
    assert!(!is_test(&t, "src/dispatch/keys.rs"));
    assert!(!is_test(&t, "src/helper.rs"));
    assert!(!is_test(&t, "src/paint.rs"));
    // ⛔ um comentário no fim da linha do `mod` escondia o nome (`mod line_seam_tests; // …`)
    assert!(is_test(&t, "src/paint/commented_tests.rs"));
    assert!(!is_test(&t, "src/paint/commented_prod.rs"));
    assert!(!Path::new(&t.0).join("src/nao_existe.rs").exists());
}

/// ⛔ **Um COMENTÁRIO entre o `#[cfg(test)]` e o `#[path]` não esconde o teste** (2026-09-16, a
/// `ph2d-panel-motion-params`) — e um comentário acima de um `mod` de PRODUTO não o torna teste.
#[test]
fn a_comment_between_the_attributes_does_not_hide_the_cfg_test() {
    let t = tree(&[
        (
            "src/lib.rs",
            "#[cfg(test)]\n// o sufixo `_tests.rs` não é cosmético\n// (e explica-o em duas linhas)\n#[path = \"lib_gradient_tests.rs\"]\nmod tests_gradient;\n// um comentário sobre produção\nmod rows;\n",
        ),
        ("src/lib_gradient_tests.rs", "fn g() {}\n"),
        ("src/rows.rs", "fn r() {}\n"),
    ]);
    assert!(is_test(&t, "src/lib_gradient_tests.rs"));
    assert!(!is_test(&t, "src/rows.rs"));
}

/// ⭐⭐⭐ **A régua lê cada ficheiro UMA vez — e o instrumento é uma CONTAGEM, nunca um relógio.**
///
/// ⛔⛔ Até 2026-09-19 a pergunta *«o meu pai gateia-me?»* relia **todos os irmãos a cada
/// ficheiro**, o que é `O(irmãos²)` em leituras de disco. Medido pela régua lexical, que chama isto
/// uma vez por ficheiro: a `ph2d-app-motion` (492 ficheiros, 5,25 MB) levava `13,61 s` e passou a
/// `0,36 s` — **38×** — com a saída **byte-idêntica** em seis crates e 1 425 ficheiros.
///
/// ⚠️ **Um gate de TEMPO aqui seria um membro da família de flakes de fan-out** (`CLAUDE.md` §5.0).
/// Esta contagem não depende de carga: `n + 1` leituras com a memória, `~n²` sem ela — e é por isso
/// que a barra pode ser apertada em vez de folgada.
#[test]
fn a_regua_le_cada_ficheiro_uma_vez_e_nao_uma_vez_por_irmao() {
    const IRMAOS: usize = 12;
    let mut files: Vec<(String, String)> = vec![(
        "src/lib.rs".into(),
        "#[cfg(test)]\nmod f0;\nmod f1;\nmod f2;\nmod f3;\nmod f4;\nmod f5;\nmod f6;\nmod f7;\nmod f8;\nmod f9;\nmod f10;\nmod f11;\n".into(),
    )];
    for i in 0..IRMAOS {
        files.push((format!("src/f{i}.rs"), format!("fn f{i}() {{}}\n")));
    }
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let t = tree(&borrowed);

    let antes = ph2d_label_census::cfg_test::leituras_do_disco();
    // ⚠️ O controlo POSITIVO vem primeiro: se a régua deixasse de responder, a contagem baixa
    //    seria trivialmente verdadeira sobre uma régua partida.
    assert!(
        is_test(&t, "src/f0.rs"),
        "o `#[cfg(test)] mod f0;` gateia-o"
    );
    for i in 1..IRMAOS {
        assert!(
            !is_test(&t, &format!("src/f{i}.rs")),
            "os outros onze são produto"
        );
    }
    let lidos = ph2d_label_census::cfg_test::leituras_do_disco() - antes;

    // ⭐ `IRMAOS + 1` ficheiros no disco, mais os dois pais que não existem (`src/mod.rs` e
    //   `<dir>.rs`) — a memória guarda também a ausência, logo o tecto é `IRMAOS + 3`.
    let tecto = IRMAOS + 3;
    assert!(
        lidos <= tecto,
        "a régua leu {lidos} ficheiros para responder sobre {IRMAOS} irmãos (tecto {tecto}). \
         Sem a memória de `declaracoes` isto é ~{}, que é o quadrático que a cura de 2026-09-19 \
         apagou.",
        IRMAOS * IRMAOS
    );
}
