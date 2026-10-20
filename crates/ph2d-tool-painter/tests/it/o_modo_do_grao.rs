//! ⭐ **O modo do GRÃO e a profundidade por dab** (L9, `docs/Painter/Pinceis/03_MOTORES.md` §4) pela porta do
//! ponteiro e pelas linhas do painel (`set_ponta`), no Digital, contra a lei MEDIDA nos lápis do Rebelle.
//!
//! O oráculo (`docs/Painter/Pinceis/fixtures/rebelle_bateria/PENCIL/*_gestos_A.png`, a `rampa_0_1` da bateria, miolo
//! ±6 px, tinta = escuro > 3 % do máximo): o cv da tinta CAI com a pressão — HB 0,41 (4.º decil) → 0,10 (10.º),
//! 8B 0,46 → 0,08, 2H 0,37 → 0,13, Colored 4B 0,36 → 0,16 — e a fracção que pinta sobe de 0 a 1. É o grão que ABRE:
//! a pressão baixa só toca os picos do papel. Num Multiply o cv não muda com a pressão (o grão é um factor fixo).

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{LinhaDaPonta, PaintMedia, PainterTool};

const W: usize = 600;
const H: usize = 120;
const PAPEL_RUGOSO: u8 = 27; // TextureKind::PaperRough
const TILED: u8 = 1; // TextureMapping::Tiled: o grão preso ao papel

/// A rampa 0 → 1 (x 40 → 560, y 60), lápis preto raio 14 com o grão rugoso preso ao papel; `f` arma a lei.
fn rampa(dura: bool, f: impl Fn(&mut PainterTool)) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(14.0);
    if dura {
        t.set_brush_falloff(8); // a ponta dura: a régua mede o grão, não a queda
    }
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_texture_kind(PAPEL_RUGOSO);
    t.set_brush_texture_mapping(TILED);
    // O dente FINO do papel (Size 6: o tile de 256 px fica ~43 px): o miolo de cada decil vê muitos dentes.
    t.set_brush_texture_size(0, 6.0);
    t.set_brush_texture_size(1, 6.0);
    f(&mut t);
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: ((x - 40.0) / 520.0).clamp(0.0, 1.0),
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(40.0, PointerPhase::Down));
    let mut x = 40.0;
    while x < 560.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    t.run_full().0
}

/// Por decil de pressão: `(fracção do miolo que pinta, cv da tinta)` — a régua do oráculo (miolo ±2 px: a
/// pressão também afina o traço).
fn decis(px: &[u8]) -> Vec<(f64, f64)> {
    let escuro = |x: usize, y: usize| 1.0 - f64::from(px[(y * W + x) * 4]) / 255.0;
    let dmax = {
        let mut v: Vec<f64> = (480..560).flat_map(|x| (58..63).map(move |y| (x, y))).map(|(x, y)| escuro(x, y)).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() * 95 / 100].max(1e-6)
    };
    (0..10)
        .map(|k| {
            let tinta: Vec<f64> = (40 + k * 52..40 + (k + 1) * 52)
                .flat_map(|x| (58..63).map(move |y| (x, y)))
                .map(|(x, y)| escuro(x, y) / dmax)
                .collect();
            let n = tinta.len() as f64;
            let com: Vec<f64> = tinta.into_iter().filter(|v| *v > 0.03).collect();
            let frac = com.len() as f64 / n;
            let m = com.iter().sum::<f64>() / com.len().max(1) as f64;
            let sd = (com.iter().map(|v| (v - m).powi(2)).sum::<f64>() / com.len().max(1) as f64).sqrt();
            (frac, if com.len() > 20 { sd / m } else { f64::NAN })
        })
        .collect()
}

/// ⭐ **O Height abre o grão com a pressão** (a lei dos lápis do Rebelle): com o modo Height/Linear Height e a
/// profundidade pela pressão, a pressão baixa só pinta os PICOS do papel (fracção do miolo < 0,5 no 4.º decil) e a
/// cheia enche (> 0,9); no Linear Height a tinta gradua e o cv cai ≥ 2× (medido 0,46 → 0,14; Rebelle HB 0,41 →
/// 0,10). Vermelho hoje: o grão de sempre (Multiply) pinta o miolo inteiro em toda pressão (0,99).
#[test]
fn o_height_abre_o_grao_com_a_pressao() {
    let pela_pressao = |modo: f64| {
        decis(&rampa(true, move |t| {
            t.set_ponta(LinhaDaPonta::GrainMode, modo);
            t.set_ponta(LinhaDaPonta::DepthJitter, 1.0);
            t.set_ponta(LinhaDaPonta::DepthPressure, 1.0);
        }))
    };
    let (height, linear, multiply) = (pela_pressao(4.0), pela_pressao(5.0), pela_pressao(0.0));
    let hoje = decis(&rampa(true, |_| {}));
    let fmt = |d: &[(f64, f64)]| d.iter().map(|(f, c)| format!("{f:.2}|{c:.2}")).collect::<Vec<_>>().join(" ");
    println!(
        "[grao] height   {}\n[grao] linear   {}\n[grao] multiply {}\n[grao] hoje     {}",
        fmt(&height),
        fmt(&linear),
        fmt(&multiply),
        fmt(&hoje)
    );
    // O Height: a ponta toca só os picos e pinta-os inteiros (a fracção abre, a tinta é binária).
    assert!(height[3].0 < 0.5 && height[9].0 > 0.9, "Height: a fracção não abre");
    // O Linear Height (o lápis): a fracção abre E a tinta gradua — o cv cai como no Rebelle (≥ 2×; Rebelle 3–6×).
    let (f_baixa, cv_baixa) = linear[3];
    let (f_cheia, cv_cheia) = linear[9];
    assert!(f_baixa < 0.5 && f_cheia > 0.9, "Linear Height: a fracção não abre: {f_baixa:.2} → {f_cheia:.2}");
    assert!(cv_baixa >= 2.0 * cv_cheia, "Linear Height: o cv não cai: {cv_baixa:.2} → {cv_cheia:.2}");
    // O controlo: o grão de hoje (Multiply) pinta o miolo INTEIRO em toda pressão — não abre.
    assert!(hoje[3].0 > 0.9, "o controlo: o grão de hoje já abria? {:.2}", hoje[3].0);
    assert!(multiply[3].0 > 0.9, "o Multiply com a profundidade pela pressão não abre: {:.2}", multiply[3].0);
}

/// O neutro: a lei de fábrica é o Multiply sem profundidade por dab — e as linhas voltam a ela.
#[test]
fn a_lei_do_grao_de_fabrica_e_o_multiply() {
    let a = rampa(false, |_| {});
    let b = rampa(false, |t| {
        t.set_ponta(LinhaDaPonta::GrainMode, 0.0);
        t.set_ponta(LinhaDaPonta::DepthJitter, 0.0);
        t.set_ponta(LinhaDaPonta::DepthMin, 0.0);
        t.set_ponta(LinhaDaPonta::DepthPressure, 0.0);
    });
    assert_eq!(a, b);
}

/// Cada modo muda a tinta (nenhum é o Multiply com outro nome) e só TIRA tinta ao Multiply de profundidade cheia
/// onde a ponta é cheia (o miolo nunca fica mais escuro que o Multiply).
#[test]
fn cada_modo_do_grao_e_outro() {
    // A ponta MACIA: na dura (`w = 1`) o Subtract e o Darken SÃO o Multiply e o Color Burn não morde (a lei).
    let base = rampa(false, |t| t.set_brush_grain_depth(0.6));
    for m in 1..6 {
        let px = rampa(false, |t| {
            t.set_brush_grain_depth(0.6);
            t.set_ponta(LinhaDaPonta::GrainMode, f64::from(m));
        });
        let n = px.iter().zip(&base).filter(|(a, b)| a != b).count();
        assert!(n > 500, "o modo {m} mudou só {n} bytes");
    }
}
