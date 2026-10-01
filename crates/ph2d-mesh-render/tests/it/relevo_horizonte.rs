//! ⭐⭐ **O HORIZONTE DA NORMAL DO RELEVO, lido nos dois lados** — o
//! `tinta_inclina` do `tinta.wgsl` na placa contra o
//! [`ph2d_mesh_render::relevo_normal::inclina`] na CPU (report do dono de
//! 01/10, 2.ª volta: *«mesma ponta vista de frente e inclinada»*).
//!
//! ⚠️ As entradas são as que a lei VÊ no produto (normal de vista, gradiente
//! em vista, corpo) e varrem as duas regiões — acima do limiar, onde a lei é o
//! gradiente de superfície ao bit, e abaixo, onde a compressão trabalha. O
//! CONTROLO de que a fixtura contém a compressão vem primeiro.

use ph2d_mesh_render::relevo_normal::{HORIZONTE_T, inclina};

const ENTRADA: &str = r#"
@group(0) @binding(0) var<storage, read> entra: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> sai: array<vec4<f32>>;
@compute @workgroup_size(64)
fn cs(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (2u * i + 1u >= arrayLength(&entra)) { return; }
    let a = entra[2u * i];
    let b = entra[2u * i + 1u];
    sai[i] = vec4<f32>(tinta_inclina(a.xyz, b.xyz, a.w), 0.0);
}
"#;

fn amostras() -> Vec<([f32; 3], [f32; 3], f32)> {
    let mut s = 0x2545_f491_4f6c_dd1d_u64;
    let mut r = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 40) as f32 / (1u64 << 24) as f32
    };
    (0..4096)
        .map(|_| {
            let n = [r() * 2.0 - 1.0, r() * 2.0 - 1.0, r() * 2.0 - 1.0];
            let g = [(r() - 0.5) * 12.0, (r() - 0.5) * 12.0, (r() - 0.5) * 12.0];
            (n, g, r() * 1.2 - 0.1)
        })
        .filter(|(n, _, _)| n.iter().map(|v| v * v).sum::<f32>() > 1e-3)
        .collect()
}

/// ⭐⭐⭐ **GATE — a placa inclina a normal e respeita o horizonte como a CPU.**
#[test]
#[ignore = "GPU: precisa de adaptador"]
fn o_horizonte_le_o_mesmo_na_placa_e_na_cpu() {
    use wgpu::util::DeviceExt as _;
    let Some((device, queue)) = super::device_de_teste::device() else {
        eprintln!("sem adaptador — skip");
        return;
    };
    let casos = amostras();
    let comprimidos = casos
        .iter()
        .filter(|(n, g, c)| {
            let cpu = inclina(*n, *g, *c);
            let l = n.iter().map(|v| v * v).sum::<f32>().sqrt();
            let t = (n[2] / l).abs().min(HORIZONTE_T);
            *c > 0.0 && t > 0.0 && (cpu[2] * n[2].signum()) < t
        })
        .count();
    assert!(
        comprimidos > 200,
        "CONTROLO: só {comprimidos} casos caem na compressão"
    );

    let entrada: Vec<f32> = casos
        .iter()
        .flat_map(|(n, g, c)| [n[0], n[1], n[2], *c, g[0], g[1], g[2], 0.0])
        .collect();
    let k = casos.len();
    let b_in = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&entrada),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let tam = (k * 16) as u64;
    let b_out = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: tam,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let b_ler = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: tam,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let fonte = format!("{}\n{ENTRADA}", ph2d_mesh_render::fonte::TINTA_WGSL);
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("horizonte"),
        source: wgpu::ShaderSource::Wgsl(fonte.into()),
    });
    let pipe = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &modulo,
        entry_point: Some("cs"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipe.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: b_in.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: b_out.as_entire_binding(),
            },
        ],
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&pipe);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(k.div_ceil(64) as u32, 1, 1);
    }
    enc.copy_buffer_to_buffer(&b_out, 0, &b_ler, 0, tam);
    queue.submit([enc.finish()]);
    let fatia = b_ler.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let dados = fatia.get_mapped_range();
    let placa: &[f32] = bytemuck::cast_slice(&dados);

    let mut pior = (0.0f32, String::new());
    for (i, (n, g, c)) in casos.iter().enumerate() {
        let cpu = inclina(*n, *g, *c);
        let gpu = &placa[4 * i..4 * i + 3];
        for e in 0..3 {
            let d = (gpu[e] - cpu[e]).abs();
            if d > pior.0 {
                pior = (
                    d,
                    format!("caso {i} eixo {e}: {} contra {}", gpu[e], cpu[e]),
                );
            }
        }
        assert!(
            gpu[2] * n[2].signum() >= 0.0,
            "caso {i}: a placa passou o horizonte ({gpu:?})"
        );
    }
    drop(dados);
    b_ler.unmap();
    eprintln!("pior desvio placa/CPU: {:.3e} ({})", pior.0, pior.1);
    assert!(pior.0 <= 2e-4, "a placa diverge da CPU: {}", pior.1);
}
