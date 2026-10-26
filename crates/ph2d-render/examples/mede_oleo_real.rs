//! W0(a) do «Óleo Real» (`docs/Painter/49_realshader/plano.md`): o custo de placa de um passe de óptica
//! (Sobel + AO por horizonte `N`×`K` + sombra macia por cone `K`) ANTES de escolher qualquer constante.
//!
//! Protótipo fiel no CUSTO, não no look. Régua de `docs/DevOps/MEDIR_VELOCIDADE.md`: todas as variantes
//! no MESMO processo (`override N/K/MODO` do WGSL), aquecidas fora da régua, intercaladas em blocos curtos
//! por ordem rodada; resumo = MÍNIMO das rodadas (mediana ao lado); relógio = timestamps da placa.
//!
//! `bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo run --profile smoke -p ph2d-render --example mede_oleo_real`

use ph2d_gpu::GpuContext;
use ph2d_render::{IMPASTO_MAX_LIGHTS, ImpastoLamp, ImpastoLightInput, ImpastoLightPass};

const RODADAS: usize = 7;
const BLOCO: u32 = 10;
const RESOLUCOES: [(u32, u32); 2] = [(1920, 1080), (3840, 2160)];
const KS: [u32; 3] = [8, 16, 32];
const NS: [u32; 2] = [4, 8];

const WGSL: &str = r#"
override N: u32 = 4u;
override K: u32 = 8u;
// 0 = só copia o albedo (as mesmas ligações), 1 = a óptica inteira.
override MODO: u32 = 1u;

struct U {
    // xy = direcção da luz (azimute), z = tan(altitude), w = ganho de profundidade
    luz: vec4<f32>,
    // xy = tamanho, z = c do cone da penumbra
    dims: vec4<f32>,
};

@group(0) @binding(0) var altura: texture_2d<f32>;
@group(0) @binding(1) var albedo: texture_2d<f32>;
@group(0) @binding(2) var saida: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> u: U;

fn h_em(p: vec2<i32>) -> f32 {
    let lim = vec2<i32>(u.dims.xy) - vec2<i32>(1);
    return textureLoad(altura, clamp(p, vec2<i32>(0), lim), 0).r;
}

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let tam = vec2<u32>(u.dims.xy);
    if (gid.x >= tam.x || gid.y >= tam.y) { return; }
    let p = vec2<i32>(gid.xy);
    let a = textureLoad(albedo, p, 0);
    if (MODO == 0u) {
        textureStore(saida, p, a);
        return;
    }
    // Sobel 3×3
    let tl = h_em(p + vec2<i32>(-1, -1)); let tc = h_em(p + vec2<i32>(0, -1)); let tr = h_em(p + vec2<i32>(1, -1));
    let ml = h_em(p + vec2<i32>(-1, 0));                                       let mr = h_em(p + vec2<i32>(1, 0));
    let bl = h_em(p + vec2<i32>(-1, 1));  let bc = h_em(p + vec2<i32>(0, 1));  let br = h_em(p + vec2<i32>(1, 1));
    let gx = (tr + 2.0 * mr + br) - (tl + 2.0 * ml + bl);
    let gy = (bl + 2.0 * bc + br) - (tl + 2.0 * tc + tr);
    let n = normalize(vec3<f32>(-gx * u.luz.w, -gy * u.luz.w, 8.0));
    let h0 = h_em(p);
    let pf = vec2<f32>(p);
    // AO por horizonte: N direcções, K passos lineares de 1 px (t = 1..K)
    var occ = 0.0;
    for (var d = 0u; d < N; d = d + 1u) {
        let ang = 6.2831853 * f32(d) / f32(N);
        let dir = vec2<f32>(cos(ang), sin(ang));
        var m = 0.0;
        for (var s = 1u; s <= K; s = s + 1u) {
            let t = f32(s);
            let q = vec2<i32>(round(pf + dir * t));
            m = max(m, (h_em(q) - h0) / t);
        }
        occ = occ + m * inverseSqrt(1.0 + m * m);
    }
    let ao = 1.0 - occ / f32(N);
    // Sombra macia: K passos de 1 px na direcção da luz, penumbra por cone
    var pen = 1.0;
    for (var s = 1u; s <= K; s = s + 1u) {
        let t = f32(s);
        let q = vec2<i32>(round(pf + u.luz.xy * t));
        let raio = h0 + t * u.luz.z;
        pen = min(pen, u.dims.z * (raio - h_em(q)) / t);
    }
    pen = clamp(pen, 0.0, 1.0);
    let l = normalize(vec3<f32>(u.luz.xy, u.luz.z));
    let dif = max(dot(n, l), 0.0) * pen;
    let rgb = a.rgb * (0.25 * ao + 0.75 * dif);
    textureStore(saida, p, vec4<f32>(rgb, a.a));
}
"#;

struct Tela {
    w: u32,
    h: u32,
    altura: Vec<f32>,
    albedo: Vec<u8>,
    bg: wgpu::BindGroup,
    albedo_tex: wgpu::Texture,
}

enum Variante {
    Oleo {
        pipe: wgpu::ComputePipeline,
        tela: usize,
    },
    Impasto {
        pass: ImpastoLightPass,
        tela: usize,
    },
}

/// Campo de altura plausível: algumas cristas suaves (até ~14 px) + ruído de valor (~±3 px), 0..~20 px.
fn campo(w: u32, h: u32) -> Vec<f32> {
    let mut v = Vec::with_capacity((w * h) as usize);
    let hash = |x: i32, y: i32| -> f32 {
        let mut n = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
        n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
        ((n ^ (n >> 16)) & 0xffff) as f32 / 65535.0
    };
    let ruido = |x: f32, y: f32| -> f32 {
        let (xi, yi) = (x.floor() as i32, y.floor() as i32);
        let (fx, fy) = (x - x.floor(), y - y.floor());
        let a = hash(xi, yi) + (hash(xi + 1, yi) - hash(xi, yi)) * fx;
        let b = hash(xi, yi + 1) + (hash(xi + 1, yi + 1) - hash(xi, yi + 1)) * fx;
        a + (b - a) * fy
    };
    let s = 1920.0 / w as f32;
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 * s, y as f32 * s);
            let mut c = 0.0f32;
            for k in 0..5 {
                let ang = 0.7 * k as f32 + 0.3;
                let d = fx * ang.cos() + fy * ang.sin() - 250.0 * k as f32;
                let off = 120.0 * ((fx + fy) * 0.004 + k as f32).sin();
                let r = (d + off).rem_euclid(380.0) - 190.0;
                c = c.max(14.0 * (-r * r / (2.0 * 25.0 * 25.0)).exp());
            }
            let nz = 3.0 * (ruido(fx / 4.0, fy / 4.0) - 0.5) + 1.5 * ruido(fx / 1.5, fy / 1.5);
            v.push((c + nz + 2.0).clamp(0.0, 20.0));
        }
    }
    v
}

fn tex(gpu: &GpuContext, w: u32, h: u32, f: wgpu::TextureFormat, u: wgpu::TextureUsages) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mede_oleo_real"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: f,
        usage: u,
        view_formats: &[],
    })
}

fn sobe(gpu: &GpuContext, t: &wgpu::Texture, bytes: &[u8], linha: u32, w: u32, h: u32) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(linha), rows_per_image: Some(h) },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim().to_string()
}

fn main() {
    eprintln!("loadavg (início): {}", loadavg());
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("GpuContext");
    let info = gpu.adapter.get_info();
    eprintln!(
        "placa: {} · {:?} · {:?} · driver {} {}",
        info.name, info.backend, info.device_type, info.driver, info.driver_info
    );
    if !gpu.device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        eprintln!("⛔ o dispositivo não tem TIMESTAMP_QUERY — sem relógio de placa, nada medido");
        std::process::exit(3);
    }
    let periodo = f64::from(gpu.queue.get_timestamp_period());

    let modulo = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mede_oleo_real"),
        source: wgpu::ShaderSource::Wgsl(WGSL.into()),
    });
    let ent = |binding: u32, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    };
    let lida = wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable: false },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    };
    let bgl = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mede_oleo_real"),
        entries: &[
            ent(0, lida),
            ent(1, lida),
            ent(
                2,
                wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
            ),
            ent(
                3,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            ),
        ],
    });
    let layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mede_oleo_real"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let pipeline = |n: u32, k: u32, modo: u32| {
        let c = [("N", f64::from(n)), ("K", f64::from(k)), ("MODO", f64::from(modo))];
        gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("mede_oleo_real"),
            layout: Some(&layout),
            module: &modulo,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &c,
                zero_initialize_workgroup_memory: true,
            },
            cache: None,
        })
    };

    // Luz: azimute 135°, altitude 42° (a do M6), ganho 1; c do cone 4.
    let (az, alt) = (135f32.to_radians(), 42f32.to_radians());
    let uni: [f32; 8] = [az.cos(), az.sin(), alt.tan(), 1.0, 0.0, 0.0, 4.0, 0.0];

    let mut telas = Vec::new();
    for &(w, h) in &RESOLUCOES {
        let altura = campo(w, h);
        let albedo: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                let (x, y) = (i % w, i / w);
                [(x * 255 / w) as u8, (y * 255 / h) as u8, 140, 255]
            })
            .collect();
        use wgpu::TextureUsages as TU;
        let ta = tex(&gpu, w, h, wgpu::TextureFormat::R32Float, TU::TEXTURE_BINDING | TU::COPY_DST);
        sobe(&gpu, &ta, bytemuck::cast_slice(&altura), w * 4, w, h);
        let tb = tex(&gpu, w, h, wgpu::TextureFormat::Rgba8Unorm, TU::TEXTURE_BINDING | TU::COPY_DST);
        sobe(&gpu, &tb, &albedo, w * 4, w, h);
        let ts = tex(&gpu, w, h, wgpu::TextureFormat::Rgba8Unorm, TU::STORAGE_BINDING);
        let mut un = uni;
        un[4] = w as f32;
        un[5] = h as f32;
        let ub = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mede_oleo_real u"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&ub, 0, bytemuck::cast_slice(&un));
        let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let (va, vb, vs) = (v(&ta), v(&tb), v(&ts));
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mede_oleo_real"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&va) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&vb) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&vs) },
                wgpu::BindGroupEntry { binding: 3, resource: ub.as_entire_binding() },
            ],
        });
        telas.push(Tela { w, h, altura, albedo, bg, albedo_tex: tb });
    }

    let mut variantes: Vec<(String, Variante)> = Vec::new();
    for (ti, &(w, h)) in RESOLUCOES.iter().enumerate() {
        variantes.push((format!("{w}×{h} · só cópia"), Variante::Oleo { pipe: pipeline(1, 1, 0), tela: ti }));
        variantes.push((format!("{w}×{h} · ImpastoLightPass (4 lâmpadas)"), Variante::Impasto {
            pass: ImpastoLightPass::new(&gpu),
            tela: ti,
        }));
        for &k in &KS {
            for &n in &NS {
                variantes.push((
                    format!("{w}×{h} · K={k} · N={n}"),
                    Variante::Oleo { pipe: pipeline(n, k, 1), tela: ti },
                ));
            }
        }
    }

    // Entrada do Impasto por tela (planos inteiros; o upload fica fora do relógio do passe).
    let lut: Vec<f32> = (0..256 * 8).map(|i| (i % 256) as f32 / 255.0).collect();
    let lamps: Vec<ImpastoLamp> = (0..IMPASTO_MAX_LIGHTS.min(4))
        .map(|i| {
            let a = i as f32 * 1.57 + 0.6;
            let d = [a.cos() * 0.6, a.sin() * 0.6, 0.53];
            ImpastoLamp { dir: d, half: [d[0] * 0.5, d[1] * 0.5, 0.86], tint: [0.3; 3] }
        })
        .collect();
    let planos: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = telas
        .iter()
        .map(|t| {
            let n = (t.w * t.h) as usize;
            (vec![255u8; n], [200u8, 128, 0, 0].repeat(n), [0u8; 4].repeat(n))
        })
        .collect();
    let entrada = |ti: usize| {
        let t = &telas[ti];
        let reg = ph2d_render::layer_compositor::Region::full(t.w, t.h);
        ImpastoLightInput {
            width: t.w,
            height: t.h,
            region: reg,
            plane_region: reg,
            relief: &t.altura,
            cover: &planos[ti].0,
            mat0: &planos[ti].1,
            mat1: &planos[ti].2,
            lamps: &lamps,
            spec_lut: &lut,
            lut_width: 256,
            rough_levels: 8,
            paper_body: 1.0,
        }
    };
    let _ = &telas[0].albedo;

    let qs = gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("mede_oleo_real"),
        ty: wgpu::QueryType::Timestamp,
        count: 2 * BLOCO,
    });
    let resolve = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mede_oleo_real resolve"),
        size: u64::from(2 * BLOCO) * 8,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mede_oleo_real leitura"),
        size: u64::from(2 * BLOCO) * 8,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    ph2d_gpu::pass_profiler::init_forced(&gpu.device, &gpu.queue);

    // Um bloco = BLOCO despachos; devolve a média por despacho em ms (relógio da placa).
    let bloco = |v: &mut Variante| -> f64 {
        match v {
            Variante::Oleo { pipe, tela } => {
                let t = &telas[*tela];
                let mut enc = gpu.device.create_command_encoder(&Default::default());
                for i in 0..BLOCO {
                    let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("mede_oleo_real"),
                        timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                            query_set: &qs,
                            beginning_of_pass_write_index: Some(2 * i),
                            end_of_pass_write_index: Some(2 * i + 1),
                        }),
                    });
                    p.set_pipeline(pipe);
                    p.set_bind_group(0, &t.bg, &[]);
                    p.dispatch_workgroups(t.w.div_ceil(8), t.h.div_ceil(8), 1);
                }
                enc.resolve_query_set(&qs, 0..2 * BLOCO, &resolve, 0);
                enc.copy_buffer_to_buffer(&resolve, 0, &leitura, 0, u64::from(2 * BLOCO) * 8);
                gpu.queue.submit([enc.finish()]);
                leitura.slice(..).map_async(wgpu::MapMode::Read, |r| r.expect("map"));
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                let soma: u64 = {
                    let d = leitura.slice(..).get_mapped_range();
                    let ts: &[u64] = bytemuck::cast_slice(&d);
                    (0..BLOCO as usize).map(|i| ts[2 * i + 1].saturating_sub(ts[2 * i])).sum()
                };
                leitura.unmap();
                soma as f64 * periodo / 1e6 / f64::from(BLOCO)
            }
            Variante::Impasto { pass, tela } => {
                let ti = *tela;
                for _ in 0..BLOCO {
                    pass.run(&gpu, &telas[ti].albedo_tex, &entrada(ti)).expect("impasto run");
                    ph2d_gpu::pass_profiler::end_frame(&gpu.device, &gpu.queue);
                }
                let t = ph2d_gpu::pass_profiler::drain(&gpu.device).expect("profiler");
                let (_, ns, n) = t
                    .per_label
                    .iter()
                    .find(|l| l.0 == "render.impasto_light")
                    .copied()
                    .expect("amostra do impasto");
                ns as f64 / 1e6 / f64::from(n.max(1))
            }
        }
    };

    // Aquecimento fora da régua.
    for (_, v) in variantes.iter_mut() {
        bloco(v);
    }
    let nv = variantes.len();
    let mut amostras: Vec<Vec<f64>> = vec![Vec::new(); nv];
    for r in 0..RODADAS {
        for k in 0..nv {
            let i = (k + r * 5) % nv;
            amostras[i].push(bloco(&mut variantes[i].1));
        }
    }
    ph2d_gpu::pass_profiler::shutdown();

    println!("\n## Custo de placa por despacho (ms): MÍNIMO de {RODADAS} rodadas (mediana), blocos de {BLOCO}");
    println!("placa: {} ({:?})", info.name, info.backend);
    println!("| variante | min ms | mediana ms |\n|---|---|---|");
    for (i, (nome, _)) in variantes.iter().enumerate() {
        let mut a = amostras[i].clone();
        a.sort_by(f64::total_cmp);
        println!("| {nome} | {:.3} | {:.3} |", a[0], a[a.len() / 2]);
    }
    eprintln!("loadavg (fim): {}", loadavg());
}
