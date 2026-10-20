//! **A PÁGINA DE AJUDA do editor de scripts** (CodeEditor Onda 2, item 5;
//! `docs/CodeEditor/04_pagina_de_ajuda.md`): as palavras da página e o texto de cada biblioteca do
//! Luau, escrito por nós. `code.help.<página>` é a frase do topo, `.about` o resto, `.<membro>` o que
//! ele faz, `.<membro>.<argumento>` (`vararg` para `...`) e `.<membro>.returns`. O censo do
//! `ph2d-code-help` reprova um nome do runtime sem texto.

mod bit32;
mod buffer;
mod coroutine;
mod debug;
mod globals;
mod integer;
mod math;
mod os;
mod string;
mod table;
mod utf8;
mod vector;

/// A tradução de uma chave `code.help.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    let rest = key.strip_prefix("code.help.")?;
    let page = rest.split('.').next()?;
    match page {
        "globals" => globals::tr(key),
        "bit32" => bit32::tr(key),
        "buffer" => buffer::tr(key),
        "coroutine" => coroutine::tr(key),
        "debug" => debug::tr(key),
        "integer" => integer::tr(key),
        "math" => math::tr(key),
        "os" => os::tr(key),
        "string" => string::tr(key),
        "table" => table::tr(key),
        "utf8" => utf8::tr(key),
        "vector" => vector::tr(key),
        _ => ours(key),
    }
}

/// As páginas nossas que não saem da `api_table` (`types`, `hooks`) e as palavras da página.
fn ours(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.types.title" => "Types",
        "code.help.types" => "The types the PH2D functions take and give.",
        "code.help.types.about" => {
            "Write them after a name to check your script: `local field: PoseField = \"x\"`. Add `--!strict` at the top and the editor checks every line."
        }
        "code.help.hooks.title" => "Script hooks",
        "code.help.hooks" => "Functions your script declares and PH2D calls.",
        "code.help.hooks.about" => {
            "Declare them at the top level with `function`, never `local function`: PH2D looks them up by name. Each one gets the object's `self`."
        }
        _ => return None,
    })
}
