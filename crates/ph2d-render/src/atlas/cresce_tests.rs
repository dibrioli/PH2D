//! Gates da F37 §1 (`docs/UI_New_and_Simple/F37_plano.md`): o átlas nasce pequeno e cresce no lugar,
//! e quem GUARDOU a UV do ladrilho branco (Motion, partículas, barras de vida, trilha de macro) segue
//! a amostrar branco depois de ele crescer.

use super::*;

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn cor(lado: u32, rgba: [u8; 4]) -> Vec<u8> {
    rgba.repeat((lado * lado) as usize)
}

/// Os texels de nível 0 que um quad com `uv` amostra (o retângulo inteiro, com a vizinhança bilinear
/// do lado de dentro), no lado ATUAL do átlas.
fn texels_da_uv(atlas: &TextureAtlas, gpu: &GpuContext, uv: [f32; 4]) -> Vec<[u8; 4]> {
    let (lado, _, px) = atlas.readback_mip(gpu, 0);
    let s = lado as f32;
    let (x0, y0) = ((uv[0] * s) as u32, (uv[1] * s) as u32);
    let (x1, y1) = ((uv[2] * s).ceil() as u32, (uv[3] * s).ceil() as u32);
    let mut out = Vec::new();
    for y in y0..=y1.min(lado - 1) {
        for x in x0..=x1.min(lado - 1) {
            let i = ((y * lado + x) * 4) as usize;
            out.push([px[i], px[i + 1], px[i + 2], px[i + 3]]);
        }
    }
    out
}

/// **G22 — a UV do ladrilho branco GUARDADA no arranque segue branca depois de o átlas crescer.**
/// A ordem é a do editor: as sprites de demonstração entram ANTES dos ladrilhos reservados
/// (`atlas_loader` → `insert_reserved_tiles`), e a sprite à volta é vermelha — o defeito
/// «vermelho→grená» do `init_subsystems.rs` se a UV cair fora do branco.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_cached_white_tile_uv_still_samples_white_after_the_atlas_grows() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let vermelho = cor(64, [255, 0, 0, 255]);
    for k in 0..4 {
        atlas.insert(&gpu, k, 64, 64, &vermelho).expect("demo");
    }
    let antes = atlas.size_px;
    let uv = atlas.insert_reserved_tiles(&gpu).expect("reservados");
    for k in 4..120 {
        atlas
            .insert_crescendo(&gpu, k, 64, 64, &vermelho)
            .expect("cresce");
    }
    assert!(
        atlas.size_px >= 4 * antes,
        "o átlas não cresceu ({antes} → {})",
        atlas.size_px
    );
    let lidos = texels_da_uv(&atlas, &gpu, uv);
    assert!(!lidos.is_empty());
    assert!(
        lidos.iter().all(|t| *t == [255, 255, 255, 255]),
        "a UV guardada amostra fora do branco depois de crescer: {:?}",
        lidos.iter().find(|t| **t != [255, 255, 255, 255])
    );
    assert_eq!(
        uv,
        atlas.insert_reserved_tiles(&gpu).expect("de novo"),
        "a UV mudou"
    );
}

/// **G23 — crescer não mexe em nenhuma região:** o mesmo retângulo de pixels, os mesmos bytes, e a
/// free-list continua a servir (um slot libertado antes é reusado depois).
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn growing_keeps_every_region_in_place_with_its_pixels_and_free_slots() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut atlas = TextureAtlas::new(&gpu, 256);
    let azul = cor(32, [0, 0, 255, 255]);
    let a = atlas.insert(&gpu, 1, 32, 32, &azul).expect("a");
    let livre = atlas.insert(&gpu, 2, 32, 32, &azul).expect("b");
    atlas.remove(2);
    assert_eq!(atlas.cresce(&gpu, 1024), Ok(1024));
    assert_eq!(atlas.region(1), Some(a), "a região mudou de lugar");
    let (lado, _, px) = atlas.readback_mip(&gpu, 0);
    for y in a.y..a.y + a.h {
        for x in a.x..a.x + a.w {
            let i = ((y * lado + x) * 4) as usize;
            assert_eq!(
                &px[i..i + 4],
                &[0, 0, 255, 255],
                "texel ({x},{y}) perdeu-se"
            );
        }
    }
    let reusado = atlas.insert(&gpu, 3, 32, 32, &azul).expect("c");
    assert_eq!(reusado, livre, "a free-list não sobreviveu ao crescimento");
    // Uma fonte maior que o lado atual também faz crescer, até ao teto e não além.
    let grande = cor(2048, [9, 9, 9, 255]);
    atlas
        .insert_crescendo(&gpu, 4, 2048, 2048, &grande)
        .expect("2048");
    assert_eq!(atlas.size_px, 2048.max(atlas.size_px));
    assert!(atlas.size_px <= atlas.max_size_px());
}
