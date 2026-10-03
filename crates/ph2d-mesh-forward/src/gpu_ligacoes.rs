//! As LIGAÇÕES do desenhista: as entradas do grupo `0` (o quadro, o céu, os mapas, as texturas) e os
//! ajudantes de entrada. O número de cada uma é o do `forward.wgsl`.

pub(crate) fn uniforme(
    binding: u32,
    dinamico: bool,
    vis: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: vis,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dinamico,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(crate) fn textura_float(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub(crate) fn textura_prof(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

/// O grupo `0` do desenhista — todas as ligações que o `forward.wgsl` declara no grupo `0`.
pub(crate) fn g0(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let tri = crate::gpu_triplanar::entradas();
    let ct = crate::gpu_contacto::entradas(17);
    let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ph2d-mesh-forward g0"),
        entries: &[
            uniforme(0, false, vf),
            uniforme(1, false, wgpu::ShaderStages::FRAGMENT),
            textura_float(2),
            textura_float(3),
            textura_prof(4),
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            textura_float(8),
            textura_float(9),
            textura_prof(10),
            textura_prof(11),
            tri[0],
            tri[1],
            tri[2],
            crate::gpu_ceu_chao::entrada(15),
            uniforme(16, false, wgpu::ShaderStages::FRAGMENT),
            ct[0],
            ct[1],
            ct[2],
        ],
    })
}
