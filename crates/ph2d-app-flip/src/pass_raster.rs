//! ⭐ **O quadro ACTUAL de um objecto Flip, rasterizado fora do ecrã** — o «Send to board…» da
//! Hierarquia (MiroClone W15). Filho do [`super`] porque é o MESMO passe: as camadas saem do
//! `collect_layers` (pelo ciclo, com a pose da chave e o `model` do objecto), e a composição é o
//! `compose` do ecrã (fatias por camada + o compositor de 22 modos) sem o blit no `game_rt` — a saída
//! do compositor já é straight sRGB8, o que um ficheiro quer.
//!
//! O que NÃO entra: os fantasmas (onion skin), o preview do traço em curso, o peek, o fundo, e a
//! paralaxe multiplano (sem câmara a panhar, cada plano fica no sítio dele: `depth = 1`).
//!
//! ⭐ **A escala é a régua do editor em píxeis**: `px_per_world` píxeis por unidade de mundo (a shell
//! passa os `pixels_per_meter` do projecto — o tamanho natural de uma sprite e o do SVG de um vector).
//! A caixa é a dos traços do quadro actual, posados, com o RAIO de cada ponto e 1 px de folga.

use super::{FlipComposite, LayerRef, collect_layers, compose};
use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipObjectId, FlipTarget};
use ph2d_flip_render::engine::new_engine_armed;
use ph2d_flip_render::{FlipCompose, FlipRenderer};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;

/// Uma imagem RGBA de 8 bits, alfa reto, linhas de `size[0]` píxeis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameImage {
    pub size: [u32; 2],
    pub rgba: Vec<u8>,
}

/// Porque não saiu imagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterError {
    /// O objecto não existe, ou nenhuma camada visível tem traço neste quadro.
    Empty,
    /// A composição na placa falhou (dito no terminal).
    Gpu,
}

/// A caixa de MUNDO `[x0, y0, x1, y1]` dos traços das camadas, cada ponto posado e alargado pelo
/// raio dele (a espessura escala com o `model`, como no `fold_model`). `None` = nenhum traço.
fn world_box(layers: &[LayerRef<'_>]) -> Option<[f64; 4]> {
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for l in layers {
        let k = l.model.mean_scale();
        for s in l.drawing.iter().flat_map(|d| &d.strokes) {
            for (p, w) in s.positions().iter().zip(s.widths()) {
                let [x, y] = l.model.apply([f64::from(p.x), f64::from(p.y)]);
                let r = f64::from(*w) * k;
                b = [
                    b[0].min(x - r),
                    b[1].min(y - r),
                    b[2].max(x + r),
                    b[3].max(y + r),
                ];
            }
        }
    }
    b[0].is_finite().then_some(b)
}

/// ⭐ O quadro de `object` em `playhead`, com o `model` LOCAL→mundo do objecto, a `px_per_world`
/// píxeis por unidade de mundo. Uma imagem maior que a textura que a placa aceita
/// (`max_texture_dimension_2d`) desce de escala até caber — o tecto é o do dispositivo.
#[allow(clippy::too_many_arguments)]
pub fn current_frame(
    flip: &FlipDoc,
    object: FlipObjectId,
    model: Xform,
    playhead: &Playhead,
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    px_per_world: f32,
    gpu: &GpuContext,
) -> Result<FrameImage, RasterError> {
    let models = [(object, model)];
    let (mut layers, _) = collect_layers(
        flip,
        playhead,
        None,
        FlipTarget::default(),
        &models,
        None,
        None,
    );
    layers.retain(|l| l.cache_key.0 == object.0 && l.drawing.is_some());
    for l in &mut layers {
        l.depth = 1.0;
    }
    let [x0, y0, x1, y1] = world_box(&layers).ok_or(RasterError::Empty)?;
    let max = f64::from(gpu.device.limits().max_texture_dimension_2d);
    let ppw = f64::from(px_per_world).min((max - 2.0) / (x1 - x0).max(y1 - y0));
    // + 1 px de folga de cada lado: a orla anti-serrilhada do traço.
    let px = |v: f64| (v * ppw - 1e-6).ceil().max(1.0) as u32 + 2;
    let (w, h) = (px(x1 - x0), px(y1 - y0));
    // A câmara 1:1 do passe, centrada na caixa: `px_per_world = h / height_world = ppw`.
    let center = [((x0 + x1) * 0.5) as f32, ((y0 + y1) * 0.5) as f32];
    let camera = Camera2d::new(center, (f64::from(h) / ppw) as f32);
    let cam = ph2d_flip_render::camera::camera_raw(&camera, WindowSize::new(w, h));
    flip_compose.set_walk_engine(&gpu.device, new_engine_armed());
    // Um compositor SÓ desta leitura: as fatias e a frescura do ecrã ficam intactas.
    let mut comp = FlipComposite::new(gpu);
    if !compose(
        &layers,
        flip_render,
        flip_compose,
        &mut comp,
        &cam,
        center,
        (w, h),
        gpu,
    ) {
        return Err(RasterError::Gpu);
    }
    let tex = comp.compositor.output_texture().ok_or(RasterError::Gpu)?;
    let rgba = read_rgba8(gpu, tex, w, h);
    Ok(FrameImage { size: [w, h], rgba })
}

/// Cópia bloqueante de uma textura `rgba8unorm` para `w·h·4` bytes (sem o preenchimento de linha).
fn read_rgba8(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let row = w * 4;
    let padded =
        row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(padded) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let mapped = buf.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((row * h) as usize);
    for r in mapped.chunks(padded as usize).take(h as usize) {
        out.extend_from_slice(&r[..row as usize]);
    }
    out
}

#[cfg(test)]
#[path = "pass_raster_tests.rs"]
mod tests;
