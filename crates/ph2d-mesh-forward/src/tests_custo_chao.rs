//! ⭐ **O instrumento do CUSTO do céu do chão** — medido ANTES de fixar lado, fatias e passos.
//!
//! Os dois passes do [`crate::gpu_ceu_chao::CeuChao`] sobre uma cobertura SINTÉTICA com a cena do
//! gate (`tests_chao`: a esfera e a caixa, no enquadramento do desenhista), `K` vezes por envio, com
//! o MESMO aparelho (`Features::empty()` + limites do WebGL2). `fatias = 0` é a base (o borrão e o
//! teste de sólido sem marcha).
//!
//! Corra (placa de exclusão):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-mesh-forward --lib
//!  instrumento_custo_ceu_do_chao -- --ignored --nocapture`

use std::time::Instant;

use crate::gpu_ceu_chao::{CeuChao, Parametros};
use crate::gpu_cobertura::COBERTURA_LADO;

const K: u32 = 20;
const ENVIOS: usize = 9;
/// O enquadramento da cobertura da cena do gate (meia-aresta, topo e fundo em mundo).
const MEIA: f32 = 3.56;
const TOPO: f32 = 0.61;
const FUNDO: f32 = 0.62;

/// A cobertura da esfera (`r 0,3` em `x = −0,8`) e da caixa (`0,4` em `x = 0,8`), com os níveis.
fn cobertura(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let n = COBERTURA_LADO.ilog2() + 1;
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("custo cobertura"),
        size: wgpu::Extent3d {
            width: COBERTURA_LADO,
            height: COBERTURA_LADO,
            depth_or_array_layers: 1,
        },
        mip_level_count: n,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let lado = COBERTURA_LADO as usize;
    let z = |y: f32| (TOPO - y) / FUNDO;
    let mut nivel: Vec<[f32; 4]> = (0..lado * lado)
        .map(|k| {
            let x = ((k % lado) as f32 + 0.5) / lado as f32 * 2.0 * MEIA - MEIA;
            let w = MEIA - ((k / lado) as f32 + 0.5) / lado as f32 * 2.0 * MEIA;
            let q = 0.09 - (x + 0.8).powi(2) - w * w;
            if q > 0.0 {
                [1.0, z(0.3 + q.sqrt()), z(0.3 - q.sqrt()), 1.0]
            } else if (x - 0.8).abs() < 0.2 && w.abs() < 0.2 {
                [1.0, z(0.4), z(0.0), 1.0]
            } else {
                [0.0; 4]
            }
        })
        .collect();
    let mut l = lado;
    for m in 0..n {
        let bytes: Vec<u8> = nivel
            .iter()
            .flat_map(|v| v.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8))
            .collect();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: m,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * l as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: l as u32,
                height: l as u32,
                depth_or_array_layers: 1,
            },
        );
        if l == 1 {
            break;
        }
        let h = l / 2;
        nivel = (0..h * h)
            .map(|k| {
                let (i, j) = (2 * (k % h), 2 * (k / h));
                let mut s = [0.0f32; 4];
                for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let v = nivel[(j + dj) * l + i + di];
                    for c in 0..4 {
                        s[c] += 0.25 * v[c];
                    }
                }
                s
            })
            .collect();
        l = h;
    }
    tex.create_view(&wgpu::TextureViewDescriptor::default())
}

fn mede(device: &wgpu::Device, queue: &wgpu::Queue, ceu: &CeuChao) -> f64 {
    let um = || {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        for _ in 0..K {
            ceu.grava_passes(&mut enc);
        }
        let t0 = Instant::now();
        queue.submit([enc.finish()]);
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        t0.elapsed().as_secs_f64() * 1000.0 / f64::from(K)
    };
    for _ in 0..3 {
        um();
    }
    let mut v: Vec<f64> = (0..ENVIOS).map(|_| um()).collect();
    v.sort_by(f64::total_cmp);
    v[ENVIOS / 2]
}

#[test]
#[ignore = "instrumento: precisa de aparelho"]
fn instrumento_custo_ceu_do_chao() {
    let Some((device, queue, adapter)) = crate::gpu_alvo::aparelho_em(wgpu::Backends::all()) else {
        eprintln!("sem aparelho");
        return;
    };
    eprintln!("placa: {:?}", adapter.get_info().name);
    let cob = cobertura(&device, &queue);
    let mut vp = crate::tests::ID;
    vp[1][2] = -1.0 / FUNDO;
    vp[3][2] = TOPO / FUNDO;
    for lado in [256u32, 512] {
        let mut ceu = CeuChao::novo(&device, &cob, lado);
        let mut base = 0.0;
        for (fatias, razao) in [(0u32, 1.25f32), (4, 1.25), (8, 1.25), (4, 1.4), (8, 1.4)] {
            ceu.parametros = Parametros {
                fatias,
                razao,
                ..Parametros::default()
            };
            ceu.prepara(&queue, [MEIA, FUNDO, 0.47, 1.0], &vp, 0.0, &[]);
            let ms = mede(&device, &queue, &ceu);
            if fatias == 0 {
                base = ms;
            }
            eprintln!(
                "lado {lado} · fatias {fatias} · razão {razao}: {ms:.3} ms ({:+.3} sobre a base)",
                ms - base
            );
        }
    }
}
