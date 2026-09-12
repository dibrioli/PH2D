//! **A ponte do carimbo para o dispositivo** — a metade do lado do shell.
//!
//! O tool publica um lote de dado simples ([`ph2d_tool_painter::DeviceStampJob`]: uma região de
//! bytes, uma tabela e uma lista de discos) e **não tem device**; quem tem é o shell. É o molde do
//! `denoise_ml_with_progress` do editor de áudio, e a contenção corta nos dois sentidos: `wgpu`
//! nunca entra na `ph2d-tool-painter`, e a `ph2d-paint-gpu` não alcança o `falloff_weight` — ela
//! não CONSEGUE ter opinião sobre a lei que carimba.
//!
//! ⚠️ **Este arquivo é a ÚNICA tradução entre os dois vocabulários**, e ela é mecânica: o
//! [`DeviceDab`] do tool é dado puro, o [`GpuDab`] é o layout que o WGSL alinha (com o rabo de
//! preenchimento que só um buffer de GPU precisa). Manter os dois separados é o que impede o tool
//! de aprender o que WGSL alinha.

use ph2d_paint_gpu::{GpuDab, Region, StampPass};
use ph2d_render::SpriteRenderer;
use ph2d_tool_painter::{DeviceStampJob, PainterTool};
use std::sync::Arc;

/// **Instala a ponte no bind do documento** — o mesmo vão humano em que o `prewarm` compila os
/// shaders do preview, e pela mesma razão: construir o [`StampPass`] COMPILA um shader, e esse custo
/// não pode cair no primeiro traço.
///
/// ⚠️ **Idempotente por construção** (`has_device_stamp`): o bind acontece a cada troca de sprite, e
/// recriar o passe por bind pagaria a compilação de novo. O passe não guarda nada do documento — ele
/// recebe a região a cada lote —, então o mesmo vive a sessão inteira.
pub(crate) fn install(painter: &mut PainterTool, renderer: &SpriteRenderer) {
    if painter.has_device_stamp() {
        return;
    }
    let pass = Arc::new(StampPass::new(renderer.gpu()));
    painter.set_device_stamp(Some(Box::new(move |job: &DeviceStampJob<'_>| {
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
    })));
}
