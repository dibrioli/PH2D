//! Gates da F36 §1 (`docs/UI_New_and_Simple/F36_plano.md`): o que a placa recusou AO NASCER renasce com
//! o seu conteúdo. A recusa é forçada no nascimento ([`crate::renasce::testes::forca_recusas`]: a
//! textura é destruída, o conteúdo perde-se de verdade).

use crate::individual::IndividualTextureStore;
use crate::renasce::testes::forca_recusas;
use crate::{SpritePipeline, TextureAtlas};
use ph2d_gpu::GpuContext;

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn padrao(lado: u32, sal: u8) -> Vec<u8> {
    (0..lado * lado)
        .flat_map(|i| [i as u8 ^ sal, 255 - i as u8, sal, 255])
        .collect()
}

/// **G17 — o átlas que nasceu recusado volta com as suas sprites, nas MESMAS regiões.** A substituição
/// feita enquanto ele estava recusado é a que volta.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn an_atlas_refused_at_birth_comes_back_with_its_sprites_after_the_episode() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut sao = TextureAtlas::new(&gpu, 64);
    assert!(
        !sao.recusada() && !sao.refaz_se_recusada(&gpu),
        "um átlas são não se refaz"
    );
    forca_recusas(1);
    let mut atlas = TextureAtlas::new(&gpu, 64);
    assert!(atlas.recusada(), "a recusa no nascimento não foi vista");
    let regiao = atlas
        .insert(&gpu, 7, 16, 16, &padrao(16, 1))
        .expect("insert");
    let segunda = padrao(16, 9);
    atlas
        .insert(&gpu, 7, 16, 16, &segunda)
        .expect("substituição");
    assert!(atlas.refaz_se_recusada(&gpu), "o átlas não renasceu");
    assert!(!atlas.recusada());
    assert_eq!(atlas.region(7), Some(regiao), "a região mudou de lugar");
    let (lado, _, px) = atlas.readback_mip(&gpu, 0);
    let mut lido = Vec::new();
    for y in regiao.y..regiao.y + 16 {
        let i = ((y * lado + regiao.x) * 4) as usize;
        lido.extend_from_slice(&px[i..i + 64]);
    }
    assert!(lido == segunda, "o átlas renasceu sem os pixels da sprite");
}

/// **G18 — a textura individual que nasceu recusada volta com o MESMO id, o seu conteúdo e o seu
/// refcount**, em 8 e em 16 bits; a escrita de região feita enquanto recusada também volta.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn an_individual_texture_refused_at_birth_comes_back_with_the_same_id() {
    let Some(gpu) = gpu() else {
        return;
    };
    let bgl = SpritePipeline::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb).material_bgl;
    let mut store = IndividualTextureStore::new(&gpu);
    forca_recusas(2);
    let mut oito = padrao(8, 3);
    let a = store.acquire(&gpu, &bgl, 8, 8, &oito).expect("8 bits");
    let meios: Vec<u16> = (0..8 * 8 * 4).map(|i| 0x3c00 ^ i as u16).collect();
    let b = store.acquire_16(&gpu, &bgl, 8, 8, &meios).expect("16 bits");
    assert_eq!(
        store.recusadas(),
        2,
        "as recusas no nascimento não foram vistas"
    );
    store.retain(a);
    let remendo = vec![200u8; 2 * 2 * 4];
    store
        .replace_pixels_region(&gpu, a, 3, 4, 2, 2, &remendo)
        .expect("região");
    for y in 4..6 {
        let i = (y * 8 + 3) * 4;
        oito[i..i + 8].copy_from_slice(&remendo[..8]);
    }
    assert_eq!(
        store.refaz_recusadas(&gpu, &bgl),
        2,
        "não renasceram as duas"
    );
    assert_eq!(store.recusadas(), 0);
    let (_, _, lido_a) = store.readback(&gpu, a).expect("readback a");
    assert!(lido_a == oito, "a de 8 bits renasceu sem o seu conteúdo");
    let (_, _, lido_b) = store.readback(&gpu, b).expect("readback b");
    assert!(
        lido_b == bytemuck::cast_slice::<u16, u8>(&meios),
        "a de 16 bits renasceu sem o seu conteúdo"
    );
    assert_eq!(
        store.release(a),
        Some(1),
        "o refcount perdeu-se no renascer"
    );
}
