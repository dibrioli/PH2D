//! **Sonda (ignorada): o custo de um QUADRO a tocar uma macro do Wet Paint** contra o de pintá-la, a
//! 1024² e 2048². A reprodução faz na thread dela os estágios que o worker fazia em paralelo (a fita),
//! e compõe no mesmo tique em que o original compôs — esta sonda diz se isso cabe num quadro.
//!
//! Corre: `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it --
//! --ignored --nocapture --test-threads=1 sonda_macro_custo`.

use super::macro_comum::{Gravador, cp};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::time::{Duration, Instant};

const QUADRO: Duration = Duration::from_micros(16_666);

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

/// Grava 3 traços com esperas reais; devolve a macro e os tempos de quadro do ORIGINAL (ms).
fn grava(n: u32) -> (ph2d_macro::MacroDoc, Vec<f32>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(30.0 * n as f32 / 1024.0);
    let mut g = Gravador::arma(&mut t);
    let mut tempos = Vec::new();
    let s = n as f32;
    let mut quadro = |t: &mut PainterTool, g: &mut Gravador, ev: &[_]| {
        let a = Instant::now();
        for &e in ev {
            t.on_canvas_pointer(e);
        }
        let _ = t.take_preview_arc();
        t.on_tick(1000.0 / 60.0);
        tempos.push(a.elapsed().as_secs_f32() * 1e3);
        g.quadro(t);
        if let Some(r) = QUADRO.checked_sub(a.elapsed()) {
            std::thread::sleep(r);
        }
    };
    for (k, y) in [0.3f32, 0.5, 0.7].into_iter().enumerate() {
        t.set_brush_color_srgb8([200, 40 + 60 * k as u8, 30]);
        let p = |i: u32| [s * (0.15 + 0.7 * i as f32 / 30.0), s * y];
        quadro(&mut t, &mut g, &[cp(p(0), PointerPhase::Down)]);
        for q in 0..10 {
            let ev: Vec<_> = (1..=3)
                .map(|m| cp(p(q * 3 + m), PointerPhase::Move))
                .collect();
            quadro(&mut t, &mut g, &ev);
        }
        quadro(&mut t, &mut g, &[cp(p(30), PointerPhase::Up)]);
        for _ in 0..30 {
            quadro(&mut t, &mut g, &[]);
        }
    }
    (g.para(&mut t), tempos)
}

/// Toca quadro a quadro (os eventos de um mesmo tempo são um quadro); devolve os tempos (ms).
fn toca_por_quadro(doc: &ph2d_macro::MacroDoc) -> Vec<f32> {
    let tr = &doc.tracks[0];
    let mut toc =
        TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
            .expect("toca");
    let mut tempos = Vec::new();
    let mut i = 0;
    while i < tr.events.len() {
        let t0 = tr.events[i].t;
        let a = Instant::now();
        while i < tr.events.len() && tr.events[i].t == t0 {
            toc.aplica(&tr.events[i].payload).expect("não diverge");
            i += 1;
        }
        tempos.push(a.elapsed().as_secs_f32() * 1e3);
    }
    tempos
}

fn resumo(v: &mut [f32]) -> (f32, f32, f32) {
    v.sort_by(f32::total_cmp);
    (v[0], v[v.len() / 2], v[v.len() - 1])
}

#[test]
#[ignore = "medição — --profile smoke --ignored --nocapture --test-threads=1"]
fn sonda_macro_custo() {
    println!("[macro custo] load antes {}", carga().trim());
    for n in [1024u32, 2048] {
        let (doc, mut orig) = grava(n);
        // Três reproduções da MESMA macro, e de cada quadro o MÍNIMO das três (o ruído da máquina sai).
        let r: Vec<Vec<f32>> = (0..3).map(|_| toca_por_quadro(&doc)).collect();
        let mut minimo: Vec<f32> = (0..r[0].len())
            .map(|k| r.iter().map(|v| v[k]).fold(f32::INFINITY, f32::min))
            .collect();
        let (om, od, ox) = resumo(&mut orig);
        let (tm, td, tx) = resumo(&mut minimo);
        let acima = minimo.iter().filter(|&&x| x > 16.666).count();
        println!(
            "[macro custo] {n}²: ORIGINAL quadro min {om:.2} med {od:.2} pior {ox:.2} ms · TOCAR quadro \
             min {tm:.2} med {td:.2} pior {tx:.2} ms · {acima}/{} quadros acima de 16,7 ms",
            minimo.len()
        );
    }
    println!("[macro custo] load depois {}", carga().trim());
}

/// **A memória de UM instantâneo** (W7, o aberto da onda 1): o Splash em Wet Paint tocado até ao fim,
/// e o que um instantâneo do tocador declara ao razão da RAM (`bytes_retidos`), a 1024² e 2048².
#[test]
#[ignore = "medição — --profile smoke --ignored --nocapture --test-threads=1"]
fn sonda_a_memoria_de_um_instantaneo() {
    for lado in [1024u32, 2048] {
        let (bytes, _) =
            ph2d_tool_painter::macro_track::roteiro::grava_o_splash(lado, 0).expect("grava");
        let doc = ph2d_macro::codec::read(&bytes).expect("lê");
        let tr = &doc.tracks[0];
        let mut toc =
            TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
                .expect("tocador");
        let mut maximo = 0usize;
        for (i, e) in tr.events.iter().enumerate() {
            let _ = toc.aplica(&e.payload);
            if i % 64 == 0 {
                maximo = maximo.max(toc.bytes_retidos());
            }
        }
        maximo = maximo.max(toc.bytes_retidos());
        println!(
            "[instantâneo] {lado}²: até {:.1} MB por instantâneo ({} eventos)",
            maximo as f64 / (1 << 20) as f64,
            tr.events.len()
        );
    }
}
