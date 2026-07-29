//! **Os RECURSOS do passe da pilha de FX** — o layout do uniform e as texturas de trabalho.
//!
//! Irmão de [`super::fx_stack`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo
//! é *o FOLD* (que passes correm, em que ordem, com que bind groups), este é *o que o passe ALOCA
//! e como ele fala com o device*.
//!
//! ⚠️ O [`Globals`] mora aqui e o gate de paridade (`the_wgsl_globals_members_match_the_rust_struct`)
//! lê ESTE arquivo: um uniform é lido por OFFSET, então a ordem dos membros dos dois lados é a
//! coisa mais frágil do módulo — trocar dois `u32` não falha ligação nenhuma, só passa a desenhar
//! outra coisa.

use ph2d_gpu::GpuContext;

use crate::fx_stack::UNIFORM_STRIDE;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Globals {
    pub(crate) dims: [u32; 2],
    pub(crate) half: u32,
    pub(crate) kind: u32,
    pub(crate) tint: [f32; 4],
    pub(crate) inv_two_sigma2: f32,
    pub(crate) opacity: f32,
    pub(crate) off_x: i32,
    pub(crate) off_y: i32,
    /// O passo do salto do JFA (só os passes de campo de distância o leem).
    pub(crate) jump: i32,
    /// A largura da banda (modo Contour) / do contorno, em pixels.
    pub(crate) band: f32,
    /// Quantos segmentos de SILHUETA o chamador entregou. `0` = semeia pela cobertura (o caminho
    /// antigo, e o único disponível quando não há geometria — os gates de raster puro).
    pub(crate) n_segs: u32,
    /// A LEI DE MISTURA deste degrau (o código de `BlendMode`; `0` = Normal, o neutro).
    pub(crate) blend: u32,
    /// O tamanho das ondulações do ruído, em pixels (só a turbulência o lê).
    pub(crate) noise_scale: f32,
    /// Quantas oitavas o ruído soma.
    pub(crate) octaves: u32,
    /// Qual realização do ruído.
    pub(crate) seed: u32,
    /// O MODO do degrau — o índice em `FxKindSpec::modes`.
    ///
    /// ⚠️ **O `mode` chega ao DEVICE pela primeira vez aqui.** Os modos anteriores (Proximity vs
    /// Contour) escolhem o PLANO, então nunca precisaram de um ramo no shader; os do ruído
    /// escolhem a lei da soma de oitavas, que só existe lá dentro.
    pub(crate) mode: u32,
    /// A ORIGEM da grade de ruído, em texels do scratch — a margem que a pilha reservou. É ela que
    /// prega o padrão na FORMA em vez de na textura.
    pub(crate) org: [f32; 2],
    pub(crate) _pad: [f32; 2],
}

pub(crate) struct Tex {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

pub(crate) fn make_tex(gpu: &GpuContext, w: u32, h: u32, format: wgpu::TextureFormat) -> Tex {
    let (w, h) = (w.max(1), h.max(1));
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render fx_stack tex"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        w,
        h,
    }
}

/// Cria uma textura de SAÍDA para o FX (o chamador a mantém viva por-forma — o Vello a copia no
/// render, DEPOIS do recook). `Rgba8Unorm` com os usos que o `register_texture` exige.
#[must_use]
pub fn make_output_texture(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    make_tex(gpu, w, h, wgpu::TextureFormat::Rgba8Unorm).texture
}

/// Escreve os globals do passe `slot` no blob, no offset dinâmico dele.
///
/// A pilha escreve os globals de TODOS os passes de uma vez e indexa por offset — um
/// `write_buffer` por passe antes de um único `submit` deixaria o ÚLTIMO a valer para todos.
pub(crate) fn write_at(blob: &mut [u8], slot: usize, g: &Globals) {
    let off = slot * UNIFORM_STRIDE as usize;
    let bytes = bytemuck::bytes_of(g);
    blob[off..off + bytes.len()].copy_from_slice(bytes);
}
