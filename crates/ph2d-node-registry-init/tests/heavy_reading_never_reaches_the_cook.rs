//! ⭐⭐⭐ **NENHUM NÓ CONSEGUE ABRIR UM FICHEIRO** — a cerca é ESTRUTURAL, não uma promessa.
//!
//! A lei é do [doc 63 §6](../../../docs/Motion%20Nodes/63_pesquisa_industria_2026_e_plano_estado_da_arte.md):
//! *"a FFT NUNCA entra no cook"*. O trabalho pesado — descodificar som, ler um disco, partir um
//! CSV — corre na shell, uma vez por ficheiro, e o resultado é **publicado** no canal externo. O
//! nó lê um stream e nada mais.
//!
//! ⚠️ **Isto não se defende com disciplina, defende-se com DEPENDÊNCIAS**: sem a crate no
//! `Cargo.toml`, o nó não CONSEGUE ter opinião. Este gate lê os `Cargo.toml` e prova-o.
//!
//! ⛔⛔ **E ele nasceu porque o antecessor dele NÃO EXISTIA.** O
//! `shells/desktop/src/render_loop/motion_bridge_params_file.rs` cita
//! *"(gate `the_fft_never_reaches_the_cook`)"* desde que foi escrito — e uma varredura da árvore
//! inteira em 2026-08-30 encontrou **zero** testes com esse nome. *Um gate citado pelo nome num
//! comentário não é um gate: é uma nota que se lê como uma.*

use std::fs;
use std::path::{Path, PathBuf};

/// `(crate do nó, o que ele não pode alcançar)` — a lista é a AFIRMAÇÃO.
const FENCED: &[(&str, &[&str])] = &[
    (
        "ph2d-node-audio-bands",
        &["ph2d-audio", "ph2d-audio-spectral", "ph2d-audio-edit"],
    ),
    ("ph2d-node-source-table", &["ph2d-table"]),
    ("ph2d-node-value-table", &["ph2d-table"]),
];

/// O que TODO nó cercado pode ter, e mais nada.
const ALLOWED: &[&str] = &["ph2d-nodegraph", "ph2d-node-registry"];

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// Os nomes de crate que aquele `Cargo.toml` declara como dependência, **em qualquer das formas
/// que o Cargo aceita**.
///
/// ⛔⛔ **A 1.ª redacção só entendia `[dependencies]` exacto, e uma auditoria adversarial passou
/// por ela DUAS vezes** (2026-08-30): `[dependencies.ph2d-table]` (a forma sub-tabela) e
/// `[target.'cfg(unix)'.dependencies]` faziam a dependência **desaparecer**, com o leitor ligado
/// e chamado dentro do `eval`, e o gate verde. *Um parser de conveniência defende a forma que o
/// autor calhou de escrever, não a lei.*
fn deps_of(manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let t = line.trim();
        if let Some(head) = t.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
            let head = head.trim();
            // `[dependencies]` · `[dev-dependencies]` · `[build-dependencies]` ·
            // `[target.'cfg(…)'.dependencies]` — e as formas sub-tabela de todas elas.
            let is_dep_table = head.ends_with("dependencies");
            if let Some(rest) = head.rsplit_once("dependencies.") {
                // `[dependencies.NOME]` ⇒ o NOME é a dependência.
                out.push(rest.1.trim().trim_matches('"').to_string());
                inside = false;
                continue;
            }
            inside = is_dep_table;
            continue;
        }
        if !inside || t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = t.split_once('=') {
            out.push(name.trim().trim_matches('"').to_string());
        }
    }
    out
}

/// O que um nó cercado não pode INVOCAR, mesmo sem uma dependência nova.
///
/// ⛔⛔⛔ **A cerca por dependências é necessária e NÃO É SUFICIENTE, e a auditoria mediu-o:**
/// `std::fs::read_to_string` dentro do `eval`, **sem tocar num `Cargo.toml`**, passava o gate —
/// porque a `std` não aparece em manifesto nenhum. O doc afirmava *"NENHUM NÓ CONSEGUE ABRIR UM
/// FICHEIRO"* e o instrumento só sabia dizer *"não depende do leitor"*, que é mais fraco.
const FORBIDDEN_IN_SRC: &[&str] = &[
    "std::fs",
    "std::net",
    "std::process",
    "include_str!",
    "include_bytes!",
];

/// Os `.rs` de `crates/<nome>/src`, recursivamente.
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn a_node_that_reads_a_file_cannot_even_depend_on_the_reader() {
    let root = crates_dir();
    for (node, forbidden) in FENCED {
        let path = root.join(node).join("Cargo.toml");
        let manifest = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{node}/Cargo.toml: {e} — o no' mudou de nome?"));
        let deps = deps_of(&manifest);
        // ⚠️ **O CONTROLE**: sem ele, um `Cargo.toml` que o parser não entendesse daria uma
        // lista vazia e o gate ficaria verde sobre qualquer coisa.
        assert!(
            deps.contains(&"ph2d-nodegraph".to_string()),
            "{node}: o parser nao achou nem o `ph2d-nodegraph` — ele nao esta' a ler nada. \
             Deps lidas: {deps:?}"
        );
        for bad in *forbidden {
            assert!(
                !deps.contains(&(*bad).to_string()),
                "{node} depende de `{bad}` — a analise passa a poder entrar no COOK, e a cerca \
                 do doc 63 §6 deixa de ser estrutural. Publique pelo canal externo."
            );
        }
        // ⭐⭐ **E o que ele INVOCA**, que nenhuma dependência denuncia.
        let mut files = Vec::new();
        rs_files(&root.join(node).join("src"), &mut files);
        assert!(
            !files.is_empty(),
            "{node}/src vazio — o gate varre o sitio errado"
        );
        for f in &files {
            let src = fs::read_to_string(f).expect("ler o fonte");
            let name = f.file_name().unwrap_or_default().to_string_lossy();
            for bad in FORBIDDEN_IN_SRC {
                assert!(
                    !src.contains(bad),
                    "{node}/{name} invoca `{bad}` — um no' NAO abre um ficheiro, nem sem \
                     dependencia nenhuma (doc 63 §6). Publique pelo canal externo."
                );
            }
        }
        for d in &deps {
            assert!(
                ALLOWED.contains(&d.as_str()),
                "{node} ganhou a dependencia `{d}`, que nao esta' na lista do que um no' cercado \
                 pode ter ({ALLOWED:?}). Se ela e' legitima, acrescente-a AQUI e diga porque' — \
                 uma cerca que cresce sem ninguem olhar deixa de ser uma."
            );
        }
    }
}
