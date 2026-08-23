//! **OS ALVOS DO PASSE DE HALO** — a cadeia de mips, os uniformes por-passe e os bind groups,
//! ou seja tudo o que depende do TAMANHO da janela.
//!
//! ⚠️ **O corte foi FORÇADO pelo HR-18** e a costura é por responsabilidade: o `motion_fx.rs`
//! fica com a lei do passe (o que se desenha e por que ordem) e isto com o que se reconstrói a
//! cada redimensionamento. É a razão de o [`Shared`] existir — ele é exactamente a fronteira
//! entre as duas metades: o que sobrevive ao resize.

use super::{GpuContext, Tex, make_tex, mip_sizes};

/// Everything size-dependent: the full-res RT, the mip chain, the per-pass
/// downsample uniforms, and all bind groups.
pub(super) struct Targets {
    pub(super) rt: Tex,
    pub(super) mips: Vec<Tex>,
    pub(super) u_down: Vec<wgpu::Buffer>,
    pub(super) bg_prefilter: wgpu::BindGroup,
    pub(super) bg_down: Vec<wgpu::BindGroup>,
    pub(super) bg_up: Vec<wgpu::BindGroup>,
    pub(super) bg_composite: wgpu::BindGroup,
}

/// **O que um bind group precisa e que NÃO depende do tamanho da janela** — o layout, o
/// sampler, os uniformes de passe e a LUT do halo.
///
/// ⚠️ **Agrupados porque são a mesma coisa**, e não para calar um lint: eles nascem uma vez na
/// construção, sobrevivem a todo redimensionamento, e o `build_targets` reconstrói só o que
/// depende do tamanho. Passá-los soltos deixava a assinatura em oito posições, que é onde um
/// `&wgpu::Buffer` troca de lugar com outro sem o compilador reparar.
pub(super) struct Shared<'a> {
    pub(super) bgl: &'a wgpu::BindGroupLayout,
    pub(super) sampler: &'a wgpu::Sampler,
    pub(super) u_prefilter: &'a wgpu::Buffer,
    pub(super) u_up: &'a wgpu::Buffer,
    pub(super) u_composite: &'a wgpu::Buffer,
    pub(super) lut: &'a Tex,
}

pub(super) fn build_targets(gpu: &GpuContext, shared: &Shared<'_>, size: (u32, u32)) -> Targets {
    let Shared {
        bgl,
        sampler,
        u_prefilter,
        u_up,
        u_composite,
        lut,
    } = *shared;
    let rt = make_tex(gpu, size, "ph2d-render motion-fx RT (Rgba16Float HDR)");
    let mips: Vec<Tex> = mip_sizes(size)
        .into_iter()
        .map(|d| make_tex(gpu, d, "ph2d-render motion-fx mip"))
        .collect();
    let passes = mips.len().saturating_sub(1);

    let u_down: Vec<wgpu::Buffer> = (0..passes)
        .map(|_| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render motion-fx u_down"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();

    let bind = |src: &wgpu::TextureView, u: &wgpu::Buffer| {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render motion-fx bg"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: u.as_entire_binding(),
                },
                // A LUT do halo — presente em todos os quatro, lida só pelo composite.
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&lut.view),
                },
            ],
        })
    };

    let bg_prefilter = bind(&rt.view, u_prefilter);
    let bg_down: Vec<_> = (0..passes)
        .map(|i| bind(&mips[i].view, &u_down[i]))
        .collect();
    let bg_up: Vec<_> = (0..passes).map(|i| bind(&mips[i + 1].view, u_up)).collect();
    let bg_composite = bind(&mips[0].view, u_composite);

    Targets {
        rt,
        mips,
        u_down,
        bg_prefilter,
        bg_down,
        bg_up,
        bg_composite,
    }
}
