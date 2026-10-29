//! ⭐ **A pena da mancha molhada fala em PX** ([`Knob::WetOpenEdge`], `trail/molhado.rs`, J6 auditoria motor 2): o
//! mesmo pincel (raio 60 px, pena 24 px) sobre o mesmo vermelho molhado a razão 1 (60 células) e 2 (30 células, a pena
//! convertida por `Params::px_por_celula`) levanta a outra tinta com a MESMA rampa em px através do traço.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

const R_PX: f64 = 60.0;
const PENA_PX: f64 = 24.0;
const BASE: [f64; 3] = [23.0, 101.0, 179.0];

/// A fracção levantada do vermelho por linha da coluna do meio do traço, à razão `razao` (grade `600/razao × 400/razao`).
fn levantado(razao: f64, pena_px: f64) -> Vec<f64> {
    let (w, h) = ((600.0 / razao) as usize, (400.0 / razao) as usize);
    let mut e = Engine::new(w, h);
    let g = e.active_grid_mut();
    for i in 0..g.susp.len() {
        g.film[i] = 0.5;
        g.susp[i] = 60.0;
        g.susp_rgb[i] = [191.0, 4.0, 18.0];
    }
    e.tuning.set(Knob::WetOpen, 0.6);
    e.tuning.set(Knob::WetOpenPush, 0.8);
    e.tuning.set(Knob::WetOpenEdge, pena_px);
    let mut p = e.sim.gather_params(&e.tuning);
    p.px_por_celula = razao;
    let tex = e.bristle_texture_for_measure();
    let antes = e.active_grid_mut().susp.clone();
    let mut t = Trail::default();
    t.start_stroke(150.0 / razao, 200.0 / razao, BASE, TrailMode::Paint);
    t.on_segment(12.0 / razao, 12.0 / razao);
    let g = e.active_grid_mut();
    for k in 0..26 {
        let d = ph2d_wet_paint::trail::Dab {
            x: (150.0 + 12.0 * f64::from(k)) / razao,
            y: 200.0 / razao,
            r: R_PX / razao,
            hardness: 0.5,
            intensity: 4.0,
            water_amount: 0.5,
            dry_gate: 0.2,
            reach: 1.0,
            window_reach: 1.0,
            opacity: 1.0,
        };
        if t.accumulate_paint(g, &p, &tex, &d, false).window_full {
            t.transfer_paint(g, &p);
        }
    }
    let cx = (300.0 / razao) as usize;
    (0..g.h + 2)
        .map(|y| {
            let i = cx + y * g.s;
            f64::from((antes[i] - g.susp[i]) / antes[i])
        })
        .collect()
}

/// A largura em px da rampa de fora do levantado (de 90 % a 10 % do máximo), média dos dois lados do traço.
fn rampa_px(v: &[f64], razao: f64) -> f64 {
    let mx = v.iter().copied().fold(0.0, f64::max);
    // Vindo de FORA para dentro: o miolo tem o pigmento que o traço pousou (a fracção cai lá), a rampa não.
    let cruza = |nivel: f64, passo: isize| -> f64 {
        let mut y: isize = if passo > 0 { v.len() as isize - 1 } else { 0 };
        while v[y as usize] < nivel * mx {
            y -= passo;
        }
        let (a, b) = (v[y as usize], v[(y + passo) as usize]);
        y as f64 + passo as f64 * (a - nivel * mx) / (a - b)
    };
    let lado = |passo: isize| (cruza(0.1, passo) - cruza(0.9, passo)).abs();
    0.5 * (lado(1) + lado(-1)) * razao
}

/// ⭐ GATE-LEI: a rampa em px (90 %→10 % do levantado) é a mesma a razão 1 e 2 — medido `33,31` × `33,33` px (sem
/// pena `18,72` × `18,79`); tolerância `1` px (½ célula da razão 2) — e a pena alarga-a de verdade (controlo: ≥ ½ da
/// pena). Com a pena em células a razão 2 ganhava o DOBRO da pena em px.
#[test]
fn a_pena_da_mancha_molhada_e_a_mesma_em_px_a_toda_a_razao() {
    let (um, dois) = (levantado(1.0, PENA_PX), levantado(2.0, PENA_PX));
    let (r1, r2) = (rampa_px(&um, 1.0), rampa_px(&dois, 2.0));
    let (s1, s2) = (rampa_px(&levantado(1.0, 0.0), 1.0), rampa_px(&levantado(2.0, 0.0), 2.0));
    println!("[pena px] rampa: razão 1 {r1:.2} px · razão 2 {r2:.2} px; sem pena {s1:.2} · {s2:.2}");
    assert!(r1 - s1 >= 0.5 * PENA_PX, "controlo: a pena alarga a rampa ({s1} → {r1})");
    assert!((r1 - r2).abs() <= 1.0, "a pena em px: razão 1 {r1} px × razão 2 {r2} px");
}
