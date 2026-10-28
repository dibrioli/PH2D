//! Ler e escrever: criar lista e cartão, mover, filtrar, comentar, marcar, desfazer, falhar.

use ph2d_kanban_auto::{Action, Kind, Scope, Spec};
use ph2d_kanban_model::Member;
use ph2d_kanban_script::script_member;

use crate::{Scene, enio, ok};

#[test]
fn um_script_cria_lista_e_cartao_e_move_o_cartao() {
    let mut s = Scene::new();
    let r = s.run(
        "plan.luau",
        r#"
local k = ph2d.kanban.get("projeto")
assert(k, "o Kanban pelo nome, sem caixa")
local review = k:add_list("Review")
local c = review:add_card({ title = "Storyboard", labels = { "Urgent" }, due = "2026-11-03T14:30" })
assert(c.list == review and c.due == "2026-11-03T14:30" and c.labels[1] == "Urgent")
c:move("Done", 1)
assert(c.list.name == "Done", c.list.name)
local first = k:list("To Do"):add_card("Primeiro")
local top = k:list("To Do"):add_card({ title = "Topo", top = true })
assert(k:list("To Do"):cards()[1] == top and k:list("To Do"):cards()[2] == first)
print(#k:lists(), c.url)
"#,
    );
    ok(&r);
    assert_eq!(
        r.printed,
        [(
            "plan.luau".to_owned(),
            format!(
                "4\tph2d://kanban/{}/card/{}",
                s.k.0,
                s.by_title("Storyboard").unwrap().id.0
            )
        )]
    );
    let c = s.by_title("Storyboard").expect("criado");
    assert_eq!(c.list, s.done);
    assert_eq!(c.labels.len(), 1);
    // A actividade diz quem foi.
    let k = s.set.get(s.k).unwrap();
    let by_script = k
        .activities
        .values()
        .filter(|a| a.by == script_member("plan.luau"))
        .count();
    assert!(by_script >= 3, "{by_script}");
}

#[test]
fn filtrar_pesquisar_e_ler_os_campos() {
    let mut s = Scene::new();
    let urgent = s.set.get(s.k).unwrap().labels.values().next().unwrap().id;
    let a = s.card(s.todo, "Som", |c| c.labels.push(urgent));
    s.card(s.doing, "Cor", |_| {});
    s.card(s.done, "Luz", |c| {
        c.labels.push(urgent);
        c.members.push(Member("Ana".into()));
    });
    s.checklist(a, &["Gravar", "Misturar"]);
    let r = s.run(
        "ler.luau",
        r##"
local k = ph2d.kanban.get(1)
local t = {}
for _, c in k:cards("#urgent") do table.insert(t, c.title) end
print(table.concat(t, ","))
local u = {}
for _, c in ph2d.kanban.search("@ana") do table.insert(u, c.title .. "@" .. c.list.name) end
print(table.concat(u, ","))
local som = k:cards()[1]
print(som.title, #som.checklists, som.checklists[1].items[2].text, tostring(som.done))
print(table.concat(k:members(), ","), #k:labels(), k:labels()[1].color)
print(#ph2d.kanban.list(), ph2d.kanban.list()[1].name, tostring(ph2d.kanban.get("nenhum")))
"##,
    );
    ok(&r);
    let lines: Vec<&str> = r.printed.iter().map(|(_, l)| l.as_str()).collect();
    assert_eq!(
        lines,
        [
            "Som,Luz",
            "Luz@Done",
            "Som\t1\tMisturar\tfalse",
            "Ana\t1\tred",
            "1\tProjeto\tnil"
        ]
    );
    assert!(!r.changed, "ler não muda nada");
}

#[test]
fn comentar_marcar_mudar_e_arquivar() {
    let mut s = Scene::new();
    let a = s.card(s.todo, "Som", |_| {});
    s.checklist(a, &["Gravar", "Misturar"]);
    let r = s.run(
        "escreve.luau",
        r#"
local c = ph2d.kanban.get("Projeto"):cards()[1]
c:comment("pronto, @Ana")
c:check("gravar")
c:add_member("Ana")
c:set({ title = "Som final", description = "**ok**", due = { year = 2026, month = 11, day = 3, hour = 9 }, done = true })
assert(c.checklists[1].items[1].done and c.title == "Som final" and c.due == "2026-11-03T09:00" and c.done)
c:remove_label("Urgent")
c:archive()
assert(c.archived)
"#,
    );
    ok(&r);
    let k = s.set.get(s.k).unwrap();
    let c = &k.cards[&a];
    assert!(c.archived && c.due.unwrap().done && c.members == [Member("Ana".into())]);
    let cm = k
        .comments
        .values()
        .find(|x| x.card == a)
        .expect("comentário");
    assert_eq!(
        (cm.text.as_str(), &cm.author),
        ("pronto, @Ana", &script_member("escreve.luau"))
    );
    assert!(k.items.values().any(|i| i.text == "Gravar" && i.done));
}

#[test]
fn desfazer_reverte_a_corrida_inteira_num_passo() {
    let mut s = Scene::new();
    s.card(s.todo, "Som", |_| {});
    let antes = s.photo();
    let r = s.run(
        "muitos.luau",
        r#"
local k = ph2d.kanban.get("Projeto")
local l = k:add_list("Review")
for i = 1, 5 do l:add_card("c" .. i):comment("x") end
k:cards()[1]:move(l)
"#,
    );
    ok(&r);
    assert!(r.changed);
    assert_ne!(s.photo(), antes);
    assert!(s.undo(), "há um passo");
    assert_eq!(s.photo(), antes, "um Ctrl+Z devolve a corrida inteira");
    assert!(!s.undo(), "e era UM passo");
}

#[test]
fn um_script_que_falha_nao_deixa_nada_e_diz_porque() {
    let mut s = Scene::new();
    s.card(s.todo, "Som", |_| {});
    let antes = s.photo();
    let r = s.run(
        "mau.luau",
        r#"
local k = ph2d.kanban.get("Projeto")
k:add_list("Meia")
k:cards()[1]:add_label("Inexistente")
"#,
    );
    assert_eq!(r.errors.len(), 1);
    let m = &r.errors[0].message;
    assert!(
        m.contains("card:add_label: no label \"Inexistente\" in \"Projeto\" (it has \"Urgent\")"),
        "{m}"
    );
    assert_eq!(s.photo(), antes, "nada da corrida pousou");
    assert!(!r.changed && !s.undo());
    assert!(
        s.scripts.loaded().is_empty(),
        "um topo que falha não fica carregado"
    );
    let r = s.run("laco.luau", "while true do end");
    assert!(
        r.errors[0].message.contains("took longer than"),
        "{:?}",
        r.errors
    );
}

#[test]
fn ordenar_e_carregar_num_botao() {
    let mut s = Scene::new();
    s.card(s.todo, "b", |_| {});
    s.card(s.todo, "a", |_| {});
    s.add(&Spec {
        name: "Arquivar feitos".into(),
        enabled: true,
        owner: enio(),
        kind: Kind::BoardButton {
            scope: Scope {
                lists: vec![s.done],
                conditions: Vec::new(),
            },
        },
        actions: vec![Action::Archive],
    });
    s.card(s.done, "feito", |_| {});
    let r = s.run(
        "botao.luau",
        r#"
local k = ph2d.kanban.get("Projeto")
k:sort("To Do", "name")
assert(k:list("To Do"):cards()[1].title == "a")
assert(ph2d.kanban.press_button("arquivar feitos"))
assert(not ph2d.kanban.press_button("nenhum"))
assert(#k:list("Done"):cards() == 0)
"#,
    );
    ok(&r);
    let k = s.set.get(s.k).unwrap();
    assert!(
        k.activities.values().any(|a| a.by == Member::butler()),
        "o botão age em nome do Butler"
    );
}
