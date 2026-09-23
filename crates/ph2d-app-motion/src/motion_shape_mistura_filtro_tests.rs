//! ⭐⭐ **O FILTRO de um quad de imagem na cena vectorial** (doc 118 §8) — a lei pura e o pixel.
//!
//! A lei ([`super::qualidade_da_imagem`]) responde a mesma pergunta que o sampler da sprite: com
//! que lei se AMPLIA a imagem. ⚠️ Duas metades, porque são dois defeitos: a lei certa com o fio
//! partido (o `encode` a ignorar o `sampling` da linha) deixa a tabela verde e o ecrã igual ao de
//! antes; o fio certo com a lei errada muda o ecrã para o lado errado.

use std::sync::Arc;

use ph2d_eval_motion::{MisturaDoSink, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_render::{ImageFilterMode, RenderInstance, VelloPass, filter_tag_magnifies_by_point};
use ph2d_vector::{Affine, ImageQuality, VectorScene};

use super::super::{VecPathStore, encode};
use super::qualidade_da_imagem;

/// **A tabela inteira**: toda tag que o sink pode escrever, nos dois projectos.
///
/// ⚠️ A expectativa sai de [`filter_tag_magnifies_by_point`] — a MESMA função que monta o sampler da
/// sprite —, e é isso que faz a imagem e a sprite concordarem por construção. O `repeat` no byte de
/// cima não pode mudar a resposta (a rota vectorial não ladrilha: doc 118 §8).
#[test]
fn a_lei_do_filtro_da_imagem_e_a_do_sampler_da_sprite() {
    let mut viu = (false, false);
    for projeto in [ImageFilterMode::Smooth, ImageFilterMode::PixelArt] {
        for tag in 0..=6u8 {
            let ponto = if tag == 0 {
                projeto == ImageFilterMode::PixelArt
            } else {
                filter_tag_magnifies_by_point(tag)
            };
            let esperado = if ponto {
                ImageQuality::Low
            } else {
                ImageQuality::Medium
            };
            for repeat in 0..=3u8 {
                let sampling = RenderInstance::pack_sampling(tag, repeat);
                assert_eq!(
                    qualidade_da_imagem(sampling, projeto),
                    esperado,
                    "tag {tag}, repeat {repeat}, projecto {projeto:?}"
                );
            }
            viu.0 |= ponto;
            viu.1 |= !ponto;
        }
    }
    // CONTROLO: a tabela tem as duas respostas — uma lei constante passaria numa tabela de uma só.
    assert!(viu.0 && viu.1, "a tabela nao exercita as duas leis");
    // ⭐ O caminho de OMISSÃO é o de antes: projecto de fábrica e sink em `Inherit` ⇒ `Medium`.
    assert_eq!(
        ph2d_editor_core::project::ProjectSettings::default().image_filter,
        ImageFilterMode::Smooth,
        "o projecto de fabrica mudou — o caminho de omissao ja nao e' byte-identico"
    );
    assert_eq!(
        qualidade_da_imagem(RenderInstance::SAMPLING_DEFAULT, ImageFilterMode::Smooth),
        ImageQuality::Medium,
        "o sink de fabrica num projecto de fabrica tem de desenhar o que sempre desenhou"
    );
}

const W: u32 = 64;
const H: u32 = 8;

fn try_headless_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// Um quad de imagem de `48×H` px centrado na faixa, com a amostragem dada.
fn quad(sampling: u32) -> VectorInstance {
    VectorInstance {
        geometry_id: 0,
        world_pos: [W as f32 / 2.0, H as f32 / 2.0],
        size: [48.0, H as f32],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [1.0; 4],
        texture_id: 7,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        sampling,
        blend_linha: 0,
        mistura: MisturaDoSink::default(),
    }
}

/// Pinta pela porta do PRODUTO uma imagem de `2×1` texels (PRETO | BRANCO) ampliada `24×` e conta
/// quantos píxeis da linha do meio ficam ENTRE os dois — a assinatura de uma lei bilinear.
fn intermedios(pass: &mut VelloPass, gpu: &GpuContext, sampling: u32, p: ImageFilterMode) -> usize {
    let store = VecPathStore::default();
    let mut art = |id: u32, _: [f32; 4]| {
        (id == 7).then(|| (2, 1, Arc::new(vec![0, 0, 0, 255, 255, 255, 255, 255])))
    };
    let mut cena = VectorScene::new();
    encode(
        &[quad(sampling)],
        &store,
        &mut art,
        Affine::IDENTITY,
        None,
        p,
        &mut cena,
    );
    let px = pass
        .render_and_readback(gpu, cena.inner(), (W, H))
        .expect("o readback");
    let linha = (H / 2) as usize * W as usize * 4;
    // Só o miolo do quad (`x ∈ [8, 56]`), longe das arestas dele, que o anti-serrilhado mistura.
    (12..52)
        .filter(|&x| (24..=231).contains(&px[linha + x * 4]))
        .count()
}

/// ⭐⭐ **O `Filter` do sink CHEGA ao pixel de uma imagem na cena vectorial** (doc 118 §8).
///
/// Quatro células, e cada uma separa uma mutação: o `Inherit` segue o PROJECTO nos dois sentidos, e
/// uma tag explícita GANHA do projecto nos dois sentidos. ⚠️ O CONTROLO vem primeiro: a fixtura tem
/// de produzir uma rampa com a lei bilinear, senão «zero intermédios» não distingue nada.
#[test]
#[ignore = "precisa de GPU"]
fn o_filtro_do_sink_chega_ao_pixel_da_imagem() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let Ok(mut pass) = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, (W, H)) else {
        println!("sem VelloPass — saltado");
        return;
    };
    let herda = RenderInstance::SAMPLING_DEFAULT;
    let (ponto, linear) = (
        RenderInstance::pack_sampling(1, 0),
        RenderInstance::pack_sampling(2, 0),
    );
    assert!(filter_tag_magnifies_by_point(1) && !filter_tag_magnifies_by_point(2));
    let casos = [
        ("Inherit · Smooth", herda, ImageFilterMode::Smooth, true),
        (
            "Inherit · PixelArt",
            herda,
            ImageFilterMode::PixelArt,
            false,
        ),
        ("Nearest · Smooth", ponto, ImageFilterMode::Smooth, false),
        ("Linear · PixelArt", linear, ImageFilterMode::PixelArt, true),
    ];
    let lidos = casos.map(|(nome, s, p, _)| {
        let n = intermedios(&mut pass, &gpu, s, p);
        println!("  {nome:<20} intermedios {n}");
        n
    });
    assert!(
        lidos[0] >= 8,
        "CONTROLO: a lei bilinear desta fixtura tem de dar uma rampa (leu {})",
        lidos[0]
    );
    for ((nome, _, _, rampa), n) in casos.iter().zip(lidos) {
        if *rampa {
            assert!(
                n >= 8,
                "{nome}: esperava a rampa bilinear, leu {n} intermedios"
            );
        } else {
            assert!(
                n <= 1,
                "{nome}: esperava a ampliacao por PONTO, leu {n} intermedios"
            );
        }
    }
}
