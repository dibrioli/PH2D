//! n.º 10 F5j: as duas réguas de placa do B20 que medem ANTES de construir (sondas ignoradas, precisam de um device).
//!
//! - `sonda_b20_3_cadeia_de_mips`: o CPU por chamada de `MipGenerator::run` e `run_rect` numa textura 4K (12 níveis: 2
//!   views + 1 bind group por nível + um `submit` próprio). Se o mínimo for < 0,1 ms, guardar views/bind groups em
//!   cache por entrada não paga a complexidade.
//! - `sonda_b20_2_pilha_abaixo_da_activa`: o compositor a 4K com `PH2D_SONDA_CAMADAS` (16) camadas, só a de CIMA muda
//!   (um retângulo de 256² por chamada, como um traço): o custo de recompor a pilha inteira debaixo dela por quadro, com
//!   a placa esperada. É o teto do que um cache f32 da pilha de baixo (B20.2) poderia poupar.
//!
//! Corra (a vez da placa): `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-render --profile smoke --test it
//! -- --ignored --nocapture --test-threads=1 sonda_b20_`.

use super::layer_compositor_gpu::try_headless_gpu;
use ph2d_render::mipgen::{MipGenerator, mip_levels};
use ph2d_render::{LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region};
use std::time::Instant;

const W: u32 = 3840;
const H: u32 = 2160;

fn minimo_e_mediana(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

#[test]
#[ignore = "sonda: precisa de um device (a vez da placa)"]
fn sonda_b20_3_cadeia_de_mips() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem placa");
        return;
    };
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sonda mipgen 4K"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_levels(W, H),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let mg = MipGenerator::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb);
    let n = mip_levels(W, H);
    for (rot, rect) in [
        ("run", None),
        ("run_rect 256²", Some((1800, 1000, 256, 256))),
    ] {
        let (mut cpu, mut tudo) = (Vec::new(), Vec::new());
        for _ in 0..120 {
            let t0 = Instant::now();
            match rect {
                None => mg.run(&gpu, &tex, n),
                Some(r) => mg.run_rect(&gpu, &tex, n, r),
            }
            cpu.push(t0.elapsed().as_secs_f64() * 1e3);
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            tudo.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        let (c0, c50) = minimo_e_mediana(cpu);
        let (g0, g50) = minimo_e_mediana(tudo);
        println!(
            "B20.3\t{rot}\t{n} níveis\tCPU mín {c0:.3} · med {c50:.3} ms\tcom a placa mín {g0:.3} · med {g50:.3} ms"
        );
    }
}

struct Pilha {
    camadas: Vec<Vec<u8>>,
    versoes: Vec<u64>,
    sujo: Option<Region>,
}

impl LayerPixelProvider for Pilha {
    fn layer_pixels(&self, key: u64) -> Option<LayerPixels<'_>> {
        let i = key as usize;
        let topo = i + 1 == self.camadas.len();
        self.camadas.get(i).map(|b| LayerPixels {
            version: self.versoes[i],
            rgba8: b,
            dirty: if topo { self.sujo } else { None },
            dirty_parts: &[],
        })
    }
}

#[test]
#[ignore = "sonda: precisa de um device (a vez da placa)"]
fn sonda_b20_2_pilha_abaixo_da_activa() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem placa");
        return;
    };
    let n: usize = std::env::var("PH2D_SONDA_CAMADAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16);
    let px = (W * H * 4) as usize;
    let mut pilha = Pilha {
        camadas: (0..n)
            .map(|i| {
                let mut v = vec![0u8; px];
                for (k, p) in v.chunks_exact_mut(4).enumerate() {
                    let a = ((k + i * 7919) % 97) as u8;
                    p.copy_from_slice(&[a.wrapping_mul(3), a.wrapping_mul(5), a, 40 + a]);
                }
                v
            })
            .collect(),
        versoes: vec![1; n],
        sujo: None,
    };
    let ops: Vec<LayerOp> = (0..n as u64)
        .map(|key| LayerOp::Layer {
            mask: None,
            clipping: false,
            key,
            blend_mode: 0,
            opacity: 0.9,
        })
        .collect();
    let mut comp = LayerCompositor::new(&gpu);
    comp.composite(&gpu, &ops, &pilha, W, H, Region::full(W, H))
        .expect("semente");
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let rect = Region {
        x: 1800,
        y: 1000,
        w: 256,
        h: 256,
    };
    for (rot, regiao) in [("região 256²", rect), ("tela inteira", Region::full(W, H))] {
        let mut ms = Vec::new();
        for k in 0..60u64 {
            pilha.versoes[n - 1] += 1;
            pilha.sujo = Some(rect);
            let byte = (k % 200) as u8;
            pilha.camadas[n - 1][((rect.y * W + rect.x) * 4) as usize] = byte;
            let t0 = Instant::now();
            comp.composite(&gpu, &ops, &pilha, W, H, regiao)
                .expect("composta");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        let (m0, m50) = minimo_e_mediana(ms);
        println!("B20.2\t{n} camadas 4K\tsó a de cima muda\t{rot}\tmín {m0:.3} · med {m50:.3} ms");
    }
}
