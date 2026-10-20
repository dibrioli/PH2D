//! G4 (`docs/CodeEditor/04_pagina_de_ajuda.md` §2): a porta `resolve` sobre as respostas REAIS do
//! `luau-lsp` 1.70.1 gravadas pela sonda `docs/CodeEditor/ferramentas/lspprobe_ajuda.py`
//! (`medicoes_onda2/ajuda_bruto.json`) — o balão decide onde é exacto, a árvore o resto.

use ph2d_code_help::resolve::{Query, resolve};
use ph2d_code_help::{Anchor, anchor};
use serde_json::Value;

const BRUTO: &str =
    include_str!("../../../../docs/CodeEditor/ferramentas/medicoes_onda2/ajuda_bruto.json");

fn hover(bruto: &Value, caso: &str) -> Option<String> {
    let h = &bruto["casos"][caso]["hover"];
    assert!(
        !h.is_null() || caso.starts_with("self"),
        "o caso «{caso}» não está na sonda"
    );
    h["contents"]["value"].as_str().map(str::to_owned)
}

fn a(page: &str, member: Option<&str>) -> Option<Anchor> {
    let r = anchor(page, member);
    assert!(r.is_some(), "o catálogo não tem {page}.{member:?}");
    r
}

/// (caso da sonda, palavra, qualificador, `:`, o sítio esperado)
type Caso = (
    &'static str,
    &'static str,
    Option<&'static str>,
    bool,
    Option<Anchor>,
);

fn casos() -> Vec<Caso> {
    vec![
        (
            "math.sin",
            "sin",
            Some("math"),
            false,
            a("math", Some("sin")),
        ),
        (
            "m.cos (math por outro nome)",
            "cos",
            Some("m"),
            false,
            a("math", Some("cos")),
        ),
        (
            "math.pi (constante)",
            "pi",
            Some("math"),
            false,
            a("math", Some("pi")),
        ),
        (
            "math.huge (constante)",
            "huge",
            Some("math"),
            false,
            a("math", Some("huge")),
        ),
        ("math (a biblioteca)", "math", None, false, a("math", None)),
        (
            "print (global)",
            "print",
            None,
            false,
            a("globals", Some("print")),
        ),
        (
            "s:upper (método de um local)",
            "upper",
            Some("s"),
            true,
            a("string", Some("upper")),
        ),
        (
            "(\"x\"):upper (método de um literal)",
            "upper",
            None,
            true,
            a("string", Some("upper")),
        ),
        (
            "string.format",
            "format",
            Some("string"),
            false,
            a("string", Some("format")),
        ),
        (
            "string (a biblioteca)",
            "string",
            None,
            false,
            a("string", None),
        ),
        (
            "typeof",
            "typeof",
            None,
            false,
            a("globals", Some("typeof")),
        ),
        (
            "tostring",
            "tostring",
            None,
            false,
            a("globals", Some("tostring")),
        ),
        (
            "table.insert",
            "insert",
            Some("table"),
            false,
            a("table", Some("insert")),
        ),
        (
            "buffer.create",
            "create",
            Some("buffer"),
            false,
            a("buffer", Some("create")),
        ),
        (
            "vector.create",
            "create",
            Some("vector"),
            false,
            a("vector", Some("create")),
        ),
        (
            "bit32.band",
            "band",
            Some("bit32"),
            false,
            a("bit32", Some("band")),
        ),
        (
            "utf8.len",
            "len",
            Some("utf8"),
            false,
            a("utf8", Some("len")),
        ),
        (
            "os.clock",
            "clock",
            Some("os"),
            false,
            a("os", Some("clock")),
        ),
        (
            "coroutine.wrap",
            "wrap",
            Some("coroutine"),
            false,
            a("coroutine", Some("wrap")),
        ),
        ("pcall", "pcall", None, false, a("globals", Some("pcall"))),
        (
            "error (passado como valor)",
            "error",
            None,
            false,
            a("globals", Some("error")),
        ),
        (
            "debug.traceback",
            "traceback",
            Some("debug"),
            false,
            a("debug", Some("traceback")),
        ),
        (
            "ph2d.set",
            "set",
            Some("ph2d"),
            false,
            a("ph2d", Some("set")),
        ),
        (
            "ph2d.property",
            "property",
            Some("ph2d"),
            false,
            a("ph2d", Some("property")),
        ),
        ("ph2d (a tabela)", "ph2d", None, false, a("ph2d", None)),
        (
            "motion.fire",
            "fire",
            Some("motion"),
            false,
            a("motion", Some("fire")),
        ),
        (
            "PoseField (tipo nosso)",
            "PoseField",
            None,
            false,
            a("types", Some("PoseField")),
        ),
        // o gancho é declarado no script: o `goto` vai lá antes de perguntar à porta
        ("update (gancho)", "update", None, false, None),
        ("self.id (campo do motor)", "id", Some("self"), false, None),
    ]
}

#[test]
fn a_tabela_da_sonda() {
    let bruto: Value = serde_json::from_str(BRUTO).expect("JSON da sonda");
    let casos = casos();
    assert_eq!(
        casos.len(),
        bruto["casos"].as_object().map_or(0, |o| o.len()),
        "um caso por linha da sonda"
    );
    let mut erros = Vec::new();
    for (caso, word, qualifier, colon, want) in casos {
        let h = hover(&bruto, caso);
        let got = resolve(&Query {
            word,
            qualifier,
            colon,
            hover: h.as_deref(),
        });
        if got != want {
            erros.push(format!("{caso}: {got:?}, esperado {want:?}"));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

/// Sem o balão (ainda por chegar), a árvore chega onde pode — e o pseudónimo e o método não abrem
/// página errada: não abrem nenhuma.
#[test]
fn sem_o_balao_a_arvore_chega_onde_pode() {
    let q = |word, qualifier, colon| {
        resolve(&Query {
            word,
            qualifier,
            colon,
            hover: None,
        })
    };
    assert_eq!(
        q("insert", Some("table"), false),
        a("table", Some("insert"))
    );
    assert_eq!(q("print", None, false), a("globals", Some("print")));
    assert_eq!(q("cos", Some("m"), false), None);
    assert_eq!(q("upper", Some("s"), true), None);
    assert_eq!(q("local", None, false), None);
}
