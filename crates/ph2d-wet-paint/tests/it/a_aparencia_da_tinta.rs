//! **O «Glaze layering» é da tinta** (W3b item 3, `grid::aparencia`): toda porta que muda pigmento de camada lê a lei
//! da CÉLULA (`Ecra::com_vidrado`). A régua: um motor que pinta com o flag ligado (vidrado `1` pousado, `esconde` de
//! fábrica) e outro cuja sessão já tapa por velatura (`esconde` = `TINTA_ESCONDE_EM_VELATURA`, sem flag) têm as MESMAS
//! grades ao bit, em toda célula pousada — pelo Paint com Glaze (o assentar do Glaze no pouso), o Blend (o levantar do
//! traço) e as ferramentas Wet e Smear (`tools.rs`). Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md`.

use ph2d_wet_paint::assenta::Lei;
use ph2d_wet_paint::colorops::{
    TINTA_ESCONDE, TINTA_ESCONDE_EM_VELATURA, VELATURA_DO_FILME, VELATURA_MEDIDA,
};
use ph2d_wet_paint::grid::aparencia::vidrado;
use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;

use crate::util::drive_stroke;

const W: usize = 48;
const H: usize = 40;

fn motor(celula: bool) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.glaze = VELATURA_MEDIDA;
    e.sim.glaze_filme = VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = Lei::Cobre;
    if celula {
        e.sim.esconde = TINTA_ESCONDE;
        e.sim.camadas_transparentes = true;
    } else {
        e.sim.esconde = TINTA_ESCONDE_EM_VELATURA;
    }
    // O Glaze do pincel (o assentar no pouso) e o levantar molhado (Blend, Wet, Smear).
    e.tuning.set(Knob::Glaze, 0.6);
    e.tuning.set(Knob::WetLift, 0.6);
    e
}

/// Vermelho seco, azul molhado por cima, amarelo molhado sobre o azul molhado, e a ferramenta `ultima` por cima.
fn cena(celula: bool, ultima: Tool) -> Engine {
    let mut e = motor(celula);
    e.tool = Tool::Paint;
    e.color = [200.0, 40.0, 40.0];
    drive_stroke(&mut e, 6.0, 20.0, 42.0, 20.0, 3.0, 0);
    e.dry_canvas_now();
    e.color = [20.0, 60.0, 190.0];
    drive_stroke(&mut e, 6.0, 22.0, 42.0, 22.0, 3.0, 0);
    e.color = [210.0, 190.0, 30.0];
    drive_stroke(&mut e, 6.0, 21.0, 42.0, 21.0, 3.0, 0);
    e.tool = ultima;
    drive_stroke(&mut e, 8.0, 21.0, 40.0, 21.0, 2.0, 0);
    e
}

#[test]
fn o_vidrado_da_celula_e_a_lei_de_velatura_em_cada_porta() {
    for ultima in [Tool::Paint, Tool::Blend, Tool::Wet, Tool::Smear] {
        let (a, b) = (cena(true, ultima), cena(false, ultima));
        let (ga, gb) = (a.active_grid(), b.active_grid());
        let mut pousadas = 0;
        for i in 0..ga.cells {
            if vidrado(&ga.aparencia, i) != 1.0 {
                continue;
            }
            pousadas += 1;
            assert!(
                ga.sett[i] == gb.sett[i]
                    && ga.susp[i] == gb.susp[i]
                    && ga.sett_rgb[i] == gb.sett_rgb[i]
                    && ga.susp_rgb[i] == gb.susp_rgb[i],
                "{ultima:?}: a célula {i} divergiu da lei de velatura ({}/{} · {}/{})",
                ga.sett[i],
                gb.sett[i],
                ga.susp[i],
                gb.susp[i]
            );
        }
        assert!(
            pousadas > 300,
            "{ultima:?}: controlo — só {pousadas} células pousadas"
        );
    }
}
