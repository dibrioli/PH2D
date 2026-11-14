use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};
use ph2d_wet_paint::tuning::Knob;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32, x0: f32) {
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    for k in 1..=60 {
        t.on_canvas_pointer(cp(
            [x0 + 30.0 * k as f32, y + 200.0 * (k as f32 / 9.0).sin()],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([x0 + 1800.0, y], PointerPhase::Up));
}

fn tela(n: u32, prepara: impl FnOnce(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    let b = BrushSpec {
        radius_px: 60.0,
        color: [0.1, 0.4, 0.9],
        falloff: Falloff::Smooth,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_tool_mode("wetpaint");
    prepara(&mut t);
    t
}

#[test]
#[ignore = "medida: o custo do Smooth Edges; corre-se à mão"]
fn medida_do_smooth_edges() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    const N: u32 = 4096;
    let y = (N / 2) as f32;
    let mut cenas: Vec<(&str, PainterTool)> = vec![
        (
            "0, sem o campo",
            tela(N, |t| {
                t.set_wet_smooth_edges(0.0);
                traco(t, y, 1000.0);
            }),
        ),
        (
            "0, com o campo",
            tela(N, |t| {
                t.paint.wetpaint.knobs.set(Knob::PaperVisibility, 0.4);
                t.set_wet_smooth_edges(0.0);
                traco(t, y, 1000.0);
            }),
        ),
        (
            "0,12",
            tela(N, |t| {
                t.set_wet_smooth_edges(0.12);
                traco(t, y, 1000.0);
            }),
        ),
        (
            "0,5 sob 0",
            tela(N, |t| {
                t.set_wet_smooth_edges(0.5);
                traco(t, y, 1000.0);
                t.set_wet_smooth_edges(0.0);
                traco(t, y + 40.0, 1000.0);
            }),
        ),
    ];
    let v = cenas.len();
    let mut minimo = vec![[f64::MAX; 2]; v + 1];
    let rects = [None, Some((1700, 1800, 2000, 2100))];
    let mut degraus = 0usize;
    // A variante `v` (a última linha): a lei de ANTES a 0,12, um só `Suave` escalar sobre a tinta da cena «0, sem o
    // campo» (`teto = 0`: sem o plano nem os degraus), o caminho inteiro de antes.
    for rodada in 0..7 {
        for k in 0..=v {
            let variante = (k + rodada) % (v + 1);
            super::raio_escalar::set((variante == v).then_some(0.72));
            let (_, t) = &mut cenas[if variante == v { 0 } else { variante }];
            for (q, rect) in rects.into_iter().enumerate() {
                let sess = t.paint.wetpaint.session.as_mut().expect("sessão viva");
                sess.bring_home();
                match rect {
                    None => sess.engine.mark_dirty_full(),
                    Some((x0, y0, x1, y1)) => sess.engine.merge_dirty(x0, y0, x1, y1),
                }
                let t0 = std::time::Instant::now();
                super::super::super::wetpaint::composite_for_measure(t);
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                let m = &mut minimo[variante][q];
                *m = m.min(ms);
            }
        }
    }
    super::raio_escalar::set(None);
    for (_, t) in &cenas {
        let sess = t.paint.wetpaint.session.as_ref().expect("sessão viva");
        let mut d = [false; 256];
        sess.planos
            .suave
            .iter()
            .for_each(|&x| d[usize::from(x)] = true);
        degraus = degraus.max(d[1..].iter().filter(|&&x| x).count());
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    for (k, (nome, _)) in cenas.iter().enumerate() {
        println!(
            "[smooth edges] {nome}: tela 4096² {:.2} ms · retângulo 300² {:.2} ms",
            minimo[k][0], minimo[k][1]
        );
    }
    println!(
        "[smooth edges] ANTES a 0,12 (raio escalar): tela 4096² {:.2} ms · retângulo 300² {:.2} ms · até {degraus} degraus · load {}",
        minimo[v][0],
        minimo[v][1],
        load.split_whitespace().next().unwrap_or("?")
    );
}
