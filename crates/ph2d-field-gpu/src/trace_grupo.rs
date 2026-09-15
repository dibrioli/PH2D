//! ⭐ **O GRUPO `0` DA MARCHA** — as sete ligações, numa ordem que o WGSL declara.
//!
//! ⚠️ **Saiu do [`crate::trace`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira é a mesma dos
//! irmãos: aqui não há decisão, só a montagem. ⛔ **A ORDEM é o contrato** — ela tem de ser a do
//! [`crate::trace_wgsl::COMUM`] e a do [`crate::trace::bgl_marcha`], e as três mudam juntas.

/// As sete entradas do grupo `0`, na ordem dos bindings.
// Sete buffers e o dispositivo: cada um é um binding declarado, e uma struct só os renomearia.
#[allow(clippy::too_many_arguments)]
pub(super) fn grupo_da_marcha(
    device: &wgpu::Device,
    bgl: &wgpu::BindGroupLayout,
    setup: &wgpu::Buffer,
    k: &wgpu::Buffer,
    centro: &wgpu::Buffer,
    luz: &wgpu::Buffer,
    conta: &wgpu::Buffer,
    borda: &wgpu::Buffer,
    grades: &wgpu::Buffer,
) -> wgpu::BindGroup {
    fn r(b: &wgpu::Buffer, i: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: i,
            resource: b.as_entire_binding(),
        }
    }
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bgl,
        entries: &[
            r(setup, 0),
            r(k, 1),
            r(centro, 2),
            r(luz, 3),
            r(conta, 4),
            r(borda, 5),
            r(grades, 6),
        ],
    })
}
