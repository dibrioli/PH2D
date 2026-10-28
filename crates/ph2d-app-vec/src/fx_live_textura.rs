//! Corte F37 (teto de LOC) de `fx_live.rs`: a capacidade em degraus da textura de saída e a
//! textura REGISTADA que a mostra (criar e copiar).

use ph2d_gpu::GpuContext;

use crate::fx_live_memo::MAX_FX_SIDE;

#[allow(clippy::cast_precision_loss)]
pub(super) fn w_f32(v: u32) -> f32 {
    v as f32
}

/// **A capacidade da textura de saída para um lado de `v` texels.** Trocar a textura re-regista-a
/// no Vello, e com o tamanho a mudar a cada quadro de um gesto (o recorte à vista, F34) ~¼ dos
/// quadros saíam SEM a imagem a partir de ~2 000 px (sonda `fx_zoom_sonda_do_piscar`). Em degraus,
/// e a textura só se troca quando o conteúdo não cabe ou cabe com folga de mais (`serve`).
pub(crate) fn capacidade(v: u32) -> u32 {
    let degrau = if v >= 1024 { 256 } else { 64 };
    v.div_ceil(degrau)
        .saturating_mul(degrau)
        .min(MAX_FX_SIDE)
        .max(v)
}

/// A textura de lado `cap` serve um conteúdo de `v`: cabe, e não sobra mais que o dobro.
pub(crate) fn serve(cap: u32, v: u32) -> bool {
    v <= cap && cap <= capacidade(v.saturating_mul(2))
}

/// O recurso de lados `cap` SOBRA para um pedido de `preciso`: um dos lados passa da `capacidade` do
/// pedido. Sem a folga do `serve` — parado, o gesto acabou. Nada pedido (`0`) = tudo sobra.
pub(crate) fn sobra(cap: (u32, u32), preciso: (u32, u32)) -> bool {
    cap.0 > capacidade(preciso.0) || cap.1 > capacidade(preciso.1)
}

/// A textura REGISTADA de uma forma: só recebe cópias (a pilha escreve na `saida`), e o Vello copia-a
/// para o atlas (`COPY_SRC`, `Rgba8Unorm`).
pub(super) fn textura_mostrada(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-app-vec::fx_live::textura_mostrada"),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Copia o retângulo `[x0, y0, x1, y1]` da saída da pilha para o canto da textura registada.
pub(super) fn copia_o_que_mostra(
    gpu: &GpuContext,
    de: &wgpu::Texture,
    para: &wgpu::Texture,
    m: [u32; 4],
) {
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-app-vec::fx_live::copia_o_que_mostra"),
        });
    enc.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: de,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: m[0],
                y: m[1],
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        para.as_image_copy(),
        wgpu::Extent3d {
            width: m[2] - m[0],
            height: m[3] - m[1],
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
}
