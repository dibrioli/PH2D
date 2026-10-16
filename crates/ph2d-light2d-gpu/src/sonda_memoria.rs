//! ⭐ W5x — DIAGNÓSTICO: a banda de um buffer sozinho (ler em sequência, ler aos saltos, escrever zero
//! ou ruído), pelo relógio da placa. A régua do regime da `c2` do High: os zeros de um buffer em páginas
//! COMPRIMÍVEIS leem-se a ~4× a banda da DRAM, fora delas à banda da DRAM (handoff W5x). Só para as
//! réguas (`PH2D_LIGHT2D_CUSTO=1` no app e as sondas dos testes).

use std::sync::Arc;

use ph2d_gpu::GpuContext;

const WGSL: &str = r"
struct P { n: u32, modo: u32 }
@group(0) @binding(0) var<storage, read_write> dados: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read_write> saida: array<u32>;
@group(0) @binding(2) var<uniform> p: P;
fn h(x: u32) -> u32 {
  var v = x * 747796405u + 2891336453u;
  v = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
  return (v >> 22u) ^ v;
}
@compute @workgroup_size(256)
fn ler(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
  let passo = nw.x * 256u;
  var acc = vec4<u32>(0u);
  var i = g.x;
  loop { if i >= p.n { break; } acc ^= dados[i]; i += passo; }
  let s = acc.x ^ acc.y ^ acc.z ^ acc.w;
  if s == 0x12345679u { saida[0] = s; }
}
@compute @workgroup_size(256)
fn saltar(@builtin(global_invocation_id) g: vec3<u32>) {
  var acc = vec4<u32>(0u);
  var j = h(g.x);
  for (var k = 0u; k < 16u; k++) { j = h(j ^ k); acc ^= dados[j % p.n]; }
  let s = acc.x ^ acc.y ^ acc.z ^ acc.w;
  if s == 0x12345679u { saida[0] = s; }
}
@compute @workgroup_size(256)
fn escrever(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
  let passo = nw.x * 256u;
  var i = g.x;
  loop {
    if i >= p.n { break; }
    let k = 4u * i;
    dados[i] = select(vec4<u32>(0u), vec4<u32>(h(k), h(k + 1u), h(k + 2u), h(k + 3u)), p.modo == 1u);
    i += passo;
  }
}
";

/// O núcleo de medida.
#[derive(Clone, Copy)]
pub enum Medida {
    /// Ler o buffer inteiro em sequência.
    Ler,
    /// 16 leituras aos saltos por fio (4096 × 256 fios).
    Saltar,
    /// Escrever zero.
    EscreverZero,
    /// Escrever ruído (incomprimível).
    EscreverRuido,
}

/// Os núcleos de medida (o buffer precisa de `STORAGE`) e o relógio de placa de um despacho.
#[doc(hidden)]
pub struct SondaDeMemoria {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    bgl: wgpu::BindGroupLayout,
    ler: wgpu::ComputePipeline,
    saltar: wgpu::ComputePipeline,
    escrever: wgpu::ComputePipeline,
    saida: wgpu::Buffer,
    consultas: wgpu::QuerySet,
    resolvido: wgpu::Buffer,
    leitura: wgpu::Buffer,
}

impl SondaDeMemoria {
    /// `None` sem `TIMESTAMP_QUERY`.
    #[must_use]
    pub fn nova(gpu: &GpuContext) -> Option<Self> {
        if !gpu
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
        {
            return None;
        }
        let d = &gpu.device;
        let modulo = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d sonda de memória"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let entrada = |binding: u32, ty: wgpu::BufferBindingType| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let armazem = wgpu::BufferBindingType::Storage { read_only: false };
        let bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d sonda de memória"),
            entries: &[
                entrada(0, armazem),
                entrada(1, armazem),
                entrada(2, wgpu::BufferBindingType::Uniform),
            ],
        });
        let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d sonda de memória"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let pipeline = |e: &str| {
            d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(e),
                layout: Some(&layout),
                module: &modulo,
                entry_point: Some(e),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let pequeno = |nome: &'static str, usage: wgpu::BufferUsages| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(nome),
                size: 256,
                usage,
                mapped_at_creation: false,
            })
        };
        Some(Self {
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
            ler: pipeline("ler"),
            saltar: pipeline("saltar"),
            escrever: pipeline("escrever"),
            bgl,
            saida: pequeno("luz2d sonda saida", wgpu::BufferUsages::STORAGE),
            consultas: d.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("luz2d sonda de memória"),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            }),
            resolvido: pequeno(
                "luz2d sonda resolvido",
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            leitura: pequeno(
                "luz2d sonda leitura",
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
        })
    }

    /// O mínimo de `n` despachos de `medida` sobre `dados` inteiro (arredondado a 16 bytes), em ms.
    #[must_use]
    pub fn ms(&self, dados: &wgpu::Buffer, medida: Medida, n: usize) -> f64 {
        let (pl, modo) = match medida {
            Medida::Ler => (&self.ler, 0),
            Medida::Saltar => (&self.saltar, 0),
            Medida::EscreverZero => (&self.escrever, 0),
            Medida::EscreverRuido => (&self.escrever, 1),
        };
        let grupo = self.grupo(dados, modo);
        (0..n.max(1))
            .map(|_| self.mede(pl, &grupo))
            .fold(f64::INFINITY, f64::min)
    }

    fn grupo(&self, dados: &wgpu::Buffer, modo: u32) -> wgpu::BindGroup {
        let tam = dados.size() / 16 * 16;
        let u = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("luz2d sonda uniforme"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut b = [0u8; 16];
        b[..4].copy_from_slice(&u32::try_from(tam / 16).unwrap_or(u32::MAX).to_le_bytes());
        b[4..8].copy_from_slice(&modo.to_le_bytes());
        self.queue.write_buffer(&u, 0, &b);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d sonda de memória"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: dados,
                        offset: 0,
                        size: wgpu::BufferSize::new(tam),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.saida.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: u.as_entire_binding(),
                },
            ],
        })
    }

    fn mede(&self, pl: &wgpu::ComputePipeline, grupo: &wgpu::BindGroup) -> f64 {
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d sonda de memória"),
                timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                    query_set: &self.consultas,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
            });
            pass.set_pipeline(pl);
            pass.set_bind_group(0, grupo, &[]);
            pass.dispatch_workgroups(4096, 1, 1);
        }
        enc.resolve_query_set(&self.consultas, 0..2, &self.resolvido, 0);
        enc.copy_buffer_to_buffer(&self.resolvido, 0, &self.leitura, 0, 16);
        self.queue.submit([enc.finish()]);
        let fatia = self.leitura.slice(..16);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        let ticks = {
            let v = fatia.get_mapped_range();
            let t0 = u64::from_le_bytes(v[..8].try_into().unwrap_or_default());
            let t1 = u64::from_le_bytes(v[8..16].try_into().unwrap_or_default());
            t1.saturating_sub(t0)
        };
        self.leitura.unmap();
        ticks as f64 * f64::from(self.queue.get_timestamp_period()) * 1e-6
    }
}
