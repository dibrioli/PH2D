//! O arrasto pelo rato real do hero: o cartão entre listas (o alvo, a inclinação, UM desfazer), a
//! lista, a rolagem na borda e o movimento reduzido.

use crate::{AREA, Mesa, pega};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, KanbanId, ListId};

/// O topo do corpo de uma lista (a cabeça: 10 + 20 + 6).
const CABECA: f32 = 36.0;

/// Onde pôr o ponteiro para que o CENTRO do cartão agarrado em `p` caia em `alvo`.
fn ponteiro_para(r: Rect, p: (f32, f32), alvo: (f32, f32)) -> (f32, f32) {
    (
        alvo.0 + (p.0 - r.x) - r.w / 2.0,
        alvo.1 + (p.1 - r.y) - r.h / 2.0,
    )
}

/// Arrasta o 1.º cartão do Backlog até ao topo da Doing, sem largar. Devolve `(cartão, doing, ponteiro)`.
fn leva_ao_topo_da_doing(m: &mut Mesa, id: KanbanId) -> (CardId, ListId, (f32, f32)) {
    let backlog = m.lista(id, "Backlog");
    let doing = m.lista(id, "Doing");
    let c = m.cartoes(id, backlog)[0];
    let r = m.rect_cartao(id, c);
    let d = m.rect_lista(id, doing);
    let p = pega(r);
    let alvo = (d.x + d.w / 2.0, d.y + CABECA + 6.0);
    let fim = ponteiro_para(r, p, alvo);
    m.arrasta(p, fim, 8);
    (c, doing, fim)
}

#[test]
fn arrastar_um_cartao_do_backlog_para_o_topo_da_doing_e_um_ctrl_z_devolve() {
    let (mut m, id) = Mesa::cena1();
    let backlog = m.lista(id, "Backlog");
    let ordem = m.cartoes(id, backlog);
    let (c, doing, fim) = leva_ao_topo_da_doing(&mut m, id);
    assert_eq!(
        m.docs().probe_drop(id.0),
        Some((doing, 0)),
        "o alvo do arrasto é o topo da Doing"
    );
    m.quadros(4);
    let tilt = m.docs().probe_tilt(id.0);
    assert!(tilt > 0.0, "quem voa inclina: {tilt}");
    assert!(m.larga(fim.0, fim.1));
    m.quadro();
    let k = m.kanban(id);
    assert_eq!(k.cards[&c].list, doing);
    assert_eq!(m.cartoes(id, doing)[0], c, "no índice 0");
    assert_eq!(m.docs().probe_drop(id.0), None);
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(m.kanban(id).cards[&c].list, backlog);
    assert_eq!(
        m.cartoes(id, backlog),
        ordem,
        "de volta ao seu lugar, num só Ctrl+Z"
    );
}

#[test]
fn arrastar_uma_lista_para_a_direita_troca_a_ordem_e_o_desfazer_devolve() {
    let (mut m, id) = Mesa::cena1();
    let ordem = |m: &Mesa| {
        m.kanban(id)
            .lists_in_order()
            .iter()
            .map(|l| l.title.clone())
            .collect::<Vec<_>>()
    };
    let antes = ordem(&m);
    let backlog = m.lista(id, "Backlog");
    let r = m.rect_lista(id, backlog);
    // A cabeça, à esquerda do título (fora dos botões).
    let p = (r.x + 30.0, r.y + 18.0);
    let fim = (p.0 + 200.0, p.1);
    m.arrasta(p, fim, 8);
    assert!(m.larga(fim.0, fim.1));
    m.quadro();
    let depois = ordem(&m);
    assert_eq!(&depois[..3], ["To Do", "Backlog", "Doing"], "{depois:?}");
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(ordem(&m), antes);
}

#[test]
fn segurar_um_cartao_na_borda_direita_rola_o_quadro() {
    let (mut m, id) = Mesa::cena1();
    let backlog = m.lista(id, "Backlog");
    let c = m.cartoes(id, backlog)[0];
    let r = m.rect_cartao(id, c);
    let p = pega(r);
    let borda = (AREA.x + AREA.w - 8.0, 300.0);
    m.arrasta(p, borda, 6);
    let antes = m.docs().probe_view(id.0).expect("vista").scroll_x;
    m.quadros(30);
    let depois = m.docs().probe_view(id.0).expect("vista").scroll_x;
    assert!(depois > antes + 50.0, "o quadro rolou: {antes} → {depois}");
    assert!(m.larga(borda.0, borda.1));
    m.quadro();
}

#[test]
fn com_movimento_reduzido_o_largar_chega_no_mesmo_quadro() {
    // Controlo: com molas, o cartão largado ainda voa no quadro seguinte.
    let (mut m, id) = Mesa::cena1();
    let (_, _, fim) = leva_ao_topo_da_doing(&mut m, id);
    assert!(m.larga(fim.0, fim.1));
    m.quadro();
    m.quadro();
    assert!(
        m.docs().probe_moving(id.0),
        "controlo: com molas, ainda se mexe"
    );

    let (mut m, id) = Mesa::cena1();
    m.hero.motion.set_reduced_motion(true);
    m.quadro();
    let (c, doing, fim) = leva_ao_topo_da_doing(&mut m, id);
    assert_eq!(m.docs().probe_drop(id.0), Some((doing, 0)));
    assert!(m.larga(fim.0, fim.1));
    m.quadro();
    assert_eq!(m.kanban(id).cards[&c].list, doing);
    assert!(
        !m.docs().probe_moving(id.0),
        "movimento reduzido: nada voa depois de largar"
    );
}
