//! ⭐⭐⭐ **O GRUPO 0 — o que o fragmento lê UMA VEZ POR QUADRO.**
//!
//! Irmão (`#[path]`) do [`super::super`] pelo mesmo corte que o pai declara: lá é *como este
//! renderizador é construído*, aqui é **o que ele deixa o shader ver por quadro**. E eles crescem
//! por motivos diferentes — o pai quando o pipeline ganha um estado, este quando a LEI ganha uma
//! entrada.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (HR-18): o pai chegou a `730` no dia em que o modo
//! `Lighting::Pbr` acrescentou o quarto uniform. A cura de um tecto é um corte para o IRMÃO, nunca
//! uma entrada nova no `FILE_OVERAGE_OK` — e ele fica melhor do que era: *as quatro entradas do
//! grupo, os quatro buffers e o `BindGroup` que os junta liam-se num bloco de 118 linhas no meio de
//! uma função de 715.*
//!
//! ⛔ **As quatro são do MESMO grupo porque a FREQUÊNCIA é a mesma** (uma escrita por quadro), e não
//! por comodidade: um uniform cuja frequência fosse outra pagaria um `set_bind_group` a mais por
//! objecto.

// ⚠️ **A CASA do tipo, e não o `use` do PAI.** Um `use super::CameraRaw` resolve-se pelo IMPORT
// dele, logo o pai ficaria com um import que só este ficheiro usa — e quem arrumasse os imports do
// pai partiria este, em silêncio. Foi exactamente o que aconteceu no sentido inverso quando este
// corte nasceu: o `RigRaw` e o `ShadeRaw` VIAJARAM para cá e as linhas ficaram lá, órfãs.
use crate::lighting::RigRaw;
use crate::pipeline::CameraRaw;
use crate::shade::ShadeRaw;

/// O que o [`super::super::MeshRenderer::new`] precisa de guardar do grupo 0.
pub(super) struct GrupoDoQuadro {
    pub bgl: wgpu::BindGroupLayout,
    pub uniform: wgpu::Buffer,
    pub rig_uniform: wgpu::Buffer,
    pub shade_uniform: wgpu::Buffer,
    pub pbr_uniform: wgpu::Buffer,
    pub bind: wgpu::BindGroup,
}

/// Monta o grupo 0 inteiro — o layout, os quatro buffers e o `BindGroup`.
pub(super) fn monta(device: &wgpu::Device) -> GrupoDoQuadro {
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ph2d-mesh bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // O RIG. Buffer separado do da câmera de propósito: são duas
            // frequências (a câmera muda a cada arrasto, o rig quando o
            // artista abre o card) e, sobretudo, o gate do layout coluna-major
            // da câmera continua olhando exatamente os mesmos 128 bytes que
            // olhava antes desta wave.
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // **AS OPÇÕES DE SOMBREAMENTO** (hoje: a cavidade). Terceira
            // entrada do grupo 0 e não um campo apendado ao rig, apesar de o
            // `RigRaw` ter padding sobrando: uma cavidade não é uma lâmpada, e
            // aquele struct é o espelho do `Lamp` do passe de luz da tinta —
            // enfiar um knob de barro nele faria a próxima wave que sincronizar
            // os dois herdar um campo que o outro lado não tem.
            //
            // A FREQUÊNCIA é a que justifica o grupo: câmera, rig e opções são
            // todos da CENA (uma escrita por frame). O `Object` é o grupo 1
            // porque ele é por-desenho.
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // ⭐⭐⭐ **O MATERIAL E O CÉU** (`crate::pbr::PbrRaw`), que o modo
            // `Lighting::Pbr` lê. Quarta entrada do MESMO grupo porque a
            // frequência é a mesma (uma escrita por quadro) — e um uniform
            // próprio, e não campos apendados ao `ShadeRaw`, porque aquele tem
            // 32 B exactos e o corte é por ASSUNTO: *com que luz* é a vista,
            // *de que matéria* é o objecto.
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-mesh camera"),
        size: size_of::<CameraRaw>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let rig_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-mesh rig"),
        size: RigRaw::SIZE as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shade_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-mesh shade"),
        size: ShadeRaw::SIZE as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let pbr_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-mesh pbr"),
        size: crate::pbr::PbrRaw::SIZE as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-mesh bind"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: rig_uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: shade_uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: pbr_uniform.as_entire_binding(),
            },
        ],
    });

    GrupoDoQuadro {
        bgl,
        uniform,
        rig_uniform,
        shade_uniform,
        pbr_uniform,
        bind,
    }
}
