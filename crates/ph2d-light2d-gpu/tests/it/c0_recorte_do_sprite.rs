//! **O sprite capturado lê só a célula dele** ([doc](../../../../docs/Sprite_projeto/22_o_recorte_nao_le_o_vizinho.md)):
//! a célula transparente de uma folha, de vizinhas opacas, não emite nem bloqueia na C0.

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Amostragem, CapturaDeSprites, Config, Solver, SpriteNaC0};

use crate::comum::{gpu, grade, ler_bytes};

const C: u32 = 8;

fn c0_da_celula(
    gpu: &ph2d_gpu::GpuContext,
    vista: &wgpu::TextureView,
    uv: Option<[f32; 4]>,
) -> Vec<u8> {
    let mut s = Solver::novo(gpu, Config::padrao(grade(64, 1.0 / 16.0))).expect("solver");
    let sprites = CapturaDeSprites {
        texturas: vec![(vista, Amostragem::Bilinear)],
        sprites: uv
            .into_iter()
            .map(|uv| SpriteNaC0 {
                origem: [1.0, 3.0],
                eixo_s: [2.0, 0.0],
                eixo_t: [0.0, -2.0],
                uv,
                emissao: [1.0; 3],
                opacidade: 1.0,
                premultiplicada: false,
                textura: 0,
                albedo: [1.0; 3],
                cor_em_srgb: false,
            })
            .collect(),
        ..Default::default()
    };
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    s.gravar_com_sprites(gpu, &mut enc, &CenaDeLuz::default(), &sprites);
    gpu.queue.submit([enc.finish()]);
    ler_bytes(gpu, s.c0())
}

#[test]
fn a_celula_transparente_nao_entra_na_c0() {
    let Some(gpu) = gpu() else { return };
    let lado = C * 3;
    let px: Vec<u8> = (0..lado * lado)
        .flat_map(|i| {
            let (x, y) = (i % lado, i / lado);
            [
                255,
                255,
                255,
                if x / C == 1 && y / C == 1 { 0 } else { 255 },
            ]
        })
        .collect();
    let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("folha"),
        size: wgpu::Extent3d {
            width: lado,
            height: lado,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        textura.as_image_copy(),
        &px,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(lado * 4),
            rows_per_image: Some(lado),
        },
        textura.size(),
    );
    let vista = textura.create_view(&Default::default());
    let (u, v) = (1.0 / 3.0, 2.0 / 3.0);
    let vazia = c0_da_celula(&gpu, &vista, None);
    let meio = c0_da_celula(&gpu, &vista, Some([u, u, v, v]));
    let pintados = meio.iter().zip(&vazia).filter(|(a, b)| a != b).count();
    assert_eq!(
        pintados, 0,
        "a célula transparente pintou {pintados} bytes da C0"
    );
    // Controlo: a célula do canto, opaca, entra (a régua não mede o nada).
    let canto = c0_da_celula(&gpu, &vista, Some([0.0, 0.0, u, u]));
    assert_ne!(canto, vazia, "a célula opaca não entrou na C0");
}
