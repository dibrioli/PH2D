//! ⭐ **O Water do Smear** (`Knob::SmearWater`, dono 25/10: *«Smear tem água em seu efeito mas não encontrei onde
//! ajustar»*): no meio (`0,5`, a fábrica) a lei de antes, ao bit; abaixo dele o rasto arrasta menos água (`0` = a
//! seco); acima, molha-o até ao `WaterCap`.

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 96;
const H: usize = 96;

/// Uma faixa MOLHADA horizontal (y ≈ 48) e o Smear de cima para baixo a atravessá-la (x = 48, y 30 → 76).
fn esfregado(agua: f64) -> Engine {
    let mut e = Engine::new(W, H);
    e.color = [220.0, 45.0, 40.0];
    e.begin_direct_stroke(0, 20.0, 48.0);
    let mut x = 20.0;
    while x < 76.0 {
        x += 4.0;
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane(0, x, 48.0, 5.0, 10.0, None, None, None);
    }
    e.end_direct_stroke();
    e.set_knob(Knob::SmearWater, agua);
    let mut prev = None;
    let mut y = 30.0;
    while y <= 76.0 {
        e.dispatch_pressure_dab_tool(Tool::Smear, 48.0, y, 6.0, 12.0, prev);
        prev = Some([48.0, y]);
        y += 2.0;
    }
    e
}

/// A água no rasto ABAIXO da faixa (onde só o Smear a pode ter levado) e a impressão digital da grade.
fn medida(e: &Engine) -> (f64, u64) {
    let g = e.active_grid();
    let mut rasto = 0.0;
    for cy in 62..=80 {
        for cx in 40..=56 {
            rasto += f64::from(g.film[cx + 1 + (cy + 1) * g.s]);
        }
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for plano in [&g.susp, &g.sett, &g.film] {
        for v in plano.iter() {
            h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
        }
    }
    for c in g.susp_rgb.iter().chain(g.sett_rgb.iter()) {
        for v in c {
            h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
        }
    }
    for &b in &g.wet {
        h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
    }
    (rasto, h)
}

/// A impressão digital do esfregado com a lei de ANTES do Water (medida 25/10 com `film += (src − film)·u`, sem o knob:
/// a mesma da lei de agora a `0,5`).
const LEI_DE_ANTES: u64 = 0xeb82_b457_b3b5_5f26;

/// ⭐ No meio, a lei de antes ao bit: a grade inteira é a que a lei sem o knob deixava.
#[test]
fn o_water_do_smear_no_meio_e_a_lei_de_antes() {
    let (_, h) = medida(&esfregado(0.5));
    assert_eq!(h, LEI_DE_ANTES, "a impressão digital: {h:#x}");
}

/// Abaixo do meio o rasto arrasta MENOS água, e a seco (`0`) nenhuma; acima ele MOLHA (mais água que o arrasto, até ao
/// tecto `WaterCap`).
#[test]
fn o_water_do_smear_arrasta_a_seco_e_molha() {
    let [seco, meio_seco, meio, molhado] = [0.0, 0.25, 0.5, 1.0].map(|a| medida(&esfregado(a)).0);
    println!(
        "[smear water] rasto: 0 → {seco:.4} · 0,25 → {meio_seco:.4} · 0,5 → {meio:.4} · 1 → {molhado:.4}"
    );
    assert!(
        meio > 0.0,
        "controlo: o Smear de fábrica leva água para o rasto"
    );
    assert!(seco == 0.0, "a seco o rasto ficou com {seco} de água");
    assert!(
        seco < meio_seco && meio_seco < meio,
        "abaixo do meio: {seco} · {meio_seco} · {meio}"
    );
    assert!(
        molhado > meio,
        "acima do meio o rasto não molhou: {molhado} ≤ {meio}"
    );
    let cap = ph2d_wet_paint::tuning::KNOB_DEFS[Knob::WaterCap as usize].default as f32;
    let e = esfregado(1.0);
    let g = e.active_grid();
    let topo = (62..=80)
        .flat_map(|cy| (40..=56).map(move |cx| cx + 1 + (cy + 1) * g.s))
        .map(|i| g.film[i])
        .fold(0.0f32, f32::max);
    assert!(
        topo <= cap,
        "o rasto molhado passou o WaterCap: {topo} > {cap}"
    );
}
