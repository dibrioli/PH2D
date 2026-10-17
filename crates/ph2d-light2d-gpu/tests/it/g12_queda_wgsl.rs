//! **G12 (lado WGSL)** — a `queda` do `queda.wgsl` contra a `ph2d_light2d_ref::queda::queda` nos
//! MESMOS 1 000 pontos (`queda::pontos_g12`, a porta partilhada), e o texto dela existe UMA vez:
//! o módulo das cascatas concatena-o, nenhum outro `.wgsl` o redefine.

use ph2d_light2d_gpu::{Precisao, QUEDA_WGSL, fonte_do_rc};
use ph2d_light2d_ref::queda::{pontos_g12, queda};

use crate::comum::gpu;

/// Erro de avaliar em `f32` (`t` e `1/R` arredondados, `x⁴`, o quadrado): a derivada da janela
/// não passa de 2 em módulo, logo o erro absoluto fica na ordem de 2·x·ε₃₂ ≈ 1e-6.
const TOL: f64 = 4e-6;

const HARNESS: &str = r"
@group(0) @binding(0) var<storage, read> entrada: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> saida: array<f32>;
@compute @workgroup_size(64)
fn principal(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x < arrayLength(&entrada) {
        saida[id.x] = queda(entrada[id.x].x, entrada[id.x].y);
    }
}
";

fn avaliar_na_placa(gpu: &ph2d_gpu::GpuContext, pontos: &[[f32; 2]]) -> Vec<f32> {
    use wgpu::util::DeviceExt;
    let d = &gpu.device;
    let fonte = format!("{QUEDA_WGSL}\n{HARNESS}");
    let modulo = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("g12 queda"),
        source: wgpu::ShaderSource::Wgsl(fonte.into()),
    });
    let pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("g12 queda"),
        layout: None,
        module: &modulo,
        entry_point: Some("principal"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let entrada = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("g12 entrada"),
        contents: bytemuck::cast_slice(pontos),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let bytes = (pontos.len() * 4) as u64;
    let saida = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("g12 saída"),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("g12 leitura"),
        size: bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let grupo = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("g12"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: entrada.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: saida.as_entire_binding(),
            },
        ],
    });
    let mut enc = d.create_command_encoder(&Default::default());
    {
        let mut p = enc.begin_compute_pass(&Default::default());
        p.set_pipeline(&pipeline);
        p.set_bind_group(0, &grupo, &[]);
        p.dispatch_workgroups((pontos.len() as u32).div_ceil(64), 1, 1);
    }
    enc.copy_buffer_to_buffer(&saida, 0, &leitura, 0, bytes);
    gpu.queue.submit([enc.finish()]);
    let fatia = leitura.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    d.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let v = bytemuck::cast_slice::<u8, f32>(&fatia.get_mapped_range()).to_vec();
    leitura.unmap();
    v
}

#[test]
fn g12_a_queda_wgsl_e_a_do_gabarito_nos_mesmos_mil_pontos() {
    let Some(gpu) = gpu() else { return };
    let tabela = pontos_g12();
    assert_eq!(tabela.len(), 1000);
    let entrada: Vec<[f32; 2]> = tabela
        .iter()
        .map(|&(t, r)| {
            [
                t as f32,
                if r.is_infinite() {
                    0.0
                } else {
                    (1.0 / r) as f32
                },
            ]
        })
        .collect();
    let placa = avaliar_na_placa(&gpu, &entrada);
    let mut pior = (0.0_f64, 0.0, 0.0);
    for (&(t, r), &q) in tabela.iter().zip(&placa) {
        let erro = (f64::from(q) - queda(t, r)).abs();
        if erro > pior.0 {
            pior = (erro, t, r);
        }
    }
    eprintln!(
        "G12 WGSL: 1000 pontos, |placa − gabarito| máximo = {:e} (t = {}, R = {}; tolerância {TOL:e})",
        pior.0, pior.1, pior.2
    );
    assert!(pior.0 <= TOL, "a gêmea WGSL divergiu: {pior:?}");
}

/// Uma cópia do texto da `queda` divergiria em silêncio: ele vive num ficheiro, e cada módulo que o
/// usa contém-no por concatenação.
#[test]
fn g12_o_texto_da_queda_existe_uma_vez() {
    for a in [Precisao::F16, Precisao::F32] {
        for b in [Precisao::F16, Precisao::F32] {
            for t in [false, true] {
                let fonte = fonte_do_rc(a, b, t);
                assert!(
                    fonte.contains(QUEDA_WGSL),
                    "o RC {a:?}/{b:?}/{t} não concatena queda.wgsl"
                );
                assert_eq!(fonte.matches("fn queda(").count(), 1, "RC {a:?}/{b:?}/{t}");
            }
        }
    }
    let pasta = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/wgsl");
    let mut vistos = 0;
    for e in std::fs::read_dir(&pasta).expect("src/wgsl") {
        let caminho = e.expect("entrada").path();
        if caminho.extension().is_some_and(|x| x == "wgsl") {
            vistos += 1;
            let texto = std::fs::read_to_string(&caminho).expect("ler wgsl");
            let define = texto.contains("fn queda(");
            let e_a_gemea = caminho.file_name().is_some_and(|n| n == "queda.wgsl");
            assert_eq!(define, e_a_gemea, "{}", caminho.display());
        }
    }
    assert!(
        vistos >= 5,
        "o censo não achou os .wgsl em {}",
        pasta.display()
    );
}
