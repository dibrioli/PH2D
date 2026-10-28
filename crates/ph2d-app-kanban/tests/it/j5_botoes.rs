//! J5 — **os botões de quadro do Butler no cabeçalho**, pela costura do hero: um botão de quadro
//! posto no MODELO aparece no cabeçalho (o mesmo `view::paint` da aba e do elemento do Quadro);
//! premi-lo corre as acções em nome do Butler, num passo de desfazer; desligado, some.

use crate::Mesa;
use ph2d_kanban_auto::{Action, Kind, Scope, Spec, specs};
use ph2d_kanban_model::{KanbanId, Member, Moment};

/// Põe no modelo o botão de quadro `nome` que arquiva os cartões da lista `lista`.
fn poe_botao(m: &mut Mesa, id: KanbanId, nome: &str, lista: &str) {
    let list = m.lista(id, lista);
    let spec = Spec {
        name: nome.to_owned(),
        enabled: true,
        owner: crate::verso::eu(),
        kind: Kind::BoardButton {
            scope: Scope {
                lists: vec![list],
                conditions: Vec::new(),
            },
        },
        actions: vec![Action::Archive],
    };
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let op = spec.create(set.get(id).unwrap(), Moment(0));
    set.apply(id, vec![op]);
}

fn botoes(m: &mut Mesa, id: KanbanId) -> Vec<ph2d_editor_core::zones::Rect> {
    m.docs()
        .probe_w6_hits(id.0)
        .into_iter()
        .filter(|(n, _)| n.starts_with("BoardButton("))
        .map(|(_, r)| r)
        .collect()
}

#[test]
fn j5_o_botao_de_quadro_do_modelo_aparece_no_cabecalho_e_corre_como_butler() {
    let (mut m, id) = Mesa::cena1();
    assert!(botoes(&mut m, id).is_empty(), "sem botões, nada no cabeçalho");
    poe_botao(&mut m, id, "Sweep", "Done");
    m.quadro();
    let rs = botoes(&mut m, id);
    assert_eq!(rs.len(), 1, "o botão aparece no cabeçalho");
    let r = rs[0];
    let head = m.docs().probe_rect(id.0, "menu").unwrap();
    assert!(
        r.y >= head.y - head.h && r.x + r.w <= head.x,
        "na faixa do cabeçalho, à esquerda do «…»: {r:?} × {head:?}"
    );
    let done = m.lista(id, "Done");
    let antes = m.cartoes(id, done);
    assert!(!antes.is_empty(), "a Done tem cartões");
    m.clica((r.x + r.w / 2.0, r.y + r.h / 2.0));
    assert!(m.cartoes(id, done).is_empty(), "premir arquivou a Done");
    let k = m.kanban(id);
    assert!(
        k.activities.values().any(|a| a.by == Member::butler()),
        "a atividade diz Butler"
    );
    assert!(m.desfaz().is_some());
    assert_eq!(m.cartoes(id, done), antes, "um passo de desfazer");
    // Desligado, o botão sai do cabeçalho.
    let (a, mut s) = specs(&m.kanban(id)).remove(0);
    s.enabled = false;
    {
        let shared = m.hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        let op = s.update(&set.get(id).unwrap().automations[&a]);
        set.apply(id, vec![op]);
    }
    m.quadro();
    assert!(botoes(&mut m, id).is_empty(), "desligado, some");
}
