//! **AS TEXTURAS DO PASSE DE HALO** — o tipo `Tex`, as duas fábricas (o alvo de tela e a tira
//! da LUT) e a escada de tamanhos da cadeia de mips.
//!
//! ⚠️ **O corte foi FORÇADO pelo teto de LOC** (700, `architecture_workspace_file_loc_cap`) e a
//! costura segue a que o `_targets` já fez: aquele responde *o que se reconstrói a cada
//! redimensionamento*, este *de que material*. As três fábricas vivem juntas porque partilham a
//! decisão que mais custa a acertar — o FORMATO, e o que ele obriga (filtrável, `Rgba16Float`
//! nas três; ver [`make_lut`]).

use super::{GpuContext, HALO_LUT_TEXELS};

pub(super) struct Tex {
    #[allow(dead_code)]
    pub(super) texture: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
    pub(super) size: (u32, u32),
}

pub(super) fn make_tex(gpu: &GpuContext, size: (u32, u32), label: &str) -> Tex {
    let size = (size.0.max(1), size.1.max(1));
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::GameRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        size,
    }
}

/// **A LUT DA RAMPA DO HALO** — `HALO_LUT_TEXELS × 1`, `Rgba16Float`.
///
/// ⚠️ **`Rgba16Float` e não `Rgba32Float`**, e a razão é a filtragem: o `rgba32float` **não é
/// filtrável** sem uma feature opcional do WebGPU, e sem filtragem linear a LUT devolveria
/// degraus — exactamente o artefacto que a resolução medida existe para não ter. E não
/// `Rgba8Unorm`, cuja quantização é `1/255`: ela **é** o passo do ecrã, ou seja gastaria o
/// orçamento inteiro do erro antes de a interpolação começar.
pub(super) fn make_lut(gpu: &GpuContext) -> Tex {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render motion-fx halo LUT"),
        size: wgpu::Extent3d {
            width: HALO_LUT_TEXELS as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::GameRt::FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        size: (HALO_LUT_TEXELS as u32, 1),
    }
}

/// Cap on mip-chain depth (6 halvings reach a wide soft glow at any editor size).
pub(super) const MAX_MIPS: usize = 6;

/// The mip resolutions: mip0 = half the RT, then halve while both dims stay ≥ 2,
/// capped at [`MAX_MIPS`]. Always at least one level.
pub(super) fn mip_sizes(size: (u32, u32)) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let mut s = (size.0.max(2) / 2, size.1.max(2) / 2);
    for _ in 0..MAX_MIPS {
        out.push(s);
        if s.0 <= 2 || s.1 <= 2 {
            break;
        }
        s = ((s.0 / 2).max(1), (s.1 / 2).max(1));
    }
    out
}
