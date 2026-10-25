//! ⭐ **A costura do azulejo repetido** (plano 06 do GameEngineFuncional, a amostra de arte): uma sprite que
//! REPETE a textura (`uv_xform` + `RepeatMode::Enabled`) amostra com o gradiente do UV CONTÍNUO. Com o
//! implícito, o quad 2×2 onde o `fract` dá a volta vê o UV saltar um azulejo inteiro e a placa escolhe o mip
//! mais pequeno — a média do atlas quase todo transparente: um risco claro a cada azulejo (medido no smoke
//! `PH2D_GAME_SMOKE=2`, foto de 25/10).

use crate::{Camera2d, GameRt, LiftedInstances, RenderInstance, SpriteRenderer, TextureAtlas};
use ph2d_host::WindowSize;

const W: u32 = 64;
const H: u32 = 32;

fn try_headless_gpu() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

/// Lê o `GameRt` (`Rgba16Float`) em `[r, g, b, a]` por texel.
fn ler(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture) -> Vec<[f32; 4]> {
    let (w, h) = (t.width(), t.height());
    let padded =
        (w * 8).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("costura leitura"),
        size: u64::from(padded * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        t.size(),
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let dados = buf.slice(..).get_mapped_range();
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let o = (y * padded + x * 8) as usize;
            let c = |k: usize| {
                half::f16::from_le_bytes([dados[o + 2 * k], dados[o + 2 * k + 1]]).to_f32()
            };
            [c(0), c(1), c(2), c(3)]
        })
        .collect()
}

/// Um azulejo opaco de cor ÚNICA repetido 4× num quad de 4 × 1 m: todo texel do interior tem a mesma cor.
/// O quad está deslocado 1 px para que a volta do `fract` caia DENTRO de um quad 2×2 (alinhada à grelha de
/// quads, a volta escapa entre dois e nada se vê). ⚠️ Controlo: o meio de cada azulejo tem a cor (a fixtura
/// desenha mesmo).
#[test]
#[ignore = "placa: precisa de GPU"]
fn um_azulejo_repetido_nao_tem_costura() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 1024);
    atlas
        .insert(&gpu, 1, 64, 64, &[200, 40, 40, 255].repeat(64 * 64))
        .expect("cabe");
    let mut renderer = SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, atlas, 8);
    let px_m = 1.0 / 16.0;
    let mut lifted = LiftedInstances::default();
    lifted.push(
        RenderInstance {
            world_pos: [px_m, px_m * 0.5],
            size: [4.0, 1.0],
            atlas_uv: renderer.atlas().region_uv(1),
            tint: [1.0; 4],
            basis: RenderInstance::IDENTITY_BASIS,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: RenderInstance::pack_repeat_bits(2),
            texture_id: RenderInstance::ATLAS_TEXTURE_ID,
            z_order: 0,
            sampling: RenderInstance::SAMPLING_DEFAULT,
            uv_xform: [4.0, 1.0, 0.0, 0.0],
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            sub_order: 0,
        },
        None,
    );
    let alvo = GameRt::new(&gpu, (W, H));
    renderer.render_lifted_instances(
        alvo.view(),
        &Camera2d::new([0.0, 0.0], 2.0),
        WindowSize::new(W, H),
        wgpu::Color::TRANSPARENT,
        &lifted,
        None,
        false,
    );
    let px = ler(&gpu, alvo.texture());
    let at = |x: u32, y: u32| px[(y * W + x) as usize];
    // O quad cobre x ∈ [1, 65) e y ∈ [8, 24) (16 px/m); o interior deixa 1 px de cada borda.
    let cor = at(9, 16);
    assert!(
        cor[3] > 0.99 && cor[0] > 2.0 * cor[1],
        "o meio do 1.º azulejo: {cor:?}"
    );
    let mut piores = Vec::new();
    for y in 9..23 {
        for x in 2..W {
            let c = at(x, y);
            let d = (0..4)
                .map(|k| (c[k] - cor[k]).abs())
                .fold(0.0_f32, f32::max);
            if d > 0.02 {
                piores.push((x, y, c));
            }
        }
    }
    assert!(
        piores.is_empty(),
        "{} texels fora da cor do azulejo, ex.: {:?}",
        piores.len(),
        &piores[..piores.len().min(4)]
    );
}
