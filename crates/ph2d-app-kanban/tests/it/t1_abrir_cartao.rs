//! T1 — o pedido de outra família (o Planner, o sino) para abrir o verso de um cartão: o lado do
//! Kanban esvazia-o no `frame`, abre a aba e o verso; um cartão apagado não abre nada e não cai.

use crate::Mesa;
use crate::verso::cartao;
use ph2d_kanban_hooks::request_open_card;
use ph2d_kanban_model::{CardRef, KanbanId, KanbanOp, RecordId};

/// Um segundo Kanban (cópia da cena 1) que nenhuma aba mostra.
fn fora_da_aba(m: &Mesa, de: KanbanId) -> KanbanId {
    let outro = KanbanId(900);
    let mut k = m.kanban(de);
    k.id = outro;
    m.hero.documents.kanbans_shared().borrow_mut().insert(k);
    outro
}

#[test]
fn pedido_para_kanban_fora_da_aba_abre_a_aba_e_o_verso() {
    let (mut m, id) = Mesa::cena1();
    let outro = fora_da_aba(&m, id);
    let c = cartao(&m, outro, "Write the logline");
    assert!(m.docs().probe_back(outro.0).is_none(), "ainda sem aba");
    request_open_card(CardRef {
        kanban: outro,
        card: c,
    });
    m.quadro();
    m.quadro();
    assert_eq!(
        m.docs().probe_back(outro.0),
        Some(c),
        "o verso do cartão abriu"
    );
    assert_eq!(
        m.activa(),
        Some(outro.0),
        "a aba do Kanban pedido ganhou o foco"
    );
    assert!(
        ph2d_kanban_hooks::take_open_cards().is_empty(),
        "o pedido foi esvaziado"
    );
}

#[test]
fn pedido_de_cartao_apagado_nao_abre_nada_e_nao_cai() {
    let (mut m, id) = Mesa::cena1();
    let outro = fora_da_aba(&m, id);
    let c = cartao(&m, outro, "Write the logline");
    m.hero
        .documents
        .kanbans_shared()
        .borrow_mut()
        .apply(outro, vec![KanbanOp::Delete(RecordId::Card(c))]);
    request_open_card(CardRef {
        kanban: outro,
        card: c,
    });
    m.quadros(2);
    assert_eq!(
        m.docs().probe_back(outro.0),
        None,
        "cartão apagado: sem verso"
    );
}
