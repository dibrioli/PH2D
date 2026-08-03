#![forbid(unsafe_code)]
//! `ph2d-paint-gpu` — o carimbo de pigmento no device.
//!
//! Um quadro de preview de um editor de figura re-carimba a figura INTEIRA, e a medição do produto
//! (doc 33 §1) diz que isso são **17,3 M visitas de texel** num canvas de 16,7 M pixels: a
//! redundância é ~10× porque o espaçamento é 10% do diâmetro. A CPU já roda isso a **1,06 ns por
//! visita** com as linhas divididas entre os núcleos — não há kernel lento a consertar, há trabalho
//! demais para um processador. É por isso que a alavanca seguinte é o dispositivo.
//!
//! # A lei desta crate
//!
//! ⚠️ **Ela não sabe o que é um falloff, e não pode saber.** Não há `ph2d-painter-brush` nas suas
//! dependências, então o `falloff_weight` está fora de alcance: o que entra é uma **TABELA** que a
//! CPU encheu com a função que já shipa, e uma lista de discos. É a defesa estrutural contra a
//! armadilha de dois motores sobre um estado — a mesma cura do LUT especular do `ImpastoLightPass`,
//! onde o único transcendental do modelo nunca roda no device.
//!
//! # O que ela NÃO faz
//!
//! Shape, Grain, imagem, os 23 modos de blend que não são o Normal, o cap de Accumulate e o AA do
//! filme ficam **todos** na rota em banda da CPU, que é testada e continua sendo o caminho de
//! todo caso que não seja o quente. O modo de falha de um caso novo é *lento*, nunca *errado*.

use ph2d_gpu::GpuContext;
use wgpu::util::DeviceExt as _;

/// Um disco de pigmento, já resolvido pela CPU.
///
/// ⚠️ **`m0`/`m1` são as duas LINHAS do mapa linear do footprint** (o *flatten & rotate*), avaliadas
/// pelo chamador nos vetores da base — identidade num pincel redondo. Um deform de dab É linear, e
/// `the_footprint_is_a_linear_map` prova isso contra o `apply` real em vez de o assumir.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuDab {
    pub center: [f32; 2],
    pub radius: f32,
    pub coverage: f32,
    pub color: [f32; 3],
    pub _pad0: f32,
    pub m0: [f32; 2],
    pub m1: [f32; 2],
    /// ⚠️ **QUATRO floats, não dois.** O `vec3<f32>` do WGSL alinha em 16, então a struct do shader
    /// arredonda para **64 bytes**; sem este rabo a de Rust mede 56 e o wgpu recusa o bind
    /// (`bound with size 56 where the shader expects 64`). Os offsets dos campos já coincidiam — o
    /// que falta num layout desalinhado é sempre o fim, e é onde ninguém olha.
    pub _pad1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    dab_count: u32,
    lut_len: u32,
    preserve_alpha: u32,
    _pad: u32,
}

/// A janela de canvas que um despacho escreve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// O passe. Construí-lo COMPILA o shader — o precedente do `prewarm` do preview diz que isso custa
/// milissegundos e tem de acontecer antes do gesto, nunca no primeiro traço.
pub struct StampPass {
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl StampPass {
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let device = gpu.device.clone();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-paint-gpu:stamp"),
            source: wgpu::ShaderSource::Wgsl(include_str!("stamp.wgsl").into()),
        });
        let storage = |read_only: bool| wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let entry = |binding: u32, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-paint-gpu:layout"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(1, storage(true)),
                entry(2, storage(true)),
                entry(3, storage(true)),
                entry(4, storage(false)),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-paint-gpu:pl"),
            bind_group_layouts: &[&layout],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-paint-gpu:stamp"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            device,
            queue: gpu.queue.clone(),
            pipeline,
            layout,
        }
    }

    /// Carimba `dabs` sobre `base` (a região já extraída, RGBA8, `region.w * region.h * 4` bytes) e
    /// devolve os bytes resultantes.
    ///
    /// `lut` é o perfil do pincel amostrado em `t ∈ [0,1]` — **quem o enche é a CPU**, com a função
    /// que o produto já usa.
    ///
    /// ⚠️ Esta v1 sobe a região e a lê de volta: o `canvas_rgba` continua **autoritativo** na CPU, e
    /// nenhum dos ~25 leitores dele precisa saber que a GPU existe. Tornar a tela residente no
    /// device é a fatia S4 do doc 33, e só se a medição da FRONTEIRA pedir.
    #[must_use]
    pub fn run(
        &self,
        base: &[u8],
        region: Region,
        lut: &[f32],
        dabs: &[GpuDab],
        preserve_alpha: bool,
    ) -> Vec<u8> {
        let n = (region.w as usize) * (region.h as usize);
        assert_eq!(base.len(), n * 4, "base tem de medir a região");
        if n == 0 || dabs.is_empty() || lut.is_empty() {
            return base.to_vec();
        }
        let params = Params {
            rx: region.x,
            ry: region.y,
            rw: region.w,
            rh: region.h,
            dab_count: u32::try_from(dabs.len()).unwrap_or(u32::MAX),
            lut_len: u32::try_from(lut.len()).unwrap_or(u32::MAX),
            preserve_alpha: u32::from(preserve_alpha),
            _pad: 0,
        };
        let dev = &self.device;
        let mk = |label: &str, data: &[u8], usage: wgpu::BufferUsages| {
            dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: data,
                usage,
            })
        };
        let p_buf = mk(
            "stamp:params",
            bytemuck::bytes_of(&params),
            wgpu::BufferUsages::UNIFORM,
        );
        let d_buf = mk(
            "stamp:dabs",
            bytemuck::cast_slice(dabs),
            wgpu::BufferUsages::STORAGE,
        );
        let l_buf = mk(
            "stamp:lut",
            bytemuck::cast_slice(lut),
            wgpu::BufferUsages::STORAGE,
        );
        let b_buf = mk("stamp:base", base, wgpu::BufferUsages::STORAGE);
        let bytes = (n * 4) as u64;
        let o_buf = dev.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stamp:out"),
            size: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = dev.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stamp:read"),
            size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = dev.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("stamp:bind"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: p_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: d_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: l_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: b_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: o_buf.as_entire_binding(),
                },
            ],
        });
        let mut enc = dev.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("stamp:enc"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("stamp:pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(region.w.div_ceil(8), region.h.div_ceil(8), 1);
        }
        enc.copy_buffer_to_buffer(&o_buf, 0, &read, 0, bytes);
        self.queue.submit(Some(enc.finish()));

        let slice = read.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        let out = match rx.recv() {
            Ok(Ok(())) => slice.get_mapped_range().to_vec(),
            // Um readback que falha devolve a BASE, não lixo: o chamador escreve o que já estava lá.
            _ => base.to_vec(),
        };
        read.unmap();
        out
    }
}
