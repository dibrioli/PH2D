//! ⭐⭐⭐ **O QUE O COMPOSITOR FAZ COM MEIA COBERTURA** — a pergunta que decide em que ESPAÇO uma
//! imagem pré-multiplicada tem de ser escrita.
//!
//! # ⛔⛔⛔ Porque ela se MEDE e não se deduz
//!
//! Report do dono (2026-09-19, com foto): *«toda forma apresenta uma falsa outline branca de 1
//! pixel»*. Um rebordo **mais claro que a peça E que o fundo** só pode nascer de um pixel de
//! cobertura parcial escrito na convenção errada — e qual é a certa é uma afirmação sobre o
//! **CONSUMIDOR**, não sobre a nossa aritmética.
//!
//! ⚠️⚠️ **E este repositório tem uma nota MEDIDA que responde ao contrário** — a
//! `ph2d_render::premul::premultiply_rgba8_in_linear` diz, por escrito, que pré-multiplicar **em
//! linear** é o que cura um *«light halo at the silhouette edge»*. Ela descreve o pipeline de
//! SPRITES, que tem dois consumidores (o shader com decode de hardware e o Vello) e por isso outra
//! resposta. *Uma lei portada traz a premissa do alvo sobre a disposição DELE* ⇒ mede-se este
//! caminho, que é o `draw_stable_image` sobre o `VelloPass`.
//!
//! # A régua: um rebordo é um pixel FORA do intervalo dos vizinhos
//!
//! A cena tem, lado a lado, os três pixels que uma silhueta produz: o fundo sozinho, a peça opaca, e
//! um pixel de **meia cobertura** — escrito nas duas convenções candidatas. *Um pixel de meia
//! cobertura tem de aterrar ENTRE os dois vizinhos; o que aterra fora deles é o rebordo que o dono
//! fotografou.*

use std::sync::Arc;

use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_vector::{StableImage, VectorScene};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill, ImageQuality};

/// O formato de superfície que o construtor do `VelloPass` pede para o blitter — esta sonda nunca
/// apresenta, e lê o intermédio.
const SURFACE: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
/// O cinzento do canvas do modelador, em byte sRGB.
const FUNDO: u8 = 110;

fn try_headless_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// A curva do [`ph2d_color::srgb`], em unidades.
fn srgb(linear: f32) -> f32 {
    ph2d_color::srgb::linear_to_srgb_unit(linear)
}

/// Uma faixa de `3×1` com os três pixels da régua, em alfa **pré-multiplicado**.
///
/// O pixel `1` é o que está em causa: a peça é branca (`C = 1,0` em linear) e cobre **meio** pixel.
fn faixa(meio: [u8; 4]) -> StableImage {
    let px = vec![
        // 0 — nada: o fundo tem de aparecer inteiro.
        0, 0, 0, 0, // 1 — meia cobertura, na convenção que o chamador escolheu.
        meio[0], meio[1], meio[2], meio[3], // 2 — a peça, opaca.
        255, 255, 255, 255,
    ];
    StableImage::from_rgba_premultiplied(Arc::new(px), 3, 1).expect("3x1")
}

fn cena(img: &StableImage) -> VectorScene {
    let mut s = VectorScene::new();
    s.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(FUNDO, FUNDO, FUNDO, 255),
        None,
        &Rect::new(0.0, 0.0, 3.0, 1.0),
    );
    s.draw_stable_image_transformed(img, Affine::IDENTITY, ImageQuality::Low);
    s
}

/// ⭐⭐⭐ **A MEDIÇÃO** — imprime o que cada convenção devolve, e afirma qual delas produz rebordo.
#[test]
#[ignore = "precisa de GPU"]
fn o_pixel_de_meia_cobertura_aterra_entre_os_vizinhos() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let Ok(mut pass) = VelloPass::new(&gpu, SURFACE, (3, 1)) else {
        println!("sem VelloPass — saltado");
        return;
    };
    // `a = 0,5`; a peça é branca, logo `C = 1,0` em linear.
    let a = 128.0 / 255.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let em_linear = (srgb(a) * 255.0 + 0.5) as u8; // `sRGB(C·a)` — o que os dois motores escrevem hoje
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let em_ecra = (srgb(1.0) * a).mul_add(255.0, 0.5) as u8; // `sRGB(C)·a`
    let mut mede = |meio: [u8; 4]| -> [u8; 3] {
        let img = faixa(meio);
        let bytes = pass
            .render_and_readback(&gpu, cena(&img).inner(), (3, 1))
            .expect("o readback");
        [bytes[0], bytes[4], bytes[8]]
    };
    let linear = mede([em_linear, em_linear, em_linear, 128]);
    let ecra = mede([em_ecra, em_ecra, em_ecra, 128]);

    println!("\n  o que a imagem leva no pixel do meio:");
    println!("    pré-multiplicado em LINEAR (hoje) ... {em_linear}");
    println!("    pré-multiplicado em ECRÃ .......... {em_ecra}");
    println!("  o que o compositor devolve (fundo · meio · peça):");
    println!("    com o de hoje ..... {linear:?}");
    println!("    com o de ecrã ..... {ecra:?}");

    // ⭐ O CONTROLO vem primeiro: os dois vizinhos têm de ser o que se pediu, senão as colunas do
    // meio não estão a ser comparadas contra nada.
    for (nome, r) in [("linear", linear), ("ecrã", ecra)] {
        assert!(
            r[2] > r[0],
            "no arranjo `{nome}` a peça ({}) não ficou mais clara que o fundo ({}) — a fixtura não \
             contém uma silhueta e as asserções abaixo não afirmam nada",
            r[2],
            r[0]
        );
    }
    assert!(
        linear[1] > linear[2]
            || i32::from(linear[1]) - i32::from(linear[0])
                > 2 * (i32::from(linear[2]) - i32::from(linear[0])) / 3,
        "o pré-multiplicado em LINEAR devolveu {} entre {} e {} — se ele já não exagera, o rebordo \
         que o dono fotografou tem outra causa e esta sonda deixou de a conter",
        linear[1],
        linear[0],
        linear[2]
    );
    assert!(
        ecra[1] > ecra[0] && ecra[1] < ecra[2],
        "o pré-multiplicado em ECRÃ devolveu {} fora do intervalo [{}, {}] — a convenção que este \
         compositor lê não é a que o report supõe",
        ecra[1],
        ecra[0],
        ecra[2]
    );

    // ⭐⭐⭐ **E A LUZ DE COBERTURA ZERO** — a premissa da wave do BRILHO (2026-09-19), medida aqui.
    // Num pré-multiplicado, `a = 0` com `rgb ≠ 0` quer dizer *«acrescenta sem tapar»*; se este
    // compositor a somasse, a cobertura que aquela wave acrescentou ao halo seria desnecessária.
    let aditiva = mede([128, 128, 128, 0]);
    println!("    luz com alfa ZERO (128) ..... {aditiva:?}");
    assert!(
        aditiva[1] >= ecra[0],
        "a luz aditiva devolveu {} contra um fundo de {} — leitura impossível",
        aditiva[1],
        ecra[0]
    );
    if aditiva[1] == ecra[0] {
        println!(
            "    ⇒ este compositor DESCARTA luz com cobertura zero (a wave do brilho tinha razão)"
        );
    } else {
        println!(
            "    ⇒ este compositor SOMA luz com cobertura zero — a wave do brilho tem de ser relida"
        );
    }
}
