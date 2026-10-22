//! ⭐⭐⭐ **A API que a VM TEM é a da tabela, nos dois sentidos** (CodeEditor T2, ADR-0188 §3).
//!
//! A VM é montada pela MESMA fiação que a cena usa (`ScriptHost::new` → `build_runtime`, com a
//! sandbox), e as chaves são lidas da VM — nunca de uma lista escrita à mão ao lado.

use std::collections::BTreeSet;

use mlua::{Lua, Table, Value};
use ph2d_script::ScriptHost;
use ph2d_script::api_table::{API, API_GLOBALS, API_TYPES, HOOKS, luau_definitions};

fn keys(t: &Table) -> BTreeSet<String> {
    t.pairs::<Value, Value>()
        .map(|p| {
            let (k, _) = p.expect("par");
            match k {
                Value::String(s) => s.to_str().expect("utf8").to_owned(),
                other => panic!("chave não textual: {other:?}"),
            }
        })
        .collect()
}

/// Os globais visíveis — o próprio ambiente e a base para onde a sandbox o faz cair.
fn globais(lua: &Lua) -> BTreeSet<String> {
    let g = lua.globals();
    let mut out = keys(&g);
    if let Some(Value::Table(base)) = g.metatable().and_then(|m| m.raw_get("__index").ok()) {
        out.extend(keys(&base));
    }
    out
}

#[test]
fn as_chaves_da_vm_sao_as_da_tabela_nos_dois_sentidos() {
    let host = ScriptHost::new().expect("vm");
    let lua = host.runtime().lua();
    let limpa = Lua::new();
    limpa.sandbox(true).expect("sandbox");
    let instalados: BTreeSet<String> = globais(lua).difference(&globais(&limpa)).cloned().collect();
    let declarados: BTreeSet<String> = API_GLOBALS.iter().map(|g| g.ident.to_owned()).collect();
    assert_eq!(instalados, declarados, "globais da fiação × API_GLOBALS");
    for g in API_GLOBALS {
        let t: Table = lua.globals().get(g.ident).expect("o global é tabela");
        let na_vm = keys(&t);
        let na_tabela: BTreeSet<String> = API
            .iter()
            .filter(|e| e.table == g.ident)
            .map(|e| e.ident.to_owned())
            .collect();
        let falta: Vec<_> = na_vm.difference(&na_tabela).collect();
        let inventada: Vec<_> = na_tabela.difference(&na_vm).collect();
        assert!(
            falta.is_empty() && inventada.is_empty(),
            "{}: na VM sem linha na tabela {falta:?} · na tabela sem função na VM {inventada:?}",
            g.ident
        );
        for k in &na_vm {
            assert!(
                matches!(t.get::<Value>(k.as_str()), Ok(Value::Function(_))),
                "{}.{k} não é função",
                g.ident
            );
        }
    }
    // Cada linha é única e aponta um global declarado.
    let mut vistas = BTreeSet::new();
    for e in API {
        assert!(
            API_GLOBALS.iter().any(|g| g.ident == e.table),
            "{}",
            e.table
        );
        assert!(
            vistas.insert((e.table, e.ident)),
            "repetida: {}.{}",
            e.table,
            e.ident
        );
    }
}

/// As definições citam cada global, cada função e cada tipo, e só usam tipos declarados.
#[test]
fn as_definicoes_saem_da_tabela() {
    let d = luau_definitions();
    for g in API_GLOBALS {
        assert!(
            d.contains(&format!("declare {}: {{", g.ident)),
            "{}",
            g.ident
        );
    }
    for e in API {
        assert!(
            d.contains(&format!("    {}: {},\n", e.ident, e.luau_type)),
            "{}.{}",
            e.table,
            e.ident
        );
    }
    for t in API_TYPES {
        assert!(
            d.contains(&format!("export type {} = {}\n", t.ident, t.definition)),
            "{}",
            t.ident
        );
    }
    // Um tipo com maiúscula citado numa assinatura tem de estar declarado.
    let declarados: BTreeSet<&str> = API_TYPES.iter().map(|t| t.ident).collect();
    for h in HOOKS {
        assert!(
            d.contains(&format!("-- @hook {}: {}\n", h.ident, h.luau_type)),
            "gancho {}",
            h.ident
        );
    }
    let sigs = API
        .iter()
        .map(|e| e.luau_type)
        .chain(API_TYPES.iter().map(|t| t.definition));
    for sig in sigs.chain(HOOKS.iter().map(|h| h.luau_type)) {
        for w in sig.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
            if w.starts_with(|c: char| c.is_ascii_uppercase()) {
                assert!(declarados.contains(w), "tipo por declarar: {w} em {sig}");
            }
        }
    }
}

/// A base de documentação do `luau-lsp` sai da tabela: cada global e cada função com a sua linha,
/// os parâmetros pelo nome e na ordem da assinatura, e a documentação de um parâmetro é um SÍMBOLO
/// (texto directo sai vazio no servidor) com o texto DELE na `api_help` — o da página de ajuda —, ou,
/// sem ele, a linha do tipo exportado que usa.
#[test]
fn a_base_de_documentacao_sai_da_tabela() {
    use ph2d_script::api_help::api_help;
    use ph2d_script::api_table::{DEFINITIONS_PACKAGE, luau_docs, luau_params, param_doc};
    let v: serde_json::Value = serde_json::from_str(&luau_docs()).expect("JSON");
    let doc = |k: &str| v[k]["documentation"].as_str().map(str::to_owned);
    for g in API_GLOBALS {
        let k = format!("{DEFINITIONS_PACKAGE}/global/{}", g.ident);
        assert_eq!(doc(&k).as_deref(), Some(g.doc), "{k}");
    }
    let (mut com_ajuda, mut com_tipo) = (0, 0);
    for e in API {
        let k = format!("{DEFINITIONS_PACKAGE}/global/{}.{}", e.table, e.ident);
        let ajuda = api_help(&format!("{}.{}", e.table, e.ident));
        assert_eq!(doc(&k).as_deref(), Some(e.doc), "{k}");
        let ps = v[&k]["params"].as_array().expect("params");
        let sig = luau_params(e.luau_type);
        // Tantos parâmetros quantos `nome:` na assinatura, cada um um identificador.
        assert_eq!(
            ps.len(),
            e.luau_type
                .split(") ->")
                .next()
                .unwrap()
                .matches(':')
                .count(),
            "{k}"
        );
        for (i, ((name, ty), p)) in sig.iter().zip(ps).enumerate() {
            assert!(
                !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "{k}: {name:?}"
            );
            assert_eq!(p["name"].as_str(), Some(*name), "{k}");
            let proprio = ajuda
                .and_then(|h| h.params.iter().find(|(n, _)| n == name))
                .map(|(_, t)| *t);
            let tipo = API_TYPES
                .iter()
                .find(|t| t.ident == ty.trim_end_matches('?'))
                .map(|t| t.doc);
            let d = p["documentation"].as_str().unwrap();
            match proprio.or(tipo) {
                Some(texto) => {
                    if proprio.is_some() {
                        com_ajuda += 1;
                    } else {
                        com_tipo += 1;
                    }
                    assert_eq!(d, format!("{k}/param/{i}"), "{k}");
                    assert_eq!(doc(d).as_deref(), Some(texto), "{k}: {name}");
                }
                None => assert_eq!(d, "", "{k}: {name}"),
            }
        }
    }
    // Controlo: o texto da ajuda chega (o `entity` do `ph2d.set`, …) e a queda para o tipo também
    // vive (uma entrada sem ajuda cai na linha do `Entity`).
    assert!(com_ajuda >= 5, "{com_ajuda} (com tipo: {com_tipo})");
    let entity = API_TYPES.iter().find(|t| t.ident == "Entity").unwrap();
    assert_eq!(
        param_doc("ninguem.tem", "entity", "Entity?"),
        Some(entity.doc)
    );
    assert_eq!(param_doc("ninguem.tem", "n", "number"), None);
    assert_eq!(
        luau_params("(f: (x: number) -> (), t: { string }, n: number?) -> ()"),
        vec![
            ("f", "(x: number) -> ()"),
            ("t", "{ string }"),
            ("n", "number?")
        ]
    );
    assert!(luau_params("() -> number").is_empty());
}

/// Os ganchos da tabela são EXACTAMENTE os nomes que o motor pede (`.hook("…")` no fonte da crate,
/// fora dos testes) — gancho novo na cena sem linha em `HOOKS` ⇒ vermelho, e vice-versa.
#[test]
fn os_ganchos_da_tabela_sao_os_que_a_cena_chama() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut chamados = BTreeSet::new();
    for f in std::fs::read_dir(&src).expect("src") {
        let p = f.expect("entrada").path();
        let nome = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        if !nome.ends_with(".rs") || nome.ends_with("_tests.rs") {
            continue;
        }
        let texto = std::fs::read_to_string(&p).expect("ler");
        for (i, _) in texto.match_indices(".hook(\"") {
            let resto = &texto[i + ".hook(\"".len()..];
            let nome = &resto[..resto.find('"').expect("fecho")];
            // Só identificadores: a prosa (`.hook("…")` num doc-comment) não é uma chamada.
            if !nome.is_empty() && nome.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                chamados.insert(nome.to_owned());
            }
        }
    }
    let na_tabela: BTreeSet<String> = HOOKS.iter().map(|h| h.ident.to_owned()).collect();
    assert!(
        !chamados.is_empty(),
        "nenhum .hook(\"…\") no fonte — o censo está cego"
    );
    assert_eq!(chamados, na_tabela, "ganchos chamados pela cena × HOOKS");
}

/// ⭐ **G5 — cada linha da API tem a sua página** (`docs/CodeEditor/04_pagina_de_ajuda.md` §6): toda
/// linha de `API`, `HOOKS` e `API_TYPES` tem EXACTAMENTE uma ajuda; os argumentos dela são os da
/// assinatura, pela ordem e cada um com texto; funções e ganchos trazem exemplo; e nenhuma ajuda
/// nomeia uma linha que não existe (texto morto).
#[test]
fn toda_entrada_da_api_tem_ajuda() {
    use ph2d_script::api_help::all_help;
    use ph2d_script::api_table::luau_params;
    // (chave, assinatura, precisa de exemplo)
    let linhas: Vec<(String, &str, bool)> = API
        .iter()
        .map(|e| (format!("{}.{}", e.table, e.ident), e.luau_type, true))
        .chain(
            HOOKS
                .iter()
                .map(|h| (format!("hook.{}", h.ident), h.luau_type, true)),
        )
        .chain(
            API_TYPES
                .iter()
                .map(|t| (format!("type.{}", t.ident), "", false)),
        )
        .collect();
    let mut erros = Vec::new();
    for (key, sig, exemplo) in &linhas {
        let ajudas: Vec<_> = all_help().filter(|h| h.key == *key).collect();
        let [h] = ajudas.as_slice() else {
            erros.push(format!("{key}: {} ajudas", ajudas.len()));
            continue;
        };
        let esperados: Vec<&str> = luau_params(sig).into_iter().map(|(n, _)| n).collect();
        let dados: Vec<&str> = h.params.iter().map(|(n, _)| *n).collect();
        if dados != esperados {
            erros.push(format!(
                "{key}: argumentos {dados:?} ≠ assinatura {esperados:?}"
            ));
        }
        for (n, t) in h.params {
            if t.trim().is_empty() {
                erros.push(format!("{key}({n}): sem texto"));
            }
        }
        let sem_retorno = sig.is_empty() || sig.trim_end().ends_with("-> ()");
        if !sem_retorno && h.returns.trim().is_empty() {
            erros.push(format!("{key}: devolve algo e não diz o quê"));
        }
        if sem_retorno && !h.returns.is_empty() {
            erros.push(format!("{key}: não devolve nada e diz «{}»", h.returns));
        }
        if *exemplo && h.example.trim().is_empty() {
            erros.push(format!("{key}: sem exemplo"));
        }
        if h.more.starts_with('\n') || h.more.ends_with('\n') || h.more.contains("\n\n\n") {
            erros.push(format!("{key}: parágrafos mal separados"));
        }
    }
    for h in all_help() {
        if !linhas.iter().any(|(k, _, _)| k == h.key) {
            erros.push(format!("{}: ajuda sem linha na tabela", h.key));
        }
    }
    assert!(
        erros.is_empty(),
        "{} problemas:\n{}",
        erros.len(),
        erros.join("\n")
    );
    // Controlo: o censo viu as três famílias.
    assert_eq!(linhas.len(), API.len() + HOOKS.len() + API_TYPES.len());
    assert!(HOOKS.len() >= 3 && API_TYPES.len() >= 8 && API.len() >= 20);
}

/// ⭐ O tipo `KeyName` do `ph2d.d.luau` é a lista do teclado (`ph2d_input::Key::script_names`), pela
/// ordem dela — escrita à mão na tabela, conferida aqui (CodeEditor item 6).
#[test]
fn os_nomes_de_tecla_sao_os_do_teclado() {
    let def = API_TYPES
        .iter()
        .find(|t| t.ident == "KeyName")
        .expect("o tipo KeyName")
        .definition;
    let na_tabela: Vec<String> = def
        .split(" | ")
        .map(|n| n.trim_matches('"').to_owned())
        .collect();
    assert_eq!(na_tabela, ph2d_input::Key::script_names());
    let botoes = API_TYPES
        .iter()
        .find(|t| t.ident == "MouseButtonName")
        .expect("o tipo MouseButtonName")
        .definition;
    let botoes: Vec<&str> = botoes.split(" | ").map(|n| n.trim_matches('"')).collect();
    let rato: Vec<&str> = ph2d_input::MouseButton::ALL
        .iter()
        .map(|b| b.script_name())
        .collect();
    assert_eq!(botoes, rato);
}
