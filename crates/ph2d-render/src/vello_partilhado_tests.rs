//! Gates da F37 §2 (`docs/UI_New_and_Simple/F37_plano.md`): UM `vello::Renderer` por placa.

use super::*;
use crate::VelloPass;
use ph2d_gpu::GpuContext;
use vello::kurbo::Affine;
use vello::peniko::{ImageBrush, ImageQuality};

const LADO: u32 = 16;
const TELA: (u32, u32) = (48, 48);

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn textura(gpu: &GpuContext, rgba: [u8; 4]) -> wgpu::Texture {
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("F37 textura"),
        size: wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        t.as_image_copy(),
        &rgba.repeat((LADO * LADO) as usize),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(LADO * 4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
    );
    t
}

/// Uma cena com a imagem registada em `(8, 8)`, e o pixel lido no meio dela.
fn desenha(vp: &mut VelloPass, gpu: &GpuContext, img: vello::peniko::ImageData) -> Vec<u8> {
    let mut cena = vello::Scene::new();
    cena.draw_image(
        ImageBrush::new(img)
            .with_quality(ImageQuality::Low)
            .as_ref(),
        Affine::translate((8.0, 8.0)),
    );
    let px = vp.render_and_readback(gpu, &cena, TELA).expect("render");
    let i = ((16 * TELA.0 + 16) * 4) as usize;
    px[i..i + 4].to_vec()
}

/// **G25 — os passes de UMA placa partilham UM renderer, e cada um desenha a SUA textura.** Largar um
/// pass tira as texturas dele do renderer; largar o último larga o renderer (o próximo nasce novo).
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_passes_of_one_card_share_one_renderer_and_each_draws_its_own_texture() {
    let Some(gpu) = gpu() else {
        return;
    };
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut a = VelloPass::new(&gpu, fmt, TELA).expect("a");
    let mut b = VelloPass::new(&gpu, fmt, TELA).expect("b");
    assert!(
        Arc::ptr_eq(&a.partilhado, &b.partilhado),
        "dois renderers na mesma placa"
    );
    let ia = a.register_texture(textura(&gpu, [255, 0, 0, 255]));
    let ib = b.register_texture(textura(&gpu, [0, 0, 255, 255]));
    for _ in 0..2 {
        assert_eq!(desenha(&mut a, &gpu, ia.clone()), [255, 0, 0, 255], "a");
        assert_eq!(desenha(&mut b, &gpu, ib.clone()), [0, 0, 255, 255], "b");
    }
    let fraco = Arc::downgrade(&a.partilhado);
    drop(a);
    assert!(
        !trava(&b.partilhado).registos.contains_key(&ia.data.id()),
        "a textura do pass largado ficou no renderer"
    );
    assert_eq!(
        desenha(&mut b, &gpu, ib),
        [0, 0, 255, 255],
        "b depois de largar a"
    );
    drop(b);
    assert!(
        fraco.upgrade().is_none(),
        "o renderer sobreviveu ao último dono"
    );
    // Outro dispositivo tem o seu.
    if let Ok(outra) = GpuContext::new(GpuContext::default_instance(), None) {
        let c = VelloPass::new(&outra, fmt, TELA).expect("c");
        let d = VelloPass::new(&gpu, fmt, TELA).expect("d");
        assert!(
            !Arc::ptr_eq(&c.partilhado, &d.partilhado),
            "duas placas, um renderer"
        );
    }
}

/// **G26 — a recusa vista por UM pass refaz o renderer da placa UMA vez, e a textura do OUTRO pass
/// volta a desenhar** (o renderer refeito herda os registos de todos os donos, F35 §2).
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_refusal_seen_by_one_pass_rebuilds_the_shared_renderer_with_every_owners_textures() {
    let Some(gpu) = gpu() else {
        return;
    };
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut a = VelloPass::new(&gpu, fmt, TELA).expect("a");
    let mut b = VelloPass::new(&gpu, fmt, TELA).expect("b");
    let ib = b.register_texture(textura(&gpu, [0, 255, 0, 255]));
    assert_eq!(desenha(&mut b, &gpu, ib.clone()), [0, 255, 0, 255]);
    a.finge_falta_de_memoria();
    let ia = a.register_texture(textura(&gpu, [255, 0, 0, 255]));
    assert_eq!(desenha(&mut a, &gpu, ia), [255, 0, 0, 255]);
    assert_eq!(
        b.estado_do_renderer(),
        (false, 1),
        "refeito uma vez, para os dois"
    );
    assert_eq!(
        desenha(&mut b, &gpu, ib),
        [0, 255, 0, 255],
        "o renderer refeito perdeu a textura do outro pass"
    );
}
