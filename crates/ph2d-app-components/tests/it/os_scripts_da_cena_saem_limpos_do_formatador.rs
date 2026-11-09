//! ⛔ Os scripts que as cenas do produto TRAZEM saem do StyLua fixado tal como entram (CodeEditor I13):
//! o editor formata ao salvar, e o 1.º Ctrl+S do dono não pode reescrever o que ele não tocou (os
//! comentários alinhados em coluna, o `if … then return end` numa linha). Precisa de
//! `bash scripts/stylua-fetch.sh`. O par do modelo do «New script»: `ph2d-app-code` `formatar::`.
//!
//! A lista é um CENSO: todo `pub const *_LUAU: &str = r#"…"#` de `ph2d-code-scenes/src` entra
//! sozinho (a lista à mão esqueceu-se de cinco).

use ph2d_app_components::code_smoke::BROKEN_LUAU;
use ph2d_app_components::script_smoke;

/// `(nome, texto)` de cada script cru das cenas, lido das fontes da crate.
fn scripts_das_cenas() -> Vec<(String, String)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../ph2d-code-scenes/src");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .expect("ph2d-code-scenes/src")
        .map(|e| e.expect("entrada").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let src = std::fs::read_to_string(&f).expect("fonte");
        let mut rest = src.as_str();
        while let Some(i) = rest.find("pub const ") {
            rest = &rest[i + "pub const ".len()..];
            let Some((name, tail)) = rest.split_once(": &str = r#\"") else {
                continue;
            };
            if !name.ends_with("_LUAU") || name.contains(char::is_whitespace) {
                continue;
            }
            let end = tail.find("\"#;").expect("o fim do texto cru");
            out.push((name.to_owned(), tail[..end].to_owned()));
            rest = &tail[end..];
        }
    }
    out
}

#[test]
fn os_scripts_da_cena_saem_limpos_do_formatador() {
    let p = ph2d_code_format::Program::pinned().expect("`bash scripts/stylua-fetch.sh`");
    let boot = include_str!("../../src/script_boot.luau");
    let mut todos = scripts_das_cenas();
    assert!(
        todos.len() >= 10,
        "o censo encontrou só {} scripts",
        todos.len()
    );
    todos.push((
        "script_smoke::BOB_LUAU".into(),
        script_smoke::BOB_LUAU.into(),
    ));
    todos.push(("script_boot.luau".into(), boot.into()));
    let sujos: Vec<&str> = todos
        .iter()
        .filter(|(_, t)| p.format(t).as_deref() != Ok(t.as_str()))
        .map(|(n, _)| n.as_str())
        .collect();
    assert!(sujos.is_empty(), "o StyLua reescreve: {sujos:?}");
    // O partido do nível 2 é para o dono VER o erro: o formatador recusa-o e ele fica.
    assert!(p.format(BROKEN_LUAU).is_err());
}
