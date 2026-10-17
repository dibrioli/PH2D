//! ⭐ W5z — **a régua que vê a compressão de uma TEXTURA** (handoff W5y §2 e §4.1, caminho (A)). A sonda
//! de texturas da W5y lia em ordem de linha e dava 541–620 GB/s em tudo, até numa textura nova de zeros.
//! Esta lê na ordem dos ladrilhos (faixas de `tw × th` texels, `tw` = 64 B de largura) e calibra-se a si
//! própria: o teto (uma textura que cabe no L2) e o chão (ruído, que não comprime), ao lado dos buffers de
//! calibração da W5x (herdado ~2 TB/s, fresco ~610).
//!
//! A pergunta: uma textura de armazém `Rg32Uint` NOVA, escrita por shader com o vácuo da holográfica
//! (`empacotar(VACUO)` = `(0, 0x3C00_0000)`), lê-se acima da DRAM?

use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::{Medida, SondaDeMemoria};

use crate::comum::gpu;

const MIB: u64 = 1024 * 1024;

const WGSL_COMUM: &str = r"
struct P { n: u32, largura: u32, tw: u32, th: u32, modo: u32, a: u32, b: u32, c: u32 }
@group(0) @binding(1) var<storage, read_write> saida: array<u32>;
@group(0) @binding(2) var<uniform> p: P;
fn posicao(i: u32) -> vec2<u32> {
  let por_faixa = p.tw * p.th;
  let f = i / por_faixa;
  let r = i % por_faixa;
  let faixas = p.largura / p.tw;
  return vec2<u32>((f % faixas) * p.tw + r % p.tw, (f / faixas) * p.th + r / p.tw);
}
fn h(x: u32) -> u32 {
  var v = x * 747796405u + 2891336453u;
  v = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
  return (v >> 22u) ^ v;
}
";

/// A declaração do recurso e o `carregar` vêm à frente ([`Via`]).
const WGSL_LER: &str = r"
@compute @workgroup_size(256)
fn ler(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
  let passo = nw.x * 256u;
  var acc = vec4<u32>(0u);
  var i = g.x;
  loop { if i >= p.n { break; } acc ^= carregar(posicao(i)); i += passo; }
  let s = acc.x ^ acc.y ^ acc.z ^ acc.w;
  if s == 0x12345679u { saida[0] = s; }
}
";

const WGSL_ESCREVER: &str = r"
@compute @workgroup_size(256)
fn escrever(@builtin(global_invocation_id) g: vec3<u32>, @builtin(num_workgroups) nw: vec3<u32>) {
  let passo = nw.x * 256u;
  var i = g.x;
  loop {
    if i >= p.n { break; }
    var v = vec4<u32>(0u);
    if p.modo == 1u { v = vec4<u32>(0u, 0x3C000000u, 0u, 0x3C000000u); }
    if p.modo == 2u { let k = 4u * i; v = vec4<u32>(h(k), h(k + 1u), h(k + 2u), h(k + 3u)); }
    textureStore(w, posicao(i), v);
    i += passo;
  }
}
";

/// O que se escreve por shader.
#[derive(Clone, Copy)]
enum Conteudo {
    Zero = 0,
    Vacuo = 1,
    Ruido = 2,
}

/// A ordem de leitura: linha (a régua da W5y), faixas de um GOB (`tw × 8`) ou de um bloco (`tw × 128`).
#[derive(Clone, Copy)]
enum Ordem {
    Linha,
    Gob,
    Bloco,
}

/// Por onde se lê: `textureLoad` de uma `texture_2d` (a unidade de texturas), de uma
/// `texture_storage_2d<…, read>` (Rg32 ou Rgba32); o buffer `array<vec2<u32>>` (o elemento dos T/R) lê-se
/// por [`Regua::ler_buffer2`].
#[derive(Clone, Copy, PartialEq)]
enum Via {
    Amostra,
    Armazem,
}

const PRELUDIOS: [&str; 4] = [
    "@group(0) @binding(0) var t: texture_2d<u32>;\nfn carregar(c: vec2<u32>) -> vec4<u32> { return textureLoad(t, c, 0); }\n",
    "@group(0) @binding(0) var t: texture_storage_2d<rg32uint, read>;\nfn carregar(c: vec2<u32>) -> vec4<u32> { return textureLoad(t, c); }\n",
    "@group(0) @binding(0) var t: texture_storage_2d<rgba32uint, read>;\nfn carregar(c: vec2<u32>) -> vec4<u32> { return textureLoad(t, c); }\n",
    "@group(0) @binding(0) var<storage, read> b: array<vec2<u32>>;\nfn carregar(c: vec2<u32>) -> vec4<u32> { return vec4<u32>(b[c.y * p.largura + c.x], 0u, 0u); }\n",
];

impl Ordem {
    fn faixa(self, largura: u32, bytes_por_texel: u32) -> (u32, u32) {
        let tw = 64 / bytes_por_texel;
        match self {
            Self::Linha => (largura, 1),
            Self::Gob => (tw, 8),
            Self::Bloco => (tw, 128),
        }
    }
}

struct Regua {
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
    /// `[amostra, armazém rg32, armazém rgba32, buffer vec2]` ([`PRELUDIOS`]).
    ler: [(wgpu::BindGroupLayout, wgpu::ComputePipeline); 4],
    /// Por formato: `[Rg32Uint, Rgba32Uint]`.
    escrever: [(wgpu::BindGroupLayout, wgpu::ComputePipeline); 2],
    saida: wgpu::Buffer,
    consultas: wgpu::QuerySet,
    resolvido: wgpu::Buffer,
    leitura: wgpu::Buffer,
}

fn entrada_buffer(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl Regua {
    fn nova(gpu: &GpuContext) -> Self {
        let d = &gpu.device;
        let pipeline = |fonte: String, entrada0: wgpu::BindingType, e: &str| {
            let modulo = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sonda compressão"),
                source: wgpu::ShaderSource::Wgsl(fonte.into()),
            });
            let bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sonda compressão"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: entrada0,
                        count: None,
                    },
                    entrada_buffer(1, wgpu::BufferBindingType::Storage { read_only: false }),
                    entrada_buffer(2, wgpu::BufferBindingType::Uniform),
                ],
            });
            let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sonda compressão"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });
            let pl = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(e),
                layout: Some(&layout),
                module: &modulo,
                entry_point: Some(e),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
            (bgl, pl)
        };
        let escrever = |formato: &str, f: wgpu::TextureFormat| {
            pipeline(
                format!(
                    "@group(0) @binding(0) var w: texture_storage_2d<{formato}, write>;\n{WGSL_COMUM}{WGSL_ESCREVER}"
                ),
                wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: f,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                "escrever",
            )
        };
        let pequeno = |usage: wgpu::BufferUsages| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sonda compressão"),
                size: 256,
                usage,
                mapped_at_creation: false,
            })
        };
        Self {
            device: d.clone(),
            queue: gpu.queue.clone(),
            ler: {
                let arm = |f| wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: f,
                    view_dimension: wgpu::TextureViewDimension::D2,
                };
                let tipos = [
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    arm(wgpu::TextureFormat::Rg32Uint),
                    arm(wgpu::TextureFormat::Rgba32Uint),
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ];
                std::array::from_fn(|k| {
                    pipeline(
                        format!("{WGSL_COMUM}{}{WGSL_LER}", PRELUDIOS[k]),
                        tipos[k],
                        "ler",
                    )
                })
            },
            escrever: [
                escrever("rg32uint", wgpu::TextureFormat::Rg32Uint),
                escrever("rgba32uint", wgpu::TextureFormat::Rgba32Uint),
            ],
            saida: pequeno(wgpu::BufferUsages::STORAGE),
            consultas: d.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("sonda compressão"),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            }),
            resolvido: pequeno(wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC),
            leitura: pequeno(wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
        }
    }

    fn grupo(
        &self,
        r: wgpu::BindingResource<'_>,
        (w, h): (u32, u32),
        bgl: &wgpu::BindGroupLayout,
        faixa: (u32, u32),
        modo: u32,
    ) -> wgpu::BindGroup {
        let u = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sonda compressão uniforme"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let v: [u32; 8] = [w * h, w, faixa.0, faixa.1, modo, 0, 0, 0];
        let bytes: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
        self.queue.write_buffer(&u, 0, &bytes);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sonda compressão"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: r,
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

    fn indice(t: &wgpu::Texture) -> usize {
        usize::from(t.format() == wgpu::TextureFormat::Rgba32Uint)
    }

    /// GB/s de ler `t` inteira na `ordem` pela `via` (mínimo de `n`).
    fn ler(&self, t: &wgpu::Texture, ordem: Ordem, via: Via, n: usize) -> f64 {
        let k = match via {
            Via::Amostra => 0,
            _ => 1 + Self::indice(t),
        };
        let vista = t.create_view(&wgpu::TextureViewDescriptor::default());
        let faixa = ordem.faixa(t.width(), bytes_por_texel(t));
        let (bgl, pl) = &self.ler[k];
        let g = self.grupo(
            wgpu::BindingResource::TextureView(&vista),
            (t.width(), t.height()),
            bgl,
            faixa,
            0,
        );
        let ms = (0..n)
            .map(|_| self.mede(pl, &g))
            .fold(f64::INFINITY, f64::min);
        bytes(t) as f64 / (ms * 1e-3) / 1e9
    }

    /// GB/s de ler o buffer `b` como `array<vec2<u32>>`, em sequência (mínimo de `n`).
    fn ler_buffer2(&self, b: &wgpu::Buffer, n: usize) -> f64 {
        let elementos = u32::try_from(b.size() / 8).unwrap_or(u32::MAX);
        let (bgl, pl) = &self.ler[3];
        let g = self.grupo(
            b.as_entire_binding(),
            (elementos, 1),
            bgl,
            (elementos, 1),
            0,
        );
        let ms = (0..n)
            .map(|_| self.mede(pl, &g))
            .fold(f64::INFINITY, f64::min);
        b.size() as f64 / (ms * 1e-3) / 1e9
    }

    /// GB/s de escrever `conteudo` em `t` inteira na ordem dos blocos (1 despacho).
    fn escrever(&self, t: &wgpu::Texture, conteudo: Conteudo) -> f64 {
        let (bgl, pl) = &self.escrever[Self::indice(t)];
        let vista = t.create_view(&wgpu::TextureViewDescriptor::default());
        let faixa = Ordem::Bloco.faixa(t.width(), bytes_por_texel(t));
        let g = self.grupo(
            wgpu::BindingResource::TextureView(&vista),
            (t.width(), t.height()),
            bgl,
            faixa,
            conteudo as u32,
        );
        let ms = self.mede(pl, &g);
        bytes(t) as f64 / (ms * 1e-3) / 1e9
    }

    fn mede(&self, pl: &wgpu::ComputePipeline, grupo: &wgpu::BindGroup) -> f64 {
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("sonda compressão"),
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

fn bytes_por_texel(t: &wgpu::Texture) -> u32 {
    t.format().block_copy_size(None).unwrap_or(8)
}

fn bytes(t: &wgpu::Texture) -> u64 {
    u64::from(bytes_por_texel(t)) * u64::from(t.width()) * u64::from(t.height())
}

fn textura(
    d: &wgpu::Device,
    formato: wgpu::TextureFormat,
    largura: u32,
    altura: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    d.create_texture(&wgpu::TextureDescriptor {
        label: Some("sonda compressão/textura"),
        size: wgpu::Extent3d {
            width: largura,
            height: altura,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: formato,
        usage,
        view_formats: &[],
    })
}

fn buffer(d: &wgpu::Device, tam: u64) -> wgpu::Buffer {
    d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sonda compressão/buffer"),
        size: tam,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[test]
#[ignore = "sonda W5z (placa): a régua que vê a compressão de uma textura"]
fn sonda_compressao_das_texturas() {
    const TAM: u64 = 48 * MIB;
    const RODADAS: usize = 3;
    const N: usize = 3;
    let Some(gpu) = gpu() else { return };
    let Some(sonda) = SondaDeMemoria::nova(&gpu) else {
        eprintln!("sem TIMESTAMP_QUERY");
        return;
    };
    let regua = Regua::nova(&gpu);
    let d = &gpu.device;
    let lado = d.limits().max_texture_dimension_2d;
    let arm = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
    let rg = wgpu::TextureFormat::Rg32Uint;
    let rgba = wgpu::TextureFormat::Rgba32Uint;
    // Os buffers de calibração da W5x: frescos, e herdados de texturas de armazém que morreram (o berço);
    // e um de 16 MiB, que cabe no L2 (o teto da régua).
    let mut buffers: Vec<(&str, wgpu::Buffer)> = vec![
        ("fresco a", buffer(d, TAM)),
        ("fresco b", buffer(d, TAM)),
        ("cache 16MiB", buffer(d, 16 * MIB)),
    ];
    {
        let berco: Vec<wgpu::Texture> = (0..4)
            .map(|_| {
                textura(
                    d,
                    rgba,
                    lado,
                    u32::try_from(TAM / 16 / u64::from(lado)).unwrap_or(1),
                    arm,
                )
            })
            .collect();
        let _segura = buffer(d, MIB);
        drop(berco);
        let _ = d.poll(wgpu::PollType::wait_indefinitely());
        buffers.push(("herdado a", buffer(d, TAM)));
        buffers.push(("herdado b", buffer(d, TAM)));
    }
    let altura_rg = u32::try_from(TAM / 8 / u64::from(lado)).unwrap_or(1);
    let texturas: Vec<(&str, wgpu::Texture)> = vec![
        ("cache rg32 16MiB", textura(d, rg, 2048, 1024, arm)),
        ("cache rgba32 16MiB", textura(d, rgba, 1024, 1024, arm)),
        ("arm rg32", textura(d, rg, lado, altura_rg, arm)),
        (
            "arm+rt rg32",
            textura(
                d,
                rg,
                lado,
                altura_rg,
                arm | wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        ),
        ("arm rgba32", textura(d, rgba, lado, altura_rg / 2, arm)),
    ];
    let estados = [
        ("vácuo", Conteudo::Vacuo),
        ("zero", Conteudo::Zero),
        ("ruído", Conteudo::Ruido),
        ("vácuo 2", Conteudo::Vacuo),
    ];
    let leituras = [
        (Ordem::Linha, Via::Amostra),
        (Ordem::Gob, Via::Amostra),
        (Ordem::Bloco, Via::Amostra),
        (Ordem::Gob, Via::Armazem),
        (Ordem::Bloco, Via::Armazem),
    ];
    // Banda = o MÍNIMO do tempo entre rodadas (o máximo da banda).
    let mut tab = vec![vec![(0.0_f64, [0.0_f64; 5]); estados.len()]; texturas.len()];
    let mut tab_buf = vec![[0.0_f64; 6]; buffers.len()];
    let max = |a: &mut f64, b: f64| *a = a.max(b);
    for rodada in 0..RODADAS {
        eprintln!("rodada {rodada}");
        for (bi, (_, b)) in buffers.iter().enumerate() {
            let gbs = |ms: f64| b.size() as f64 / (ms * 1e-3) / 1e9;
            let c = &mut tab_buf[bi];
            for (k, m) in [
                Medida::EscreverZero,
                Medida::EscreverRuido,
                Medida::EscreverZero,
            ]
            .into_iter()
            .enumerate()
            {
                let _ = sonda.ms(b, m, 1);
                max(&mut c[2 * k], gbs(sonda.ms(b, Medida::Ler, N)));
                max(&mut c[2 * k + 1], regua.ler_buffer2(b, N));
            }
        }
        for (ti, (_, t)) in texturas.iter().enumerate() {
            for (ei, (_, conteudo)) in estados.iter().enumerate() {
                let c = &mut tab[ti][ei];
                max(&mut c.0, regua.escrever(t, *conteudo));
                for (li, (ordem, via)) in leituras.iter().enumerate() {
                    max(&mut c.1[li], regua.ler(t, *ordem, *via, N));
                }
            }
        }
    }
    eprintln!("BUFFERS (GB/s ler vec4 · vec2): zero shader · ruído shader · zero de novo");
    for (bi, (nome, _)) in buffers.iter().enumerate() {
        let v = tab_buf[bi];
        eprintln!(
            "  {nome:12} · {:5.0} {:5.0} · {:5.0} {:5.0} · {:5.0} {:5.0}",
            v[0], v[1], v[2], v[3], v[4], v[5]
        );
    }
    eprintln!(
        "TEXTURAS (GB/s): estado · escrever · amostra linha · amostra gob · amostra bloco · armazém gob · armazém bloco"
    );
    for (ti, (nome, _)) in texturas.iter().enumerate() {
        for (ei, (estado, _)) in estados.iter().enumerate() {
            let (w, l) = tab[ti][ei];
            eprintln!(
                "  {nome:18} {estado:8} · {w:5.0} · {:5.0} · {:5.0} · {:5.0} · {:5.0} · {:5.0}",
                l[0], l[1], l[2], l[3], l[4]
            );
        }
    }
}
