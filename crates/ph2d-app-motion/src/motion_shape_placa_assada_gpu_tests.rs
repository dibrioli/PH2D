//! ⭐⭐ **P4 — a textura ASSADA dá a mesma imagem nas duas rotas** (doc 132 §4.1). Uma imagem de grupo
//! cuja textura é um KTX2 carregado pela porta do produto (`SpriteRenderer::ensure_cooked_texture`):
//! a cena vectorial da CPU lê a arte pela porta de leitura do renderer (`le_rgba8`, via
//! `motion_leaf_images`) e a placa liga a textura pela porta irmã (`texture_view_and_dims`). Até ao P4
//! a arte de uma assada não resolvia e a CPU não a desenhava.
//!
//! `#[ignore]`: precisa de adaptador real.

use ph2d_eval_motion::{BlendWith, MisturaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_vector::Affine;

use super::gpu_tests::{LADO, camara, compara, le_a_camada};
use super::{GeometriasDaPlaca, Misturas, PlacaDeFormas};
use crate::motion_shape_gen::VecPathStore;

const L: u32 = 8;

/// A arte `8×8`: cor a variar nos dois eixos e um anel de meia alfa.
fn arte() -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..L {
        for x in 0..L {
            let a = if x == 0 || y == 0 || x == L - 1 || y == L - 1 {
                128
            } else {
                255
            };
            #[expect(clippy::cast_possible_truncation, reason = "bytes da fixtura")]
            v.extend_from_slice(&[(x * 30 + 10) as u8, (y * 30 + 10) as u8, 160, a]);
        }
    }
    v
}

/// Duas imagens da assada que se sobrepõem, num grupo `Add · Copies`.
fn imagens(id: u32) -> Vec<VectorInstance> {
    [-0.8f32, 0.6]
        .iter()
        .map(|&x| VectorInstance {
            geometry_id: 0,
            texture_id: id,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            premultiplied: 0.0,
            world_pos: [x, 0.2],
            size: [2.4, 2.4],
            basis: [0.96, 0.28, -0.28, 0.96],
            tint: [1.0; 4],
            anchor: [0.0, 0.0],
            sampling: 0,
            blend_linha: 0,
            mistura: MisturaDoSink {
                blend: 1,
                com: BlendWith::Copies,
                sink: 0,
            },
        })
        .collect()
}

/// ⭐⭐ **A assada: a arte da CPU é o texel da placa, e as duas rotas desenham o mesmo.** (1) A porta
/// de leitura devolve a arte AO BYTE (`±1`, a ida e volta sRGB) — o CONTROLO de antes da cura é `None`;
/// (2) o Vello com essa arte e a placa com a textura ligada, à barra das imagens (`4 · 4 · 0`).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_textura_assada_da_a_mesma_imagem_nas_duas_rotas() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a placa");
    };
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::new(&gpu, 64),
        8,
    );
    let px = arte();
    let blob = ph2d_asset_ktx2::encode_uncompressed_rgba8(L, L, true, &px).expect("o KTX2");
    let id = renderer
        .ensure_cooked_texture(
            ph2d_asset::LogicalTextureId::from_source_bytes(b"p4 assada"),
            ph2d_asset::AssetId::from_bytes(&blob),
            &blob,
        )
        .expect("a assada carrega");
    assert!(ph2d_render::RenderInstance::is_cooked_texture_id(id));
    // (1) A arte pela porta da CPU.
    let mut cache = crate::motion_leaf_images::LeafImages::default();
    let (w, h, lida) = cache
        .synced(renderer.atlas().epoch())
        .art(&gpu, &renderer, id, [0.0, 0.0, 1.0, 1.0])
        .expect("a CPU resolve a arte da assada");
    assert_eq!((w, h), (L, L));
    let pior = lida
        .iter()
        .zip(&px)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(pior <= 1, "a arte da assada desvia {pior} do texel");
    // (2) As duas rotas.
    let insts = imagens(id);
    let mut cena = ph2d_vector::VectorScene::new();
    let janela = ph2d_vector::Rect::new(0.0, 0.0, f64::from(LADO), f64::from(LADO));
    let mut sync = cache.synced(renderer.atlas().epoch());
    crate::motion_shape_gen::encode(
        &insts,
        &VecPathStore::default(),
        &mut |tex, uv| sync.art(&gpu, &renderer, tex, uv),
        camara(),
        Some(janela),
        ph2d_render::ImageFilterMode::Smooth,
        &mut cena,
    );
    let mut vp =
        ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (LADO, LADO))
            .expect("vello");
    let vello = vp
        .render_and_readback(&gpu, cena.inner(), (LADO, LADO))
        .expect("o Vello desenha");
    let mut placa = PlacaDeFormas::default();
    let mut geos = GeometriasDaPlaca::default();
    let store = VecPathStore::default();
    assert!(placa.decide_com_misturas(
        true,
        &insts,
        &store,
        &mut geos,
        camara(),
        Misturas::do_quadro(&gpu, false),
    ));
    let m = crate::motion_state::MotionState::new();
    let fontes = super::imagens::fontes_do_quadro(
        &placa,
        &gpu,
        &m,
        &renderer,
        ph2d_render::ImageFilterMode::Smooth,
    );
    assert!(
        fontes.as_ref().is_some_and(|f| f.texturas.len() == 1),
        "a placa liga a assada"
    );
    placa.com_imagens(fontes);
    let _ = placa
        .desenha(&gpu, (LADO, LADO), &geos, None)
        .expect("a placa desenha");
    let da_placa = le_a_camada(&gpu, &placa);
    let (alfa, cor, nv, np, fora) = compara(&vello, &da_placa);
    eprintln!("assada: alfa max {alfa} · cor max {cor} · pixels {nv}/{np} · > 16: {fora}");
    assert!(nv > 2_000, "a cena pinta ({nv})");
    assert!(
        alfa <= 4 && cor <= 4 && fora == 0,
        "alfa {alfa} cor {cor} fora {fora}"
    );
    let _ = Affine::IDENTITY;
}
