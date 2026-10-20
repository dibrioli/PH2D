//! O catálogo contra o runtime REAL (`ScriptHost`, com a sandbox): todo nome que um script alcança tem
//! página, todo membro de página existe no runtime, todo texto foi escrito, e todo exemplo corre e
//! escreve o que diz (`docs/CodeEditor/04_pagina_de_ajuda.md` §6, G1 e G2).

use mlua::{Table, Value};
use ph2d_code_help::{Kind, catalog, page};
use ph2d_script::ScriptHost;
use std::collections::BTreeSet;

/// As páginas que não são uma tabela do runtime.
const NOT_IN_RUNTIME: &[&str] = &["types", "hooks"];

/// `PH2D_HELP_SO=<página>` restringe os três gates a uma página (a escrever o texto de uma biblioteca);
/// sem ela, todas.
fn wanted(page: &str) -> bool {
    std::env::var("PH2D_HELP_SO").map_or(true, |so| so == page)
}

/// A bancada de quem escreve o texto: corre o Luau de `PH2D_HELP_RUN=<ficheiro>` no runtime REAL e
/// imprime o que o `print` escreveu (vários valores como `a, b`, como as linhas `-->` do exemplo).
/// `PH2D_HELP_RUN=x.luau cargo test -p ph2d-code-help --test it corre -- --ignored --nocapture`
#[test]
#[ignore = "bancada: só com PH2D_HELP_RUN"]
fn corre() {
    let path = std::env::var("PH2D_HELP_RUN").expect("PH2D_HELP_RUN=<ficheiro .luau>");
    let src = std::fs::read_to_string(&path).expect("o ficheiro lê-se");
    let host = ScriptHost::new().expect("o runtime monta");
    let r = host.runtime().eval(&src);
    for l in host.drain_console() {
        println!("{}", l.text.replace('\t', ", "));
    }
    if let Err(e) = r {
        println!("ERRO: {e}");
    }
}

/// Os pares de `t` e da cadeia `__index` da metatabela dele: com a sandbox, os globais de um script
/// são uma tabela VAZIA cujo `__index` é a dos globais congelados.
fn reachable(t: &Table) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = t
        .pairs::<String, Value>()
        .map(|p| p.expect("chave de texto"))
        .collect();
    if let Some(Value::Table(up)) = t
        .metatable()
        .map(|m| m.raw_get::<Value>("__index").expect("__index"))
    {
        out.extend(reachable(&up));
    }
    out
}

/// `(página, membro)` de tudo o que o runtime põe ao alcance de um script.
fn runtime_names() -> BTreeSet<(String, String)> {
    let host = ScriptHost::new().expect("o runtime monta");
    let mut out = BTreeSet::new();
    for (name, value) in reachable(&host.runtime().lua().globals()) {
        match value {
            Value::Table(t) if name != "_G" => {
                for (m, _) in reachable(&t) {
                    out.insert((name.clone(), m));
                }
            }
            _ => {
                out.insert(("globals".to_owned(), name));
            }
        }
    }
    out
}

fn is_text(s: &str) -> bool {
    !s.trim().is_empty() && !s.starts_with("code.help.")
}

#[test]
fn toda_global_do_runtime_tem_pagina() {
    let runtime: BTreeSet<_> = runtime_names()
        .into_iter()
        .filter(|(p, _)| wanted(p))
        .collect();
    let documented: BTreeSet<(String, String)> = catalog()
        .iter()
        .filter(|p| !NOT_IN_RUNTIME.contains(&p.id) && wanted(p.id))
        .flat_map(|p| {
            p.members
                .iter()
                .map(|m| (p.id.to_owned(), m.name.to_owned()))
        })
        .collect();
    let missing: Vec<String> = runtime
        .difference(&documented)
        .map(|(p, m)| format!("{p}.{m}"))
        .collect();
    let dead: Vec<String> = documented
        .difference(&runtime)
        .map(|(p, m)| format!("{p}.{m}"))
        .collect();
    assert!(
        missing.is_empty() && dead.is_empty(),
        "{} nomes do runtime sem página: {missing:?}\n{} membros sem nome no runtime: {dead:?}",
        missing.len(),
        dead.len()
    );
}

#[test]
fn todo_texto_foi_escrito() {
    let mut falta = Vec::new();
    for p in catalog().iter().filter(|p| wanted(p.id)) {
        for (what, s) in [("title", &p.title), ("summary", &p.summary)] {
            if !is_text(s) {
                falta.push(format!("{} {what}: {s:?}", p.id));
            }
        }
        if p.about.starts_with("code.help.") {
            falta.push(format!("{} about: {:?}", p.id, p.about));
        }
        for m in &p.members {
            let at = format!("{}.{}", p.id, m.name);
            if !is_text(&m.text) {
                falta.push(format!("{at}: {:?}", m.text));
            }
            if m.sigs.is_empty() || m.sigs.iter().any(|s| s.trim().is_empty()) {
                falta.push(format!("{at}: sem assinatura"));
            }
            for prm in &m.params {
                if !is_text(&prm.text) {
                    falta.push(format!("{at}({}): {:?}", prm.name, prm.text));
                }
            }
            if let Some(r) = &m.returns {
                if !is_text(r) {
                    falta.push(format!("{at} returns: {r:?}"));
                }
            }
            if m.kind == Kind::Function && m.example.is_none() {
                falta.push(format!("{at}: sem exemplo"));
            }
        }
    }
    assert!(
        falta.is_empty(),
        "{} textos por escrever:\n{}",
        falta.len(),
        falta.join("\n")
    );
}

/// O que as linhas `--> x` do exemplo dizem que o `print` escreve (vários valores: `a, b`).
fn promised(example: &str) -> Vec<String> {
    example
        .lines()
        .filter_map(|l| l.split_once("--> ").map(|(_, r)| r.trim_end().to_owned()))
        .collect()
}

#[test]
fn cada_exemplo_corre_e_da_o_que_diz() {
    let mut erros = Vec::new();
    let mut corridos = 0;
    for p in catalog()
        .iter()
        .filter(|p| wanted(p.id) && !matches!(p.id, "ph2d" | "motion" | "types" | "hooks"))
    {
        for m in &p.members {
            let Some(example) = m.example else { continue };
            let host = ScriptHost::new().expect("o runtime monta");
            let at = format!("{}.{}", p.id, m.name);
            if let Err(e) = host.runtime().eval(example) {
                erros.push(format!("{at}: não correu: {e}"));
                continue;
            }
            corridos += 1;
            let got: Vec<String> = host
                .drain_console()
                .into_iter()
                .map(|l| l.text.replace('\t', ", "))
                .collect();
            let want = promised(example);
            if got != want {
                erros.push(format!("{at}: escreveu {got:?}, a página diz {want:?}"));
            }
        }
    }
    assert!(
        erros.is_empty(),
        "{} de {corridos} exemplos:\n{}",
        erros.len(),
        erros.join("\n")
    );
    assert!(
        !wanted("math")
            || page("math").is_some_and(|p| p.members.iter().all(|m| m.example.is_some())),
        "o controlo: math tem exemplos"
    );
}
