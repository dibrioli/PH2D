//! **O levantar do molhado × molhado pousa o MESMO grid nas duas rotas** (`Knob::WetOpen`, `trail/molhado.rs`): a outra
//! tinta debaixo do tubo é levantada linha a linha do grid, em série ou em paralelo; cada célula só lê e escreve a sua,
//! e o retângulo tocado é um mínimo/máximo. O MESMO traço azul sobre o MESMO vermelho molhado, pelas duas rotas forçadas
//! (`transfer_paint_rows`), comparado **byte a byte** — o grid, os planos da janela e os retângulos declarados.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::{MIN_CELLS_DEPOSIT, Rows};
use ph2d_wet_paint::trail::{Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 600;
const H: usize = 400;
const R: f64 = 60.0;
const BASE: [f64; 3] = [23.0, 101.0, 179.0];

/// O grid inteiro molhado de vermelho (a película acima do «molhado», tinta suspensa), com massas que variam por célula.
fn vermelho_molhado() -> Engine {
    let mut e = Engine::new(W, H);
    let g = e.active_grid_mut();
    for i in 0..g.susp.len() {
        g.film[i] = 0.5;
        g.susp[i] = 20.0 + (i % 97) as f32;
        g.susp_rgb[i] = [191.0, (i % 7) as f32, 18.0];
    }
    e
}

struct Saida {
    grid: Vec<u32>,
    pousos: Vec<Option<(i32, i32, i32, i32)>>,
    levantadas: usize,
}

fn pousa(mode: Rows, abre: f64) -> Saida {
    let mut e = vermelho_molhado();
    e.tuning.set(Knob::WetOpen, abre);
    e.tuning.set(Knob::WetOpenPush, 0.8);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let antes = e.active_grid_mut().susp.clone();
    let mut t = Trail::default();
    t.start_stroke(150.0, 200.0, BASE, TrailMode::Paint);
    t.on_segment(12.0, 12.0);
    let g = e.active_grid_mut();
    let mut pousos = Vec::new();
    for k in 0..24 {
        let d = ph2d_wet_paint::trail::Dab {
            x: 150.0 + 12.0 * f64::from(k),
            y: 200.0 + 2.0 * f64::from(k),
            r: R,
            hardness: 0.5,
            intensity: 4.0,
            water_amount: 0.5,
            dry_gate: 0.2,
            reach: 1.0,
            window_reach: 1.0,
            opacity: 1.0,
        };
        if t.accumulate_paint(g, &p, &tex, &d, false).window_full {
            pousos.push(
                t.transfer_paint_rows(g, &p, mode)
                    .map(|r| (r.x0, r.y0, r.x1, r.y1)),
            );
        }
    }
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
    let mut grid = bits(&g.susp);
    grid.extend(bits(&g.sett));
    grid.extend(bits(&g.film));
    grid.extend(g.susp_rgb.iter().flatten().map(|x| x.to_bits()));
    let levantadas = antes.iter().zip(&g.susp).filter(|(a, b)| b < a).count();
    Saida {
        grid,
        pousos,
        levantadas,
    }
}

#[test]
fn o_levantar_do_molhado_e_o_mesmo_nas_duas_rotas() {
    let (serie, paralela, sem) = (
        pousa(Rows::Serial, 0.6),
        pousa(Rows::Parallel, 0.6),
        pousa(Rows::Serial, 0.0),
    );
    // Premissas: o tubo abriu e levantou de verdade (mais células que sem a lei), numa caixa acima do piso.
    assert!(
        serie.levantadas > sem.levantadas + MIN_CELLS_DEPOSIT,
        "a fixtura não levanta: {} células com a lei, {} sem",
        serie.levantadas,
        sem.levantadas
    );
    assert!(
        serie.pousos.len() >= 2,
        "o traço transferiu {} janelas",
        serie.pousos.len()
    );
    assert_eq!(
        serie.pousos, paralela.pousos,
        "os retângulos declarados diferem entre as rotas"
    );
    let diferentes = serie
        .grid
        .iter()
        .zip(&paralela.grid)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        diferentes, 0,
        "o grid difere entre as rotas em {diferentes} palavras"
    );
}
