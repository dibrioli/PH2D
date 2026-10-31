//! E9 §5: Pigmento → Velha → Pigmento não lê as somas de antes da volta.

use crate::grid::tinta::M_UM;
use crate::painter::Engine;
use crate::sim::LeiDaTinta;

fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64) {
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 6.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

#[test]
fn voltar_a_lei_velha_e_regressar_nao_le_somas_velhas() {
    let mut e = Engine::new(96, 64);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.set_lei_da_tinta(LeiDaTinta::Pigmento);
    e.color = [25.0, 60.0, 200.0];
    traco(&mut e, 10.0, 80.0, 30.0);
    assert!(
        !e.layers[0].grid.tinta.is_empty(),
        "a Pigmento escreveu somas"
    );
    e.set_lei_da_tinta(LeiDaTinta::Velha);
    assert!(
        e.layers[0].grid.tinta.is_empty(),
        "sair de Pigmento esvazia as somas"
    );
    // A lei velha move a massa sem as somas: um 2.º traço por cima e um passo da água.
    e.color = [240.0, 210.0, 20.0];
    traco(&mut e, 10.0, 80.0, 34.0);
    e.step_simulation();
    e.set_lei_da_tinta(LeiDaTinta::Pigmento);
    e.color = [240.0, 210.0, 20.0];
    traco(&mut e, 90.0, 92.0, 60.0); // longe: só o `garante` corre na faixa
    let g = &e.layers[0].grid;
    let mut n = 0;
    for i in 0..g.cells {
        let c = g.tinta[i];
        let (f, a) = (c.filme.v * M_UM, c.aberto.v * M_UM);
        if g.susp[i] + g.sett[i] > 0.0 && (i % g.s) < 85 {
            n += 1;
            assert!(
                (f - g.susp[i]).abs() <= 1e-3 * g.susp[i].max(1.0)
                    && (a - g.sett[i]).abs() <= 1e-3 * g.sett[i].max(1.0),
                "célula {i}: filme·M_UM {f} × susp {} · aberto·M_UM {a} × sett {}",
                g.susp[i],
                g.sett[i]
            );
        }
    }
    assert!(n > 100, "a tela tinha tinta ({n})");
}
