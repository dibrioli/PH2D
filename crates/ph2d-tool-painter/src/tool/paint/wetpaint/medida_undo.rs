//! **Medição (ignorada, W1g; dono 10/10: «confira se Undo não reduz performance»)** — o Ctrl+Z de um
//! traço molhado e o 1.º quadro do traço seguinte (onde a sessão de água renasce), a 4096², nas quatro
//! combinações das opções: fábrica · Heavy Edge 1 · Live Water · as duas. Uma ferramenta por variante,
//! no MESMO processo, 7 rodadas de ordem rodada, o resumo é o MÍNIMO (a mediana ao lado), a água no
//! worker como no produto (MEDIR_VELOCIDADE.md). Medição, nunca barra.
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::{Duration, Instant};

const LADO: u32 = 4096;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn quadro(t: &mut PainterTool) {
    let a = Instant::now();
    t.on_tick(1000.0 / 60.0);
    if let Some(r) = Duration::from_micros(16_666).checked_sub(a.elapsed()) {
        std::thread::sleep(r);
    }
}

fn ferramenta(borda: f64, viva: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(60.0);
    t.set_wet_borda_que_pesa(borda);
    t.set_wet_agua_viva(viva);
    t
}

/// Um traço de 40 movimentos de 40 px (um quadro cada) e meio segundo de água depois do Up.
fn traco(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([300.0, y], PointerPhase::Down));
    for k in 1..=40u16 {
        t.on_canvas_pointer(cp([300.0 + 40.0 * f32::from(k), y], PointerPhase::Move));
        let _ = t.take_preview_arc();
        quadro(t);
    }
    t.on_canvas_pointer(cp([1900.0, y], PointerPhase::Up));
    for _ in 0..30 {
        quadro(t);
    }
}

/// Uma rodada: o traço, o Ctrl+Z (cronometrado) e o 1.º quadro do traço seguinte (cronometrado).
fn rodada(t: &mut PainterTool, y: f32) -> (f32, f32) {
    traco(t, y);
    let a = Instant::now();
    assert!(t.undo_last(), "havia um traço para desfazer");
    let undo = a.elapsed().as_secs_f32() * 1e3;
    let a = Instant::now();
    t.on_canvas_pointer(cp([300.0, y + 120.0], PointerPhase::Down));
    for k in 1..=3u16 {
        t.on_canvas_pointer(cp(
            [300.0 + 40.0 * f32::from(k), y + 120.0],
            PointerPhase::Move,
        ));
        let _ = t.take_preview_arc();
    }
    t.on_tick(1000.0 / 60.0);
    let primeiro = a.elapsed().as_secs_f32() * 1e3;
    t.on_canvas_pointer(cp([460.0, y + 120.0], PointerPhase::Up));
    assert!(t.undo_last(), "o traço curto também se desfaz");
    (undo, primeiro)
}

#[test]
#[ignore = "medicao — rode com --profile smoke --ignored --nocapture --test-threads=1"]
fn medida_o_undo_com_as_opcoes() {
    let nomes = ["fábrica", "Heavy Edge 1", "Live Water", "as duas"];
    let mut ts = [
        ferramenta(0.0, false),
        ferramenta(1.0, false),
        ferramenta(0.0, true),
        ferramenta(1.0, true),
    ];
    for t in &mut ts {
        let _ = rodada(t, 800.0); // aquece fora da régua
    }
    let mut res: [Vec<(f32, f32)>; 4] = Default::default();
    for r in 0..7usize {
        for i in 0..4 {
            let k = (r + i) % 4;
            res[k].push(rodada(&mut ts[k], 1200.0 + 300.0 * r as f32));
        }
    }
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    for (k, nome) in nomes.iter().enumerate() {
        let mut u: Vec<f32> = res[k].iter().map(|v| v.0).collect();
        let mut p: Vec<f32> = res[k].iter().map(|v| v.1).collect();
        u.sort_by(f32::total_cmp);
        p.sort_by(f32::total_cmp);
        println!(
            "[undo] {nome:>12}: Ctrl+Z mín {:.1} ms (mediana {:.1}) · 1.º quadro do traço seguinte mín {:.1} ms (mediana {:.1}) · undo guardado {:.1} MB",
            u[0],
            u[u.len() / 2],
            p[0],
            p[p.len() / 2],
            ts[k].undo_retained_bytes() as f64 / 1e6
        );
    }
    println!("[undo] carga: {}", carga.trim());
}
