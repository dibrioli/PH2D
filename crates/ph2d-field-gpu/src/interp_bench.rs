//! ⏱️⭐⭐⭐ **O BANCO: a mesma fita, COMPILADA contra INTERPRETADA, nos mesmos pontos.**
//!
//! Mede o factor que decide a poda por região (ver [`ph2d_field_eval::interp`]): o custo de uma
//! instrução interpretada sobre o de uma compilada, na placa desta máquina. ⚠️ A resposta das duas
//! é comparada ponto a ponto — um interpretador que desse outro número mediria outro programa.
//! ⛔ Instrumento: nada no produto o chama.

use ph2d_field::FieldDoc;
use ph2d_field_eval::Field;

/// O que o banco mediu.
pub struct Banco {
    /// Mínimo de relógio de parede por despacho (submit + espera), compilada.
    pub ms_compilado: f64,
    /// O mesmo, interpretada.
    pub ms_interpretado: f64,
    /// O pior `|interpretada − compilada|` nos pontos.
    pub pior_desvio: f32,
    /// Instruções que fazem trabalho, e os registos do interpretador.
    pub operacoes: usize,
    pub registos: usize,
}

const KERNEL: &str = r"
@group(0) @binding(1) var<storage, read> pts: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i * 3u + 2u >= arrayLength(&pts)) { return; }
    out[i] = field(vec3<f32>(pts[i * 3u], pts[i * 3u + 1u], pts[i * 3u + 2u]));
}
";

/// ⭐ As ESTRUTURAS de kernel do banco de estrutura — a mesma fita, só muda à volta dela.
///
/// `laço`: dez passos de marcha dependentes do valor (como a marcha real). `normal à parte`: mais
/// quatro chamadas escritas uma a uma (como o estêncil da normal). `normal no laço`: as mesmas
/// quatro, mas de um só ponto de chamada.
pub const ESTRUTURAS: [(&str, &str); 4] = [
    (
        "uma chamada",
        r"
    out[i] = field(p);",
    ),
    (
        "laço de 10",
        r"
    var t = 0.0;
    for (var s = 0u; s < 10u; s = s + 1u) { t = t + abs(field(p + vec3<f32>(0.0, 0.0, t))) + 1e-3; }
    out[i] = t;",
    ),
    (
        "laço + normal à parte",
        r"
    var t = 0.0;
    for (var s = 0u; s < 10u; s = s + 1u) { t = t + abs(field(p + vec3<f32>(0.0, 0.0, t))) + 1e-3; }
    let q = p + vec3<f32>(0.0, 0.0, t);
    let e = 1e-3;
    let o0 = vec3<f32>( 1.0, -1.0, -1.0);
    let o1 = vec3<f32>(-1.0, -1.0,  1.0);
    let o2 = vec3<f32>(-1.0,  1.0, -1.0);
    let o3 = vec3<f32>( 1.0,  1.0,  1.0);
    let n = o0 * field(q + o0 * e) + o1 * field(q + o1 * e) + o2 * field(q + o2 * e) + o3 * field(q + o3 * e);
    out[i] = t + n.x + n.y + n.z;",
    ),
    (
        "laço + normal no laço",
        r"
    var t = 0.0;
    var n = vec3<f32>(0.0);
    let e = 1e-3;
    for (var s = 0u; s < 14u; s = s + 1u) {
        var o = vec3<f32>(0.0);
        if (s == 10u) { o = vec3<f32>( 1.0, -1.0, -1.0); }
        if (s == 11u) { o = vec3<f32>(-1.0, -1.0,  1.0); }
        if (s == 12u) { o = vec3<f32>(-1.0,  1.0, -1.0); }
        if (s == 13u) { o = vec3<f32>( 1.0,  1.0,  1.0); }
        let f = field(p + vec3<f32>(0.0, 0.0, t) + o * e);
        if (s < 10u) { t = t + abs(f) + 1e-3; } else { n = n + o * f; }
    }
    out[i] = t + n.x + n.y + n.z;",
    ),
];

/// ⭐ O banco de ESTRUTURA: a fita COMPILADA, dentro de cada uma das [`ESTRUTURAS`].
#[must_use]
pub fn mede_estrutura(doc: &FieldDoc, pontos: &[[f32; 3]], repeticoes: usize) -> Option<Vec<f64>> {
    let campo = Field::new(doc);
    let fita = campo.tape_wgsl()?;
    let consts = if fita.consts.is_empty() {
        vec![0.0f32]
    } else {
        fita.consts.clone()
    };
    let mut v = Vec::new();
    for (_, corpo) in ESTRUTURAS {
        let kernel = format!(
            "@group(0) @binding(0) var<storage, read> k: array<f32>;\n{}
@group(0) @binding(1) var<storage, read> pts: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {{
    let i = g.x;
    if (i * 3u + 2u >= arrayLength(&pts)) {{ return; }}
    let p = vec3<f32>(pts[i * 3u], pts[i * 3u + 1u], pts[i * 3u + 2u]);{corpo}
}}
",
            fita.source
        );
        v.push(corre_um(&kernel, &consts, pontos, repeticoes)?);
    }
    Some(v)
}

/// ⭐⭐ **A MARCHA REAL num kernel MAGRO** — os raios do produto (origem, direcção, entrada e
/// saída na caixa, oito `f32` por pixel), o mesmo passo, orçamento e limiar, e a normal de quatro
/// amostras; e NADA mais (sem luz, chão, sombra, bordas). Separa o custo da marcha do custo do
/// kernel que a hospeda.
#[must_use]
pub fn mede_marcha_magra(
    doc: &FieldDoc,
    raios: &[[f32; 8]],
    acerto: f32,
    passo: f32,
    orcamento: u32,
    normal_eps: f32,
    repeticoes: usize,
) -> Option<f64> {
    let campo = Field::new(doc);
    let fita = campo.tape_wgsl()?;
    let consts = if fita.consts.is_empty() {
        vec![0.0f32]
    } else {
        fita.consts.clone()
    };
    let kernel = format!(
        "@group(0) @binding(0) var<storage, read> k: array<f32>;\n{}
@group(0) @binding(1) var<storage, read> pts: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {{
    let i = g.y * 1920u + g.x;
    if (g.x >= 1920u || i * 8u + 7u >= arrayLength(&pts)) {{ return; }}
    let o = vec3<f32>(pts[i * 8u], pts[i * 8u + 1u], pts[i * 8u + 2u]);
    let d = vec3<f32>(pts[i * 8u + 3u], pts[i * 8u + 4u], pts[i * 8u + 5u]);
    var t = pts[i * 8u + 6u];
    let fim = pts[i * 8u + 7u];
    var n = 0u;
    var acertou = false;
    loop {{
        if (n >= {orcamento}u || t >= fim) {{ break; }}
        let f = field(o + d * t);
        n = n + 1u;
        if (f < {acerto:e}) {{ acertou = true; break; }}
        t = t + f * {passo:e};
    }}
    if (!acertou) {{ out[i] = -1.0; return; }}
    let p = o + d * t;
    let e = {normal_eps:e};
    let o0 = vec3<f32>( 1.0, -1.0, -1.0);
    let o1 = vec3<f32>(-1.0, -1.0,  1.0);
    let o2 = vec3<f32>(-1.0,  1.0, -1.0);
    let o3 = vec3<f32>( 1.0,  1.0,  1.0);
    let w = o0 * field(p + o0 * e) + o1 * field(p + o1 * e) + o2 * field(p + o2 * e) + o3 * field(p + o3 * e);
    out[i] = t + w.x;
}}
",
        fita.source
    );
    let planos: Vec<f32> = raios.iter().flat_map(|r| *r).collect();
    #[allow(clippy::cast_possible_truncation)]
    corre_generico(&kernel, &consts, &planos, raios.len() as u32, repeticoes)
}

/// Um kernel com o `k` em `0`, dados planos em `1`, saída de `n` em `2`, despachado em `8×8`
/// sobre `1920 × ⌈n/1920⌉`.
fn corre_generico(
    kernel: &str,
    consts: &[f32],
    dados: &[f32],
    n: u32,
    repeticoes: usize,
) -> Option<f64> {
    use wgpu::util::DeviceExt;
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    let buf = |b: &[u8]| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: b,
            usage: wgpu::BufferUsages::STORAGE,
        })
    };
    let pb = buf(&bytes(dados, f32::to_le_bytes));
    let kb = buf(&bytes(consts, f32::to_le_bytes));
    let ob = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(n).max(1) * 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(kernel.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &modulo,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: kb.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: pb.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: ob.as_entire_binding(),
            },
        ],
    });
    let linhas = n.div_ceil(1920);
    let despacha = || {
        let mut enc =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cp.set_pipeline(&pipeline);
            cp.set_bind_group(0, &bind, &[]);
            cp.dispatch_workgroups(1920u32.div_ceil(8), linhas.div_ceil(8), 1);
        }
        queue.submit([enc.finish()]);
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
    };
    despacha();
    despacha();
    let mut melhor = f64::INFINITY;
    for _ in 0..repeticoes {
        let t0 = std::time::Instant::now();
        despacha();
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    Some(melhor)
}

/// Um kernel com o `k` em `0`, os pontos em `1` e a saída em `2`: o mínimo do relógio de parede.
fn corre_um(kernel: &str, consts: &[f32], pontos: &[[f32; 3]], repeticoes: usize) -> Option<f64> {
    use wgpu::util::DeviceExt;
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    let buf = |dados: &[u8]| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: dados,
            usage: wgpu::BufferUsages::STORAGE,
        })
    };
    let n = pontos.len() as u64;
    let planos: Vec<f32> = pontos.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
    let pb = buf(&bytes(&planos, f32::to_le_bytes));
    let kb = buf(&bytes(consts, f32::to_le_bytes));
    let ob = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (n * 4).max(4),
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(kernel.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &modulo,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: kb.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: pb.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: ob.as_entire_binding(),
            },
        ],
    });
    let despacha = || {
        let mut enc =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cp.set_pipeline(&pipeline);
            cp.set_bind_group(0, &bind, &[]);
            #[allow(clippy::cast_possible_truncation)]
            cp.dispatch_workgroups((n as u32).div_ceil(64).max(1), 1, 1);
        }
        queue.submit([enc.finish()]);
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
    };
    despacha();
    despacha();
    let mut melhor = f64::INFINITY;
    for _ in 0..repeticoes {
        let t0 = std::time::Instant::now();
        despacha();
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    Some(melhor)
}

fn bytes<T: Copy>(v: &[T], f: impl Fn(T) -> [u8; 4]) -> Vec<u8> {
    v.iter().flat_map(|x| f(*x)).collect()
}

/// ⭐ Mede `doc` sobre `pontos`, `repeticoes` despachos de cada lado.
#[must_use]
pub fn mede(doc: &FieldDoc, pontos: &[[f32; 3]], repeticoes: usize) -> Option<Banco> {
    use wgpu::util::DeviceExt;
    let campo = Field::new(doc);
    let fita = campo.tape_wgsl()?;
    let bc = campo.tape_bytecode()?;
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("banco do interpretador"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .ok()?;
    let buf = |dados: &[u8], usage| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: dados,
            usage,
        })
    };
    let n = pontos.len() as u64;
    let planos: Vec<f32> = pontos.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
    let pb = buf(
        &bytes(&planos, f32::to_le_bytes),
        wgpu::BufferUsages::STORAGE,
    );
    let consts = if fita.consts.is_empty() {
        vec![0.0f32]
    } else {
        fita.consts.clone()
    };
    let kb = buf(
        &bytes(&consts, f32::to_le_bytes),
        wgpu::BufferUsages::STORAGE,
    );
    let cb = buf(
        &bytes(&bc.palavras, u32::to_le_bytes),
        wgpu::BufferUsages::STORAGE,
    );

    let corre = |fonte: String, primeiro: &wgpu::Buffer| -> (f64, Vec<f32>) {
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(fonte.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &modulo,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let ob = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (n * 4).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let leitura = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (n * 4).max(4),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: primeiro.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pb.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: ob.as_entire_binding(),
                },
            ],
        });
        let despacha = |copiar: bool| {
            let mut enc =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                cp.set_pipeline(&pipeline);
                cp.set_bind_group(0, &bind, &[]);
                #[allow(clippy::cast_possible_truncation)]
                cp.dispatch_workgroups((n as u32).div_ceil(64).max(1), 1, 1);
            }
            if copiar {
                enc.copy_buffer_to_buffer(&ob, 0, &leitura, 0, (n * 4).max(4));
            }
            queue.submit([enc.finish()]);
            device.poll(wgpu::PollType::wait_indefinitely()).ok();
        };
        despacha(false);
        despacha(false);
        let mut melhor = f64::INFINITY;
        for _ in 0..repeticoes {
            let t0 = std::time::Instant::now();
            despacha(false);
            melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        despacha(true);
        let fatia = leitura.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
        let dados = fatia.get_mapped_range();
        let v: Vec<f32> = dados
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();
        drop(dados);
        leitura.unmap();
        (melhor, v)
    };

    let compilada = format!(
        "@group(0) @binding(0) var<storage, read> k: array<f32>;\n{}{KERNEL}",
        fita.source
    );
    let interpretada = format!(
        "@group(0) @binding(0) var<storage, read> codigo: array<u32>;\n{}{KERNEL}",
        ph2d_field_eval::interp::interpretador_wgsl(bc.registos)
    );
    let (ms_c, vc) = corre(compilada, &kb);
    let (ms_i, vi) = corre(interpretada, &cb);
    let pior_desvio = vc
        .iter()
        .zip(&vi)
        .map(|(a, b)| {
            if a.is_nan() && b.is_nan() {
                0.0
            } else {
                (a - b).abs()
            }
        })
        .fold(0.0f32, f32::max);
    Some(Banco {
        ms_compilado: ms_c,
        ms_interpretado: ms_i,
        pior_desvio,
        operacoes: bc.operacoes,
        registos: bc.registos,
    })
}
