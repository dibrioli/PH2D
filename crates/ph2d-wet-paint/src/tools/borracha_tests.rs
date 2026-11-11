//! B35.9 — a Borracha do Wet tem UM controlo (o Erase Strength do cartão): o par Eraser (Tuning) ×
//! Erase Strength fundiu-se em `força = erase'`, com `erase' = 0,4·erase` (o 0,4 da fábrica do Eraser).
//! No máximo (`1`) a borracha limpa o miolo do traço.

use super::*;
use crate::painter::{Engine, Tool};

const W: usize = 160;
const H: usize = 120;

fn traco(e: &mut Engine, simula: bool) {
    e.pointer_down(30.0, 60.0, None);
    let mut x: f64 = 30.0;
    while x < 130.0 {
        x = (x + 4.0).min(130.0);
        e.pointer_frame(x, 60.0);
        if simula && e.sim_should_run() {
            e.step_simulation();
        }
    }
    e.pointer_up();
}

fn pintado() -> Engine {
    let mut e = Engine::new(W, H);
    e.color = [190.0, 40.0, 30.0];
    traco(&mut e, true);
    e
}

/// A massa (suspensa, assente) no miolo do traço: x 50..110, as três linhas do eixo.
fn miolo(e: &Engine) -> (f64, f64) {
    let g = e.active_grid();
    let (mut su, mut se) = (0.0f64, 0.0f64);
    for y in 59..=61 {
        for x in 50..110 {
            let i = y * g.w + x;
            su += f64::from(g.susp[i]);
            se += f64::from(g.sett[i]);
        }
    }
    (su, se)
}

/// F26-07 / DF-59c: com a força no máximo e o Hardness (portão do papel) a 0, o carimbo cheio dava
/// `gg = 1`, `k = 0` e a célula ficava INTACTA — anel apagado, miolo cheio. Pela porta do produto
/// (silhueta e grão cheios do anfitrião), o miolo perde toda a tinta solta.
#[test]
fn borracha_no_maximo_apaga_o_miolo() {
    let mut e = pintado();
    let (su0, se0) = miolo(&e);
    assert!(
        su0 > 1000.0,
        "fixtura: o traço tem tinta solta no miolo ({su0} + {se0})"
    );
    e.set_knob(Knob::PaperGate, 0.0);
    e.sliders.erase = 1.0;
    e.sliders.pressure = 1.0; // carimbo cheio: `0,5·b·0,9` passa de 1
    let cheia = |_: i32, _: i32| 1.0;
    let mut x = 30.0;
    while x <= 130.0 {
        e.dispatch_pressure_dab_erase(x, 60.0, 4.0, 1.0, 8.0, Some(&cheia), Some(&cheia));
        x += 2.0;
    }
    let (su, se) = miolo(&e);
    assert_eq!(
        su, 0.0,
        "a borracha no máximo deixou tinta solta no miolo ({su0} → {su})"
    );
    assert!(se <= se0, "a assente no miolo cresceu: {se0} → {se}");
}

/// A lei de antes, verbatim (só o caminho sem silhueta): `força = Eraser · erase` e a célula com `k ≤ 0`
/// intocada.
fn apagar_lei_antiga(g: &mut Grid, p: &Params, tex: &[f32], dab: &Dab, erase_slider: f64) {
    let gate = p.k(Knob::PaperGate);
    let pig_cap = p.k(Knob::GateSaturation);
    let force = p.k(Knob::Eraser) * erase_slider;
    let force_sett = force * (1.0 - 0.5 * p.k(Knob::ExtStaining));
    let clamp_i = dab.intensity.min(3.0);
    let Grid {
        s,
        w,
        h,
        susp,
        sett,
        film,
        paper,
        ..
    } = g;
    let body = |i: usize, _x: i32, _y: i32, fall: f64, texv: f64| {
        let mut stamp = fall * texv * clamp_i;
        if stamp > 1.0 {
            stamp = 1.0;
        }
        let tooth = if (susp[i] as f64 + sett[i] as f64) < pig_cap {
            paper[i] as f64
        } else {
            0.45
        };
        let mut gg = stamp - (1.0 - tooth) * gate;
        if gg <= 0.0 {
            return;
        }
        if gg > 1.0 {
            gg = 1.0;
        }
        let k = 1.0 - force * gg;
        if k <= 0.0 {
            return;
        }
        susp[i] = (susp[i] as f64 * k) as f32;
        film[i] = (film[i] as f64 * k) as f32;
        if (susp[i] as f64) < 1.0 {
            susp[i] = 0.0;
        }
        let k_sett = 1.0 - force_sett * gg;
        sett[i] = (sett[i] as f64 * k_sett) as f32;
        if (sett[i] as f64) < 1.0 {
            sett[i] = 0.0;
        }
    };
    for_each_stamp_pixel(*s, *w, *h, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, body);
}

fn bits(g: &Grid) -> Vec<u32> {
    g.susp
        .iter()
        .chain(&g.sett)
        .chain(&g.film)
        .map(|v| v.to_bits())
        .collect()
}

#[test]
fn eraser_fundido_da_a_mesma_borracha_ao_bit() {
    let mut e = pintado();
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.texture().to_vec();
    let base = e.active_grid().clone();
    for erase in [0.05, 0.25, 0.4, 0.7, 1.0] {
        let (mut antiga, mut nova) = (base.clone(), base.clone());
        for (x, b) in [(40.0, 0.3), (61.5, 0.8), (80.0, 1.0), (99.25, 0.55)] {
            let (dab, _) = e.pressure_dab(&p, x, 60.0, b, 6.0);
            apagar_lei_antiga(&mut antiga, &p, &tex, &dab, erase);
            apply_erase_impl(
                &mut nova,
                &p,
                &tex,
                &dab,
                ERASER_DA_FABRICA * erase,
                None,
                None,
            );
        }
        assert!(
            bits(&antiga) != bits(&base),
            "fixtura: a borracha {erase} apagou algo"
        );
        assert!(
            bits(&antiga) == bits(&nova),
            "erase {erase}: a borracha fundida difere da antiga"
        );
    }
}
