//! ⭐⭐⭐ **DOIS MOTORES, UMA LEI** — o arnês que mede o campo do dispositivo contra o da CPU.
//!
//! É o mesmo molde que o Flip e o pincel de tecido usam: quem shipa é o rápido, e quem julga é o
//! lento. ⚠️ **A barra não é `0`**, e a razão está declarada: a fita da CPU é `f64` e o dispositivo
//! é `f32`. O que o arnês devolve é o **erro**, e quem escolhe a barra é o gate.

use ph2d_field::FieldDoc;
use ph2d_field_eval::Field;

/// O que o dispositivo respondeu e o que a CPU respondeu, ponto a ponto.
pub struct Parity {
    pub gpu: Vec<f32>,
    pub cpu: Vec<f64>,
}

impl Parity {
    /// O maior desvio absoluto — em unidades de MUNDO.
    #[must_use]
    pub fn worst(&self) -> f64 {
        self.gpu
            .iter()
            .zip(&self.cpu)
            .map(|(g, c)| (f64::from(*g) - c).abs())
            .fold(0.0, f64::max)
    }

    /// O desvio RMS.
    #[must_use]
    pub fn rms(&self) -> f64 {
        if self.cpu.is_empty() {
            return 0.0;
        }
        let s: f64 = self
            .gpu
            .iter()
            .zip(&self.cpu)
            .map(|(g, c)| {
                let d = f64::from(*g) - c;
                d * d
            })
            .sum();
        (s / self.cpu.len() as f64).sqrt()
    }
}

/// O molde de um shader que só avalia o campo nos pontos que lhe derem.
const MOLDE: &str = r"
@group(0) @binding(0) var<storage, read> k: array<f32>;
@group(0) @binding(1) var<storage, read> pts: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
{FIELD}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i * 3u + 2u >= arrayLength(&pts)) { return; }
    out[i] = field(vec3<f32>(pts[i * 3u], pts[i * 3u + 1u], pts[i * 3u + 2u]));
}
";

/// ⭐ **Avalia `doc` nos `pontos`, nos DOIS motores.**
///
/// `None` quando não há adaptador de GPU, ou quando o documento não tem fita — a mesma resposta que
/// o [`Field::at`] dá com `NaN`.
#[must_use]
pub fn compare(doc: &FieldDoc, pontos: &[[f32; 3]]) -> Option<Parity> {
    let campo = Field::new(doc);
    let fita = campo.tape_wgsl()?;

    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("paridade do campo"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .ok()?;

    let mut cache = crate::FieldPipelines::new();
    let pipeline = cache.get(&device, MOLDE, &fita).clone();

    let bytes = |v: &[f32]| -> Vec<u8> {
        let mut b = Vec::with_capacity(v.len() * 4);
        for x in v {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b
    };
    // ⚠️ Um buffer de storage VAZIO é inválido; um documento sem constantes recebe uma.
    let consts = if fita.consts.is_empty() {
        vec![0.0f32]
    } else {
        fita.consts.clone()
    };
    let planos: Vec<f32> = pontos.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
    let n = pontos.len() as u64;

    let buf = |dados: &[u8], usage: wgpu::BufferUsages| {
        use wgpu::util::DeviceExt;
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: dados,
            usage,
        })
    };
    let kb = buf(&bytes(&consts), wgpu::BufferUsages::STORAGE);
    let pb = buf(&bytes(&planos), wgpu::BufferUsages::STORAGE);
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

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
    enc.copy_buffer_to_buffer(&ob, 0, &leitura, 0, (n * 4).max(4));
    queue.submit([enc.finish()]);

    let fatia = leitura.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = fatia.get_mapped_range();
    let gpu: Vec<f32> = dados
        .as_chunks::<4>()
        .0
        .iter()
        .take(pontos.len())
        .map(|c| f32::from_le_bytes(*c))
        .collect();
    drop(dados);
    leitura.unmap();

    let cpu: Vec<f64> = pontos
        .iter()
        .map(|p| campo.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2])))
        .collect();
    Some(Parity { gpu, cpu })
}
