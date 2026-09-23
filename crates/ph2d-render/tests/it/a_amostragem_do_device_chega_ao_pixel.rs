//! ⭐⭐⭐ **O `Filter` DE UMA SAÍDA COZIDA NA PLACA CHEGA AO PIXEL?** (doc 119 §3, ciclo 11 W1)
//!
//! O irmão [`super::individual_texture_honours_its_sampling`] prova a amostragem pelo caminho
//! da CPU (as instâncias da cena). Este desenha a MESMA textura pelo buffer do DISPOSITIVO
//! (`gpu_extra`), que é por onde o cozimento do Motion desenha — e é por onde o defeito vivia:
//! o laço dos runs ligava `material_bg(texture_id, 0)`, logo o `Filter` do sink ficava no
//! buffer (palavra 43) e **nunca chegava ao sampler**. A mesma cena era nítida pela CPU e
//! borrada pela placa, que é a rota de omissão.
//!
//! ⚠️ A régua é a do irmão, de propósito: um xadrez 2×2 ampliado, lido FORA do centro, onde
//! `Nearest` dá o texel preto e `Linear` o clareia. Skip gracioso sem adapter.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{
    Camera2d, GpuTexRun, LiftedInstances, RenderInstance, SpriteRenderer, TextureAtlas,
};

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

/// ⭐⭐⭐ **`Nearest` e `Linear` DESENHAM DIFERENTE quando a textura vem do buffer da placa.**
///
/// ⚠️ A chave viaja no RUN (`GpuTexRun::sampling`) e **também** na instância: a instância leva
/// a do sink (o lowering escreve-a) e o run é o que o desenho lê. Para o gate medir o RUN e não
/// um acaso, a instância leva SEMPRE a chave de fábrica — com o run cravado em `0` os dois modos
/// leriam o mesmo byte.
#[test]
fn o_filtro_do_sink_chega_ao_pixel_pela_rota_da_placa() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping o_filtro_do_sink_chega_ao_pixel_pela_rota_da_placa: no GPU");
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
    let vazio = LiftedInstances::default();

    let draw = |renderer: &mut SpriteRenderer, filter: u8| -> u8 {
        let mut present = PresentWorld::new();
        let frente = [quad(tex, RenderInstance::SAMPLING_DEFAULT)];
        let bytes: &[u8] = bytemuck::cast_slice(&frente);
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("amostragem no device: instancias"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, bytes);
        let runs = [GpuTexRun {
            texture_id: tex,
            start: 0,
            end: 1,
            blend: 0,
            // ⭐ O SUJEITO deste gate: é por aqui que o `Filter` chega ao sampler.
            sampling: RenderInstance::pack_sampling(filter, 0),
        }];
        renderer.render_with_streams(
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
            &vazio,
            Some((&buffer, 1, &runs)),
            None,
            None,
            None,
        );
        channel(&readback(&gpu, &target), 24, 24)
    };

    let nearest = draw(&mut renderer, NEAREST);
    let linear = draw(&mut renderer, LINEAR);
    assert!(
        nearest.abs_diff(linear) > 8,
        "o Filter do sink nao alcanca a rota da placa: Nearest={nearest} Linear={linear} — \
         o laco dos runs voltou a ligar o sampler 0"
    );
    assert!(
        nearest < linear,
        "no texel preto, `Linear` tem de clarear: Nearest={nearest} Linear={linear}"
    );
}
