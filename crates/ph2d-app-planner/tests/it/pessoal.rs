//! O que é de UMA pessoa: a Inbox que vira cartão, o arrasto da coluna para um bloco de tempo,
//! «My cards», e a menção que aparece no sino.

use ph2d_app_planner::notify::What;
use ph2d_app_planner::{Hit, Mode};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_kanban_dates::{Zone, civil, day_of};
use ph2d_kanban_model::{BlockOf, InboxId};

use crate::{Mesa, centro};

fn item(m: &Mesa, texto: &str) -> InboxId {
    m.set()
        .agenda()
        .inbox
        .values()
        .find(|i| i.text == texto && !i.meta.deleted)
        .map(|i| i.id)
        .unwrap_or_else(|| panic!("item {texto}"))
}

#[test]
fn inbox_captura_e_move_para_o_kanban_num_passo_que_se_desfaz() {
    let (mut m, k) = Mesa::cena();
    assert!(m.hero.apply_event(WidgetEvent::Click(ph2d_app_planner::CMD_INBOX)));
    m.quadros(2);
    assert_eq!(m.docs().probe_mode().0, Mode::Planner);
    m.escreve("Buy paper");
    m.enter();
    let id = item(&m, "Buy paper");
    let me = m.set().agenda().inbox[&id].owner.clone();
    m.clica_em(Hit::InboxToCard(id));
    m.enter(); // «Move» no destino (a 1.ª lista, no fundo)
    let c = m.cartao(k, "Buy paper");
    assert!(c.members.contains(&me), "o cartão é meu, como no Trello");
    assert!(m.set().agenda().inbox[&id].meta.deleted, "o item saiu da Inbox");
    assert_eq!(m.desfaz(), Some(true));
    assert!(!m.set().agenda().inbox[&id].meta.deleted, "o desfazer devolve o item…");
    assert!(m.cartao(k, "Buy paper").meta.deleted, "…e tira o cartão: UM passo");
}

#[test]
fn arrastar_da_coluna_para_um_dia_cria_um_bloco_as_nove() {
    let (mut m, _) = Mesa::cena();
    m.clica_em(Hit::Mode(Mode::Planner));
    let id = item(&m, "Buy a new pen nib");
    let de = centro(m.docs().probe_hit(Hit::InboxItem(id)).expect("na coluna"));
    let z = Zone::utc();
    let hoje = day_of(m.agora().0, &z);
    let alvo = hoje.add_days(2);
    let para = {
        let cal = m.docs().probe_calendar();
        centro(cal.cell_rect(alvo).expect("o dia à vista"))
    };
    m.arrasta(de, para);
    let set = m.set();
    let b = set
        .agenda()
        .blocks
        .values()
        .find(|b| b.of == BlockOf::Inbox(id))
        .expect("um bloco novo");
    let ct = civil(b.start.0, &z);
    assert_eq!((ct.date, ct.hour), (alvo, 9));
}

#[test]
fn my_cards_mostra_os_meus_cartoes_de_todos_os_kanbans() {
    let (mut m, k) = Mesa::cena();
    m.clica_em(Hit::Mode(Mode::MyCards));
    let fox = m.cartao(k, "Design the fox");
    let r = ph2d_kanban_model::CardRef {
        kanban: k,
        card: fox.id,
    };
    let row = m.docs().probe_hit(Hit::Card(r)).expect("o cartão está na lista");
    m.clica(centro(row));
    // O verso abre-o a família Kanban, pelo pedido da porta baixa (as famílias não se conhecem).
    assert_eq!(ph2d_kanban_hooks::take_open_cards(), vec![r], "abrir um cartão pede o verso dele");
}

#[test]
fn a_mencao_aparece_no_sino_por_ler_e_ler_a_abre() {
    let (mut m, _) = Mesa::cena();
    let n = m
        .docs()
        .probe_notices()
        .iter()
        .find(|n| matches!(n.what, What::Mention { .. }))
        .cloned()
        .expect("a menção da Ana");
    assert!(n.unread);
    assert!(m.docs().unread() > 0);
    m.clica_em(Hit::Bell);
    m.clica_em(Hit::Notice(n.key));
    let lida = m
        .docs()
        .probe_notices()
        .iter()
        .find(|x| x.key == n.key)
        .map(|x| x.unread);
    assert_eq!(lida, Some(false), "lida");
    let card = n.card.map(|card| ph2d_kanban_model::CardRef { kanban: n.kanban, card });
    assert_eq!(ph2d_kanban_hooks::take_open_cards(), card.into_iter().collect::<Vec<_>>(), "e o verso do cartão foi pedido");
}
