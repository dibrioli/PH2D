//! ⭐ A API que a VM do script TEM é a da tabela (`ph2d_script::api_table_kanban`), nos dois
//! sentidos — e a cena tem as mesmas chaves, a dizer onde correm.

use std::collections::BTreeSet;

use mlua::{Lua, Table, Value};
use ph2d_kanban_script::{EVENT_NAMES, wire};
use ph2d_script::api_table::api;
use ph2d_script::api_table_kanban::{KANBAN_CLASSES, KANBAN_EVENT_NAMES, KANBAN_GLOBAL};

use crate::{Scene, ok};

#[test]
fn as_funcoes_de_ph2d_kanban_sao_as_da_tabela() {
    let lua = Lua::new();
    wire(&lua).expect("fiação");
    let ph2d: Table = lua.globals().get("ph2d").expect("ph2d");
    let t: Table = ph2d.get("kanban").expect("ph2d.kanban");
    let na_vm: BTreeSet<String> = t
        .pairs::<String, Value>()
        .map(|p| p.expect("par").0)
        .collect();
    let na_tabela: BTreeSet<String> = api()
        .filter(|e| e.table == KANBAN_GLOBAL.ident)
        .map(|e| e.ident.to_owned())
        .collect();
    assert_eq!(na_vm, na_tabela);
}

/// Cada método e cada campo de cada classe da tabela existe na pega — pedido por um script.
#[test]
fn as_classes_da_tabela_existem_nas_pegas() {
    let mut s = Scene::new();
    s.card(s.todo, "Som", |_| {});
    let mut src = String::from(
        "local k = ph2d.kanban.get(\"Projeto\")\nlocal objs = { Kanban = k, KanbanList = k:lists()[1], KanbanCard = k:cards()[1] }\n",
    );
    let mut vistos = 0;
    for c in KANBAN_CLASSES {
        for e in api().filter(|e| e.table == c.ident) {
            src.push_str(&format!(
                "assert(type(objs.{0}.{1}) == \"function\", \"{0}:{1}\")\n",
                c.ident, e.ident
            ));
            vistos += 1;
        }
        for f in c.fields.iter().filter(|f| !f.luau_type.ends_with('?')) {
            src.push_str(&format!(
                "assert(objs.{0}.{1} ~= nil, \"{0}.{1}\")\n",
                c.ident, f.ident
            ));
            vistos += 1;
        }
    }
    ok(&s.run("censo.luau", &src));
    assert!(vistos >= 30, "{vistos}");
}

#[test]
fn os_nomes_dos_acontecimentos_sao_os_do_tipo() {
    let tipo: Vec<&str> = KANBAN_EVENT_NAMES
        .split(" | ")
        .map(|n| n.trim_matches('"'))
        .collect();
    assert_eq!(tipo, EVENT_NAMES);
}

#[test]
fn na_cena_ph2d_kanban_diz_onde_corre() {
    let host = ph2d_script::ScriptHost::new().expect("vm da cena");
    let e = host
        .runtime()
        .eval("ph2d.kanban.list()")
        .expect_err("na cena não corre");
    assert!(
        e.to_string().contains(
            "ph2d.kanban.list: the project's Kanbans are reached only by a Kanban script"
        ),
        "{e}"
    );
}

/// ⭐ Cada exemplo das páginas de `ph2d.kanban` (o que o Ctrl+clique abre) corre sem erro numa VM de
/// Kanban — o censo do `ph2d-code-help` deixa-os para aqui.
#[test]
fn cada_exemplo_da_ajuda_corre_num_script_de_kanban() {
    use ph2d_kanban_model::{KanbanOp, Moment};
    use ph2d_script::api_help::all_help;
    let mut corridos = 0;
    for h in all_help().filter(|h| !h.example.is_empty()) {
        let kanban = h.key.starts_with("ph2d.kanban.") || h.key.starts_with("Kanban");
        if !kanban {
            continue;
        }
        let mut s = Scene::new();
        let mut header = s.set.get(s.k).unwrap().header.clone();
        header.name = "Short film".into();
        s.set.apply(s.k, vec![KanbanOp::Header(header)]);
        s.list("Review");
        for l in ["Late", "Art"] {
            s.label(l);
        }
        s.card(s.todo, "Storyboard", |c| {
            c.due = Some(ph2d_kanban_model::Due {
                at: Moment(0),
                done: false,
                remind_before_min: None,
            });
        });
        let r = s.run(h.key, h.example);
        assert!(r.errors.is_empty(), "{}: {:?}", h.key, r.errors);
        corridos += 1;
    }
    assert!(corridos >= 20, "{corridos}");
}
