//! O aparelho com os limites do CELULAR, e os alvos do quadro (cor com MSAA, profundidade, a imagem
//! resolvida e a de saída).

/// ⭐ **O aparelho que a engine promete servir**: `Features::empty()` e os limites do WebGL2, só com
/// as dimensões de textura do aparelho real (um ecrã 4K passa os `2048` do WebGL2).
///
/// ⚠️ Pedir os limites do celular TAMBÉM no desktop é a metade que importa: um shader ou um passe
/// que precisasse de mais falharia AQUI, na máquina do programador, e não no telemóvel do jogador.
///
/// O backend é escolhido por quem chama — o gate `cabe_no_gles` pede o `GL`, o caminho do WebGL2.
#[must_use]
pub fn aparelho_em(backends: wgpu::Backends) -> Option<(wgpu::Device, wgpu::Queue, wgpu::Adapter)> {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = backends;
    let instance = wgpu::Instance::new(desc);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let limites = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-mesh-forward"),
        required_features: wgpu::Features::empty(),
        required_limits: limites,
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .ok()?;
    Some((device, queue, adapter))
}

/// O formato da cor do quadro: `Rgba16Float` (linear em meia precisão) quando a placa o desenha com
/// `4×` amostras; senão `Rgba8UnormSrgb`, que todo GLES3 garante e que a placa decodifica ao ler.
#[must_use]
pub fn formato_da_cor(adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
    let f = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba16Float);
    let precisa = wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4
        | wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE;
    if f.allowed_usages
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        && f.flags.contains(precisa)
    {
        wgpu::TextureFormat::Rgba16Float
    } else {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}

/// Os alvos de um tamanho.
pub(crate) struct Alvos {
    pub tamanho: (u32, u32),
    pub cor_msaa: wgpu::TextureView,
    pub profundidade: wgpu::TextureView,
    pub resolvida: wgpu::TextureView,
    pub saida: wgpu::Texture,
    pub saida_vista: wgpu::TextureView,
    pub ecra_bind: wgpu::BindGroup,
}

pub(crate) const PROFUNDIDADE: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub(crate) const SAIDA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn textura(
    device: &wgpu::Device,
    (w, h): (u32, u32),
    format: wgpu::TextureFormat,
    amostras: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh-forward alvo"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: amostras,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

impl Alvos {
    pub(crate) fn novos(
        device: &wgpu::Device,
        tamanho: (u32, u32),
        cor: wgpu::TextureFormat,
        ecra_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let ra = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let cor_msaa = v(&textura(device, tamanho, cor, crate::MSAA, ra));
        let profundidade = v(&textura(device, tamanho, PROFUNDIDADE, crate::MSAA, ra));
        let resolvida = v(&textura(
            device,
            tamanho,
            cor,
            1,
            ra | wgpu::TextureUsages::TEXTURE_BINDING,
        ));
        let saida = textura(
            device,
            tamanho,
            SAIDA,
            1,
            ra | wgpu::TextureUsages::COPY_SRC,
        );
        let saida_vista = v(&saida);
        let ecra_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            layout: ecra_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&resolvida),
            }],
        });
        Self {
            tamanho,
            cor_msaa,
            profundidade,
            resolvida,
            saida,
            saida_vista,
            ecra_bind,
        }
    }
}
