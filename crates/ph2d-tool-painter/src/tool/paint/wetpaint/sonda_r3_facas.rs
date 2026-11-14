//! **Sonda R3 — o preço da composição nas facas do oráculo** (4K, as 20 pinceladas de `t_facas.jsonl`, a drenagem
//! a cada 8 eventos): quantas composições, quantas células o motor sujou e quantas se renderizaram, e os ms do
//! campo, do pigmento e da sombra no papel. Mede o build de TESTE (o journal do undo vivo): serve para PARTIR,
//! não para o número do produto (`examples/mede_o_dab.rs`).
//! ```text
//! bash scripts/cargo-test-narrow.sh ph2d-tool-painter sonda_r3_facas -- --ignored --nocapture
//! ```

use super::composite_contagem as contagem;
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const FACAS: [&str; 6] = [
    "Knife Smooth 3",
    "Knife Textured",
    "Knife Oily",
    "Knife Dry",
    "Knife Grainy",
    "Knife Stain",
];

#[test]
#[ignore = "sonda R3: o preço da composição nas facas 4K"]
fn sonda_r3_facas() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let (w, h) = (3840u32, 2160u32);
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes/WetPainter/Oil/Knife");
    let presets: Vec<_> = FACAS
        .iter()
        .map(|n| {
            let b = std::fs::read(raiz.join(format!("{n}.ph2d-brush.png"))).expect("faca");
            ph2d_brush_presets::ler(&b).expect("preset")
        })
        .collect();
    // Os contadores são do fio (thread_local): a ótica fica no fio da sonda.
    super::offthread::otica::forca_no_teste(Some(false));
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_compor_por_quadro(true);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_wet_oleo_real_ligado(true);
    let _ = contagem::take();
    let _ = contagem::take_campo();
    let _ = contagem::take_tempos();
    let t0 = std::time::Instant::now();
    let mut quadros = 0;
    let mut acum = (0u64, 0u64, 0u64);
    for k in 0..20 {
        assert!(t.wet_preset_aplicar(&presets[k % FACAS.len()]).is_empty());
        t.sonda_raio(45.0);
        t.set_brush_color_srgb8([20, 150, 230]);
        let (x0, y) = (
            if k % 10 < 5 { 70.0 } else { 650.0 },
            90.0 + 150.0 * (k % 5) as f32,
        );
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        let sujo = |t: &PainterTool| {
            t.paint
                .wetpaint
                .session
                .as_ref()
                .map(|s| match s.engine.dirty {
                    ph2d_wet_paint::painter::Dirty::Rect { x0, y0, x1, y1 } => {
                        format!("{}×{}", x1 - x0 + 1, y1 - y0 + 1)
                    }
                    d => format!("{d:?}"),
                })
        };
        if k < 6 {
            let b = &t.paint.brush;
            println!(
                "  faca {k} depois do preset: {:?} · papel activo {} · dente {} · chave {:?}",
                sujo(&t),
                b.paper.is_active(),
                b.paper_depth,
                t.paint
                    .wetpaint
                    .session
                    .as_ref()
                    .map(|s| s.paper_key.is_some())
            );
        }
        t.on_canvas_pointer(p(x0, PointerPhase::Down));
        if k < 3 {
            println!("  faca {k} depois do Down: {:?}", sujo(&t));
        }
        for i in 1..=150 {
            t.on_canvas_pointer(p(x0 + 3.0 * i as f32, PointerPhase::Move));
            // O sujo de UM Move: o primeiro depois da drenagem.
            if i % 8 == 1
                && let Some(s) = t.paint.wetpaint.session.as_ref()
                && let ph2d_wet_paint::painter::Dirty::Rect { x0, y0, x1, y1 } = s.engine.dirty
                && k < 2
                && i < 40
            {
                println!(
                    "  faca {k} move {i}: sujo {}×{} em ({x0},{y0})",
                    x1 - x0 + 1,
                    y1 - y0 + 1
                );
            }
            if i % 8 == 0 {
                let _ = t.take_preview_arc();
                quadros += 1;
            }
        }
        t.on_canvas_pointer(p(x0 + 450.0, PointerPhase::Up));
        if k < 3 {
            println!("  faca {k} depois do Up: {:?}", sujo(&t));
        }
        let (n0, m0, r0) = contagem::take();
        let _ = t.take_preview_arc();
        let (n1, m1, r1) = contagem::take();
        if k < 3 {
            println!(
                "  faca {k}: traço {n0} composições, {m0} células · drenagem do Up {n1}: {m1}/{r1}"
            );
        }
        acum.0 += n0 + n1;
        acum.1 += m0 + m1;
        acum.2 += r0 + r1;
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let (n, motor, render) = acum;
    super::offthread::otica::forca_no_teste(None);
    assert!(
        n > 0 && render > 0,
        "régua morta: {n} composições, {render} células renderizadas"
    );
    let campo = contagem::take_campo();
    let (pig, som) = contagem::take_tempos();
    println!(
        "sonda_r3_facas: {ms:.0} ms · {quadros} quadros · {n} composições · células/composição: motor {} render {} · campo {campo:.0} ms · pigmento {pig:.0} ms · sombra {som:.0} ms · loadavg {}",
        motor / n.max(1),
        render / n.max(1),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// B44.5 (GT-24, S-18): a contagem que a sonda lê VÊ a composição — a sonda imprimia `render` constante 0 (descartava
/// o 3.º campo). Uma tela pequena, a ótica no fio: composições, células do motor e renderizadas > 0.
#[test]
fn a_contagem_da_sonda_r3_ve_o_render() {
    super::offthread::otica::forca_no_teste(Some(false));
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
    t.set_compor_por_quadro(true);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_wet_oleo_real_ligado(true);
    t.sonda_raio(12.0);
    let _ = contagem::take();
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, 128.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(40.0, PointerPhase::Down));
    for i in 1..=40 {
        t.on_canvas_pointer(p(40.0 + 4.0 * i as f32, PointerPhase::Move));
        if i % 8 == 0 {
            let _ = t.take_preview_arc();
        }
    }
    t.on_canvas_pointer(p(200.0, PointerPhase::Up));
    let _ = t.take_preview_arc();
    let (n, motor, render) = contagem::take();
    super::offthread::otica::forca_no_teste(None);
    assert!(
        n > 0 && motor > 0 && render > 0,
        "{n} composições · motor {motor} · render {render}"
    );
}
