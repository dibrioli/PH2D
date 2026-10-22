//! ⭐ **O dual brush na PLACA pinta o que pinta na CPU** (L6, `docs/Painter/Pinceis/03_MOTORES.md` §4): a 1.ª ponta de
//! falloff puro sobe ao dispositivo DENTRO do plano do traço (o lote é o mesmo, só a tela é o plano), a 2.ª ponta e o
//! recorte são da CPU — o traço com a ponte ligada e desligada, pela porta do ponteiro, tem de dar a mesma tela (a
//! barra do `the_device_paints_what_the_cpu_paints`), e a ponte tem de ter sido CHAMADA.
//!
//! ⚠️ A tradução `DeviceDab → GpuDab` é a do `measure_product_stamp` (a cópia deliberada; o arch-gate
//! `the_shell_installs_the_stamp_bridge_at_the_document_bind` pina a do produto).

use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_paint_gpu::{GpuDab, Region, StampPass};
use ph2d_tool_painter::{DeviceStamp, DeviceStampJob, LinhaDaPonta, PaintMedia, PainterTool};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const W: usize = 640;
const H: usize = 400;

fn ponte(gpu: &ph2d_gpu::GpuContext, chamadas: &Arc<AtomicUsize>) -> DeviceStamp {
    let pass = Arc::new(StampPass::new(gpu));
    let chamadas = Arc::clone(chamadas);
    Box::new(move |job: &DeviceStampJob<'_>| {
        chamadas.fetch_add(1, Ordering::Relaxed);
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
                c0: [0.0; 8],
                c1: [0.0; 8],
            })
            .collect();
        pass.run(
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
        )
    })
}

/// Um S largo em saltos de 40 px a passo 0,02 (o lote denso que o dispositivo quer) com o pincel de raio 40; `dual` arma a 2.ª
/// ponta (um disco espalhado, Multiply).
fn pinta(gpu: Option<(&ph2d_gpu::GpuContext, &Arc<AtomicUsize>)>, dual: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(80.0);
    t.set_brush_spacing(0.02); // o lote denso: redundância ≥ 4 (`stamp_device::MIN_REDUNDANCY`)
    t.set_brush_color_srgb8([20, 30, 160]);
    if dual {
        t.set_ponta(LinhaDaPonta::DualMode, 1.0);
        t.set_ponta(LinhaDaPonta::DualSize, 0.3);
        t.set_ponta(LinhaDaPonta::DualSpacing, 0.5);
        t.set_ponta(LinhaDaPonta::DualScatter, 0.7);
        t.set_ponta(LinhaDaPonta::DualCount, 3.0);
    }
    if let Some((g, c)) = gpu {
        t.set_device_stamp(Some(ponte(g, c)));
    }
    let cp = |k: usize, phase| {
        let u = k as f32 / 14.0;
        CanvasPointer {
            pos: [
                60.0 + 520.0 * u,
                200.0 + 110.0 * (u * std::f32::consts::TAU).sin(),
            ],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    };
    t.on_canvas_pointer(cp(0, PointerPhase::Down));
    for k in 1..=14 {
        t.on_canvas_pointer(cp(k, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(14, PointerPhase::Up));
    t.run_full().0
}

fn pior_e_quantos(a: &[u8], b: &[u8]) -> (u8, usize) {
    a.iter()
        .zip(b)
        .filter(|(x, y)| x != y)
        .fold((0, 0), |(p, n), (x, y)| (p.max(x.abs_diff(*y)), n + 1))
}

#[test]
#[ignore = "precisa de adapter — rode com `-- --ignored` na máquina com GPU"]
fn a_placa_pinta_o_dual_como_a_cpu() {
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        eprintln!("sem adapter: skip");
        return;
    };
    for dual in [false, true] {
        let chamadas = Arc::new(AtomicUsize::new(0));
        let cpu = pinta(None, dual);
        let placa = pinta(Some((&gpu, &chamadas)), dual);
        let n = chamadas.load(Ordering::Relaxed);
        let (pior, quantos) = pior_e_quantos(&cpu, &placa);
        eprintln!(
            "dual {dual}: {n} lotes na placa · pior {pior} · {quantos} de {} bytes",
            cpu.len()
        );
        assert!(
            n > 0,
            "dual {dual}: a ponte nem foi chamada — o gate estaria verde sobre a rota desligada"
        );
        assert!(cpu.iter().any(|&b| b != 255), "a fixture não pintou");
        assert!(pior <= 2, "dual {dual}: pior delta {pior}");
        assert!(
            quantos * 400 <= cpu.len(),
            "dual {dual}: {quantos} bytes diferem (> 0,25 %)"
        );
    }
    assert!(
        pinta(None, true) != pinta(None, false),
        "a 2.ª ponta não mudou a tela"
    );
}
