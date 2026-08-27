//! **OS ALVOS DO PASSE DE HALO** — a cadeia de mips, os uniformes por-passe e os bind groups,
//! ou seja tudo o que depende do TAMANHO da janela.
//!
//! ⚠️ **O corte foi FORÇADO pelo HR-18** e a costura é por responsabilidade: o `motion_fx.rs`
//! fica com a lei do passe (o que se desenha e por que ordem) e isto com o que se reconstrói a
//! cada redimensionamento. É a razão de o [`Shared`] existir — ele é exactamente a fronteira
//! entre as duas metades: o que sobrevive ao resize.

use super::{GpuContext, Tex, UNIFORM_BYTES, make_tex, mip_sizes};

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
    /// A view da MÁSCARA DE SUJIDADE que está ligada agora — a do artista, ou o preto de
    /// 1×1 quando não há nenhuma escolhida (doc 89 folha 11).
    ///
    /// ⚠️ **Ela é o único membro do `Shared` que NÃO sobrevive a tudo**, e é por isso que
    /// [`bind_all`] existe separado do [`build_targets`]: escolher outra imagem tem de
    /// refazer os bind groups e **não** a cadeia de mips, que é onde mora a memória.
    pub(super) dirt: &'a wgpu::TextureView,
}

pub(super) fn build_targets(gpu: &GpuContext, shared: &Shared<'_>, size: (u32, u32)) -> Targets {
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
                size: UNIFORM_BYTES,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();

    let (bg_prefilter, bg_down, bg_up, bg_composite) = bind_all(gpu, shared, &rt, &mips, &u_down);

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

/// **REFAZ só os bind groups**, mantendo as texturas — o caminho de quando o artista escolhe
/// outra máscara de sujidade (doc 89 folha 11).
///
/// ⚠️ **Ele existe para NÃO reconstruir a cadeia de mips.** A view da máscara vive nos quatro
/// bind groups (o layout é partilhado, como a LUT), então trocá-la obriga a refazê-los; refazer
/// os ALVOS junto seria realocar seis texturas de tela por causa de um descritor, e num arrasto
/// de escolha de imagem isso é um pico de memória de GPU por quadro.
pub(super) fn bind_all(
    gpu: &GpuContext,
    shared: &Shared<'_>,
    rt: &Tex,
    mips: &[Tex],
    u_down: &[wgpu::Buffer],
) -> (
    wgpu::BindGroup,
    Vec<wgpu::BindGroup>,
    Vec<wgpu::BindGroup>,
    wgpu::BindGroup,
) {
    let Shared {
        bgl,
        sampler,
        u_prefilter,
        u_up,
        u_composite,
        lut,
        dirt,
    } = *shared;
    let passes = mips.len().saturating_sub(1);
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
                // A máscara de sujidade — idem: no layout partilhado, lida só pelo composite.
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(dirt),
                },
            ],
        })
    };
    (
        bind(&rt.view, u_prefilter),
        (0..passes)
            .map(|i| bind(&mips[i].view, &u_down[i]))
            .collect(),
        (0..passes).map(|i| bind(&mips[i + 1].view, u_up)).collect(),
        bind(&mips[0].view, u_composite),
    )
}
