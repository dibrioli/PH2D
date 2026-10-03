//! ⭐⭐⭐ **O COMPOSTO DAS CAMADAS ACHATADO NO PLANO DE TINTA, NA PLACA**
//! (`docs/3D/30` §13, W1b) — filho (`#[path]`) do `pipeline`, irmão do
//! [`super::tinta_gpu`]: lá o plano sobe da CPU, aqui ele é ESCRITO na placa a
//! partir do composto que o compositor de camadas do Painter deixou numa
//! textura — o plano inteiro deixa de ter de subir a cada mudança do painel.
//!
//! A lei é a de `ph2d_app_sculpt3d::pilha_da_peca::achata` (shader
//! `shaders/tinta_achata.wgsl`): opaco = `byte / 255` exacto pela tabela da
//! CPU; translúcido = mistura em luz sobre o fundo (linear, da CPU).

use wgpu::util::DeviceExt;

use crate::MeshRenderer;

/// O shader do achatamento.
pub const TINTA_ACHATA_WGSL: &str = include_str!("shaders/tinta_achata.wgsl");

const GRUPO: u32 = 256;

/// ⭐ **A grade do despacho de `n` amostras**: `[gx, gy, passo_y]`, com no
/// máximo `max_grupos` grupos numa dimensão — o shader lê a amostra
/// `i = y · passo_y + x`, e cada `i < n` sai UMA vez (gate `a_grade_cobre_cada_amostra_uma_vez`).
#[must_use]
pub(crate) fn grade(n: u32, max_grupos: u32) -> [u32; 3] {
    let grupos = n.div_ceil(GRUPO).max(1);
    let gx = grupos.min(max_grupos.max(1));
    [gx, grupos.div_ceil(gx), gx * GRUPO]
}

/// ⭐ **O pipeline do achatamento** — um por cena, de quem compõe as camadas.
pub struct AchataDaTinta {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    /// `[0, 256)`: `byte / 255` · `[256, 512)`: `srgb_to_linear_byte(byte)` —
    /// os números da CPU, ao bit.
    tabela: wgpu::Buffer,
    cfg: wgpu::Buffer,
}

fn entrada(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

fn armazem(so_leitura: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage {
            read_only: so_leitura,
        },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

impl AchataDaTinta {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh tinta achata"),
            source: wgpu::ShaderSource::Wgsl(TINTA_ACHATA_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh tinta achata"),
            entries: &[
                entrada(
                    0,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entrada(1, armazem(true)),
                entrada(2, armazem(false)),
                entrada(3, armazem(true)),
                entrada(
                    4,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh tinta achata"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-mesh tinta achata"),
            layout: Some(&pl),
            module: &shader,
            entry_point: Some("cs_achata"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let tabela: Vec<f32> = (0..=255u8)
            .map(|b| f32::from(b) / 255.0)
            .chain((0..=255u8).map(ph2d_color::srgb::srgb_to_linear_byte))
            .collect();
        let tabela = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ph2d-mesh tinta achata tabela"),
            contents: bytemuck::cast_slice(&tabela),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let cfg = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh tinta achata cfg"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            layout,
            tabela,
            cfg,
        }
    }
}

impl AchataDaTinta {
    /// ⭐ **O fundo na placa** — três `f32` LINEARES por amostra, a entrada
    /// `fundo` do [`MeshRenderer::achata_tinta_at`].
    #[must_use]
    pub fn fundo(device: &wgpu::Device, luz: &[[f32; 3]]) -> wgpu::Buffer {
        let vazio = [[0.0f32; 3]];
        let luz = if luz.is_empty() { &vazio[..] } else { luz };
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ph2d-mesh tinta achata fundo"),
            contents: bytemuck::cast_slice(luz),
            usage: wgpu::BufferUsages::STORAGE,
        })
    }
}

impl MeshRenderer {
    /// ⭐⭐⭐ **Achata o composto no plano de tinta do slot `k`** — as `n`
    /// amostras, da dobra `largura × ⌈n/largura⌉` em `composto`, sobre o
    /// `fundo` (três `f32` LINEARES por amostra). `false` (e nada escrito) se o
    /// slot não tem plano armado de `n` amostras: o chamador sobe-o da CPU.
    #[allow(clippy::too_many_arguments)] // device+queue+slot+pipeline+as três entradas
    pub fn achata_tinta_at(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        k: usize,
        achata: &AchataDaTinta,
        composto: &wgpu::TextureView,
        largura: u32,
        fundo: &wgpu::Buffer,
        n: usize,
    ) -> bool {
        let Some(slot) = self.slots.get(k) else {
            return false;
        };
        let g = &slot.gpu.tinta;
        if !g.armado || g.n_amostras != n || n == 0 {
            return false;
        }
        let Ok(n) = u32::try_from(n) else {
            return false;
        };
        let [gx, gy, passo_y] = grade(n, device.limits().max_compute_workgroups_per_dimension);
        queue.write_buffer(
            &achata.cfg,
            0,
            bytemuck::cast_slice(&[n, largura, passo_y, 0]),
        );
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh tinta achata"),
            layout: &achata.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(composto),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fundo.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: g.amostras.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: achata.tabela.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: achata.cfg.as_entire_binding(),
                },
            ],
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-mesh tinta achata"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-mesh tinta achata"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&achata.pipeline);
            pass.set_bind_group(0, &grupo, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        queue.submit([enc.finish()]);
        true
    }

    /// O plano de tinta do slot `k` lido de volta (BLOQUEIA — gates).
    #[must_use]
    pub fn le_tinta_at(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        k: usize,
    ) -> Option<Vec<[f32; 3]>> {
        let g = &self.slots.get(k)?.gpu.tinta;
        if !g.armado {
            return None;
        }
        let bytes = (g.n_amostras * 12) as u64;
        let destino = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh tinta leitura"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-mesh tinta leitura"),
        });
        enc.copy_buffer_to_buffer(&g.amostras, 0, &destino, 0, bytes);
        queue.submit([enc.finish()]);
        let fatia = destino.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let lido = bytemuck::cast_slice::<u8, [f32; 3]>(&fatia.get_mapped_range()).to_vec();
        destino.unmap();
        Some(lido)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ **GATE — A grade cobre cada amostra UMA vez** — também quando as
    /// amostras passam do tecto de grupos de uma dimensão (a `64x` não passam:
    /// o tecto pequeno aqui é o que faz a segunda linha de grupos existir).
    #[test]
    fn a_grade_cobre_cada_amostra_uma_vez() {
        for (n, max) in [
            (1u32, 4u32),
            (256, 4),
            (257, 1),
            (5_000, 3),
            (1_000, 1),
            (3_000, 2),
        ] {
            let [gx, gy, passo] = grade(n, max);
            assert!(gx <= max, "n {n}: {gx} grupos numa dimensão");
            let mut vistas = vec![0u8; n as usize];
            for y in 0..gy {
                for x in 0..gx * GRUPO {
                    let i = y * passo + x;
                    if i < n {
                        vistas[i as usize] += 1;
                    }
                }
            }
            assert!(
                vistas.iter().all(|&v| v == 1),
                "n {n}, max {max}: {vistas:?}"
            );
        }
    }
}
