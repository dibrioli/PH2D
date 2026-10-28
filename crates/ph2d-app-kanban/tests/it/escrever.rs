//! Criar e escrever no lugar: «New Kanban» do menu File, «Add another list», «Add a card».

use crate::{AREA, Mesa, centro};
use ph2d_editor_core::interaction::WidgetEvent;

#[test]
fn new_kanban_abre_a_aba_com_o_nome_em_edicao_e_enter_renomeia() {
    let mut m = Mesa::new();
    m.quadro();
    assert!(
        m.hero
            .apply_event(WidgetEvent::Click(ph2d_app_kanban::CMD_NEW)),
        "o comando foi tomado"
    );
    m.quadro();
    let id = m
        .hero
        .documents
        .kanbans()
        .iter()
        .last()
        .map(|k| k.id)
        .expect("um Kanban criado");
    assert_eq!(m.activa(), Some(id.0), "a aba activa é a do Kanban novo");
    let nome = m.kanban(id).header.name;
    assert_eq!(
        m.docs().probe_field(id.0),
        Some(nome),
        "o nome abre em edição, com o texto todo"
    );
    m.quadro();
    assert!(
        m.docs().probe_field(id.0).is_some(),
        "o campo sobrevive aos quadros seguintes"
    );
    m.escreve("Sprint 1");
    m.enter();
    assert_eq!(m.kanban(id).header.name, "Sprint 1");
    assert_eq!(
        m.docs().probe_field(id.0),
        None,
        "o nome não é compositor: fecha"
    );
}

#[test]
fn add_another_list_cria_a_lista_e_o_compositor_fica_aberto() {
    let (mut m, id) = Mesa::cena1();
    // O «Add another list» está à direita, fora da vista: a roda no fundo do quadro rola-o.
    let fora = m.docs().probe_rect(id.0, "add_list").expect("add_list");
    assert!(
        fora.x + fora.w > AREA.w,
        "a cena começa com ele fora: {fora:?}"
    );
    for _ in 0..20 {
        assert!(m.roda(600.0, AREA.h - 20.0, 1.0), "a roda foi tomada");
    }
    let add = m.docs().probe_rect(id.0, "add_list").expect("add_list");
    assert!(add.x + add.w <= AREA.w, "rolou até se ver: {add:?}");
    m.clica(centro(add));
    assert_eq!(m.docs().probe_field(id.0), Some(String::new()));
    m.escreve("QA");
    m.enter();
    let k = m.kanban(id);
    let ultima = k.lists_in_order().last().map(|l| l.title.clone());
    assert_eq!(ultima.as_deref(), Some("QA"), "a lista nova é a última");
    assert_eq!(
        m.docs().probe_field(id.0),
        Some(String::new()),
        "o compositor fica aberto, vazio"
    );
}

#[test]
fn add_a_card_cria_no_fim_e_o_segundo_enter_depois_dele() {
    let (mut m, id) = Mesa::cena1();
    let todo = m.lista(id, "To Do");
    let antes = m.cartoes(id, todo);
    let (add, _, _) = m.docs().probe_list_buttons(id.0, todo).expect("botões");
    m.clica(centro(add));
    assert_eq!(m.docs().probe_field(id.0), Some(String::new()));
    m.escreve("Rig the fox");
    m.enter();
    let k = m.kanban(id);
    let agora = m.cartoes(id, todo);
    assert_eq!(agora.len(), antes.len() + 1);
    assert_eq!(&agora[..antes.len()], &antes[..]);
    assert_eq!(k.cards[agora.last().unwrap()].title, "Rig the fox");
    assert_eq!(m.docs().probe_field(id.0), Some(String::new()));
    m.escreve("Rig the owl");
    m.enter();
    let k = m.kanban(id);
    let fim: Vec<_> = m
        .cartoes(id, todo)
        .iter()
        .map(|c| k.cards[c].title.clone())
        .collect();
    assert_eq!(&fim[fim.len() - 2..], ["Rig the fox", "Rig the owl"]);
}
