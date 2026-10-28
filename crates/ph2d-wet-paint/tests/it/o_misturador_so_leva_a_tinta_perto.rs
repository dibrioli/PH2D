//! ⭐ **O misturador só leva a tinta PERTO dela** ([`Knob::BlendReach`], J6 lacuna 6 — `docs/Painter/Pinceis/exames/
//! mistura.md`): a média para onde o Blend relaxa cada célula é a das células da janela a `≤ BlendReach × raio` dela. O
//! Blend com grão do Rebelle (`rebelle_mistura/BLEND/default_textured`) borra ~30 px para fora da faixa; a média da
//! janela inteira arrastava-a ao longo do traço todo (52–66 px no Blender Grainy).

use ph2d_wet_paint::painter::{Engine, Tool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 300;
const H: usize = 320;
/// A faixa MOLHADA (y 140..170, x 20..280) e o traço do Blend em vertical por x = 150, de y 20 a 300.
const Y0: usize = 140;
const Y1: usize = 170;
const X: usize = 150;
const RAIO: f64 = 30.0;
const PASSO: f64 = 3.0;
/// O `BlendReach` do Blender Grainy (J6).
const ALCANCE: f64 = 0.75;

/// A faixa misturada por um traço do Blend com o grão em xadrez e o `BlendReach` dado (`None` = o knob por omissão);
/// devolve o motor.
fn mistura(alcance: Option<f64>) -> Engine {
    let mut e = Engine::new(W, H);
    {
        let g = e.active_grid_mut();
        for y in Y0..=Y1 {
            for x in 20..=280 {
                let i = x + 1 + (y + 1) * g.s;
                g.susp[i] = 120.0;
                g.susp_rgb[i] = [191.0, 0.0, 18.0];
                g.film[i] = 0.3;
            }
        }
    }
    if let Some(a) = alcance {
        e.tuning.set(Knob::BlendReach, a);
    }
    let xadrez = |x: i32, y: i32| 0.4 + 0.6 * f64::from(u8::from((x / 3 + y / 3) % 2 == 0));
    e.tool = Tool::Blend;
    e.begin_direct_stroke(0, X as f64, 20.0);
    let mut y = 20.0;
    while y <= 300.0 {
        e.direct_segment(0, PASSO);
        e.dispatch_pressure_dab_lane_blend_grao(0, X as f64, y, 5.0, RAIO, Some(&xadrez));
        y += PASSO;
    }
    e.end_direct_stroke();
    e
}

/// Quantas linhas ACIMA (o lado de onde o traço vem: a janela arrasta-a para trás) e ABAIXO da faixa a banda do traço
/// (x ± raio) ainda tem tinta VISÍVEL (`susp > 20`, ~1/6 da faixa).
fn alcance(e: &Engine) -> (usize, usize) {
    let g = e.active_grid();
    let tem = |y: usize| (X - 30..=X + 30).any(|x| g.susp[x + 1 + (y + 1) * g.s] > 20.0);
    let acima = (0..Y0).rev().take_while(|&y| tem(y)).count();
    let abaixo = (Y1 + 1..H - 2).take_while(|&y| tem(y)).count();
    (acima, abaixo)
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn o_misturador_so_leva_a_tinta_perto() {
    let janela = mistura(None);
    let curto = mistura(Some(ALCANCE / 2.0));
    let perto = mistura(Some(ALCANCE));
    let (ja, jb) = alcance(&janela);
    let (ca, cb) = alcance(&curto);
    let (pa, pb) = alcance(&perto);
    println!(
        "[mistura] alcance acima · abaixo da faixa (raio {RAIO}): janela {ja} · {jb} · perto {} {pa} · {pb} · perto {} {ca} · {cb}",
        ALCANCE,
        ALCANCE / 2.0
    );
    // Medido (J6): janela 41 · 15, perto 0,75 → 28 · 18. Para trás o alcance cai com o BlendReach — a mistura deixa de ir
    // aonde a janela a levava; para a frente o traço continua a arrastá-la (o Rebelle também: 30 acima, 41 abaixo).
    assert!(
        (pa as f64) <= ja as f64 - RAIO / 3.0,
        "o misturador perto leva a tinta tão longe como a janela: {pa} × {ja}"
    );
    assert!(ca < pa, "o alcance não segue o BlendReach: {ca} ({}) × {pa} ({ALCANCE})", ALCANCE / 2.0);
    // … e continua a MISTURAR perto: a tinta sai da faixa dos dois lados.
    assert!(ca.min(cb) >= 4, "o misturador perto não mistura: {ca} · {cb}");
    // Para a frente a caixa local arrasta o que a janela arrasta (± 6 px), nem menos nem MAIS (uma caixa torta — só
    // para trás — leva a mistura mais longe à frente: a prova de mutação M2).
    assert!(
        pb + 6 >= jb && pb.max(cb) <= jb + 6,
        "para a frente o misturador perto não arrasta como a janela: {pb} · {cb} × {jb}"
    );
    // `0` dito é a média da janela, ao bit.
    let zero = mistura(Some(0.0));
    let (gj, gz) = (janela.active_grid(), zero.active_grid());
    assert_eq!(bits(&gj.susp), bits(&gz.susp));
    assert_eq!(bits(&gj.film), bits(&gz.film));
    assert_eq!(gj.susp_rgb, gz.susp_rgb);
}
