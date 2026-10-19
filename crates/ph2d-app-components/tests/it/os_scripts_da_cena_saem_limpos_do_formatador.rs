//! ⛔ Os scripts que as cenas do produto TRAZEM saem do StyLua fixado tal como entram (CodeEditor I13):
//! o editor formata ao salvar, e o 1.º Ctrl+S do dono não pode reescrever o que ele não tocou (os
//! comentários alinhados em coluna, o `if … then return end` numa linha). Precisa de
//! `bash scripts/stylua-fetch.sh`. O par do modelo do «New script»: `ph2d-app-code` `formatar::`.

use ph2d_app_components::{code_smoke, script_smoke};

#[test]
fn os_scripts_da_cena_saem_limpos_do_formatador() {
    let p = ph2d_code_format::Program::pinned().expect("`bash scripts/stylua-fetch.sh`");
    let boot = include_str!("../../src/script_boot.luau");
    for (nome, t) in [
        ("code_smoke::BOB_LUAU", code_smoke::BOB_LUAU),
        ("code_smoke::ONDA_LUAU", code_smoke::ONDA_LUAU),
        ("script_smoke::BOB_LUAU", script_smoke::BOB_LUAU),
        ("script_boot.luau", boot),
    ] {
        assert_eq!(p.format(t).as_deref(), Ok(t), "{nome}");
    }
    // O partido do nível 2 é para o dono VER o erro: o formatador recusa-o e ele fica.
    assert!(p.format(code_smoke::BROKEN_LUAU).is_err());
}
