//! ⭐⭐⭐ **A MARCHA NO DISPOSITIVO** — o G-buffer do quadro, feito na GPU.
//!
//! # O que vai e o que fica
//!
//! Medido (`docs/Render3d/05` §34), o quadro assente a `1920×1080` reparte-se assim:
//!
//! | | CPU | |
//! |---|---:|---|
//! | traçado | `30,04 ms` | ⇒ **vai** |
//! | sombra directa | `61,73 ms` | ⇒ **vai** |
//! | **pintura** | **`10,87 ms`** | ⇒ **FICA** |
//!
//! ⭐⭐ **A pintura fica, e isso é a decisão importante.** Eu supunha que ela custava `~45 ms` e
//! teria escrito o material em WGSL por causa disso — uma **segunda** implementação do OpenPBR,
//! com tudo o que este repositório sabe sobre duas respostas para a mesma pergunta. Medida, ela
//! custa `10,87 ms` e cabe. *A lei do material continua a viver num sítio só.*
//!
//! # ⚠️ A câmera é a ÚNICA coisa escrita duas vezes, e é deliberado
//!
//! O [`ph2d_field_render::march`] avisa por escrito contra *«duas respostas para «que raio sai
//! daqui?»»*. Mandar os raios prontos seriam `50 MB` por quadro, logo o WGSL reconstrói o
//! `ray_at_plane`. ⇒ ela nasce **com o gate de paridade em cima**: qualquer divergência de câmera
//! move o ponto de acerto em unidades de mundo, e o gate mede exactamente isso.

use ph2d_field_eval::wgsl::TapeWgsl;

/// O que a marcha do dispositivo devolve, por pixel.
pub struct DeviceGbuffer {
    pub width: u32,
    pub height: u32,
    /// `t` do acerto, ou negativo quando o raio não acertou.
    pub t: Vec<f32>,
    /// A normal em espaço de **VISTA** — a mesma convenção do G-buffer da CPU.
    pub normal: Vec<[f32; 3]>,
}

impl DeviceGbuffer {
    #[must_use]
    pub fn hit(&self, i: usize) -> bool {
        self.t.get(i).is_some_and(|t| *t >= 0.0)
    }
}

/// Tudo o que a marcha precisa de saber e que **não** sai da fita — os mesmos números que a
/// [`ph2d_field_render::Scene`] carrega.
#[derive(Clone, Copy, Debug)]
pub struct MarchSetup {
    pub half_extent: f32,
    /// `min(w, h) * 0.5` — o `half` do [`ph2d_field_render::Screen`].
    pub half_px: f32,
    pub target: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    /// O `toward_eye` da base. ⚠️ A direcção do raio na paralela é **`-fwd`**.
    pub fwd: [f32; 3],
    /// `ORTHO_START`, ou a distância ao olho quando há lente.
    pub ortho_start: f32,
    /// `0` = paralela. Caso contrário, a distância do olho ao alvo.
    pub eye_distance: f32,
    pub hit_eps: f32,
    pub normal_eps: f32,
    pub step: f32,
    pub budget: u32,
    pub t_max: f32,
}

const MOLDE: &str = r"
struct Setup {
    w: u32, h: u32, budget: u32, _p0: u32,
    half_extent: f32, half_px: f32, ortho_start: f32, eye_distance: f32,
    hit_eps: f32, normal_eps: f32, step: f32, t_max: f32,
    alvo: vec3<f32>, right: vec3<f32>, up: vec3<f32>, fwd: vec3<f32>,
};
@group(0) @binding(0) var<uniform> s: Setup;
@group(0) @binding(1) var<storage, read> k: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<vec4<f32>>;
{FIELD}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;

    // ⚠️ `Screen::plane_at(x + 0.5, y + 0.5)` — o CENTRO do pixel, e o `v` com o sinal trocado.
    let u = (f32(g.x) + 0.5 - f32(s.w) * 0.5) / s.half_px * s.half_extent;
    let v = -(f32(g.y) + 0.5 - f32(s.h) * 0.5) / s.half_px * s.half_extent;

    // `Orbit::ray_at_plane`.
    let on_plane = s.alvo + s.right * u + s.up * v;
    var origin: vec3<f32>;
    var dir: vec3<f32>;
    if (s.eye_distance == 0.0) {
        origin = on_plane + s.fwd * s.ortho_start;
        dir = -s.fwd;
    } else {
        let eye = s.alvo + s.fwd * s.eye_distance;
        let d = on_plane - eye;
        let len = length(d);
        origin = eye;
        if (len <= 0.0) { dir = -s.fwd; } else { dir = d / len; }
    }

    // A marcha: `t += d * passo`, com o orçamento derivado do passo.
    var t = 0.0;
    var acertou = false;
    for (var n: u32 = 0u; n < s.budget; n = n + 1u) {
        let d = field(origin + dir * t);
        if (d < s.hit_eps) { acertou = true; break; }
        t = t + d * s.step;
        if (t >= s.t_max) { break; }
    }
    if (!acertou) { out[i] = vec4<f32>(-1.0, 0.0, 0.0, 0.0); return; }

    // A normal pelo estêncil **Tetra4** — os quatro vértices, e os deslocamentos são os DOIS
    // papéis (onde amostrar, e com que peso somar).
    let p = origin + dir * t;
    let e = s.normal_eps;
    let o0 = vec3<f32>( 1.0, -1.0, -1.0);
    let o1 = vec3<f32>(-1.0, -1.0,  1.0);
    let o2 = vec3<f32>(-1.0,  1.0, -1.0);
    let o3 = vec3<f32>( 1.0,  1.0,  1.0);
    let world = o0 * field(p + o0 * e) + o1 * field(p + o1 * e)
              + o2 * field(p + o2 * e) + o3 * field(p + o3 * e);
    let len = length(world);
    if (len <= 0.0) { out[i] = vec4<f32>(-1.0, 0.0, 0.0, 0.0); return; }
    let nrm = world / len;
    // Para o espaço de VISTA, como o G-buffer da CPU.
    out[i] = vec4<f32>(t, dot(nrm, s.right), dot(nrm, s.up), dot(nrm, s.fwd));
}
";

/// ⭐⭐⭐ **O G-buffer do quadro, marchado no dispositivo.**
///
/// `None` sem adaptador. ⚠️ Ela abre o dispositivo a cada chamada — é a forma de **sonda**; o
/// produto vai segurar o contexto e o cache de pipelines.
#[must_use]
pub fn march(
    fita: &TapeWgsl,
    setup: MarchSetup,
    width: u32,
    height: u32,
) -> Option<DeviceGbuffer> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("marcha do campo"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .ok()?;

    let mut cache = crate::FieldPipelines::new();
    let pipeline = cache.get(&device, MOLDE, fita).clone();

    // O uniforme, campo a campo — a mesma ordem da `struct Setup`. ⚠️ Um `vec3` alinha a 16 B.
    let mut u: Vec<u8> = Vec::with_capacity(112);
    for v in [width, height, setup.budget, 0] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    for f in [
        setup.half_extent,
        setup.half_px,
        setup.ortho_start,
        setup.eye_distance,
        setup.hit_eps,
        setup.normal_eps,
        setup.step,
        setup.t_max,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    for v in [setup.target, setup.right, setup.up, setup.fwd] {
        for f in v {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes()); // o padding do `vec3`
    }

    use wgpu::util::DeviceExt;
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("setup"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let consts = if fita.consts.is_empty() {
        vec![0.0f32]
    } else {
        fita.consts.clone()
    };
    let mut kb_bytes = Vec::with_capacity(consts.len() * 4);
    for c in &consts {
        kb_bytes.extend_from_slice(&c.to_le_bytes());
    }
    let kb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("k"),
        contents: &kb_bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let n = u64::from(width) * u64::from(height);
    let bytes = n * 16;
    let ob = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gbuffer"),
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
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: kb.as_entire_binding(),
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
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    enc.copy_buffer_to_buffer(&ob, 0, &leitura, 0, bytes.max(16));
    queue.submit([enc.finish()]);

    let fatia = leitura.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = fatia.get_mapped_range();
    let quads = dados.as_chunks::<16>().0;
    let mut t = Vec::with_capacity(quads.len());
    let mut normal = Vec::with_capacity(quads.len());
    for q in quads {
        let f = |o: usize| f32::from_le_bytes([q[o], q[o + 1], q[o + 2], q[o + 3]]);
        t.push(f(0));
        normal.push([f(4), f(8), f(12)]);
    }
    drop(dados);
    leitura.unmap();

    Some(DeviceGbuffer {
        width,
        height,
        t,
        normal,
    })
}
