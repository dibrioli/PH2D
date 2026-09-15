//! ⭐⭐⭐ **DOIS MOTORES, UMA LEI — o material** (`docs/Render3d/05` §39).
//!
//! O [`ph2d_material::wgsl`] é o port do [`ph2d_material`] para WGSL, e o que prova que ele é a
//! MESMA lei é este arnês: as duas correm sobre a **mesma grelha de entradas** e as respostas
//! comparam-se uma a uma.
//!
//! ⚠️ **Nada aqui está no caminho do produto** — é o oráculo que autoriza o passe de sombreamento
//! a existir. *Um port de 700 linhas sem esta medição é uma esperança.*

/// Uma amostra: a normal, a direcção para o observador, a direcção para a luz.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    /// A normal da superfície, unitária.
    pub n: [f32; 3],
    /// A direcção **para o observador**, unitária.
    pub v: [f32; 3],
    /// A direcção **para a luz**, unitária.
    pub l: [f32; 3],
}

/// O que as duas leis devolvem para uma amostra.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Answer {
    /// `Surface::direct` com radiância `1`.
    pub direct: [f32; 3],
    /// `Surface::indirect`.
    pub indirect: [f32; 3],
    /// `Surface::emission`.
    pub emission: [f32; 3],
}

/// ⭐ **O CÉU DO ARNÊS — analítico, e o MESMO dos dois lados.**
///
/// ⛔ Um céu **chapado** não serviria: a lei indirecta especular pergunta pela direcção **espelhada**
/// (`mx_environment_radiance`), e com uma resposta constante essa direcção deixa de ser observável —
/// o gate passaria com o espelho apontado ao sítio errado. ⇒ ele depende de `dir.y` e de `alpha`.
pub struct HarnessSky;

impl HarnessSky {
    /// A mesma expressão que o WGSL do [`SKY`] escreve.
    const BASE: [f32; 3] = [0.20, 0.24, 0.31];
    const SLOPE: [f32; 3] = [0.55, 0.47, 0.38];
}

impl ph2d_material::Environment for HarnessSky {
    fn radiance(&self, dir: [f32; 3], alpha: f32) -> [f32; 3] {
        let k = 1.0 / (1.0 + 4.0 * alpha);
        [0, 1, 2].map(|i| Self::BASE[i] + Self::SLOPE[i] * (0.5 + 0.5 * dir[1]) * k)
    }
    fn irradiance(&self, n: [f32; 3]) -> [f32; 3] {
        [0, 1, 2].map(|i| Self::BASE[i] + Self::SLOPE[i] * (0.5 + 0.5 * n[1]) * 0.5)
    }
}

/// O céu do arnês, em WGSL — preenche o [`ph2d_material::wgsl::ENV_SLOT`].
pub const SKY: &str = r#"
const HARNESS_BASE: vec3<f32> = vec3<f32>(0.20, 0.24, 0.31);
const HARNESS_SLOPE: vec3<f32> = vec3<f32>(0.55, 0.47, 0.38);
// ⚠️ O `shrink` chega e este céu ignora-o — ele não é direccional por lóbulo. Ver o `ENV_SLOT`.
fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    let k = 1.0 / (1.0 + 4.0 * alpha);
    return HARNESS_BASE + HARNESS_SLOPE * ((0.5 + 0.5 * dir.y) * k);
}
fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    return HARNESS_BASE + HARNESS_SLOPE * ((0.5 + 0.5 * n.y) * 0.5);
}
"#;

const HARNESS: &str = r#"
struct Amostra { n: vec4<f32>, v: vec4<f32>, l: vec4<f32> };
@group(0) @binding(0) var<uniform> mat: Mat;
@group(0) @binding(1) var<storage, read> entrada: array<Amostra>;
@group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn avalia(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i >= arrayLength(&entrada)) { return; }
    let a = entrada[i];
    let n = a.n.xyz;
    let v = a.v.xyz;
    let l = a.l.xyz;
    saida[i * 3u + 0u] = vec4<f32>(mx_direct(mat, n, v, l, vec3<f32>(1.0)), 0.0);
    saida[i * 3u + 1u] = vec4<f32>(mx_indirect(mat, n, v), 0.0);
    saida[i * 3u + 2u] = vec4<f32>(mx_emission(mat, n, v), 0.0);
}
"#;

/// ⭐ **A lei do dispositivo, sobre as amostras dadas.** `None` sem adaptador.
#[must_use]
pub fn on_device(surface: &ph2d_material::Surface, samples: &[Sample]) -> Option<Vec<Answer>> {
    use wgpu::util::DeviceExt;
    let t = crate::trace::Tracer::new()?;
    let (device, queue) = t.parts();
    let fonte = format!(
        "{}\n{HARNESS}",
        ph2d_material::wgsl::SOURCE.replace(ph2d_material::wgsl::ENV_SLOT, SKY)
    );
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("material"),
        source: wgpu::ShaderSource::Wgsl(fonte.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("material"),
        layout: None,
        module: &modulo,
        entry_point: Some("avalia"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let empacotado = ph2d_material::wgsl::pack(surface, ph2d_material::wgsl::EnvLobe::IGNORED);
    let mut u = Vec::with_capacity(empacotado.len() * 4);
    for f in empacotado {
        u.extend_from_slice(&f.to_le_bytes());
    }
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mat"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    // ⚠️ Cada direcção viaja como `vec4` — um `vec3` num array alinha a 16 B de qualquer forma.
    let mut inb = Vec::with_capacity(samples.len() * 48);
    for s in samples {
        for v in [s.n, s.v, s.l] {
            for c in v {
                inb.extend_from_slice(&c.to_le_bytes());
            }
            inb.extend_from_slice(&0f32.to_le_bytes());
        }
    }
    let entrada = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("entrada"),
        contents: &inb,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let bytes = (samples.len() * 3 * 16) as u64;
    let saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("saida"),
        size: bytes.max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: bytes.max(16),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: entrada.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: saida.as_entire_binding(),
            },
        ],
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&pipeline);
        cp.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        cp.dispatch_workgroups((samples.len() as u32).div_ceil(64), 1, 1);
    }
    enc.copy_buffer_to_buffer(&saida, 0, &leitura, 0, bytes.max(16));
    queue.submit([enc.finish()]);
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = leitura.slice(..).get_mapped_range();
    let f4 = |q: &[u8; 16], o: usize| f32::from_le_bytes([q[o], q[o + 1], q[o + 2], q[o + 3]]);
    let quads = dados.as_chunks::<16>().0;
    let out = (0..samples.len())
        .map(|i| {
            let ler = |k: usize| [0, 1, 2].map(|c| f4(&quads[i * 3 + k], c * 4));
            Answer {
                direct: ler(0),
                indirect: ler(1),
                emission: ler(2),
            }
        })
        .collect();
    drop(dados);
    Some(out)
}

/// A MESMA pergunta, na CPU — o lado que é a referência.
#[must_use]
pub fn on_cpu(surface: &ph2d_material::Surface, samples: &[Sample]) -> Vec<Answer> {
    let sky = HarnessSky;
    samples
        .iter()
        .map(|s| Answer {
            direct: surface.direct(s.n, s.v, s.l, [1.0; 3]),
            indirect: surface.indirect(s.n, s.v, &sky),
            emission: surface.emission(s.n, s.v),
        })
        .collect()
}

#[cfg(test)]
#[path = "material_parity_tests.rs"]
mod tests;
