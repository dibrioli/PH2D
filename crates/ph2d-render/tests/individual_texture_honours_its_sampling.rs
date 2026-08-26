//! **O FILTRO POR-NÓ VALE PARA UMA TEXTURA INDIVIDUAL** (doc 89, folha 17).
//!
//! ⚠️ **Este gate nasceu de um DEFEITO DE PRODUTO pré-existente, achado em 2026-08-25.**
//! O `material_bg` do `renderer_draw` honrava a `RenderInstance::sampling` **só para o
//! átlas partilhado**; para toda textura individual ele devolvia `individual.bind_group(id)`,
//! que é UM grupo construído contra o sampler **default do projecto**. ⇒ o filtro por-nó
//! do Inspector (§9) estava **inerte em toda textura individual do app**, e uma sprite
//! promovida a Individual por um `commit_edited_texture` perdia o filtro dela **em
//! silêncio**.
//!
//! ⚠️ **E o caso que o expôs é aquele para que o filtro EXISTE**: *pixel-art*, que chega
//! por importação e portanto quase nunca está no átlas partilhado. *Um knob pode estar
//! ligado, ter gate de unidade dos dois lados, e não alcançar o único sítio onde alguém o
//! usaria.*
//!
//! A régua é o pixel, não a API: uma textura 2×2 de xadrez preto-e-branco, ampliada para o
//! alvo inteiro, amostrada FORA do centro. Com `Nearest` aquele pixel é o texel — preto ou
//! branco; com `Linear` ele é a mistura dos quatro. Skip gracioso sem adapter.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, RenderInstance, SpriteRenderer, TextureAtlas};

const W: u32 = 64;
const H: u32 = 64;

/// `FilterMode::Nearest` / `Linear` — os tags que o `sampler_from_tags` lê.
const NEAREST: u8 = 1;
const LINEAR: u8 = 2;

fn try_headless_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn make_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sampling gate target"),
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
    })
}

fn readback(gpu: &GpuContext, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sampling gate staging"),
        size: (padded * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    rx.recv().expect("map channel").expect("map ok");
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * H) as usize);
    for row in 0..H as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

fn channel(px: &[u8], x: u32, y: u32) -> u8 {
    px[((y * W + x) * 4) as usize]
}

/// Um xadrez 2×2: preto · branco / branco · preto.
fn checker_2x2() -> Vec<u8> {
    let b = [0u8, 0, 0, 255];
    let w = [255u8, 255, 255, 255];
    [b, w, w, b].concat()
}

fn quad(texture_id: u32, sampling: u32) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [8.0, 8.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id,
        z_order: 0,
        sampling,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// ⭐⭐⭐ **`Nearest` e `Linear` DESENHAM DIFERENTE numa textura individual.**
///
/// ⚠️ A amostra é tirada em `(24, 24)` — dentro do quadrante superior-esquerdo, longe da
/// fronteira. Com `Nearest` aquele pixel é o texel PRETO inteiro; com `Linear` a
/// interpolação já o clareia. *Uma amostra no centro exacto mediria a mesma média nos
/// dois modos e o gate passaria sobre o defeito.*
#[test]
fn a_per_node_filter_changes_what_an_individual_texture_draws() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping a_per_node_filter_changes_what_an_individual_texture_draws: no GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let tex = renderer
        .acquire_individual(2, 2, &checker_2x2())
        .expect("individual checker");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    let draw = |renderer: &mut SpriteRenderer, filter: u8| -> u8 {
        let mut present = PresentWorld::new();
        present
            .world_mut()
            .spawn(quad(tex, RenderInstance::pack_sampling(filter, 0)));
        renderer.render(
            &view,
            &mut present,
            &camera,
            window,
            wgpu::Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        );
        channel(&readback(&gpu, &target), 24, 24)
    };

    let nearest = draw(&mut renderer, NEAREST);
    let linear = draw(&mut renderer, LINEAR);
    assert!(
        nearest.abs_diff(linear) > 8,
        "o filtro por-no' nao alcanca uma textura individual: Nearest={nearest} \
         Linear={linear} — o `material_bg` voltou a devolver o grupo do default do projecto"
    );
    // E a leitura tem SENTIDO, não só diferença: o ponto amostrado cai no texel PRETO, e
    // é o `Linear` que o clareia (a mistura puxa para o branco vizinho).
    assert!(
        nearest < linear,
        "no texel preto, `Linear` tem de clarear: Nearest={nearest} Linear={linear}"
    );
}

/// **`sampling = 0` continua a ser o default do projecto** — a metade que impede a cura de
/// mudar o mundo de antes.
///
/// ⚠️ Sem ela, uma cache que passasse a servir também o `0` congelaria o default no valor
/// que ele tinha no primeiro desenho, e o `set_filter_mode` do projecto deixaria de ter
/// efeito **sem uma linha de erro**.
#[test]
fn an_inherit_sampling_still_follows_the_project_default() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping an_inherit_sampling_still_follows_the_project_default: no GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let tex = renderer
        .acquire_individual(2, 2, &checker_2x2())
        .expect("individual checker");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    let draw = |renderer: &mut SpriteRenderer| -> u8 {
        let mut present = PresentWorld::new();
        present
            .world_mut()
            .spawn(quad(tex, RenderInstance::SAMPLING_DEFAULT));
        renderer.render(
            &view,
            &mut present,
            &camera,
            window,
            wgpu::Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        );
        channel(&readback(&gpu, &target), 24, 24)
    };

    renderer.set_filter_mode(ph2d_render::ImageFilterMode::PixelArt);
    let with_pixel_art = draw(&mut renderer);
    renderer.set_filter_mode(ph2d_render::ImageFilterMode::Smooth);
    let with_smooth = draw(&mut renderer);
    assert!(
        with_pixel_art.abs_diff(with_smooth) > 8,
        "trocar o default do PROJECTO tem de mover uma linha que herda: \
         PixelArt={with_pixel_art} Smooth={with_smooth}"
    );
}
