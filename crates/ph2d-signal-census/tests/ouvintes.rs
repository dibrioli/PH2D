//! Quem ouve um nome (doc Timeline 17 item 8): o objecto e a linha da tabela, só os que OUVEM.

use ph2d_ecs::World;
use ph2d_ecs::name::Name;
use ph2d_ecs::signal_actions::{SignalAction, SignalActions};
use ph2d_ecs::signal_on_action::{ActionTriggerRow, SignalOnAction};
use ph2d_script::{LuauScript, ProjectScripts};
use ph2d_script_doc::ScriptLibrary;
use ph2d_signal_census::{Fora, Listener, Quem, listeners};

fn acao(on: &str) -> SignalAction {
    SignalAction {
        on: on.into(),
        ..Default::default()
    }
}

fn ouvem(w: &World, name: &str) -> Vec<Listener> {
    listeners(w, &Fora::default(), name).list
}

fn hero_signal_actions() -> Listener {
    Listener {
        object: Some("Hero".into()),
        what: Quem::Component("SignalActions"),
    }
}

/// *Mutação: tirar o filtro `Role::Listen` ⇒ o emissor entra; tirar o `n == name` ⇒ «jump» entra.*
#[test]
fn o_signal_actions_que_ouve_land_e_listado_e_o_emissor_nao() {
    let mut w = World::new();
    w.spawn((Name::new("Hero"), SignalActions(vec![acao("land")])));
    w.spawn((
        Name::new("Coin"),
        SignalOnAction(vec![ActionTriggerRow {
            signal: "land".into(),
            ..Default::default()
        }]),
    ));
    w.spawn((Name::new("Door"), SignalActions(vec![acao("jump")])));
    assert_eq!(ouvem(&w, "land"), vec![hero_signal_actions()]);
    assert_eq!(
        ouvem(&w, "jump"),
        vec![Listener {
            object: Some("Door".into()),
            what: Quem::Component("SignalActions"),
        }]
    );
    assert!(ouvem(&w, "").is_empty(), "nome vazio: ninguém");
    assert!(ouvem(&w, "lan").is_empty(), "nome exato");
}

/// Duas acções do mesmo objecto sobre o mesmo nome são UMA linha. *Mutação: tirar o `contains` ⇒ duas.*
#[test]
fn duas_linhas_do_mesmo_objeto_sao_um_ouvinte() {
    let mut w = World::new();
    w.spawn((
        Name::new("Hero"),
        SignalActions(vec![acao("land"), acao("land")]),
    ));
    assert_eq!(ouvem(&w, "land"), vec![hero_signal_actions()]);
}

/// O script que faz `ph2d.connect(obj, "land", …)` é listado no objecto dele; o `on_signal` é «may».
#[test]
fn o_script_que_conecta_o_nome_e_listado_como_script() {
    let mut w = World::new();
    let mut lib = ScriptLibrary::new();
    let id = lib.insert("s", "ph2d.connect(other, \"land\", function(self) end)\n");
    let tudo = lib.insert("t", "function on_signal(self, s, from) end");
    w.insert_resource(ProjectScripts::new(lib));
    w.spawn((Name::new("Boss"), LuauScript::library(id)));
    let l = listeners(&w, &Fora::default(), "land");
    assert_eq!(
        l.list,
        vec![Listener {
            object: Some("Boss".into()),
            what: Quem::Script,
        }]
    );
    assert!(!l.script_may);
    w.spawn((Name::new("Any"), LuauScript::library(tudo)));
    let l = listeners(&w, &Fora::default(), "land");
    assert_eq!(l.list.len(), 1, "o on_signal não é um ouvinte nomeado");
    assert!(l.script_may, "mas pode ouvir");
}

/// Dois objectos com o MESMO nome (ou sem nome) que ouvem são DUAS linhas: a deduplicação é por
/// entidade. *Mutação: deduplicar por (nome, o que ouve) ⇒ uma linha só.*
#[test]
fn dois_objetos_com_o_mesmo_nome_sao_dois_ouvintes() {
    let mut w = World::new();
    w.spawn((Name::new("Hero"), SignalActions(vec![acao("land")])));
    w.spawn((Name::new("Hero"), SignalActions(vec![acao("land")])));
    w.spawn(SignalActions(vec![acao("land")]));
    w.spawn(SignalActions(vec![acao("land")]));
    let l = ouvem(&w, "land");
    assert_eq!(l.len(), 4, "{l:?}");
    assert_eq!(l.iter().filter(|x| x.object.is_none()).count(), 2);
}
