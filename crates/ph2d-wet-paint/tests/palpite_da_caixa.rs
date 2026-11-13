//! **A caixa «Faster wet paint» chega ao motor** (n.º 8, cartão Performance do dono 13/11). Binário de teste À PARTE:
//! a chave é global ao processo, e nenhum dos gates ao bit do `it`/lib pode correr com ela ligada a meio.
//!
//! De fábrica DESLIGADA; ligada, a simulação pinta outra coisa (o palpite da `assenta`); desligada de novo, volta AO
//! BIT à de fábrica — o memo da `assenta` (que guardou contas feitas com o palpite) é esquecido na troca.

use ph2d_wet_paint::assenta::{Lei, liga_o_palpite, palpite_ligado};
use ph2d_wet_paint::painter::Engine;
use std::hash::{Hash, Hasher};

fn corre() -> u64 {
    let mut e = Engine::new(160, 120);
    e.sim.lei = Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    for (i, c) in [
        [40.0, 70.0, 200.0],
        [200.0, 40.0, 40.0],
        [230.0, 200.0, 30.0],
    ]
    .into_iter()
    .enumerate()
    {
        e.color = c;
        let y = 30.0 + 25.0 * i as f64;
        e.pointer_down(20.0, y, None);
        for k in 1..=16 {
            e.dispatch_pressure_dab(20.0 + 7.0 * f64::from(k), y + 6.0, 6.0, 60.0);
        }
        e.pointer_up();
        while e.release_frame(130.0, y) {}
    }
    for _ in 0..30 {
        e.step_simulation();
    }
    let g = e.active_grid();
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for v in [&g.film, &g.susp, &g.sett] {
        v.iter().for_each(|x| x.to_bits().hash(&mut h));
    }
    for v in [&g.susp_rgb, &g.sett_rgb] {
        v.iter().flatten().for_each(|x| x.to_bits().hash(&mut h));
    }
    h.finish()
}

#[test]
fn a_caixa_faster_wet_paint_liga_o_palpite_e_desligada_volta_ao_bit() {
    assert!(!palpite_ligado(), "de fábrica a caixa nasce desligada");
    let fabrica = corre();
    liga_o_palpite(true);
    assert!(palpite_ligado());
    let palpite = corre();
    liga_o_palpite(false);
    assert!(!palpite_ligado());
    let de_volta = corre();
    assert_ne!(fabrica, palpite, "ligada, a caixa não chegou ao motor");
    assert_eq!(
        fabrica, de_volta,
        "desligada, o motor não voltou ao bit de fábrica"
    );
}
