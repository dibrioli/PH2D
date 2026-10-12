//! W18 — a leitura nova (`gif` + a composição) contra a do `image` (o oráculo que corre): os mesmos
//! quadros, píxel a píxel, e as mesmas durações — em GIFs com cada descarte, quadros parciais e
//! transparência, e o GIF do oráculo da W16.

use std::borrow::Cow;

use gif::{DisposalMethod, Encoder, Frame, Repeat};

use super::*;

/// Os quadros pelo `image` 0.25 (o caminho da W16).
fn by_image(bytes: &[u8]) -> Vec<(Vec<u8>, u32)> {
    use image::AnimationDecoder as _;
    let dec = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(bytes.to_vec())).unwrap();
    dec.into_frames()
        .map_while(Result::ok)
        .map(|f| {
            let (n, d) = f.delay().numer_denom_ms();
            (f.into_buffer().into_raw(), n / d.max(1))
        })
        .collect()
}

fn by_us(bytes: &[u8]) -> Vec<(Vec<u8>, u32)> {
    GifFrames::new(bytes)
        .unwrap()
        .map(|(px, ms)| (px.rgba, ms))
        .collect()
}

/// Um GIF 6×5 de 4 quadros: o 1.º inteiro; os outros parciais, com um índice transparente, cada um
/// com o descarte `dispose[k]`.
fn gif_with(dispose: [DisposalMethod; 4]) -> Vec<u8> {
    let palette = [0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    let mut out = Vec::new();
    {
        let mut e = Encoder::new(&mut out, 6, 5, &palette).unwrap();
        e.set_repeat(Repeat::Infinite).unwrap();
        let rects: [(u16, u16, u16, u16); 4] =
            [(0, 0, 6, 5), (1, 1, 3, 2), (2, 0, 4, 4), (0, 3, 6, 2)];
        for (k, (l, t, w, h)) in rects.into_iter().enumerate() {
            let buf: Vec<u8> = (0..usize::from(w) * usize::from(h))
                .map(|i| ((i + k) % 5) as u8)
                .collect();
            let f = Frame {
                left: l,
                top: t,
                width: w,
                height: h,
                buffer: Cow::Owned(buf),
                dispose: dispose[k],
                // O índice 4 (branco) é o transparente dos quadros parciais.
                transparent: (k > 0).then_some(4),
                delay: [2, 7, 0, 13][k],
                ..Frame::default()
            };
            e.write_frame(&f).unwrap();
        }
    }
    out
}

/// ⭐⭐ As duas leituras dão os MESMOS quadros e durações, em cada combinação de descartes.
#[test]
fn our_gif_frames_are_the_image_crates_pixel_for_pixel() {
    use DisposalMethod::{Any, Background, Keep, Previous};
    let cases = [
        [Keep; 4],
        [Background; 4],
        [Previous; 4],
        [Any, Previous, Background, Keep],
        [Keep, Background, Previous, Any],
    ];
    for d in cases {
        let g = gif_with(d);
        assert_eq!(by_us(&g), by_image(&g), "descartes {d:?}");
    }
    let w16 = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w16_anim.gif"
    ))
    .unwrap();
    let ours = by_us(&w16);
    assert!(ours.len() > 1, "o GIF da W16 tem vários quadros");
    assert_eq!(ours, by_image(&w16), "o GIF do oráculo da W16");
}

/// O ecrã lógico acima do tecto da porta das imagens não se lê (o `image` tentava alocá-lo).
#[test]
fn a_gif_screen_over_the_cap_is_refused() {
    let side = u16::try_from(ph2d_asset::loader::MAX_DIMENSION + 1).unwrap();
    let mut out = Vec::new();
    {
        let mut e = Encoder::new(&mut out, side, 1, &[0, 0, 0]).unwrap();
        let f = Frame {
            width: 1,
            height: 1,
            buffer: Cow::Owned(vec![0]),
            ..Frame::default()
        };
        e.write_frame(&f).unwrap();
    }
    assert!(GifFrames::new(&out).is_none());
}
