//! «To Kanban» escolhe o destino: o popover com a pesquisa, os Kanbans e as listas, topo ou fundo.

use ph2d_app_planner::{DestHit, Hit, Mode};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_kanban_model::{InboxId, KanbanId, ListRef};

use crate::{Mesa, centro};

fn item(m: &Mesa, texto: &str) -> InboxId {
    m.set().agenda().inbox.values().find(|i| i.text == texto && !i.meta.deleted).map(|i| i.id).expect("item")
}

fn lista(m: &Mesa, kanban: &str, titulo: &str) -> ListRef {
    let set = m.set();
    let k = set.iter().find(|k| k.header.name == kanban).expect("Kanban");
    let l = k.lists_in_order().into_iter().find(|l| l.title == titulo).expect("lista");
    ListRef { kanban: k.id, list: l.id }
}

fn clica_no_destino(m: &mut Mesa, h: DestHit) {
    let r = {
        let set = m.set();
        let d = m.docs().probe_dest().expect("o destino está aberto");
        d.hits(&set).into_iter().find(|(x, _)| *x == h).map(|(_, r)| r).unwrap_or_else(|| panic!("{h:?} não está pintado"))
    };
    m.clica(centro(r));
}

#[test]
fn to_kanban_pesquisa_escolhe_a_lista_e_o_topo_e_move_num_passo() {
    let (mut m, k) = Mesa::cena();
    m.clica_em(Hit::Mode(Mode::Planner));
    let id = item(&m, "Call the sound studio");
    m.clica_em(Hit::InboxToCard(id));
    assert!(m.docs().probe_dest().is_some(), "o clique abre o destino, não move às cegas");
    assert!(!m.set().agenda().inbox[&id].meta.deleted, "nada mudou ainda");
    // A pesquisa deixa só as listas que casam (e o Kanban delas por cima).
    m.escreve("record");
    let alvo = lista(&m, "Sound & music", "Recording");
    let rows = ph2d_app_planner::rows(&m.set(), &m.docs().probe_dest().unwrap().search.text);
    assert_eq!(rows.len(), 2, "o Kanban e a lista: {rows:?}");
    assert_eq!(m.docs().probe_dest().unwrap().pick, Some(alvo), "a escolha segue a pesquisa");
    clica_no_destino(&mut m, DestHit::Top(true));
    clica_no_destino(&mut m, DestHit::Move);
    assert!(m.docs().probe_dest().is_none(), "Move fecha");
    let set = m.set();
    let kb = set.get(alvo.kanban).unwrap();
    let topo = kb.cards_in(alvo.list)[0];
    assert_eq!(topo.title, "Call the sound studio");
    assert!(set.agenda().inbox[&id].meta.deleted);
    assert_ne!(alvo.kanban, k, "foi para o OUTRO Kanban");
    assert_eq!(m.desfaz(), Some(true));
    assert!(!m.set().agenda().inbox[&id].meta.deleted, "um passo de desfazer devolve o item");
}

#[test]
fn to_kanban_por_omissao_vai_para_o_fundo_e_esc_fecha_sem_mudar() {
    let (mut m, k) = Mesa::cena();
    m.clica_em(Hit::Mode(Mode::Planner));
    let id = item(&m, "Buy a new pen nib");
    m.clica_em(Hit::InboxToCard(id));
    m.tecla(BoardKey::Escape, None);
    m.quadro();
    assert!(m.docs().probe_dest().is_none());
    assert!(!m.set().agenda().inbox[&id].meta.deleted, "Esc não move");
    m.clica_em(Hit::InboxToCard(id));
    let pick = m.docs().probe_dest().unwrap().pick.expect("uma lista escolhida");
    assert_eq!(pick.kanban, k, "a primeira lista do primeiro Kanban");
    m.enter();
    let set = m.set();
    let cards = set.get(k).unwrap().cards_in(pick.list);
    assert_eq!(cards.last().map(|c| c.title.as_str()), Some("Buy a new pen nib"), "no fundo");
    let _ = KanbanId::AGENDA;
}
