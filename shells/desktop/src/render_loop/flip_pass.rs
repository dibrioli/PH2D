//! ADR-0113 W1 T1.4 — o passe que põe o traço do Flip na tela.
//!
//! Roda no present phase, logo APÓS o passe de sprites e ANTES do tonemap: o
//! traço é artwork de canvas, então compõe no mesmo `game_rt` (HDR) com a MESMA
//! câmera, e é tonemapeado junto. Amostra a cena Flip no playhead, achata as
//! camadas ativas de cada objeto num só buffer (o blend por-camada é o T1.7) e
//! desenha com `LoadOp::Load` (preserva os sprites por baixo).

use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipDrawing};
use ph2d_flip_render::{CameraRaw, FlipRenderer, pack_drawings};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, GameRt};

/// Desenha o traço do Flip amostrado em `playhead` no `game_rt`. No-op se não há
/// desenho ativo (cena vazia = o caso default sem `PH2D_FLIP_DEMO`).
pub(crate) fn render(
    flip: &FlipDoc,
    flip_render: &mut FlipRenderer,
    playhead: &Playhead,
    game_rt: &GameRt,
    camera: &Camera2d,
    window: WindowSize,
    gpu: &GpuContext,
) {
    let active = collect_active(flip, playhead);
    if active.is_empty() {
        return;
    }
    let data = pack_drawings(&active);
    if data.is_empty() {
        return;
    }
    let cam = camera_raw(camera, window);
    flip_render.upload(&gpu.device, &gpu.queue, &cam, &data);
    flip_render.ensure_depth(&gpu.device, (window.width, window.height));

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-flip pass encoder"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-flip pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: game_rt.view(),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // preserva sprites + fundo
                    store: wgpu::StoreOp::Store,
                },
            })],
            // Depth próprio do Flip (ordem 2D), limpo a 0 → GREATER deixa tudo
            // passar; o descarte é ok (só vale dentro do passe).
            depth_stencil_attachment: flip_render.depth_view().map(|v| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: v,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        flip_render.draw(&mut pass);
    }
    gpu.queue.submit([encoder.finish()]);
}

/// Os desenhos ativos AGORA, na ordem de composição (objeto por objeto; dentro de
/// cada objeto, camadas de baixo p/ cima; só as visíveis). Cada objeto amostra
/// pelo SEU FPS.
fn collect_active<'a>(flip: &'a FlipDoc, playhead: &Playhead) -> Vec<&'a FlipDrawing> {
    let mut out = Vec::new();
    for obj in flip.objects() {
        let frame = obj.frame_at(playhead);
        for layer in obj.layers() {
            if !layer.visible {
                continue;
            }
            if let Some(did) = layer.drawing_at(frame)
                && let Some(drawing) = obj.drawing(did)
            {
                out.push(drawing);
            }
        }
    }
    out
}

/// Converte a `Camera2d` (mundo→clip ortográfico) no uniform do passe. O
/// `view_proj` é o MESMO afim que os sprites usam; `px_per_world` = altura da
/// janela / altura de mundo (pixels isotrópicos), o zoom que vira a espessura.
fn camera_raw(camera: &Camera2d, window: WindowSize) -> CameraRaw {
    let vp = camera.view_proj(window).to_cols_array_2d();
    let px_per_world = window.height.max(1) as f32 / camera.height_world.max(f32::EPSILON);
    CameraRaw::new(
        vp,
        [window.width as f32, window.height as f32],
        px_per_world,
    )
}
