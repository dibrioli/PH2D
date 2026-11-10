//! B12.1: o grão de pó solto marca só o alcance do dab dele no registo aberto, e o undo devolve a grade ao bit.

use super::*;
use crate::painter::historia_tests::{corre, hash_grid, traco};

/// Bytes que o registo aberto ganha com `porta`.
fn ganho(e: &mut Engine, porta: impl FnOnce(&mut Engine)) -> usize {
    let antes = e.bytes_do_historico(0);
    porta(e);
    e.bytes_do_historico(0) - antes
}

#[test]
fn historia_o_grao_de_po_marca_o_dab_e_nao_a_tela() {
    let (w, h) = (960, 640);
    let mut e = Engine::new(w, h);
    e.sliders.water = 1.0;
    // Uma camada que já correu um passo (o anel da borda selado), como em toda régua `historia_`.
    traco(&mut e, 60.0, 60.0, 4);
    corre(&mut e, 4);
    e.drain_step();
    e.capture_history();
    let antes = hash_grid(e.active_grid());
    let sil = |_: i32, _: i32| 1.0;
    let mut t = crate::trail::Trail::default();
    let grao = ganho(&mut e, |e| {
        e.pousa_grao_solto(
            &mut t,
            [220.0, 45.0, 40.0],
            480.0,
            320.0,
            5.0,
            3.0,
            1.0,
            &sil,
        );
        e.pousa_grao_solto(
            &mut t,
            [40.0, 45.0, 220.0],
            488.0,
            324.0,
            5.0,
            4.0,
            0.7,
            &sil,
        );
    });
    corre(&mut e, 4);
    e.drain_step();
    let fim = hash_grid(e.active_grid());
    assert_ne!(fim, antes, "o grão não pousou");
    let tela = {
        let mut c = Engine::new(w, h);
        c.capture_history();
        ganho(&mut c, Engine::marca_tudo)
    };
    eprintln!("B12.1 pó: registo {grao} B · tela {tela} B");
    assert!(e.undo());
    e.drain_step();
    assert_eq!(hash_grid(e.active_grid()), antes, "undo do grão");
    assert!(e.redo());
    e.drain_step();
    assert_eq!(hash_grid(e.active_grid()), fim, "redo do grão");
    // O alcance de um dab é `r + FOLGA_DAB` (130): ≈ 5×5 tiles dos 15×10 desta tela.
    assert!(
        grao * 4 < tela,
        "o grão guardou {grao} B de {tela} B: a tela inteira"
    );
}
