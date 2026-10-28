//! `ph2d.kanban.on`: os mesmos acontecimentos do Butler, a cascata com as regras e o anti-laço.

use ph2d_kanban_auto::{Action, Added, By, Dest, End, Kind, MAX_DEPTH, Spec, Stop, Trigger};
use ph2d_kanban_script::script_member;

use crate::{Scene, enio, ok};

#[test]
fn um_handler_reage_ao_gesto_de_uma_pessoa() {
    let mut s = Scene::new();
    let c = s.card(s.todo, "Som", |_| {});
    let r = s.run(
        "aviso.luau",
        r#"
ph2d.kanban.on("card_moved", function(e)
  e.card:comment(e.by .. " moved it from " .. e.from.name .. " to " .. e.list.name)
end)
"#,
    );
    ok(&r);
    assert_eq!(s.scripts.loaded(), [("aviso.luau", 1)]);
    let r = s.person_moves(c, s.doing);
    ok(&r);
    assert_eq!(r.handlers, 1);
    let k = s.set.get(s.k).unwrap();
    let cm = k
        .comments
        .values()
        .find(|x| x.card == c)
        .expect("o handler comentou");
    assert_eq!(cm.text, "Enio moved it from To Do to Doing");
    assert_eq!(cm.author, script_member("aviso.luau"));
    // O que o handler fez é o passo de cima: o 1.º Ctrl+Z tira o comentário, o 2.º o movimento.
    assert!(s.undo());
    assert_eq!(s.photo().2, 0);
    assert_eq!(s.set.get(s.k).unwrap().cards[&c].list, s.doing);
    assert!(s.undo());
    assert_eq!(s.set.get(s.k).unwrap().cards[&c].list, s.todo);
}

#[test]
fn um_handler_nao_corre_pelo_que_ele_mesmo_fez() {
    let mut s = Scene::new();
    let c = s.card(s.todo, "Som", |_| {});
    // Mover de volta acorda «card_moved» de novo — sem a lei, correria para sempre.
    let r = s.run(
        "teimoso.luau",
        r#"
ph2d.kanban.on("card_moved", function(e)
  if e.list.name == "Doing" then e.card:move("To Do") else e.card:move("Doing") end
end)
"#,
    );
    ok(&r);
    let r = s.person_moves(c, s.done);
    ok(&r);
    assert_eq!((r.handlers, r.stopped), (1, None), "corre uma vez e pára");
    assert_eq!(s.set.get(s.k).unwrap().cards[&c].list, s.doing);
}

#[test]
fn dois_scripts_e_uma_regra_em_ping_pong_param() {
    let mut s = Scene::new();
    let c = s.card(s.todo, "Som", |_| {});
    // A regra: entra em Done → vai para Doing. O script: entra em Doing → vai para Done.
    s.add(&Spec {
        name: String::new(),
        enabled: true,
        owner: enio(),
        kind: Kind::Rule {
            trigger: Trigger::CardAdded {
                list: Some(s.done),
                how: Added::Any,
            },
            by: By::Anyone,
            conditions: Vec::new(),
        },
        actions: vec![Action::MoveTo {
            to: Dest {
                kanban: None,
                list: s.doing,
            },
            at: End::Bottom,
        }],
    });
    let src = r#"
ph2d.kanban.on("card_moved", function(e)
  if e.list.name == "Doing" then e.card:move("Done") end
end)
"#;
    ok(&s.run("a.luau", src));
    ok(&s.run("b.luau", src));
    let r = s.person_moves(c, s.doing);
    ok(&r);
    // Acaba SOZINHO, sem tecto: cada handler e a regra encontram-se na própria cadeia (o mesmo
    // gesto dá sempre a mesma contagem).
    assert_eq!((r.handlers, r.butler.len(), r.stopped), (6, 4, None));
    assert_eq!(s.set.get(s.k).unwrap().cards[&c].list, s.doing);
}

#[test]
fn uma_cadeia_mais_longa_que_max_depth_para_ali() {
    let mut s = Scene::new();
    let n = MAX_DEPTH + 4;
    for i in 0..=n {
        s.label(&format!("L{i}"));
    }
    let c = s.card(s.todo, "Som", |_| {});
    let mut src = String::new();
    for i in 0..n {
        src.push_str(&format!(
            "ph2d.kanban.on(\"label_added\", function(e) if e.label == \"L{i}\" then e.card:add_label(\"L{}\") end end)\n",
            i + 1
        ));
    }
    src.push_str(&format!(
        "ph2d.kanban.get(\"Projeto\"):cards()[1]:add_label(\"L0\") -- {}\n",
        c.0
    ));
    let r = s.run("cadeia.luau", &src);
    ok(&r);
    assert_eq!(r.stopped, Some(Stop::Depth));
    // Os 20 handlers ouvem «label_added»: cada nível corre os que ainda não estão na cadeia.
    let corridas: usize = (0..MAX_DEPTH).map(|d| n - d).sum();
    assert_eq!(r.handlers, corridas);
    let labels = s.set.get(s.k).unwrap().cards[&c].labels.len();
    assert_eq!(labels, MAX_DEPTH + 1, "L0 e uma por handler da cadeia");
}

#[test]
fn um_handler_que_falha_perde_as_suas_escritas_e_o_resto_continua() {
    let mut s = Scene::new();
    let c = s.card(s.todo, "Som", |_| {});
    ok(&s.run(
        "bom.luau",
        r#"ph2d.kanban.on("card_moved", function(e) e.card:comment("ok") end)"#,
    ));
    ok(&s.run(
        "mau.luau",
        r#"ph2d.kanban.on("card_moved", function(e) e.card:comment("meio"); error("parou") end)"#,
    ));
    let r = s.person_moves(c, s.doing);
    assert_eq!(r.errors.len(), 1);
    assert_eq!(r.errors[0].script, "mau.luau");
    assert!(r.errors[0].message.contains("parou"), "{:?}", r.errors);
    let k = s.set.get(s.k).unwrap();
    let texts: Vec<&str> = k.comments.values().map(|x| x.text.as_str()).collect();
    assert_eq!(texts, ["ok"]);
    let r = s.run(
        "evento.luau",
        r#"ph2d.kanban.on("card_teleported", function() end)"#,
    );
    assert!(
        r.errors[0].message.contains("no event \"card_teleported\""),
        "{:?}",
        r.errors
    );
}
