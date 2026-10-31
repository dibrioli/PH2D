//! F4-2 E6b/E8: o INVARIANTE `filme.v·M_UM == susp`, `aberto.v·M_UM == sett` no caminho de produto (`Engine`):
//! traços de duas cores, Smear, Blend, Wet, borracha, passos da água e secar.
//! Doc: `docs/Painter/Arquitetura/E6b_pickup_e_ferramentas.md`.

use crate::grid::Grid;
use crate::grid::tinta::M_UM;
use crate::painter::{Engine, Tool};
use crate::sim::LeiDaTinta;

/// A pior violação por célula, `|v·M_UM − massa| / max(1, massa)`, nas duas fatias.
fn pior(g: &Grid) -> f64 {
    let rel = |v: f32, m: f32| {
        (f64::from(v) * f64::from(M_UM) - f64::from(m)).abs() / f64::from(m).max(1.0)
    };
    (0..g.cells)
        .map(|i| rel(g.tinta[i].filme.v, g.susp[i]).max(rel(g.tinta[i].aberto.v, g.sett[i])))
        .fold(0.0, f64::max)
}

fn traco(e: &mut Engine, tool: Tool, (x0, x1, y): (f64, f64, f64)) {
    e.tool = tool;
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 4.0f64).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 40.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn com_ferramenta(e: &mut Engine, tool: Tool, x: f64, (y0, y1): (f64, f64)) {
    let mut prev = None;
    let mut y = y0;
    while y <= y1 {
        e.dispatch_pressure_dab_tool(tool, x, y, 6.0, 12.0, prev);
        prev = Some([x, y]);
        y += 2.0;
    }
}

#[test]
fn o_invariante_soma_massa_vale_no_motor() {
    let mut e = Engine::new(96, 96);
    e.sim.lei = crate::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.sim.set_tinta(LeiDaTinta::Pigmento);
    let mut fases: Vec<(&str, f64)> = Vec::new();
    let mede = |e: &Engine, nome: &'static str, fases: &mut Vec<(&str, f64)>| {
        fases.push((nome, pior(&e.layers[0].grid)));
    };
    e.color = [220.0, 45.0, 40.0];
    traco(&mut e, Tool::Paint, (10.0, 80.0, 30.0));
    e.color = [30.0, 60.0, 210.0];
    traco(&mut e, Tool::Paint, (10.0, 80.0, 40.0));
    mede(&e, "duas cores", &mut fases);
    for _ in 0..20 {
        e.step_simulation();
    }
    mede(&e, "passos", &mut fases);
    com_ferramenta(&mut e, Tool::Smear, 48.0, (22.0, 50.0));
    mede(&e, "smear", &mut fases);
    e.color = [240.0, 210.0, 20.0];
    traco(&mut e, Tool::Blend, (20.0, 70.0, 35.0));
    mede(&e, "blend", &mut fases);
    com_ferramenta(&mut e, Tool::Wet, 30.0, (24.0, 46.0));
    mede(&e, "wet", &mut fases);
    for _ in 0..20 {
        e.step_simulation();
    }
    mede(&e, "passos 2", &mut fases);
    e.fast_dry_now();
    mede(&e, "secar", &mut fases);
    e.color = [20.0, 160.0, 60.0];
    traco(&mut e, Tool::Paint, (15.0, 75.0, 36.0));
    com_ferramenta(&mut e, Tool::Smear, 60.0, (26.0, 46.0));
    for _ in 0..20 {
        e.step_simulation();
    }
    mede(&e, "re-molhar + smear + passos", &mut fases);
    let g = &e.layers[0].grid;
    assert!(g.tinta.len() == g.cells, "a lei nova correu");
    assert!(g.susp.iter().chain(&g.sett).any(|&m| m > 0.0), "há tinta");
    for (nome, p) in &fases {
        println!("[inv] {nome}: pior {p:.2e}");
    }
    for (nome, p) in fases {
        assert!(p <= 1e-3, "{nome}: soma ≠ massa ({p:e})");
    }
}
