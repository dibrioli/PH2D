//! ⭐ **A shell liga EXACTAMENTE as famílias que este registo regista** — nem uma a mais, nem uma a
//! menos (auditoria de arquitectura 2026-09-12, A7).
//!
//! # O que foi medido
//!
//! O cabeçalho desta crate prometia *«a shell depende de UMA crate — esta — e nunca de seis»*. A
//! promessa era impossível por desenho: o `render_loop` chama os corpos de cada família PELO NOME e em
//! ordem (HOWTO §4), então a shell depende de cada família directamente — e o `cargo machete` do
//! `ship.sh` acusou a dependência da shell neste registo como morta. O registo é um **catálogo de
//! declarações** (as `const FAMILY`, com os roteadores de smoke que o gate de colisão lê).
//!
//! # A invariante que de facto importa
//!
//! As DUAS listas concordarem: uma família que a shell liga e o registo não conhece **escapa ao gate
//! de colisão de roteadores**; uma que o registo conhece e a shell não liga é uma **declaração morta**.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .to_path_buf()
}

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

/// As famílias de um bloco `[dependencies]`: chaves `ph2d-app-*`, menos o substrato e o registo.
fn familias(dependencias: &str) -> BTreeSet<String> {
    dependencias
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_owned()))
        .filter(|k| {
            k.starts_with("ph2d-app-")
                && !matches!(k.as_str(), "ph2d-app-host" | "ph2d-app-registry-init")
        })
        .collect()
}

#[test]
fn the_shell_links_exactly_the_registered_families() {
    let root = root();
    let registadas: BTreeSet<String> = ph2d_app_sync::scan_app_crates(&root.join("crates"))
        .into_iter()
        .collect();
    assert!(
        registadas.len() >= 9,
        "a varredura viu só {} famílias — ela partiu-se",
        registadas.len()
    );
    let shell =
        fs::read_to_string(root.join("shells/desktop/Cargo.toml")).expect("o manifesto da shell");
    let ligadas = familias(&table(&shell, "[dependencies]"));
    let so_ligadas: Vec<&String> = ligadas.difference(&registadas).collect();
    let so_registadas: Vec<&String> = registadas.difference(&ligadas).collect();
    assert!(
        so_ligadas.is_empty(),
        "a shell liga famílias que o registo NÃO conhece — os roteadores delas escapam ao gate de \
         colisão: {so_ligadas:?} (cura: `cargo run -p ph2d-app-sync`)"
    );
    assert!(
        so_registadas.is_empty(),
        "o registo declara famílias que a shell NÃO liga — declarações mortas: {so_registadas:?}"
    );
}

/// ⚠️ **E o leitor sabe dizer «não».**
#[test]
fn the_reader_can_say_no() {
    let t = "[dependencies]\nph2d-app-a = { path = \"x\" }\n# ph2d-app-b = 1\nph2d-app-host = 1\nph2d-app-registry-init = 1\nph2d-core = 1\n[dev-dependencies]\nph2d-app-c = 1\n";
    let f = familias(&table(t, "[dependencies]"));
    assert_eq!(
        f.into_iter().collect::<Vec<_>>(),
        vec!["ph2d-app-a".to_owned()]
    );
}
