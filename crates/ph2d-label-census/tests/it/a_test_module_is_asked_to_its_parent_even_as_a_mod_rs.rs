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
            "#[cfg(test)]\n#[path = \"paint_tests.rs\"]\nmod paint_tests;\nmod helper;\n",
        ),
        ("src/paint_tests.rs", "fn t() {}\n"),
        ("src/helper.rs", "fn h() {}\n"),
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
    assert!(!Path::new(&t.0).join("src/nao_existe.rs").exists());
}
