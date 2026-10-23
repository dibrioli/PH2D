//! Os layouts e as ligações dos grupos da C3 — módulo-filho de `solver_c3.rs` (o tecto de 700 LOC).

pub(super) fn layout(
    device: crate::Placa<'_>,
    rotulo: &str,
    entradas: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(rotulo),
        entries: entradas,
    })
}

/// Uma textura de 1 × 1 para uma ligação que o shader cita e este solver não alimenta.
pub(super) fn textura_vazia(
    device: &wgpu::Device,
    rotulo: &str,
    formato: wgpu::TextureFormat,
    uso: wgpu::TextureUsages,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(rotulo),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: formato,
            usage: uso,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

pub(super) fn vista(binding: u32, v: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(v),
    }
}
