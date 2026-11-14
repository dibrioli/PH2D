//! n.º 10 (B20.6) — **sonda**: a subida de uma camada 4K (3840×2160 RGBA8, 33 MB) à placa por `queue.write_texture`
//! (o caminho de `replace_pixels`: o wgpu copia para um staging interno novo) × por um `StagingBelt` reusado (a cópia
//! vai direto ao buffer mapeado e `copy_buffer_to_texture`), intercalados no MESMO processo em blocos ABBA; o MÍNIMO
//! de cada, a parte da CPU (até o `submit` voltar) e a total (até a placa acabar). Só se constrói o anel se B ganhar.
//! Recusa conhecida: `write_texture` em paralelo (o wgpu-core copia com o `pending_writes` trancado).

use ph2d_gpu::GpuContext;
use std::time::{Duration, Instant};

const W: u32 = 3840;
const H: u32 = 2160;

fn textura(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sonda subida 4k"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn destino(t: &wgpu::Texture) -> wgpu::TexelCopyTextureInfo<'_> {
    wgpu::TexelCopyTextureInfo {
        texture: t,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    }
}

const TAMANHO: wgpu::Extent3d = wgpu::Extent3d {
    width: W,
    height: H,
    depth_or_array_layers: 1,
};

/// (CPU, total) de uma subida por `write_texture`.
fn por_write_texture(gpu: &GpuContext, t: &wgpu::Texture, px: &[u8]) -> (Duration, Duration) {
    let t0 = Instant::now();
    gpu.queue.write_texture(
        destino(t),
        px,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        TAMANHO,
    );
    gpu.queue.submit([]);
    let cpu = t0.elapsed();
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    (cpu, t0.elapsed())
}

/// (CPU, total) de uma subida pelo `StagingBelt` reusado.
fn por_belt(
    gpu: &GpuContext,
    belt: &mut wgpu::util::StagingBelt,
    t: &wgpu::Texture,
    px: &[u8],
) -> (Duration, Duration) {
    let t0 = Instant::now();
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    {
        let n = wgpu::BufferSize::new(px.len() as u64).expect("não vazio");
        let al = wgpu::BufferSize::new(wgpu::COPY_BUFFER_ALIGNMENT).expect("alinhamento");
        let fatia = belt.allocate(n, al);
        fatia
            .get_mapped_range_mut()
            .expect("o belt mapeado")
            .copy_from_slice(px);
        enc.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: fatia.buffer(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: fatia.offset(),
                    bytes_per_row: Some(W * 4),
                    rows_per_image: Some(H),
                },
            },
            destino(t),
            TAMANHO,
        );
    }
    belt.finish();
    gpu.queue.submit([enc.finish()]);
    belt.recall();
    let cpu = t0.elapsed();
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    (cpu, t0.elapsed())
}

#[test]
#[ignore = "sonda de placa (B20.6): --ignored --nocapture --test-threads=1, profile smoke"]
fn mede_a_subida_4k_write_texture_contra_staging_belt() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa: a sonda não mede nada");
        return;
    };
    let px: Vec<u8> = (0..(W * H * 4) as usize)
        .map(|i| (i * 31 % 251) as u8)
        .collect();
    let tex = textura(&gpu);
    let mut belt = wgpu::util::StagingBelt::new((*gpu.device).clone(), px.len() as u64);
    // Aquecer os dois (o 1.º belt cria o buffer; o 1.º write_texture o staging do wgpu).
    for _ in 0..2 {
        por_write_texture(&gpu, &tex, &px);
        por_belt(&gpu, &mut belt, &tex, &px);
    }
    let mut a: Vec<(Duration, Duration)> = Vec::new();
    let mut b: Vec<(Duration, Duration)> = Vec::new();
    for bloco in 0..6 {
        for ordem in if bloco % 2 == 0 {
            [0, 1, 1, 0]
        } else {
            [1, 0, 0, 1]
        } {
            if ordem == 0 {
                a.push(por_write_texture(&gpu, &tex, &px));
            } else {
                b.push(por_belt(&gpu, &mut belt, &tex, &px));
            }
        }
    }
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let resumo = |v: &[(Duration, Duration)]| {
        let mut c: Vec<f64> = v.iter().map(|x| ms(x.0)).collect();
        let mut t: Vec<f64> = v.iter().map(|x| ms(x.1)).collect();
        c.sort_by(f64::total_cmp);
        t.sort_by(f64::total_cmp);
        (c[0], c[c.len() / 2], t[0], t[t.len() / 2])
    };
    let (ac, acm, at, atm) = resumo(&a);
    let (bc, bcm, bt, btm) = resumo(&b);
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "B20.6 subida 4K 33 MB · {} amostras cada · load {}",
        a.len(),
        load.trim()
    );
    println!("  write_texture  cpu mín {ac:.2} med {acm:.2} · total mín {at:.2} med {atm:.2} ms");
    println!("  staging_belt   cpu mín {bc:.2} med {bcm:.2} · total mín {bt:.2} med {btm:.2} ms");
    println!(
        "  veredito (mínimo CPU): {}",
        if bc < ac * 0.95 {
            "o belt ganha ≥ 5 %"
        } else {
            "o belt NÃO ganha"
        }
    );
}
