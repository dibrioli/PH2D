//! **Sonda (ignorada): os 4 pincéis de fábrica do Wet Paint na bancada do Rebelle** (conferência do dono
//! 10/10: «o Wet Wash é seco mesmo? o nome confere?»). A geometria de `ferramentas/gotas.py` (tela 1000×640,
//! traços horizontais de 150 px a 40 px, pressão 1, o azul da bancada, x0 = 60 + 240·i, y = 150), um pincel
//! por coluna — Wet Round · Wet Wash · Wet Dry Brush · Wet Drip —, cada um numa tela SUA (a água de um não
//! corre com os knobs do outro) e as quatro coladas. Grava `presets_<modo>_2s.ppm` e `presets_<modo>.ppm`
//! (30 s) em `PH2D_SONDA_OUT`, para `modo` ∈ `plano` (tela direita) · `inclinado` (anel 8, para baixo) ·
//! `preset` (o Tilt que cada preset guarda). A régua: `ferramentas/presets_regua.py`.
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 1000;
const H: usize = 640;
const PINCEIS: [&str; 4] = ["Wet Round", "Wet Wash", "Wet Dry Brush", "Wet Drip"];

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 150.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um pincel na coluna `i`: as telas a 2 s e a 30 s.
fn coluna(b: &Biblioteca, i: usize, modo: &str) -> (Vec<u8>, Vec<u8>) {
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == PINCEIS[i])
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(40.0),
        ..p.parametros.volumes.clone()
    });
    match modo {
        "plano" => t.paint.wetpaint.tilt_on = false,
        "inclinado" => {
            t.paint.wetpaint.tilt_on = true;
            t.paint.wetpaint.tilt_ring = 8;
            t.paint.wetpaint.tilt_spoke = 3;
        }
        _ => {}
    }
    t.set_brush_color_srgb8([23, 101, 179]);
    assert!(t.ensure_wet_session());
    let x0 = 60.0 + i as f32 * 240.0;
    t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
    for k in 1..=50 {
        t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(x0 + 150.0, PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    let dois = t.canvas_rgba.to_vec();
    for _ in 120..30 * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    (dois, t.canvas_rgba.to_vec())
}

fn grava(dir: &str, nome: &str, px: &[u8]) {
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for q in px.as_chunks::<4>().0 {
        ppm.extend_from_slice(&q[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava a tela");
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_presets() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    for modo in ["plano", "inclinado", "preset"] {
        let (mut a, mut z) = (vec![255u8; W * H * 4], vec![255u8; W * H * 4]);
        for i in 0..4 {
            let (dois, fim) = coluna(&b, i, modo);
            // Cada tela só tem a sua coluna: cola a faixa x 240·i … 240·(i+1).
            for y in 0..H {
                for x in 240 * i..(240 * (i + 1)).min(W) {
                    let k = (y * W + x) * 4;
                    a[k..k + 4].copy_from_slice(&dois[k..k + 4]);
                    z[k..k + 4].copy_from_slice(&fim[k..k + 4]);
                }
            }
        }
        grava(&dir, &format!("presets_{modo}_2s"), &a);
        grava(&dir, &format!("presets_{modo}"), &z);
    }
}

/// A régua de `ferramentas/presets_regua.py` em Rust, coluna `i`: (tinta, largura, orla, buracos, escorre).
pub(super) fn regua(px: &[u8], i: usize) -> (f64, f64, f64, f64, usize) {
    let e = |x: usize, y: usize| {
        let k = (y * W + x) * 4;
        255.0 - f64::from(px[k].min(px[k + 1]).min(px[k + 2]))
    };
    let med = |v: &mut Vec<f64>| {
        if v.is_empty() {
            return 0.0;
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let x0 = 60 + 240 * i;
    let (mut larg, mut orlas, mut buracos, mut centro) = (vec![], vec![], vec![], vec![]);
    for x in x0 + 40..=x0 + 110 {
        let ys: Vec<usize> = (100..260)
            .filter(|&y| e(x, y) > 25.0)
            .filter(|&y| y.abs_diff(150) < 60)
            .collect();
        let (Some(&top), Some(&bot)) = (ys.first(), ys.last()) else {
            continue;
        };
        larg.push((bot - top + 1) as f64);
        let n = bot - top + 1;
        let mut cen: Vec<f64> = (top + n / 5..=bot - n / 5).map(|y| e(x, y)).collect();
        let c = med(&mut cen.clone());
        centro.push(c);
        let o = ((top..top + 3).map(|y| e(x, y)).sum::<f64>()
            + (bot - 2..=bot).map(|y| e(x, y)).sum::<f64>())
            / 6.0;
        orlas.push(o);
        let fura = cen.iter().filter(|&&v| v < c / 4.0).count() as f64 / cen.len().max(1) as f64;
        buracos.push(fura);
        cen.clear();
    }
    let t = med(&mut centro);
    let l = med(&mut larg);
    let o = med(&mut orlas);
    let fb = 150 + (l as usize) / 2;
    // O escorrimento conta a partir de 30 % da tinta do próprio traço (mínimo 6 níveis): com a Opacity os
    // fios de uma tinta clara ficam abaixo dos 25 absolutos e a régua lia 0 onde a foto mostra fios.
    let limiar = (0.3 * t).max(6.0);
    let mut desc = 0;
    for x in x0.saturating_sub(10)..=x0 + 160 {
        let mut k = 0;
        while fb + k < H && e(x, fb + k) > limiar {
            k += 1;
        }
        desc = desc.max(k);
    }
    let b = if buracos.is_empty() {
        0.0
    } else {
        buracos.iter().sum::<f64>() / buracos.len() as f64
    };
    (t, l, if t > 0.0 { o / t } else { 0.0 }, b, desc)
}

/// Um pincel com knobs e água trocados (a varredura da calibração contra o Rebelle).
fn coluna_com(
    b: &Biblioteca,
    i: usize,
    inclinado: bool,
    knobs: &[(Knob, f64)],
    agua: Option<f64>,
    opacidade: Option<f32>,
) -> Vec<u8> {
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == PINCEIS[i])
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    let mut v = p.parametros.volumes.clone();
    v.size_px = Some(40.0);
    if agua.is_some() {
        v.water = agua;
    }
    t.wet_preset_aplicar_volumes(&v);
    for (k, x) in knobs {
        t.paint.wetpaint.knobs.set(*k, *x);
    }
    if let Some(o) = opacidade {
        t.paint.wetpaint.opacidade = o;
    }
    t.paint.wetpaint.tilt_on = inclinado;
    t.paint.wetpaint.tilt_ring = 8;
    t.paint.wetpaint.tilt_spoke = 3;
    t.set_brush_color_srgb8([23, 101, 179]);
    assert!(t.ensure_wet_session());
    let x0 = 60.0 + i as f32 * 240.0;
    t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
    for k in 1..=50 {
        t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(x0 + 150.0, PointerPhase::Up));
    for _ in 0..30 * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// **Sonda: a varredura dos 4 pincéis contra o Rebelle** (fixtures/rebelle_presets). `PH2D_SONDA_VARRE` =
/// `<pincel 0..3>|<knob>=<v>,…|<água ou ->|<opacity>` por linha (`;` separa); cada linha mede-se plana e inclinada.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_presets_varre() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let linhas = std::env::var("PH2D_SONDA_VARRE").expect("PH2D_SONDA_VARRE");
    for l in linhas.split(';').map(str::trim).filter(|l| !l.is_empty()) {
        let partes: Vec<&str> = l.split('|').collect();
        let i: usize = partes[0].parse().expect("pincel");
        let knobs: Vec<(Knob, f64)> = partes[1]
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|kv| {
                let (k, v) = kv.split_once('=').expect("knob=v");
                let def = ph2d_wet_paint::tuning::KNOB_DEFS
                    .iter()
                    .find(|d| d.key == k)
                    .unwrap_or_else(|| panic!("knob {k}"));
                (def.knob, v.parse().expect("valor"))
            })
            .collect();
        let agua = partes.get(2).and_then(|a| a.parse().ok());
        let op = partes.get(3).and_then(|a| a.parse().ok());
        let (t, la, o, bu, _) = regua(&coluna_com(&b, i, false, &knobs, agua, op), i);
        let (ti, li, oi, _, es) = regua(&coluna_com(&b, i, true, &knobs, agua, op), i);
        println!(
            "[{l}] plano: tinta {t:.0} largura {la:.0} orla {o:.2} buracos {bu:.2} · inclinado: tinta {ti:.0} largura {li:.0} orla {oi:.2} escorre {es}"
        );
    }
}
