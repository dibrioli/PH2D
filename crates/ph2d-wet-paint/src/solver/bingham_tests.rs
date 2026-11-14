//! E7 — a lei de Bingham do óleo grosso pela porta do produto (`Engine`): o óleo com pouca água NÃO escorre fora do
//! pincel. Doc: `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md` §6.

use crate::painter::Engine;
use crate::tuning::Knob;

const L: usize = 96;
const Y1: f64 = 48.0;
const X2: f64 = 48.0;

fn traco(e: &mut Engine, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
    e.pointer_down(x0, y0, None);
    for i in 1..=12 {
        let t = f64::from(i) / 12.0;
        e.pointer_frame(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
    }
    e.pointer_up();
    while e.release_frame(x1, y1) {}
}

fn massa(e: &Engine) -> Vec<f64> {
    let g = e.active_grid();
    (0..g.cells)
        .map(|i| f64::from(g.susp[i]) + f64::from(g.sett[i]))
        .collect()
}

fn corre(e: &mut Engine, n: usize) {
    for _ in 0..n {
        e.step_simulation();
    }
}

/// A tinta da camada 1 FORA do contacto do traço 2 (`|x − X2| > folga`): quanto mudou depois do traço 2, ÷ a mediana.
fn deriva_fora(agua: f64) -> f64 {
    let mut e = Engine::new(L, L);
    e.sim.oleo = true;
    e.set_knob(Knob::Medium, 1.0); // E10: o `m` de Bingham é o meio, não o Real Oil
    e.set_knob(Knob::PigmentPerDab, 3000.0);
    e.sliders.water = agua;
    e.sliders.size = 9.0;
    traco(&mut e, (12.0, Y1), (84.0, Y1));
    corre(&mut e, 120);
    let m1 = massa(&e);
    traco(&mut e, (X2, 12.0), (X2, 84.0));
    corre(&mut e, 120);
    let m2 = massa(&e);
    let s = e.active_grid().s;
    let mmax = m1.iter().copied().fold(0.0, f64::max);
    assert!(mmax > 0.0, "a camada 1 não pousou");
    let fora: Vec<usize> = (0..m1.len())
        .filter(|&i| m1[i] > 0.05 * mmax && ((i % s) as f64 - X2).abs() > 16.0)
        .collect();
    assert!(fora.len() > 50, "fora do contacto: {} células", fora.len());
    let mut v: Vec<f64> = fora.iter().map(|&i| m1[i]).collect();
    v.sort_by(f64::total_cmp);
    let med = v[v.len() / 2];
    fora.iter()
        .map(|&i| (m2[i] - m1[i]).abs() / med)
        .sum::<f64>()
        / fora.len() as f64
}

/// ⭐ A queixa do dono (31/10): o óleo grosso com pouca água corria como líquido — o nivelamento da água arrastava a
/// tinta assentada da camada 1 fora do traço 2 (ondas e crateras longe do pincel). Com a tensão de cedência da
/// receita, a camada 1 fora do contacto fica onde estava.
#[test]
fn o_oleo_grosso_com_pouca_agua_nao_escorre_fora_do_pincel() {
    let seco = deriva_fora(0.0);
    for agua in [0.2, 0.5] {
        let d = deriva_fora(agua);
        assert!(
            d <= seco + 0.01,
            "Water {agua}: a camada 1 escorre fora do pincel — deriva {d:.4} (Water 0: {seco:.4})"
        );
    }
}

/// B13.9 (PA-30): `MaxVelocity` é o TECTO da velocidade, não a unidade da tensão de cedência. Abaixo do tecto o fluxo
/// do óleo é o mesmo bit com o tecto a 1 ou a 0,5 (antes `τ_y` era multiplicado por ele: baixar o tecto amolecia a tinta).
/// O mesmo raio (`destination_reach` = ⌈tecto⌉ = 1) nas duas: o raio é a outra leitura do tecto, legítima.
#[test]
fn maxvelocity_nao_muda_a_cedencia() {
    let mut e = Engine::new(L, L);
    e.sim.oleo = true;
    e.set_knob(Knob::Medium, 1.0);
    e.set_knob(Knob::PigmentPerDab, 3000.0);
    e.sliders.water = 0.2;
    e.sliders.size = 9.0;
    traco(&mut e, (12.0, Y1), (84.0, Y1));
    corre(&mut e, 6);
    let fluxo = |e: &mut Engine, max_v: f64| {
        e.tuning.set(Knob::MaxVelocity, max_v);
        let p = e.sim.gather_params(&e.tuning);
        let [gx, gy] = e.sim.gravity(&e.tuning);
        // A grade INTEIRA: a fotografia do undo não leva os planos de velocidade.
        let mut g = e.active_grid().clone();
        super::build_flow_field_jacobi(&mut g, &p, gx, gy, false);
        let f: Vec<(f32, f32)> = g
            .flow_x
            .iter()
            .copied()
            .zip(g.flow_y.iter().copied())
            .collect();
        f
    };
    assert!(
        !e.active_grid().oleo.is_empty(),
        "a cena não tem o campo do óleo"
    );
    let a = fluxo(&mut e, 1.0);
    assert!(a == fluxo(&mut e, 1.0), "a régua repete-se");
    let b = fluxo(&mut e, 0.5);
    let abaixo = |v: (f32, f32)| v.0.abs() < 0.49 && v.1.abs() < 0.49;
    let (mut n, mut mexe) = (0, 0);
    for (va, vb) in a.iter().zip(&b) {
        if abaixo(*va) && abaixo(*vb) {
            n += 1;
            mexe += usize::from(va.0 != 0.0 || va.1 != 0.0);
            assert_eq!(
                (va.0.to_bits(), va.1.to_bits()),
                (vb.0.to_bits(), vb.1.to_bits()),
                "o tecto mudou o fluxo abaixo dele"
            );
        }
    }
    assert!(
        mexe > 20,
        "a cena não escoa: {mexe} de {n} células com fluxo"
    );
}
