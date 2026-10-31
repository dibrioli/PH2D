//! O ARRASTO do Water Brush (missão «superar o Rebelle», 31/10).

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 260;
const H: usize = 140;
const Y: usize = 70;

/// Uma mancha de tinta (x 50..80, y 58..82), MOLHADA ou assentada.
fn mancha(seca: bool) -> Engine {
    let mut e = Engine::new(W, H);
    let g = e.active_grid_mut();
    for y in Y - 12..=Y + 12 {
        for x in 50..=80 {
            let i = x + 1 + (y + 1) * g.s;
            if seca {
                g.sett[i] = 120.0;
                g.sett_rgb[i] = [191.0, 0.0, 18.0];
            } else {
                g.susp[i] = 120.0;
                g.susp_rgb[i] = [191.0, 0.0, 18.0];
                g.film[i] = 0.3;
            }
        }
    }
    e
}

fn water_brush(e: &mut Engine, extra: &[(Knob, f64)]) {
    // A ponta do Water Brush (a do Blender Soft): feltro 0,2, 4000 cerdas de 2.
    e.set_knob(Knob::Felt, 0.2);
    e.set_knob(Knob::BristleCount, 4000.0);
    e.set_knob(Knob::BristleSize, 2.0);
    e.tuning.set(Knob::WaterPerDab, 1.5);
    e.tuning.set(Knob::WetLift, 0.7);
    e.tuning.set(Knob::WaterPush, 0.35);
    for (k, v) in extra {
        e.tuning.set(*k, *v);
    }
    e.sliders.water = 1.0;
    let mut prev: Option<[f64; 2]> = None;
    let mut x = 30.0;
    while x <= 230.0 {
        e.dispatch_pressure_dab_tool(Tool::Wet, x, Y as f64, 5.0, 10.0, prev);
        prev = Some([x, Y as f64]);
        x += 2.0;
    }
}

/// Pigmento total (susp + sett) por coluna, na faixa y 58..82.
fn colunas(e: &Engine) -> Vec<f64> {
    let g = e.active_grid();
    (0..W)
        .map(|x| {
            (Y - 12..=Y + 12)
                .map(|y| {
                    let i = x + 1 + (y + 1) * g.s;
                    f64::from(g.susp[i]) + f64::from(g.sett[i])
                })
                .sum()
        })
        .collect()
}

fn soma(v: &[f64], a: usize, b: usize) -> f64 {
    v[a..=b].iter().sum()
}

struct Medida {
    origem: f64,
    jusante: f64,
    desloca: f64,
    max: f64,
    total: f64,
}

fn mede(seca: bool, drag: Option<f64>) -> (Medida, Engine) {
    let mut e = mancha(seca);
    let antes = colunas(&e);
    water_brush(&mut e, drag.map(|d| (Knob::WaterDrag, d)).as_slice());
    let c = colunas(&e);
    let cx = |v: &[f64]| {
        let m: f64 = v.iter().sum();
        v.iter().enumerate().map(|(x, p)| x as f64 * p).sum::<f64>() / m
    };
    let g = e.active_grid();
    let max = g
        .susp
        .iter()
        .zip(&g.sett)
        .map(|(a, b)| f64::from(*a + *b))
        .fold(0.0, f64::max);
    let m = Medida {
        origem: soma(&c, 50, 80) / soma(&antes, 50, 80),
        jusante: soma(&c, 90, 240) / soma(&antes, 50, 80),
        desloca: cx(&c) - cx(&antes),
        max,
        total: c.iter().sum::<f64>() / antes.iter().sum::<f64>(),
    };
    println!(
        "[arrasto] seca={seca} drag={drag:?}: origem {:.3} · jusante {:.3} · centróide +{:.2} px · máx {:.1} · total {:.4}",
        m.origem, m.jusante, m.desloca, m.max, m.total
    );
    (m, e)
}

/// ⭐ **O Water Brush ARRASTA o que levantou** (`Knob::WaterDrag`, 31/10): a mancha (molhada ou seca) atravessada por um
/// traço da água para +x. Sem o knob a composição de hoje (`wetLift` 0,7 + `waterPush` 0,35) não leva nada pelo
/// traço — molhada: centróide +0,00 px, 0,0 % depois da mancha; seca: +1,57 px, 0,1 % (o `WATER_DRAG` da tinta seca).
/// Com `waterDrag` 0,8: molhada +1,55 px, 4,4 % depois da mancha, a origem a 0,912; seca +5,19 px, 7,0 %, origem 0,833.
/// E nada escurece: a massa conserva-se (TRANSPORTE) e nenhuma célula passa do máximo que já havia sem o knob (o anel
/// do Mop, R1 §2).
#[test]
fn a_agua_arrasta_o_levantado_pelo_traco() {
    let mut falhas = Vec::new();
    for seca in [false, true] {
        let (sem, _) = mede(seca, None);
        let (com, _) = mede(seca, Some(0.8));
        let lado = if seca { "seca" } else { "molhada" };
        let ganho = if seca { 3.0 } else { 1.0 };
        if com.desloca < sem.desloca + ganho {
            falhas.push(format!(
                "{lado}: o centróide andou {:.2} px, não passa {:.2} + {ganho}",
                com.desloca, sem.desloca
            ));
        }
        if com.jusante < 0.02 || sem.jusante > 0.002 {
            falhas.push(format!(
                "{lado}: depois da mancha {:.3} com · {:.3} sem (quer ≥ 0,02 · ≤ 0,002)",
                com.jusante, sem.jusante
            ));
        }
        if com.origem > sem.origem - 0.03 {
            falhas.push(format!(
                "{lado}: a origem não clareou: {:.3} contra {:.3}",
                com.origem, sem.origem
            ));
        }
        if com.max > sem.max.max(120.0) + 0.5 {
            falhas.push(format!(
                "{lado}: escureceu além da tinta: máx {:.1} contra {:.1}",
                com.max, sem.max
            ));
        }
        if (com.total - 1.0).abs() > 2e-3 {
            falhas.push(format!(
                "{lado}: a massa não se conservou: {:.4}",
                com.total
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// O knob a 0 é a água de antes AO BIT, e o papel sem tinta fica sem tinta (a água não tem o que levar).
#[test]
fn a_agua_sem_arrasto_e_a_de_antes() {
    for seca in [false, true] {
        let (_, a) = mede(seca, None);
        let (_, b) = mede(seca, Some(0.0));
        let (ga, gb) = (a.active_grid(), b.active_grid());
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&ga.susp), bits(&gb.susp));
        assert_eq!(bits(&ga.sett), bits(&gb.sett));
        assert_eq!(bits(&ga.film), bits(&gb.film));
        assert_eq!(bits(&ga.susp_rgb.concat()), bits(&gb.susp_rgb.concat()));
        let (_, c) = mede(seca, Some(0.8));
        let g = c.active_grid();
        let antes_da_mancha: f64 = (Y - 12..=Y + 12)
            .flat_map(|y| (20..48).map(move |x| (x, y)))
            .map(|(x, y)| f64::from(g.susp[x + 1 + (y + 1) * g.s] + g.sett[x + 1 + (y + 1) * g.s]))
            .sum();
        assert_eq!(
            antes_da_mancha, 0.0,
            "a água levou tinta para TRÁS do traço"
        );
    }
}
