//! **O PERCURSO NO DEVICE** — o port do [`crate::binning::walk_pixel`] para compute
//! ([doc 12](../../../docs/Flip/12_novo_motor_pesquisa.md) §14).
//!
//! ⚠️ **O binning fica na CPU, e isso é MEDIDO, não conveniência:** `bin_segments` custa
//! **1,3 ms para 7800 segmentos** a 1080p (doc 12 §10.2) — 1× por frame, contra o percurso que é
//! por-PIXEL. Portar o binner é uma wave própria com ganho conhecido e pequeno; portar o percurso
//! é a wave que muda a ordem de grandeza.
//!
//! ⚠️ **Um workgroup de 16×16 É um ladrilho** (`DEFAULT_TILE = 16`, o número que a §6.2 mediu):
//! as 256 threads leem a MESMA lista, que é a razão inteira de o binning existir.
//!
//! ⚠️ **A saída é `vec4<f32>`, não uma textura de 8 bits** — de propósito. O gate de paridade tem
//! de medir a divergência entre os dois motores, e um alvo `rgba8unorm` a misturaria com a
//! quantização. O alvo do produto é uma textura, e a troca é de uma linha.

use crate::binning::{ScreenSpace, TileBins};
use crate::pack::FlipGpuData;
use wgpu::util::DeviceExt;

/// Os três números da grade + a câmera, no layout que o `walk.wgsl` declara.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    world_to_clip: [[f32; 4]; 4],
    view: [f32; 4],
    grid: [u32; 4],
}

/// O passe. Guarda só o pipeline — os buffers seguem o tamanho da cena, então nascem por chamada
/// (esta é a wave que mede o percurso; um pool é da wave de integração).
pub struct WalkPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl WalkPass {
    /// Compila o kernel.
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-flip walk"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/walk.wgsl").into()),
        });
        let entry = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-flip walk bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                entry(1, true),
                entry(2, true),
                entry(3, true),
                entry(4, true),
                entry(5, false),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-flip walk pl"),
            bind_group_layouts: &[&layout],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-flip walk pipeline"),
            layout: Some(&pl),
            module: &module,
            entry_point: Some("walk"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self { pipeline, layout }
    }

    /// Percorre a tela inteira e devolve o RGBA **premultiplicado** em `f32`, na mesma ordem
    /// row-major que o [`crate::binning::walk_pixel`] preencheria.
    ///
    /// ⚠️ **Um buffer de storage vazio é ILEGAL em wgpu**, e uma cena pode legitimamente não ter
    /// segmento nenhum (tela em branco) — por isso cada `Vec` sobe com um elemento sentinela
    /// quando vazio. Sem isso o primeiro frame de um documento novo é um panic de validação.
    #[must_use]
    pub fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &FlipGpuData,
        screen: &ScreenSpace,
        bins: &TileBins,
    ) -> Vec<[f32; 4]> {
        let Some(job) = self.prepare(device, data, screen, bins) else {
            return Vec::new();
        };
        let out_size = (job.n_px * 16) as u64;
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("walk read"),
            size: out_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("walk enc"),
        });
        self.record(&mut enc, &job);
        enc.copy_buffer_to_buffer(&job.out, 0, &read, 0, out_size);
        queue.submit(Some(enc.finish()));

        let slice = read.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let _ = rx.recv();
        let view = slice.get_mapped_range();
        let px: Vec<[f32; 4]> = bytemuck::cast_slice::<u8, [f32; 4]>(&view)[..job.n_px].to_vec();
        drop(view);
        read.unmap();
        px
    }

    /// Sobe os buffers da cena e monta o bind group. **É a metade por-FRAME**: o produto chama
    /// isto uma vez e depois [`Self::record`] no encoder do frame, sem readback nenhum.
    #[must_use]
    pub fn prepare(
        &self,
        device: &wgpu::Device,
        data: &FlipGpuData,
        screen: &ScreenSpace,
        bins: &TileBins,
    ) -> Option<WalkJob> {
        let (w, h) = (screen.viewport[0] as u32, screen.viewport[1] as u32);
        let n_px = (w as usize) * (h as usize);
        if n_px == 0 {
            return None;
        }
        let uni = ScreenUniform {
            world_to_clip: screen.world_to_clip,
            view: [
                screen.viewport[0],
                screen.viewport[1],
                screen.px_per_world,
                0.0,
            ],
            grid: [bins.tile, bins.cols, bins.rows, 0],
        };
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("walk uniform"),
            contents: bytemuck::bytes_of(&uni),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let storage = |label: &str, bytes: &[u8]| {
            // O sentinela do buffer vazio (ver o doc acima).
            let fallback = [0u8; 16];
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: if bytes.is_empty() { &fallback } else { bytes },
                usage: wgpu::BufferUsages::STORAGE,
            })
        };
        let pts = storage("walk points", bytemuck::cast_slice(&data.points));
        let strs = storage("walk strokes", bytemuck::cast_slice(&data.strokes));
        let rng = storage("walk ranges", bytemuck::cast_slice(&bins.ranges));
        let sgs = storage("walk segs", bytemuck::cast_slice(&bins.segs));
        let out_size = (n_px * 16) as u64;
        let out = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("walk out"),
            size: out_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("walk bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ubuf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: strs.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: rng.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: sgs.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: out.as_entire_binding(),
                },
            ],
        });
        Some(WalkJob {
            bg,
            out,
            n_px,
            groups: [w.div_ceil(16), h.div_ceil(16)],
        })
    }

    /// Grava o dispatch no encoder do chamador. **Sem submit, sem readback** — é o que o produto
    /// faz, e é por isso que a sonda de custo mede ISTO e não o `run`, cujo readback de 33 MB a
    /// 1080p não pertence a frame nenhum.
    pub fn record(&self, enc: &mut wgpu::CommandEncoder, job: &WalkJob) {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("walk pass"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&self.pipeline);
        cp.set_bind_group(0, &job.bg, &[]);
        cp.dispatch_workgroups(job.groups[0], job.groups[1], 1);
    }
}

/// Os recursos de uma cena já no device, prontos para [`WalkPass::record`].
pub struct WalkJob {
    bg: wgpu::BindGroup,
    out: wgpu::Buffer,
    n_px: usize,
    groups: [u32; 2],
}
