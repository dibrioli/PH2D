//! ⭐ doc 121 §9.19 (5) — **o preço de um DESPACHO na placa**: a prova de custo do contacto por cores no
//! dispositivo, que pede `~1 000` despachos dependentes por tique (`8` sub-passos × separação, montagem,
//! cores e `8` iterações × as cores). Um passe de cálculo com `k` despachos de `4 096` fios que dependem um do
//! outro (cada um lê o que o anterior escreveu), e o mesmo com um passe por despacho; o relógio é a parede
//! do `submit` até ao fim, menos a de `k = 1`, por despacho. O mínimo de `9` rodadas.

use super::*;

const SHADER: &str = "
@group(0) @binding(0) var<storage, read_write> buf: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {
    if g.x < arrayLength(&buf) {
        buf[g.x] = buf[g.x] * 0.999 + 1.0;
    }
}
";

#[test]
#[ignore = "sonda de relógio"]
fn custo_de_um_despacho() {
    let Some(gpu) = gpu() else { return };
    let d = &gpu.device;
    let modulo = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("despachos"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("despachos"),
        layout: None,
        module: &modulo,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let buf = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("despachos"),
        size: 4096 * 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let grupo = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("despachos"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buf.as_entire_binding(),
        }],
    });
    let corre = |k: u32, um_passe: bool| {
        let t0 = std::time::Instant::now();
        let mut enc = d.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        if um_passe {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            p.set_pipeline(&pipeline);
            p.set_bind_group(0, &grupo, &[]);
            for _ in 0..k {
                p.dispatch_workgroups(64, 1, 1);
            }
        } else {
            for _ in 0..k {
                let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                p.set_pipeline(&pipeline);
                p.set_bind_group(0, &grupo, &[]);
                p.dispatch_workgroups(64, 1, 1);
            }
        }
        gpu.queue.submit([enc.finish()]);
        let _ = d.poll(wgpu::PollType::wait_indefinitely());
        t0.elapsed().as_secs_f64() * 1e3
    };
    for _ in 0..3 {
        corre(1000, true);
    }
    eprintln!("DESPACHO placa «{}»", gpu.adapter.get_info().name);
    for um_passe in [true, false] {
        let minimo = |k: u32| (0..9).map(|_| corre(k, um_passe)).fold(f64::INFINITY, f64::min);
        let base = minimo(1);
        for k in [100_u32, 1000] {
            let ms = minimo(k);
            eprintln!(
                "DESPACHO {} · {k} despachos: {ms:.3} ms · {:.2} µs por despacho",
                if um_passe { "um passe" } else { "um passe por despacho" },
                (ms - base) * 1e3 / f64::from(k - 1)
            );
        }
    }
}
