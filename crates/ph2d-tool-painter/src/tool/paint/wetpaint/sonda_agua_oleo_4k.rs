//! **Sonda (ignorada) B11.3/B13.5 — a água a 4K com o Óleo Real** no relógio fixo: traços de Wet Wash numa tela
//! 3840×2160 com o Real Oil ligado; depois de cada traço, quadros sem pincel (passo da água + imagem síncrona),
//! cada um cronometrado. Imprime ms por quadro (mín/mediana) por traço e o hash da imagem e da grade no fim —
//! a régua ao bit de quem mexe no caminho. `PH2D_SONDA_TRACOS` (omissão 4), `PH2D_SONDA_QUADROS` (omissão 24).

use super::sonda_bordas_longe::por_nome;
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fnv(h: &mut u64, b: &[u8]) {
    for &x in b {
        *h ^= u64::from(x);
        *h = h.wrapping_mul(0x0100_0000_01b3);
    }
}

fn env(n: &str, d: usize) -> usize {
    std::env::var(n)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(d)
}

#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_agua_oleo_4k() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let (w, h) = (3840usize, 2160usize);
    let (tracos, quadros) = (env("PH2D_SONDA_TRACOS", 4), env("PH2D_SONDA_QUADROS", 24));
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_wet_oleo_real_ligado(true);
    // `PH2D_SONDA_OLEO=1`: um traço de Dabber (óleo) por baixo, que a água cruza — a óptica do óleo trabalha.
    if env("PH2D_SONDA_OLEO", 0) == 1 {
        t.preset_aplicar(por_nome(&lib, "Dabber")).expect("Dabber");
        t.sonda_raio(160.0);
        t.on_canvas_pointer(cp([1800.0, 300.0], PointerPhase::Down));
        for s in 1..=40 {
            t.on_canvas_pointer(cp([1800.0, 300.0 + 40.0 * s as f32], PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(cp([1800.0, 1900.0], PointerPhase::Up));
        for _ in 0..30 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    t.preset_aplicar(por_nome(&lib, "Wet Wash"))
        .expect("preset");
    t.sonda_raio(240.0);
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\nAGUA4K cab rayon={} carga={}",
        rayon::current_num_threads(),
        carga().trim()
    );
    let mut total = 0f64;
    for lane in 0..tracos {
        let y = 500.0 + 300.0 * lane as f32;
        t.on_canvas_pointer(cp([400.0, y], PointerPhase::Down));
        let a = Instant::now();
        for s in 1..=40 {
            t.on_canvas_pointer(cp([400.0 + 70.0 * s as f32, y], PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        let ms_traco = a.elapsed().as_secs_f64() * 1e3 / 40.0;
        t.on_canvas_pointer(cp([3200.0, y], PointerPhase::Up));
        let mut ms = Vec::with_capacity(quadros);
        for _ in 0..quadros {
            let b = Instant::now();
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
            ms.push(b.elapsed().as_secs_f64() * 1e3);
        }
        total += ms.iter().sum::<f64>();
        ms.sort_by(f64::total_cmp);
        println!(
            "AGUA4K traco={} ms/quadro_traco={ms_traco:.1} espera min={:.1} med={:.1} max={:.1} carga={}",
            lane + 1,
            ms[0],
            ms[ms.len() / 2],
            ms[ms.len() - 1],
            carga().split(' ').next().unwrap_or("")
        );
    }
    t.invalidate_composite();
    let (px, _, _) = t.take_preview_arc().expect("a imagem");
    let mut hi = 0xcbf2_9ce4_8422_2325u64;
    fnv(&mut hi, &px);
    t.wet_bring_home();
    let mut hg = 0xcbf2_9ce4_8422_2325u64;
    let e = &t.paint.wetpaint.session.as_ref().expect("sessao").engine;
    for l in &e.layers {
        let g = &l.grid;
        for plano in [&g.film, &g.sett, &g.susp] {
            for v in plano.iter() {
                fnv(&mut hg, &v.to_bits().to_le_bytes());
            }
        }
    }
    println!("AGUA4K fim espera_total_ms={total:.0} hash_imagem={hi:016x} hash_grade={hg:016x}");
}
