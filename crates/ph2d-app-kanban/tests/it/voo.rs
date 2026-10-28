//! A LISTA em voo (dono 28/10: «arrastar uma lista não pode fazê-la sumir»): pintada a cada quadro,
//! inclinada, com os cartões dela dentro; as vizinhas abrem o espaço com mola; largar assenta com
//! mola e os cartões vão junto; a borda rola o quadro.

use crate::{AREA, Mesa};
use ph2d_kanban_model::KanbanId;

/// O canto do corpo de uma lista: o cartão está `8` para dentro da borda dela.
const CORPO_X: f32 = 8.0;

/// Agarra a Backlog pela cabeça (fora do título e dos botões). Devolve o ponto.
fn pega_backlog(m: &mut Mesa, id: KanbanId) -> (f32, f32) {
    let backlog = m.lista(id, "Backlog");
    let r = m.rect_lista(id, backlog);
    let p = (r.x + 30.0, r.y + 18.0);
    assert!(m.carrega(p.0, p.1), "o carregar não foi tomado");
    p
}

#[test]
fn a_lista_arrastada_e_pintada_inclinada_a_cada_quadro_com_os_cartoes_dentro() {
    let (mut m, id) = Mesa::cena1();
    let backlog = m.lista(id, "Backlog");
    let dela = m.cartoes(id, backlog);
    let p = pega_backlog(&mut m, id);
    let mut inclinacao = 0.0;
    for i in 1..=12_u8 {
        let at = (p.0 + 25.0 * f32::from(i), p.1 + 4.0);
        m.move_(at.0, at.1);
        m.quadro();
        let (l, r, tilt, cartoes) = m
            .docs()
            .probe_painted_flight(id.0)
            .unwrap_or_else(|| panic!("quadro {i}: a lista em voo não foi pintada"));
        assert_eq!(l, backlog);
        assert!(
            (r.x + 30.0 - at.0).abs() < 0.5 && (r.y + 18.0 - at.1).abs() < 0.5,
            "quadro {i}: a lista vai na mão ({r:?} × {at:?})"
        );
        assert!(tilt > 0.0, "quadro {i}: inclinada ({tilt})");
        inclinacao = tilt;
        assert!(
            !cartoes.is_empty(),
            "quadro {i}: os cartões dela vão dentro"
        );
        assert!(
            cartoes.iter().all(|c| dela.contains(c)) && cartoes[0] == dela[0],
            "quadro {i}: os cartões pintados são os dela, pela ordem"
        );
    }
    assert!(inclinacao > 3.0, "assenta perto dos 4°: {inclinacao}");
    let fim = (p.0 + 300.0, p.1 + 4.0);
    assert!(m.larga(fim.0, fim.1));
    m.quadro();
    assert!(
        m.docs().probe_painted_flight(id.0).is_none(),
        "largada, já não voa"
    );
}

/// O `x` da To Do a cada quadro, depois de a Backlog passar por cima dela.
fn x_da_todo_ao_passar(m: &mut Mesa, id: KanbanId, quadros: usize) -> (f32, Vec<f32>) {
    let todo = m.lista(id, "To Do");
    let backlog = m.lista(id, "Backlog");
    let destino = m.rect_lista(id, backlog).x;
    let p = pega_backlog(m, id);
    m.move_(p.0 + 12.0, p.1);
    m.quadro();
    m.move_(p.0 + 300.0, p.1);
    m.quadro();
    let mut xs = Vec::new();
    for _ in 0..quadros {
        m.quadro();
        xs.push(m.rect_lista(id, todo).x);
    }
    (destino, xs)
}

#[test]
fn as_vizinhas_abrem_o_espaco_com_mola_e_sem_mola_chegam_no_quadro() {
    let (mut m, id) = Mesa::cena1();
    let todo = m.lista(id, "To Do");
    let antes = m.rect_lista(id, todo).x;
    let (destino, xs) = x_da_todo_ao_passar(&mut m, id, 40);
    let meio = xs
        .iter()
        .filter(|x| **x < antes - 1.0 && **x > destino + 1.0)
        .count();
    assert!(meio >= 3, "a To Do passa pelo meio do caminho: {xs:?}");
    assert!(
        (xs[xs.len() - 1] - destino).abs() < 0.01,
        "e assenta no lugar da Backlog ({destino}): {xs:?}"
    );

    let (mut m, id) = Mesa::cena1();
    m.hero.motion.set_reduced_motion(true);
    m.quadro();
    let (destino, xs) = x_da_todo_ao_passar(&mut m, id, 2);
    assert!(
        (xs[0] - destino).abs() < 0.01,
        "movimento reduzido: chega no quadro ({destino}): {xs:?}"
    );
}

#[test]
fn largar_a_lista_assenta_com_mola_e_os_cartoes_vao_junto() {
    let (mut m, id) = Mesa::cena1();
    let backlog = m.lista(id, "Backlog");
    let primeiro = m.cartoes(id, backlog)[0];
    let p = pega_backlog(&mut m, id);
    for i in 1..=8_u8 {
        m.move_(p.0 + 40.0 * f32::from(i), p.1 + 30.0);
        m.quadro();
    }
    let solta = m.rect_lista(id, backlog);
    assert!(m.larga(p.0 + 320.0, p.1 + 30.0));
    let mut ys = Vec::new();
    for _ in 0..40 {
        m.quadro();
        let l = m.rect_lista(id, backlog);
        let c = m.rect_cartao(id, primeiro);
        assert!(
            (c.x - l.x - CORPO_X).abs() < 0.01,
            "o cartão vai dentro da lista: {c:?} em {l:?}"
        );
        ys.push(l.y);
    }
    assert!(
        ys[0] > ys[ys.len() - 1] + 1.0 && ys[0] < solta.y,
        "parte de onde a mão a deixou e desce com mola: {} → {ys:?}",
        solta.y
    );
    m.assenta(id);
    let lugar = m.rect_lista(id, backlog);
    let c = m.rect_cartao(id, primeiro);
    assert!((c.x - lugar.x - CORPO_X).abs() < 0.01);
}

#[test]
fn segurar_uma_lista_na_borda_direita_rola_o_quadro() {
    let (mut m, id) = Mesa::cena1();
    let p = pega_backlog(&mut m, id);
    let borda = (AREA.x + AREA.w - 8.0, p.1);
    for i in 1..=6_u8 {
        let t = f32::from(i) / 6.0;
        m.move_(p.0 + (borda.0 - p.0) * t, p.1);
        m.quadro();
    }
    let antes = m.docs().probe_view(id.0).expect("vista").scroll_x;
    m.quadros(30);
    let depois = m.docs().probe_view(id.0).expect("vista").scroll_x;
    assert!(depois > antes + 50.0, "o quadro rolou: {antes} → {depois}");
    assert!(
        m.docs().probe_painted_flight(id.0).is_some(),
        "e a lista continua na mão"
    );
    assert!(m.larga(borda.0, borda.1));
    m.quadro();
}
