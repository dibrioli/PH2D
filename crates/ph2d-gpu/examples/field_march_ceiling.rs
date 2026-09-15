//! ⭐⭐⭐ **O TECTO DESTA MÁQUINA para marchar um campo implícito** — a medição que o `CLAUDE.md`
//! §0.0 exige antes de qualquer teto, e que o modo *Render* do modelador nunca tinha corrido.
//!
//! # Porque ela existe
//!
//! O traçador do modelador é **todo de CPU**, e os números da `docs/Render3d/05` §27–§31 são todos
//! dela: `28 ms` para um quadro cheio, `123 ms` por passagem de oclusão, `2 s` até assentar. O
//! report do dono — *«muito demorado e em etapas estranhas… bastante inferior a app como Unreal»* —
//! é sobre esses números.
//!
//! ⛔ **E o §0.0 proíbe exactamente isto:** *«nunca deixe o fallback definir o produto… o caminho
//! mais lento definiu o teto do mais rápido, no módulo cuja razão de existir é o mais rápido»*.
//! Esta máquina tem uma **RTX 5060 Ti** parada enquanto a CPU marcha.
//!
//! # O que ela mede
//!
//! A **mesma lei de marcha** do produto (`t += d · passo`, com o mesmo passo seguro e o mesmo
//! orçamento) sobre a **mesma peça** (três cilindros cruzados com união suave), escrita à mão em
//! WGSL. ⚠️ **Não é o produto** — é o TECTO: o que esta máquina faz quando lhe pedem este trabalho.
//! Um traçador a sério tem de compilar a árvore do documento, e é isso que a medição autoriza (ou
//! não) a construir.
//!
//! ```text
//! cargo run --release -p ph2d-gpu --example field_march_ceiling
//! ```

const SHADER: &str = r#"
struct Params {
    width: u32,
    height: u32,
    rays: u32,
    budget: u32,
    half_extent: f32,
    step: f32,
    hit: f32,
    _pad: f32,
};
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read_write> out: array<f32>;

// A peça: três cilindros cruzados, união suave de raio 0,12 — a mesma da sonda de CPU.
fn sd_capped_cylinder(q: vec3<f32>, r: f32, h: f32) -> f32 {
    let d = abs(vec2<f32>(length(q.xz), q.y)) - vec2<f32>(r, h);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0)));
}
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}
fn field(pt: vec3<f32>) -> f32 {
    let a = sd_capped_cylinder(pt, 0.22, 0.78) - 0.05;
    let b = sd_capped_cylinder(vec3<f32>(pt.x, pt.z, pt.y), 0.22, 0.78) - 0.05;
    let c = sd_capped_cylinder(vec3<f32>(pt.y, pt.x, pt.z), 0.22, 0.78) - 0.05;
    return smin(smin(a, b, 0.12), c, 0.12);
}

// A MESMA lei de marcha do produto: `t += d * passo`, com o mesmo orçamento de passos.
fn march(origin: vec3<f32>, dir: vec3<f32>, t_max: f32) -> f32 {
    var t = p.hit * 4.0;
    for (var i: u32 = 0u; i < p.budget; i = i + 1u) {
        let d = field(origin + dir * t);
        if (d < p.hit) { return 0.0; }
        t = t + d * p.step;
        if (t >= t_max) { break; }
    }
    return 1.0;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.width || gid.y >= p.height) { return; }
    let i = gid.y * p.width + gid.x;

    // O raio da câmera: ortográfica, como a do modelador.
    let u = (f32(gid.x) + 0.5) / f32(p.width) * 2.0 - 1.0;
    let v = 1.0 - (f32(gid.y) + 0.5) / f32(p.height) * 2.0;
    let aspect = f32(p.width) / f32(p.height);
    let origin = vec3<f32>(u * p.half_extent * aspect, v * p.half_extent, 3.0);
    let dir = vec3<f32>(0.0, 0.0, -1.0);

    // 1) O TRAÇADO: marcha até acertar, guardando onde parou.
    var t = 0.0;
    var acertou = false;
    var pt = vec3<f32>(0.0);
    for (var k: u32 = 0u; k < p.budget; k = k + 1u) {
        pt = origin + dir * t;
        let d = field(pt);
        if (d < p.hit) { acertou = true; break; }
        t = t + d * p.step;
        if (t > 8.0) { break; }
    }
    if (!acertou) { out[i] = 1.0; return; }

    // 2) A NORMAL, pelo mesmo estêncil de quatro pontos.
    let e = p.hit;
    let n = normalize(vec3<f32>(
        field(pt + vec3<f32>(e, -e, -e)) - field(pt + vec3<f32>(-e, e, e)),
        field(pt + vec3<f32>(-e, e, -e)) - field(pt + vec3<f32>(e, -e, e)),
        field(pt + vec3<f32>(-e, -e, e)) - field(pt + vec3<f32>(e, e, -e)),
    ));

    // 3) A OCLUSÃO: `rays` raios no hemisfério, a mesma cerca do produto.
    var a = vec3<f32>(1.0, 0.0, 0.0);
    if (abs(n.x) > 0.9) { a = vec3<f32>(0.0, 1.0, 0.0); }
    let t1 = normalize(cross(a, n));
    let t2 = cross(n, t1);
    var vis = 0.0;
    let alcance = p.half_extent;
    for (var k: u32 = 0u; k < p.rays; k = k + 1u) {
        let u1 = (f32(k) + 0.5) / f32(p.rays);
        let u2 = fract(f32(k) * 0.618034 + f32(i) * 0.7548777);
        let r = sqrt(u1);
        let phi = 6.2831853 * u2;
        let z = sqrt(max(1.0 - u1, 0.0));
        let d3 = t1 * (r * cos(phi)) + t2 * (r * sin(phi)) + n * z;
        vis = vis + march(pt + n * (p.hit * 4.0), d3, alcance);
    }
    out[i] = vis / f32(p.rays);
}
"#;

fn main() {
    let instance = ph2d_gpu::GpuContext::default_instance();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) else {
        println!("sem adaptador de GPU — nada a medir");
        return;
    };
    let info = adapter.get_info();
    println!("GPU: {} ({:?}, {:?})", info.name, info.device_type, info.backend);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("field march ceiling"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device");

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("field march"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("field march"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    println!(
        "\n  px        · raios/px ·      GPU · a CPU faz (medido, docs/Render3d/05) · ganho"
    );
    for &(w, h) in &[(640u32, 360u32), (1920, 1080)] {
        for &rays in &[0u32, 1, 16] {
            let n = (w as u64) * (h as u64);
            let out = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("out"),
                size: n * 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            #[derive(Clone, Copy)]
            struct P {
                width: u32,
                height: u32,
                rays: u32,
                budget: u32,
                half_extent: f32,
                step: f32,
                hit: f32,
                _pad: f32,
            }
            // ⚠️ Os quatro números são os do PRODUTO: o passo seguro daquele documento, a
            // tolerância de acerto do quadro, e o orçamento de passos da marcha.
            let params = P {
                width: w,
                height: h,
                rays: rays.max(1),
                budget: 512,
                half_extent: 0.8,
                step: 0.7,
                hit: 0.8 * 2.0 / (h as f32) * 0.5,
                _pad: 0.0,
            };
            let ub = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("params"),
                size: std::mem::size_of::<P>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            // ⚠️ Sem `unsafe`: a crate proíbe-o (`-F unsafe-code`), e o uniforme são oito
            // palavras de 32 bits — escrevê-las à mão é honesto e cabe numa linha.
            let mut bytes = Vec::with_capacity(32);
            bytes.extend_from_slice(&params.width.to_le_bytes());
            bytes.extend_from_slice(&params.height.to_le_bytes());
            bytes.extend_from_slice(&params.rays.to_le_bytes());
            bytes.extend_from_slice(&params.budget.to_le_bytes());
            bytes.extend_from_slice(&params.half_extent.to_le_bytes());
            bytes.extend_from_slice(&params.step.to_le_bytes());
            bytes.extend_from_slice(&params.hit.to_le_bytes());
            bytes.extend_from_slice(&params._pad.to_le_bytes());
            queue.write_buffer(&ub, 0, &bytes);
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
                        resource: out.as_entire_binding(),
                    },
                ],
            });

            let corrida = || {
                let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });
                {
                    let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: None,
                        timestamp_writes: None,
                    });
                    cp.set_pipeline(&pipeline);
                    cp.set_bind_group(0, &bind, &[]);
                    cp.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
                }
                queue.submit([enc.finish()]);
                device.poll(wgpu::PollType::wait_indefinitely()).ok();
            };
            corrida(); // aquece (compila o pipeline, aloca)
            let t = std::time::Instant::now();
            const N: u32 = 5;
            for _ in 0..N {
                corrida();
            }
            let ms = t.elapsed().as_secs_f64() * 1e3 / f64::from(N);

            // O que a CPU faz, medido em `docs/Render3d/05` §27–§31, a `load ~3`.
            let cpu = match (w, rays) {
                (640, 0) => 5.80,
                (640, 1) => 5.80 + 12.91,
                (640, 16) => 5.80 + 207.0,
                (_, 0) => 29.18,
                (_, 1) => 29.18 + 123.06,
                _ => 29.18 + 1969.0,
            };
            let que = match rays {
                0 => "só o traçado",
                1 => "traçado + 1 passagem",
                _ => "traçado + oclusão INTEIRA",
            };
            println!(
                "{w:5}×{h:<4} · {rays:8} · {ms:6.2} ms · {cpu:8.2} ms  ({que:24}) · {:5.1}×",
                cpu / ms
            );
        }
    }
}
