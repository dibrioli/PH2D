//! Diagnóstico (à mão) do bug 7 (`docs/Luz2D/BUGS_luz2d.md`): o que muda de placa para placa, medido na
//! placa e não suposto — o arredondamento do meio float, o vão que o alocador conta, o sinal do erro do
//! rebote. Corre-se nas DUAS placas e compara-se a saída:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-light2d-gpu --test it bug7_ -- --ignored --nocapture
//! ```
//! (a Radeon: o bloco do bug 7.)

use half::f16;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};

use crate::cenas::DEGRAUS;
use crate::comum::{gpu, grade, ler_bytes, ler_fluencia};

fn placa(gpu: &ph2d_gpu::GpuContext) -> String {
    let i = gpu.adapter.get_info();
    format!("{} ({:?})", i.name, i.backend)
}

/// O f16 abaixo de `x` (para `x > 0`): o que o arredondamento PARA ZERO dá.
fn para_zero(x: f32) -> f16 {
    let p = f16::from_f32(x);
    if p.to_f32() > x {
        f16::from_bits(p.to_bits() - 1)
    } else {
        p
    }
}

/// ⭐ Os valores que separam os dois arredondamentos: 3/4 de ULP acima de um f16 (o mais próximo sobe, o
/// para-zero desce) e os meios exatos (o mais próximo vai ao par).
fn amostras() -> Vec<f32> {
    let mut v = Vec::new();
    let mut s = 0x1234_5678_u32;
    while v.len() < 65_536 {
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        // f16 normais positivos de 2^-10 a 2^10.
        let b = 0x1400 + (s >> 16) % (0x6400 - 0x1400);
        let (a, c) = (
            f16::from_bits(b as u16).to_f32(),
            f16::from_bits(b as u16 + 1).to_f32(),
        );
        v.push(if v.len() % 2 == 0 {
            a + 0.75 * (c - a)
        } else {
            a + 0.5 * (c - a)
        });
    }
    v
}

#[test]
#[ignore = "diagnóstico do bug 7: corre-se à mão"]
fn bug7_o_arredondamento_do_meio_float() {
    let Some(gpu) = gpu() else { return };
    let cont = arredondamentos(&gpu);
    eprintln!("bug7 · {} · [mais próximo, para zero, outro]", placa(&gpu));
    for (nome, c) in [
        "pack2x16float ¾ ULP",
        "pack2x16float meio",
        "textura f16 ¾ ULP",
        "textura f16 meio",
    ]
    .iter()
    .zip(cont)
    {
        eprintln!("  {nome}: {c:?}");
    }
}

/// Como a placa arredonda f32 → f16: `[pack ¾ ULP, pack meio, textura ¾ ULP, textura meio]` × `[mais
/// próximo, para zero, outro]` sobre as [`amostras`].
pub fn arredondamentos(gpu: &ph2d_gpu::GpuContext) -> [[usize; 3]; 4] {
    let x = amostras();
    let d = &gpu.device;
    let src = "
@group(0) @binding(0) var<storage, read> x: array<f32>;
@group(0) @binding(1) var<storage, read_write> p: array<u32>;
@group(0) @binding(2) var t: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if i >= arrayLength(&x) { return; }
    p[i] = pack2x16float(vec2<f32>(x[i], 0.0));
    textureStore(t, vec2<i32>(i32(i % 256u), i32(i / 256u)), vec4<f32>(x[i], 0.0, 0.0, 1.0));
}";
    use wgpu::util::DeviceExt;
    let m = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bug7 f16"),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });
    let pipe = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &m,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bx = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&x),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let bytes = 4 * x.len() as u64;
    let bp = d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let tex = d.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let vista = tex.create_view(&Default::default());
    let bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipe.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: bx.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: bp.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&vista),
            },
        ],
    });
    let mut enc = d.create_command_encoder(&Default::default());
    {
        let mut c = enc.begin_compute_pass(&Default::default());
        c.set_pipeline(&pipe);
        c.set_bind_group(0, &bg, &[]);
        c.dispatch_workgroups((x.len() as u32).div_ceil(64), 1, 1);
    }
    enc.copy_buffer_to_buffer(&bp, 0, &leitura, 0, bytes);
    gpu.queue.submit([enc.finish()]);
    let f = leitura.slice(..);
    f.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    d.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let p: Vec<u32> = bytemuck::cast_slice(&f.get_mapped_range()).to_vec();
    let t = ler_bytes(gpu, &tex);
    let mut cont = [[0usize; 3]; 4]; // [pack ¾, pack ½, textura ¾, textura ½] × [mais próximo, para zero, outro]
    for (i, &xi) in x.iter().enumerate() {
        let (prox, zero) = (f16::from_f32(xi).to_bits(), para_zero(xi).to_bits());
        let meio = i % 2;
        for (k, b) in [
            (meio, (p[i] & 0xffff) as u16),
            (2 + meio, u16::from_le_bytes([t[8 * i], t[8 * i + 1]])),
        ] {
            let c = if b == prox {
                0
            } else if b == zero {
                1
            } else {
                2
            };
            cont[k][c] += 1;
        }
    }
    cont
}

#[test]
#[ignore = "diagnóstico do bug 7: corre-se à mão"]
fn bug7_o_alocador_conta_o_vao() {
    let Some(gpu) = gpu() else { return };
    let d = &gpu.device;
    let usos = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    let tamanhos = [
        16_u64, 512, 4096, 65_536, 524_288, 524_288, 1_048_576, 524_288, 16, 524_288,
    ];
    let _vivos: Vec<wgpu::Buffer> = tamanhos
        .iter()
        .enumerate()
        .map(|(k, &size)| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("bug7/{k:02}_{size}")),
                size,
                usage: usos,
                mapped_at_creation: false,
            })
        })
        .collect();
    let r = d.generate_allocator_report().expect("relatório");
    let mut a: Vec<_> = r
        .allocations
        .iter()
        .filter(|a| a.name.starts_with("bug7/"))
        .collect();
    a.sort_by_key(|a| a.name.clone());
    eprintln!(
        "bug7 · {} · nome | offset | size (o relatório) | offset % 1 MiB",
        placa(&gpu)
    );
    for a in a {
        eprintln!(
            "  {} | {} | {} | {}",
            a.name,
            a.offset,
            a.size,
            a.offset % 1_048_576
        );
    }
    // O solver do G5 que reprova («Médio com o rebote e a cascata holográfica»): cada peça no seu bloco, com o
    // vão que a separa da anterior — o relatório soma esse vão ao tamanho dela.
    let (_, lado, texel) = DEGRAUS[1];
    let _s = Solver::novo(
        &gpu,
        Config {
            rebote: true,
            holografica: Some(HOLOGRAFICA_PADRAO),
            ..Config::padrao(grade(lado, texel))
        },
    )
    .expect("solver");
    let r = d.generate_allocator_report().expect("relatório");
    eprintln!("bug7 · o mapa do G5: bloco | offset | size (o relatório) | fim da anterior | nome");
    for (b, bloco) in r.blocks.iter().enumerate() {
        let mut fim = None;
        for a in &r.allocations[bloco.allocations.clone()] {
            if a.name.starts_with("luz2d/") || a.name.starts_with("bug7/") {
                let anterior = fim.map_or_else(|| "-".to_owned(), |f: u64| f.to_string());
                eprintln!(
                    "  {b} | {} | {} | {anterior} | {}",
                    a.offset, a.size, a.name
                );
            }
            fim = Some(a.offset + a.size);
        }
    }
}

#[test]
#[ignore = "diagnóstico do bug 7: corre-se à mão"]
fn bug7_o_sinal_do_rebote() {
    use crate::g9_rebote::{luz_na_placa, pontos, sala};
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.01,
            rebotes: 300,
        },
    );
    eprintln!(
        "bug7 · {} · G9 holográfica: (placa/gabarito − 1) por canal",
        placa(&gpu)
    );
    for (degrau, lado, texel) in DEGRAUS {
        let cfg = Config {
            rebote: true,
            holografica: Some(HOLOGRAFICA_PADRAO),
            ..Config::padrao(grade(lado, texel))
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        for _ in 0..60 {
            s.quadro(&gpu, &cena);
        }
        let flu = ler_fluencia(&gpu, &s);
        let gf = s.grade_da_fluencia();
        for p in pontos() {
            let (l, r) = (luz_na_placa(&flu, &gf, p), serie.luz_em(p));
            let e: Vec<String> = (0..3)
                .map(|c| format!("{:+.4}", l[c] / r[c] - 1.0))
                .collect();
            eprintln!("  {degrau} {p:?}: {}", e.join(" "));
        }
    }
}

/// O custo da cura da holográfica (`Config::hrc_meio_float_mais_proximo`): a soma do `c2` com e sem ela, no
/// mesmo processo, intercalado, o MÍNIMO de 9 rodadas, cada degrau (`docs/DevOps/MEDIR_VELOCIDADE.md`); o
/// controlo é o sem-a-cura contra si mesmo.
#[test]
#[ignore = "diagnóstico do bug 7: corre-se à mão"]
fn bug7_o_custo_do_f16_mais_proximo() {
    use crate::g9_rebote::sala;
    let Some(gpu) = gpu() else { return };
    let cena = sala();
    eprintln!(
        "bug7 · {} · c2 da holográfica (ms, mínimo de 9): com a cura · sem · sem de novo",
        placa(&gpu)
    );
    for (degrau, lado, texel) in DEGRAUS {
        let mut s: Vec<Solver> = [true, false, false]
            .into_iter()
            .map(|cura| {
                Solver::novo(
                    &gpu,
                    Config {
                        rebote: true,
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        hrc_meio_float_mais_proximo: cura,
                        ..Config::padrao(grade(lado, texel))
                    },
                )
                .expect("solver")
            })
            .collect();
        let mut minimo = [f64::INFINITY; 3];
        for x in &mut s {
            for _ in 0..5 {
                x.quadro(&gpu, &cena);
            }
        }
        for rodada in 0..9 {
            for o in 0..3 {
                let k = (o + rodada) % 3;
                for _ in 0..3 {
                    s[k].quadro(&gpu, &cena);
                }
                let t = s[k].tempos(&gpu).expect("TIMESTAMP_QUERY");
                minimo[k] = minimo[k].min(t.c2.iter().sum());
            }
        }
        eprintln!(
            "  {degrau}: {:.3} · {:.3} · {:.3} (razão {:.3}, controlo {:.3})",
            minimo[0],
            minimo[1],
            minimo[2],
            minimo[0] / minimo[1],
            minimo[2] / minimo[1]
        );
    }
}
