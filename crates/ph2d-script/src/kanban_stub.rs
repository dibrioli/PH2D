//! **`ph2d.kanban` na cena** — a VM da cena tem as chaves da tabela (o gate `a_api_e_a_tabela`), mas
//! os Kanbans só se alcançam num script de Kanban (`ph2d-kanban-script`, que instala as funções de
//! verdade na VM dele). Aqui cada função diz isso, em vez de um «attempt to call a nil value».

use mlua::Lua;

use crate::api_table_kanban::{KANBAN_API, KANBAN_GLOBAL};

/// O que cada função diz na cena.
pub(crate) const SCENE_REFUSAL: &str =
    "the project's Kanbans are reached only by a Kanban script, not while the scene plays";

pub(crate) fn wire(lua: &Lua) -> mlua::Result<()> {
    let t = lua.create_table()?;
    for e in KANBAN_API.iter().filter(|e| e.table == KANBAN_GLOBAL.ident) {
        let name = format!("{}.{}", e.table, e.ident);
        let f = lua.create_function(move |_, _: mlua::MultiValue| -> mlua::Result<()> {
            Err(mlua::Error::RuntimeError(format!(
                "{name}: {SCENE_REFUSAL}"
            )))
        })?;
        t.set(e.ident, f)?;
    }
    let ph2d: mlua::Table = lua.globals().get("ph2d")?;
    ph2d.set("kanban", t)
}
