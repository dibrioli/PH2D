//! B02.5: para lá do tecto, desfazer com a água a correr longe não a enterra.

use super::super::undo_tinta_morre_tests::{AGUADA, tela, teto, tinta_viva_na_linha, traco};

/// A variante sem o casco (o enterro de todo o píxel que difere), para o vermelho.
pub(in crate::tool::paint) mod sem_casco {
    use std::cell::Cell;
    thread_local! { static V: Cell<bool> = const { Cell::new(false) }; }
    pub(in crate::tool::paint) fn get() -> bool {
        V.with(Cell::get)
    }
    pub(in crate::tool::paint) fn set(v: bool) {
        V.with(|c| c.set(v));
    }
}

/// O que o desfazer para lá do tecto fez à faixa de cima (longe do traço desfeito), com a aguada dela a correr:
/// `(células da grade que mudaram, das quais a zero, tinta viva na linha desfeita, na de cima)`. A tela é a do documento
/// (`para_la_do_tecto_so_o_traco_desfeito_acaba`).
fn desfaz_com_a_agua_a_correr(sem: bool) -> (usize, usize, f32, f32) {
    sem_casco::set(sem);
    teto::set(Some(1));
    let mut t = tela(900, 700, AGUADA, 40.0);
    traco(&mut t, 60.0, 840.0, 80.0);
    t.ticks_em_blocos(4);
    traco(&mut t, 60.0, 840.0, 500.0);
    traco(&mut t, 60.0, 840.0, 620.0);
    assert!(t.undo_last(), "o passo do topo, com fotografia");
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
    s.engine.drain_step();
    let g = &s.engine.layers[0].grid;
    let corte = 250 * g.s;
    let antes = (g.sett[..corte].to_vec(), g.susp[..corte].to_vec());
    assert!(t.undo_last(), "para lá do tecto");
    teto::set(None);
    sem_casco::set(false);
    let desfeita = tinta_viva_na_linha(&mut t, 500);
    let cima = tinta_viva_na_linha(&mut t, 80);
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let g = &s.engine.layers[0].grid;
    let pares = || {
        g.sett[..corte]
            .iter()
            .zip(&antes.0)
            .chain(g.susp[..corte].iter().zip(&antes.1))
    };
    let difere = pares().filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    let zeradas = pares()
        .filter(|(a, b)| a.to_bits() != b.to_bits() && **a == 0.0)
        .count();
    (difere, zeradas, desfeita, cima)
}

/// ⭐ Longe do casco do traço desfeito a água que corre fica ao bit; o traço desfeito acaba.
#[test]
fn para_la_do_tecto_a_agua_que_corre_longe_fica_ao_bit() {
    let (difere, zeradas, _, _) = desfaz_com_a_agua_a_correr(true);
    assert!(
        zeradas > 0,
        "CONTROLO: sem o casco a água longe enterra-se ({difere} células, {zeradas} a zero)"
    );
    let (difere, zeradas, desfeita, cima) = desfaz_com_a_agua_a_correr(false);
    assert_eq!((difere, zeradas), (0, 0), "células longe mudadas, a zero");
    assert_eq!(desfeita, 0.0, "o traço desfeito acaba");
    assert!(cima > 0.0, "a aguada de cima vive");
}
