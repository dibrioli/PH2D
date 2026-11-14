//! B13.4: o mapa «corre?» por tile da secagem (`Sim::corre_tile`) é relógio do motor — o undo vivo devolve-o ao bit.

use super::super::undo_tinta_morre_tests::{AGUADA, tela, traco};
use crate::tool::PainterTool;

fn relogio(t: &mut PainterTool) -> (Vec<bool>, u64, u64, u64) {
    t.wet_bring_home();
    let s = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão viva")
        .engine
        .sim;
    (
        s.corre_tile.clone(),
        s.dry_every,
        s.evap_scale.to_bits(),
        s.rewet_base.to_bits(),
    )
}

#[test]
fn o_undo_depois_de_agua_a_correr_devolve_o_mapa_por_tile_ao_bit() {
    let mut t = tela(384, 192, AGUADA, 20.0);
    traco(&mut t, 20.0, 60.0, 40.0);
    t.wet_bring_home();
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessão viva")
        .engine
        .sim
        .gravity_override = Some([0.0, 1.0]);
    let antes = relogio(&mut t);
    traco(&mut t, 170.0, 214.0, 60.0);
    let depois = relogio(&mut t);
    assert!(
        depois.0.iter().any(|&c| c) && depois.0 != antes.0,
        "a cena não pôs água a correr: {:?} → {:?}",
        antes.0.len(),
        depois.0.len()
    );
    assert!(t.undo_last(), "o undo");
    assert_eq!(
        relogio(&mut t),
        antes,
        "o undo não devolveu o relógio por tile"
    );
}
