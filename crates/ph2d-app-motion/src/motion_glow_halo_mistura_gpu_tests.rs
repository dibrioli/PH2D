//! ⭐ **O HALO das formas que misturam em grupo** (B6, item 2) — MEDIDO nas duas fontes do halo:
//! - a rota Vello: o TILE de cada forma ([`super::vector_instance_as_tile`]) no passe de sprites
//!   (`render_instances_only` sobre a lista da CPU, sem runs ⇒ `Mix`);
//! - a placa (CPU e dispositivo): [`crate::motion_shape_placa::PlacaDeFormas::redesenha_em`], que
//!   repete as cópias num só desenho.
//!
//! A pergunta: com `Add` entre as cópias, a fonte do halo é a SILHUETA em `Normal` nas duas rotas
//! (concordam), ou uma delas mistura? Medido no `Rgba16Float` do halo, na sobreposição (o pixel
//! de alfa máximo): a placa com as corridas do `Add` contra a placa em `Normal` e contra o tile.
//!
//! `#[ignore]`: precisa de adaptador.
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib --release \
//!        o_halo_das_formas_misturadas -- --ignored --nocapture`

use crate::motion_shape_gen::VecPathStore;
use crate::motion_shape_gen::mistura::gpu_tests::{H, W, copia, try_headless_gpu};
use crate::motion_shape_placa::{Misturas, PlacaDeFormas};
use ph2d_eval_motion::{BlendWith, MisturaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_render::Camera2d;
use ph2d_vector::Affine;

/// Um tom médio meio transparente (o branco opaco saturaria o `Add` e esconderia a diferença).
const TINTA: [f32; 4] = [0.4, 0.5, 0.6, 0.7];
const LADO: u32 = 64;

fn alvo(gpu: &GpuContext, (w, h): (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test halo"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ph2d_render::GameRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let v = t.create_view(&wgpu::TextureViewDescriptor::default());
    (t, v)
}

fn f16(b: u16) -> f32 {
    let s = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
    let e = i32::from((b >> 10) & 0x1f);
    let m = f32::from(b & 0x3ff);
    match e {
        0 => s * m * 2f32.powi(-24),
        31 => f32::NAN,
        _ => s * (1.0 + m / 1024.0) * 2f32.powi(e - 15),
    }
}

/// O RGBA do pixel de alfa máximo (a sobreposição das duas cópias).
fn o_mais_denso(gpu: &GpuContext, t: &wgpu::Texture, (w, h): (u32, u32)) -> [f32; 4] {
    let linha =
        (w * 8).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test leitura do halo"),
        size: u64::from(linha * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let px = buf.slice(..).get_mapped_range();
    let mut melhor = [0.0f32; 4];
    for y in 0..h {
        for x in 0..w {
            let o = (y * linha + x * 8) as usize;
            let c: [f32; 4] = std::array::from_fn(|k| {
                f16(u16::from_le_bytes([px[o + 2 * k], px[o + 2 * k + 1]]))
            });
            if c[3] > melhor[3] {
                melhor = c;
            }
        }
    }
    melhor
}

fn copias(tag: u8) -> Vec<VectorInstance> {
    let mistura = MisturaDoSink {
        blend: tag,
        com: BlendWith::Copies,
        sink: 0,
    };
    let mut c = vec![
        copia(16.0, 24.0, 0.5, mistura, None),
        copia(32.0, 24.0, 0.5, mistura, None),
    ];
    for i in &mut c {
        i.tint = TINTA;
    }
    c
}

/// A fonte do halo pela PLACA: desenha (com as corridas da mistura) e redesenha no alvo do halo.
fn pela_placa(gpu: &GpuContext, tag: u8) -> [f32; 4] {
    let mut store = VecPathStore::default();
    let _ = store.push(ph2d_vec_scene::rectangle([-0.5, -0.5], [0.5, 0.5]));
    let mut geos = crate::motion_shape_placa::GeometriasDaPlaca::default();
    let mut placa = PlacaDeFormas::default();
    assert!(placa.decide_com_misturas(
        true,
        &copias(tag),
        &store,
        &mut geos,
        Affine::IDENTITY,
        Misturas::do_quadro(gpu, false),
    ));
    if tag != 0 {
        assert!(
            !placa.corridas_do_quadro().is_empty(),
            "a cena desenha COM a mistura"
        );
    }
    let _ = placa.desenha(gpu, (W, H), &geos, None).expect("desenha");
    let (t, v) = alvo(gpu, (W, H));
    assert!(placa.redesenha_em(gpu, &v, (W, H)));
    o_mais_denso(gpu, &t, (W, H))
}

/// A fonte do halo pela rota VELLO: o tile de cada forma no passe de sprites.
fn pelo_tile(gpu: &GpuContext) -> [f32; 4] {
    let mut atlas = ph2d_render::TextureAtlas::dummy(gpu);
    let branco = atlas.insert_reserved_tiles(gpu).expect("o tile branco");
    let mut renderer =
        ph2d_render::SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    // As duas cópias no mundo da câmara de omissão (10 de altura), sobrepostas como na placa.
    let tiles: Vec<_> = [0.0f32, 0.6]
        .iter()
        .map(|&x| {
            let vi = VectorInstance {
                world_pos: [x, 0.0],
                size: [1.0, 1.0],
                tint: TINTA,
                ..copias(1)[0]
            };
            let mut t = super::vector_instance_as_tile(&vi, 0);
            t.atlas_uv = branco;
            t
        })
        .collect();
    let (t, v) = alvo(gpu, (LADO, LADO));
    renderer.render_instances_only(
        &v,
        &Camera2d::default(),
        ph2d_host::WindowSize {
            width: LADO,
            height: LADO,
        },
        wgpu::Color::TRANSPARENT,
        &tiles,
        None,
        None,
    );
    o_mais_denso(gpu, &t, (LADO, LADO))
}

/// ⭐ **As duas fontes do halo desenham a SILHUETA em `Normal`, com ou sem mistura** — medido.
///
/// **Medido** (12/10, RTX, tinta `0,4 · 0,5 · 0,6 @ 0,7`, a sobreposição de duas cópias):
///
/// | fonte | sobreposição RGBA (pré-multiplicada) |
/// |---|---|
/// | placa `Normal` · `Add` · `Screen` | `0,3638 · 0,4548 · 0,5459 · 0,9097` (as três iguais ao bit) |
/// | tile (rota Vello) | `0,3638 · 0,4548 · 0,5459 · 0,9097` — desvio `0` |
///
/// ⇒ o `over` exacto (`0,28 + 0,28·0,3`): as duas rotas já concordam — o halo é a silhueta, e o
/// brilho do grupo não depende da mistura das cópias.
#[test]
#[ignore = "placa: precisa de GPU"]
fn o_halo_das_formas_misturadas_e_a_silhueta_nas_duas_rotas() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let normal = pela_placa(&gpu, 0);
    let soma = pela_placa(&gpu, 1);
    let tela = pela_placa(&gpu, 4);
    let tile = pelo_tile(&gpu);
    println!(
        "halo · placa Normal {normal:?} · placa Add {soma:?} · placa Screen {tela:?} · tile {tile:?}"
    );
    assert_eq!(
        soma, normal,
        "a placa repete as cópias em Normal (a silhueta)"
    );
    assert_eq!(tela, normal);
    let desvio = normal
        .iter()
        .zip(tile)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    println!("desvio placa × tile: {desvio}");
    assert!(desvio <= 2.0 / 255.0, "as duas rotas concordam na silhueta");
}
