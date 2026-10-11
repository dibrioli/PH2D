//! ⭐ doc 129 §3.4 do Motion (o `fx.filter` em CADA forma) — o pipeline das CAIXAS das cópias no
//! ecrã ([`crate::ShapePass::caixas`]): a mesma `caixa_estimada` do desenho, num passe de cálculo.

/// O pipeline do [`crate::ShapePass::caixas`]: o grupo `0` do desenho e o grupo `3` da saída.
pub(crate) fn pipeline_das_caixas(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    layout: &wgpu::BindGroupLayout,
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let entrada = |binding, ty| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ph2d-shape-gpu caixas"),
        entries: &[
            entrada(0, wgpu::BufferBindingType::Storage { read_only: false }),
            entrada(1, wgpu::BufferBindingType::Uniform),
        ],
    });
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ph2d-shape-gpu caixas"),
        bind_group_layouts: &[Some(layout), None, None, Some(&bgl)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("ph2d-shape-gpu caixas"),
        layout: Some(&pl),
        module,
        entry_point: Some("cs_caixas"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bgl)
}

pub(crate) fn entrada(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
