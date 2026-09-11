//! **A wave chega ao PRODUTO?** — a sonda que fecha a S3, pela porta do ARTISTA.
//!
//! A `measure_boundary` mede o passe isolado (2,80 ms contra os 18,30 da CPU). Isso é a FRONTEIRA,
//! não o produto: entre um e outro estão a extração da região, a tabela, a resolução dos discos e o
//! predicado. Esta sonda dirige o `on_canvas_pointer` — a mesma porta que o mouse do artista usa —
//! com a ponte ligada e desligada, **costas-com-costas no mesmo processo**.
//!
//! ⚠️ **Costas-com-costas, e não duas corridas:** a máquina desta linha é compartilhada, e o MESMO
//! trabalho já variou 2× numa sessão sem uma linha mudar (doc 28 §5.46). Uma razão dentro da corrida
//! torna a carga um fator comum.
//!
//! ⚠️ **A tradução `DeviceDab → GpuDab` abaixo é uma SEGUNDA cópia da que o shell instala**, e ela é
//! deliberada e pequena: o shell não tem alvo de lib, então nenhum teste alcança o
//! `painter_stamp_device::install`. Quem impede as duas de divergirem é o arch-gate
//! `the_shell_installs_the_stamp_bridge_at_the_document_bind`, que pina a tradução do produto num
//! arquivo só. Se esta cópia envelhecer, o número que ela imprime deixa de descrever o produto —
//! releia-a antes de acreditar nele.

use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_paint_gpu::{GpuDab, Region, StampPass};
use ph2d_tool_painter::{DeviceStamp, DeviceStampJob, PainterTool};
use std::sync::Arc;

const SIDE: usize = 4096;

type Ledger = Arc<std::sync::Mutex<Vec<(f64, usize, usize)>>>;

fn bridge(gpu: &ph2d_gpu::GpuContext, ledger: &Ledger) -> DeviceStamp {
    let pass = Arc::new(StampPass::new(gpu));
    let ledger = Arc::clone(ledger);
    Box::new(move |job: &DeviceStampJob<'_>| {
        let t0 = std::time::Instant::now();
        let dabs: Vec<GpuDab> = job
            .dabs
            .iter()
            .map(|d| GpuDab {
                center: d.center,
                radius: d.radius,
                coverage: d.coverage,
                color: d.color,
                _pad0: 0.0,
                m0: d.m0,
                m1: d.m1,
                _pad1: [0.0; 4],
            })
            .collect();
        let out = pass.run(
            job.base,
            Region {
                x: job.x,
                y: job.y,
                w: job.w,
                h: job.h,
            },
            job.lut,
            &dabs,
            job.preserve_alpha,
        );
        ledger.lock().expect("mutex").push((
            t0.elapsed().as_secs_f64() * 1000.0,
            (job.w as usize) * (job.h as usize),
            dabs.len(),
        ));
        out
    })
}

fn cpx(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O tool na armação do smoke: Digital de fábrica, pincel grande, elipse viva sobre 4096².
fn tool() -> PainterTool {
    let mut t = PainterTool::default();
    #[allow(clippy::cast_possible_truncation)]
    t.set_source(vec![255u8; SIDE * SIDE * 4], SIDE as u32, SIDE as u32);
    t.set_brush_size_px(155.0); // ⚠️ este setter é o RAIO: o log do artista mede ~155
    t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::Ellipse as u8);
    t
}

#[test]
#[ignore = "mede o PRODUTO com a ponte: precisa de adapter; `-- --ignored --nocapture --test-threads=1`"]
fn the_device_road_is_faster_through_the_artists_door() {
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        eprintln!("sem adapter: skip");
        return;
    };
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    // ⚠️ **O eixo é o RAIO DA FIGURA, e é ele que decide** — não o raio do pincel. O trabalho de um
    // lote é `Σ pegadas` (as visitas) e a fronteira é a ÁREA DA REGIÃO; a razão entre os dois é a
    // REDUNDÂNCIA, e é ela que diz se subir a região se paga. Uma figura pequena tem muita
    // redundância (os dabs se empilham); uma enorme espalha os mesmos discos por uma região grande.
    let run = |half: f32, with_device: bool| -> (f64, f64, f64, u64, u32) {
        let mut t = tool();
        let ledger: Ledger = Arc::new(std::sync::Mutex::new(Vec::new()));
        if with_device {
            t.set_device_stamp(Some(bridge(&gpu, &ledger)));
        }
        t.on_canvas_pointer(cpx([2048.0, 2048.0], PointerPhase::Down));
        let mut mv = |i: usize| {
            #[allow(clippy::cast_precision_loss)]
            let d = (i % 5) as f32;
            t.on_canvas_pointer(cpx([2048.0 + half + d, 2048.0 + half], PointerPhase::Move));
        };
        mv(0);
        let _ = ph2d_tool_painter::band_diag::take();
        for i in 1..8 {
            mv(i);
        }
        let d = ph2d_tool_painter::band_diag::take();
        let per = |us: u64| us as f64 / f64::from(d.deliveries.max(1)) / 1000.0;
        let l = ledger.lock().expect("mutex");
        let bridge_ms = if l.is_empty() {
            0.0
        } else {
            med(l.iter().skip(1).map(|(m, _, _)| *m).collect())
        };
        #[allow(clippy::cast_precision_loss)]
        let region_mpx = l.last().map_or(0.0, |(_, px, _)| *px as f64 / 1.0e6);
        let _ = t
            .take_preview_arc()
            .expect("a fixture não publicou preview");
        (
            per(d.stamp_us),
            bridge_ms,
            region_mpx,
            d.visits,
            d.deliveries,
        )
    };
    eprintln!(
        "[produto] re-stamp de UMA figura a {SIDE}x{SIDE}, pela porta do artista (pincel r=155)"
    );
    eprintln!(
        "[produto] {:>7} {:>9} {:>9} {:>7} | {:>9} {:>9} {:>7}",
        "figura", "regiao", "visitas", "redund", "CPU", "DEVICE", "ganho"
    );
    for half in [300.0f32, 600.0, 1200.0, 1900.0] {
        // A CPU primeiro, para o balde de visitas (a rota do device não passa pelo `note`).
        let (cpu_ms, _, _, visits, n) = run(half, false);
        let (dev_ms, bridge_ms, region_mpx, _, _) = run(half, true);
        #[allow(clippy::cast_precision_loss)]
        let mvis = visits as f64 / f64::from(n.max(1)) / 1.0e6;
        // ⚠️ A região só existe quando a ponte foi CHAMADA. Abaixo do piso o lote nem é publicado,
        // e imprimir uma redundância derivada de zero seria um número inventado.
        if region_mpx <= 0.0 {
            eprintln!(
                "[produto] {half:>7.0}   (abaixo do piso: fica na CPU) {mvis:>7.2} M visitas | \
{cpu_ms:>6.2} ms {dev_ms:>6.2} ms {:>6.2}x",
                cpu_ms / dev_ms.max(1e-9)
            );
            continue;
        }
        eprintln!(
            "[produto] {half:>7.0} {region_mpx:>7.2} M {mvis:>7.2} M {:>7.1}x | {cpu_ms:>6.2} ms \
{dev_ms:>6.2} ms ({bridge_ms:.1} na ponte) {:>6.2}x",
            mvis / region_mpx.max(1e-9),
            cpu_ms / dev_ms.max(1e-9)
        );
    }
}
