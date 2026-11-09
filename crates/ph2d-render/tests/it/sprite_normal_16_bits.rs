//! ⭐ P14 — **a normal de 16 bits sobe sem degraus** (`docs/PixelArt/P14_plano.md` §11, «Camadas e 16
//! bits»). A medição (`docs/PixelArt/ferramentas/mede_normal_16_bits.py`) deu até 2 níveis de 8 bits no
//! quadro com a normal em `Rgba8Unorm`, na luz forte (`F/2π = 2`, `|M|/F = 0,95`); aqui, na placa, a
//! mesma luz e 256 normais na encosta escura (`n_x ∈ [−0,99, −0,55]`, onde a luz não satura e a
//! inclinação é a maior): a fonte de 16 bits pela porta `definir_normal_texeis` fica a ≤ 1 nível da lei
//! exata; o CONTROLO (as mesmas normais em 8 bits) mostra os degraus que a cena consegue ver.

use std::collections::BTreeSet;
use std::f64::consts::TAU;

use ph2d_ecs::PresentWorld;
use ph2d_host::WindowSize;
use ph2d_render::{
    Camera2d, LiftedInstances, LuzRecebida, RenderInstance, SpriteRenderer, TexeisDaNormal,
    TextureAtlas, normal_de_meio_float_linear,
};

use crate::sprite_com_normal::{H, ORIGEM, TAM, TEXEL, W, campo, gpu, ler};

/// 16 × 16 texels sobre o quad de 1 m = 16 × 16 px (x 24..40, y 8..24): um texel por píxel.
const LADO: u32 = 16;
/// A luz forte da medição: `F/2π = 2`, `M = 0,95·F` para a direita.
const EXPOSICAO: f64 = 2.0;
const MOMENTO: f64 = 1.9;

/// A normal do texel `k` (linha a linha), na encosta escura; `ny = 0`.
fn n_x(k: u32) -> f64 {
    -0.99 + 0.44 * (f64::from(k) + 0.5) / f64::from(LADO * LADO)
}

/// O valor CODIFICADO (`n·½+½`) dos três canais do texel `k`.
fn codificada(k: u32) -> [f64; 3] {
    let nx = n_x(k);
    let nz = (1.0 - nx * nx).max(0.0).sqrt();
    [nx * 0.5 + 0.5, 0.5, nz * 0.5 + 0.5]
}

fn srgb_para_linear(v: f64) -> f64 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// O quadro com a normal `texeis`; devolve o verde de cada um dos 256 píxeis (linha a linha).
fn verdes(gpu: &ph2d_gpu::GpuContext, texeis: TexeisDaNormal<'_>) -> Vec<u8> {
    let f = campo(gpu, EXPOSICAO * TAU);
    let mx = campo(gpu, MOMENTO * TAU);
    let my = campo(gpu, 0.0);
    let (fv, mxv, myv) = (
        f.create_view(&Default::default()),
        mx.create_view(&Default::default()),
        my.create_view(&Default::default()),
    );
    let ninguem = BTreeSet::new();
    let luz = LuzRecebida {
        fluencia: &fv,
        origem_m: ORIGEM,
        texel_m: TEXEL,
        tamanho: TAM,
        sem_luz: &ninguem,
        momento: Some([&mxv, &myv]),
    };
    let alvo = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("normal 16 bits alvo"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let atlas = TextureAtlas::new(gpu, 256);
    let mut r = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 8);
    let branco = vec![255_u8; (LADO * LADO * 4) as usize];
    let tex = r
        .acquire_individual(LADO, LADO, &branco)
        .expect("textura do sprite");
    r.definir_normal_texeis(tex, Some((LADO, LADO, texeis)))
        .expect("normal");
    let mut extra = LiftedInstances::default();
    extra.push(
        RenderInstance {
            world_pos: [0.0, 0.0],
            size: [1.0, 1.0],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0; 4],
            basis: RenderInstance::IDENTITY_BASIS,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            texture_id: tex,
            z_order: 0,
            sampling: RenderInstance::SAMPLING_DEFAULT,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            sub_order: 0,
        },
        None,
    );
    r.receber_luz(Some(&luz));
    r.render_with_streams(
        &alvo.create_view(&Default::default()),
        &mut PresentWorld::new(),
        &Camera2d::new([0.0, 0.0], 2.0),
        WindowSize::new(W, H),
        wgpu::Color::TRANSPARENT,
        &extra,
        None,
        None,
        None,
        None,
    );
    let b = ler(gpu, &alvo);
    (0..LADO * LADO)
        .map(|k| {
            let (x, y) = (24 + (k % LADO) as usize, 8 + (k / LADO) as usize);
            b[(y * W as usize + x) * 4 + 1]
        })
        .collect()
}

/// A lei exata (`receber.wgsl` `luz_em_n` com a normal unitária), sobre a luz como a placa a guarda
/// (meio-float), no alvo `Rgba8Unorm`.
fn lei(k: u32) -> f64 {
    let f = f64::from(half::f16::from_f64(EXPOSICAO * TAU));
    let m = f64::from(half::f16::from_f64(MOMENTO * TAU));
    let e = (f + n_x(k) * m).max(0.0) / TAU;
    (e.clamp(0.0, 1.0) * 255.0).round()
}

/// O pior desvio, em níveis, e a contagem por desvio.
fn desvio(v: &[u8]) -> (u32, [usize; 4]) {
    let mut conta = [0; 4];
    let mut pior = 0;
    for (k, &g) in v.iter().enumerate() {
        let d = (f64::from(g) - lei(k as u32)).abs() as u32;
        conta[d.min(3) as usize] += 1;
        pior = pior.max(d);
    }
    (pior, conta)
}

/// *Mutação: a subida de `Rgba16F` forçada a 8 bits (o `bytes` de `TexeisDaNormal` arredonda ao byte).*
#[test]
fn uma_normal_de_16_bits_sobe_sem_degraus() {
    let Some(gpu) = gpu() else {
        eprintln!("normal 16 bits: sem placa — saltado");
        return;
    };
    // A fonte de 16 bits como o `Asset::ImageRgba16` a guarda: meio-float LINEAR.
    let linear: Vec<u16> = (0..LADO * LADO)
        .flat_map(|k| {
            let c = codificada(k).map(|v| half::f16::from_f64(srgb_para_linear(v)).to_bits());
            [c[0], c[1], c[2], half::f16::ONE.to_bits()]
        })
        .collect();
    let dezasseis = normal_de_meio_float_linear(&linear);
    let oito: Vec<u8> = (0..LADO * LADO)
        .flat_map(|k| {
            let c = codificada(k).map(|v| (v * 255.0).round() as u8);
            [c[0], c[1], c[2], 255]
        })
        .collect();
    let (p16, c16) = desvio(&verdes(&gpu, TexeisDaNormal::Rgba16F(&dezasseis)));
    let (p8, c8) = desvio(&verdes(&gpu, TexeisDaNormal::Rgba8(&oito)));
    eprintln!("normal 16 bits: 16 → pior {p16} {c16:?} · 8 (controlo) → pior {p8} {c8:?}");
    assert!(
        p8 >= 2,
        "o controlo de 8 bits não mostra degraus ({p8}): a cena não mede"
    );
    assert!(
        p16 <= 1,
        "a normal de 16 bits tem degraus: {p16} níveis ({c16:?})"
    );
}
