//! **A tabela `ph2d.kanban`** — as funções de topo (as linhas `ph2d.kanban.*` da
//! `ph2d_script::api_table_kanban`).

use mlua::{Function, Lua, Table, Value};
use ph2d_kanban_auto::{Kind, press_board_button, press_card_button, specs};
use ph2d_kanban_filter::normalize;
use ph2d_kanban_model::{KanbanId, Member};

use crate::card::{CardH, card_ud};
use crate::event::EVENT_NAMES;
use crate::handles::{found, id_of, kanban_ud};
use crate::lane::{err, with};

fn list(lua: &Lua, (): ()) -> mlua::Result<Vec<Table>> {
    with(lua, "ph2d.kanban.list", |l| {
        l.set
            .iter()
            .map(|k| {
                let t = lua.create_table()?;
                t.set("id", k.id.0)?;
                t.set("name", k.header.name.clone())?;
                Ok(t)
            })
            .collect()
    })
}

fn get(lua: &Lua, which: Value) -> mlua::Result<Option<mlua::AnyUserData>> {
    let f = "ph2d.kanban.get";
    let id = with(lua, f, |l| {
        Ok(match &which {
            Value::String(s) => {
                let want = normalize(&s.to_string_lossy());
                l.set
                    .iter()
                    .find(|k| normalize(&k.header.name) == want)
                    .map(|k| k.id)
            }
            v => match id_of(v) {
                Some(n) => l.set.iter().find(|k| k.id.0 == n).map(|k| k.id),
                None => {
                    return Err(err(
                        f,
                        format!("got a {} — a name or a number", v.type_name()),
                    ));
                }
            },
        })
    })?;
    id.map(|k| kanban_ud(lua, k)).transpose()
}

fn search(lua: &Lua, query: String) -> mlua::Result<Vec<mlua::AnyUserData>> {
    let hits = with(lua, "ph2d.kanban.search", |l| Ok(found(l, &query)))?;
    hits.into_iter().map(|(k, c)| card_ud(lua, k, c)).collect()
}

fn on(lua: &Lua, (event, handler): (String, Function)) -> mlua::Result<()> {
    let f = "ph2d.kanban.on";
    if !EVENT_NAMES.contains(&event.as_str()) {
        return Err(err(
            f,
            format!("no event \"{event}\" (one of: {})", EVENT_NAMES.join(", ")),
        ));
    }
    with(lua, f, |l| {
        l.handlers.push((event, handler));
        Ok(())
    })
}

fn press_button(
    lua: &Lua,
    (name, card): (String, Option<mlua::AnyUserData>),
) -> mlua::Result<bool> {
    let f = "ph2d.kanban.press_button";
    let card = card
        .map(|u| {
            u.borrow::<CardH>()
                .map(|c| *c)
                .map_err(|_| err(f, "the second argument is a card"))
        })
        .transpose()?;
    with(lua, f, |l| {
        let want = normalize(&name);
        let kanbans: Vec<KanbanId> = match card {
            Some(c) => {
                l.card(f, c.kanban, c.card)?;
                vec![c.kanban]
            }
            None => l.set.iter().map(|k| k.id).collect(),
        };
        for k in kanbans {
            let Some(kb) = l.set.get(k) else { continue };
            let hit = specs(kb).into_iter().find(|(_, s)| {
                s.enabled
                    && normalize(&s.name) == want
                    && match card {
                        Some(_) => s.kind == Kind::CardButton,
                        None => matches!(s.kind, Kind::BoardButton { .. }),
                    }
            });
            let Some((id, _)) = hit else { continue };
            let out = match card {
                Some(c) => press_card_button(&l.set, k, id, c.card, &l.by, l.now, &l.zone),
                None => press_board_button(&l.set, k, id, &l.by, l.now, &l.zone),
            };
            l.write_plan(out.plan, &Member::butler());
            return Ok(true);
        }
        Ok(false)
    })
}

/// Põe `ph2d.kanban` na VM (antes da sandbox).
///
/// # Errors
/// Se a VM recusar criar as funções.
pub fn wire(lua: &Lua) -> mlua::Result<()> {
    let t = lua.create_table()?;
    t.set("list", lua.create_function(list)?)?;
    t.set("get", lua.create_function(get)?)?;
    t.set("search", lua.create_function(search)?)?;
    t.set("on", lua.create_function(on)?)?;
    t.set("press_button", lua.create_function(press_button)?)?;
    let ph2d: Table = match lua.globals().get::<Option<Table>>("ph2d")? {
        Some(p) => p,
        None => {
            let p = lua.create_table()?;
            lua.globals().set("ph2d", p.clone())?;
            p
        }
    };
    ph2d.set("kanban", t)
}
