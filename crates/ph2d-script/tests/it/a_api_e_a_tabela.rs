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
        assert!(API_GLOBALS.iter().any(|g| g.ident == e.table), "{}", e.table);
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
        assert!(d.contains(&format!("declare {}: {{", g.ident)), "{}", g.ident);
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
