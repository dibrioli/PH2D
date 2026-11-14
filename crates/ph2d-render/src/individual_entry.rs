//! **COMO UMA ENTRADA DE TEXTURA INDIVIDUAL É CONSTRUÍDA** — irmão do
//! [`crate::individual`], que trata de como o *store* as gere.
//!
//! ⚠️ **Saiu de lá por medição** (2026-08-20): o `individual.rs` chegou a **1112** linhas contra um
//! tecto de 969 quando o caminho de 16 bits entrou (plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W2). A regra registada deste projeto é **cortar, nunca alargar a allowlist** — e o corte é por
//! responsabilidade: lá fica *quem possui e liberta*, aqui *como os bytes viram textura*.
//!
//! ⚠️ **As três construtoras partilham uma armadilha:** o `bytes_per_row` é `width × 4` para 8 bits
//! e `width × 8` para 16. Trocá-los não dá erro — dá a imagem cortada ao meio na horizontal.

use ph2d_gpu::GpuContext;

use crate::individual::{IndividualTextureEntry, IndividualTextureStore};
use crate::individual_renasce::FonteRecusada;

pub(crate) fn create_entry(
    gpu: &GpuContext,
    material_bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> IndividualTextureEntry {
    let entry = create_entry_empty(
        gpu,
        material_bgl,
        sampler,
        width,
        height,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );
    let mut entry = entry;
    if entry.fonte.is_some() {
        entry.fonte = Some(FonteRecusada::Rgba8(rgba.to_vec()));
    } else {
        write_pixels(gpu, &entry.texture, width, height, rgba);
    }
    entry
}

/// Irmã de [`create_entry`] para a precisão alta — plano `docs/Sprite_projeto/18`.
///
/// ⚠️ **`halves` são bits de meio-float em espaço LINEAR.** Escrever aqui os bytes sRGB
/// convertidos a `u16` compilaria, subiria para a GPU sem uma queixa, e renderizaria a sprite
/// visivelmente mais clara — porque o `Rgba8UnormSrgb` do caminho normal é decodificado pelo
/// **hardware** na amostragem e o `Rgba16Float` **não é**. É esse o defeito que o gate
/// `the_two_precisions_deliver_the_same_colour` mede.
pub(crate) fn create_entry_16(
    gpu: &GpuContext,
    material_bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    halves: &[u16],
) -> IndividualTextureEntry {
    let entry = create_entry_empty(
        gpu,
        material_bgl,
        sampler,
        width,
        height,
        IndividualTextureStore::FORMAT_16,
    );
    let mut entry = entry;
    if entry.fonte.is_some() {
        entry.fonte = Some(FonteRecusada::Rgba16(halves.to_vec()));
    } else {
        write_pixels_16(gpu, &entry.texture, width, height, halves);
    }
    entry
}

/// Build an [`IndividualTextureEntry`] with an UNINITIALISED texture (no pixel
/// upload). Shared by [`create_entry`] (which then writes pixels) and
/// [`IndividualTextureStore::acquire_empty`] (which leaves the fill to a later
/// GPU copy). The texture/view/bind-group are otherwise identical, so a slot
/// created either way is sampled the same way. Cleared transparent on creation
/// ([`crate::texture_clear`]) so a sample before the first fill reads transparent.
pub(crate) fn create_entry_empty(
    gpu: &GpuContext,
    material_bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> IndividualTextureEntry {
    let mip_count = crate::mipgen::mip_levels(width, height);
    let (texture, recusada) = crate::renasce::nasce_textura(&gpu.device, || {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render individual texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // COPY_SRC: `readback()` staging copy (Image Tools edit on Individual sprites). COPY_DST: feeds
            // `copy_from_texture` (Painter GPU preview blit). RENDER_ATTACHMENT: `MipGenerator` mip blits.
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    });
    // Clear-on-alloc: an EMPTY slot (`acquire_empty`) is sampled before its first fill → else garbage.
    // Os níveis DA TEXTURA: a substituta da recusada tem um só (F38 §3).
    crate::texture_clear::clear_all_mips_transparent(gpu, &texture, texture.mip_level_count());
    // The sampled view spans ALL mip levels (default) so the trilinear sampler can
    // pick the right level; the generator makes its own single-level views.
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render individual bg"),
        layout: material_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    IndividualTextureEntry {
        texture,
        view,
        bind_group,
        width,
        height,
        mip_count,
        format,
        refcount: 1,
        // Nasce vazia e morre com o entry — ver o doc do campo.
        sampler_bgs: std::collections::BTreeMap::new(),
        fonte: recusada.then_some(FonteRecusada::Vazia),
    }
}

pub(crate) fn write_pixels(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    rgba: &[u8],
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// Upload a tightly-packed `width × height` RGBA8 sub-rect to `(x, y)` of
/// `texture`, leaving the rest of the texture untouched. Caller guarantees
/// the rect is in-bounds and `region_rgba.len() == width * height * 4`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_pixels_region(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    region_rgba: &[u8],
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        region_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// Escreve o nível 0 de uma textura de 16 bits (`halves` = meio-float linear, 4 por texel).
pub(crate) fn write_pixels_16(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    halves: &[u16],
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(halves),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            // 4 canais × 2 bytes. ⚠️ O `× 4` do caminho de 8 bits aqui daria metade da linha, e o
            // resultado seria a imagem cortada ao meio na horizontal — não um erro.
            bytes_per_row: Some(width * 8),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// O pedaço do anel: o `chunk_size` do [`wgpu::util::StagingBelt`]. Uma subida maior cria o seu
/// pedaço do tamanho dela (33 MB a 4K) e REUSA-o nas seguintes (B20.6).
pub(crate) const ANEL_PEDACO: u64 = 4 << 20;

std::thread_local! {
    /// Teste: força o caminho (`Some(true)` = o anel, `Some(false)` = `write_texture`).
    pub(crate) static PELO_ANEL: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// `PH2D_SUBIDA_SEM_ANEL=1` = antes (`queue.write_texture`, um staging novo por subida). Lida uma vez.
pub(crate) fn subida_pelo_anel() -> bool {
    static SEM: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    PELO_ANEL.with(std::cell::Cell::get).unwrap_or_else(|| {
        !*SEM.get_or_init(|| std::env::var("PH2D_SUBIDA_SEM_ANEL").is_ok_and(|v| v == "1"))
    })
}

/// **B20.6 — a subida inteira pelo anel**: os bytes vão direto ao buffer mapeado de um
/// [`wgpu::util::StagingBelt`] reusado e `copy_buffer_to_texture`, submetido já (a ordem com as
/// `write_texture` pendentes fica: elas descem no MESMO `submit`, antes do comando). Os mesmos
/// texels que [`write_pixels`]; a linha alinha-se a 256 B quando `width × 4` não o é.
/// Sonda `mede_a_subida_4k_write_texture_contra_staging_belt`: CPU 2,68 → 1,28 ms a 4K.
pub(crate) fn write_pixels_pelo_anel(
    gpu: &GpuContext,
    anel: &mut wgpu::util::StagingBelt,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    rgba: &[u8],
) {
    let linha = (width as usize) * 4;
    let passo = linha.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
    let Some(tamanho) = wgpu::BufferSize::new((passo * height as usize) as u64) else {
        return;
    };
    let alinha = wgpu::BufferSize::new(wgpu::COPY_BUFFER_ALIGNMENT).expect("4 ≠ 0");
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render subida pelo anel"),
        });
    {
        let fatia = anel.allocate(tamanho, alinha);
        let mut vista = fatia
            .get_mapped_range_mut()
            .expect("o anel entrega o pedaço mapeado");
        if passo == linha {
            vista.copy_from_slice(rgba);
        } else {
            for (y, src) in rgba.chunks_exact(linha).enumerate() {
                vista
                    .slice(y * passo..y * passo + linha)
                    .copy_from_slice(src);
            }
        }
        drop(vista);
        enc.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: fatia.buffer(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: fatia.offset(),
                    bytes_per_row: Some(passo as u32),
                    rows_per_image: Some(height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
    anel.finish();
    gpu.queue.submit([enc.finish()]);
    anel.recall();
}

#[cfg(test)]
#[path = "individual_anel_tests.rs"]
mod individual_anel_tests;
