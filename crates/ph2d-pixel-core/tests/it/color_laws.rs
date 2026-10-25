//! **Leis próprias da P2** (cada uma com o seu controlo): conversão de modo, tinta, operações da
//! paleta.

use ph2d_pixel_core::color::ToGray;
use ph2d_pixel_core::convert::{Bayer, Convert, Dither, convert};
use ph2d_pixel_core::ink::{Ink, apply as ink, shade};
use ph2d_pixel_core::palette::{SortKey, is_permutation, sort_permutation};
use ph2d_pixel_core::{
    Background, ColorMode, NewSprite, Op, OpError, Patch, Rect, Sprite, Whole, apply,
};

fn sprite(mode: ColorMode, bg: Background, w: u32, h: u32) -> Sprite {
    Sprite::new(NewSprite {
        width: w,
        height: h,
        mode,
        background: bg,
    })
    .expect("sprite")
}

fn paint(s: &mut Sprite, r: Rect, pixels: Vec<u8>) {
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    apply(
        s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
}

fn cel_bytes(s: &Sprite) -> Vec<u8> {
    let key = (s.layers()[0].id, s.frames()[0].id);
    s.cel(key).map(|c| c.data.clone()).unwrap_or_default()
}

/// A paleta inteira pintada em RGBA vai a Indexed e volta byte-igual: cada cor da paleta é a mais
/// próxima de si mesma (controlo: uma cor FORA da paleta não volta igual).
#[test]
fn palette_colors_survive_rgb_to_indexed_to_rgb_and_an_outsider_does_not() {
    let mut s = sprite(ColorMode::Rgba, Background::Transparent, 31, 1);
    let pal: Vec<[u8; 4]> = s.palette()[1..].to_vec();
    paint(&mut s, Rect::new(0, 0, 31, 1), pal.concat());
    let before = cel_bytes(&s);
    let w = convert(&s, ColorMode::Indexed, Convert::default()).expect("muda");
    let inv = apply(&mut s, Op::Whole(Box::new(w))).expect("to indexed");
    assert_eq!(s.mode(), ColorMode::Indexed);
    let back = convert(&s, ColorMode::Rgba, Convert::default()).expect("muda");
    apply(&mut s, Op::Whole(Box::new(back))).expect("to rgb");
    assert_eq!(cel_bytes(&s), before, "ida e volta pela paleta");
    // Desfazer a 1.ª conversão do estado Indexed devolve o RGBA original ao byte.
    let mut t = sprite(ColorMode::Rgba, Background::Transparent, 31, 1);
    paint(&mut t, Rect::new(0, 0, 31, 1), pal.concat());
    let w = convert(&t, ColorMode::Indexed, Convert::default()).expect("muda");
    let _ = inv;
    let inv = apply(&mut t, Op::Whole(Box::new(w))).expect("to indexed");
    apply(&mut t, inv).expect("undo");
    assert_eq!(t.mode(), ColorMode::Rgba);
    assert_eq!(cel_bytes(&t), before, "desfazer devolve o RGBA");
    // Controlo.
    let mut u = sprite(ColorMode::Rgba, Background::Transparent, 1, 1);
    paint(&mut u, Rect::new(0, 0, 1, 1), vec![1, 2, 200, 255]);
    let w = convert(&u, ColorMode::Indexed, Convert::default()).expect("muda");
    apply(&mut u, Op::Whole(Box::new(w))).expect("idx");
    let w = convert(&u, ColorMode::Rgba, Convert::default()).expect("muda");
    apply(&mut u, Op::Whole(Box::new(w))).expect("rgb");
    assert_ne!(cel_bytes(&u), vec![1, 2, 200, 255]);
}

/// Numa camada normal o alfa 0 vai ao índice transparente e nenhuma célula opaca cai nele (o
/// Aseprite, medido); na Background o índice 0 é uma cor como as outras.
#[test]
fn the_transparent_index_is_reserved_except_on_the_background_layer() {
    let black = [0, 0, 0, 255];
    let mut s = sprite(ColorMode::Rgba, Background::Transparent, 2, 1);
    paint(
        &mut s,
        Rect::new(0, 0, 2, 1),
        [black, [9, 9, 9, 0]].concat(),
    );
    let w = convert(&s, ColorMode::Indexed, Convert::default()).expect("muda");
    let data = &w.cels.values().next().expect("cel").data;
    assert_eq!(data[1], s.transparent_index(), "alfa 0 = transparente");
    assert_ne!(
        data[0],
        s.transparent_index(),
        "o preto opaco não é o transparente"
    );
    let mut b = sprite(ColorMode::Rgba, Background::Black, 1, 1);
    paint(&mut b, Rect::new(0, 0, 1, 1), black.to_vec());
    let w = convert(&b, ColorMode::Indexed, Convert::default()).expect("muda");
    assert_eq!(
        w.cels.values().next().expect("cel").data,
        vec![0],
        "Background: o 0 é o preto"
    );
}

#[test]
fn rgb_to_gray_uses_the_chosen_method_and_keeps_alpha() {
    let mut s = sprite(ColorMode::Rgba, Background::Transparent, 1, 1);
    paint(&mut s, Rect::new(0, 0, 1, 1), vec![0, 0, 255, 77]);
    for (m, g) in [
        (ToGray::Luminance, 18),
        (ToGray::Hsv, 255),
        (ToGray::Hsl, 127),
    ] {
        let w = convert(
            &s,
            ColorMode::Gray,
            Convert {
                to_gray: m,
                ..Convert::default()
            },
        )
        .expect("muda");
        assert_eq!(
            w.cels.values().next().expect("cel").data,
            vec![g, 77],
            "{m:?}"
        );
    }
    assert!(
        convert(&s, ColorMode::Rgba, Convert::default()).is_none(),
        "já é RGB"
    );
}

/// O dither ordenado é ancorado na TELA: dois cels com caixas diferentes dão o mesmo padrão nas
/// mesmas células do sprite; e um meio-tom entre duas cores mistura as duas (controlo: sem dither
/// sai uma só).
#[test]
fn ordered_dither_is_anchored_to_the_canvas_and_mixes_a_midtone() {
    let mid = [128, 128, 128, 255];
    let run = |x0: i32, dither: Dither| {
        let mut s = sprite(ColorMode::Rgba, Background::Transparent, 16, 16);
        paint(&mut s, Rect::new(x0, 0, 8, 8), [mid; 64].concat());
        let bw = vec![[0, 0, 0, 0], [0, 0, 0, 255], [255, 255, 255, 255]];
        apply(
            &mut s,
            Op::Palette {
                colors: bw,
                transparent: 0,
            },
        )
        .expect("pal");
        let w = convert(
            &s,
            ColorMode::Indexed,
            Convert {
                dither,
                ..Convert::default()
            },
        )
        .expect("muda");
        let cel = w.cels.values().next().expect("cel").clone();
        (cel.bbox, cel.data)
    };
    for b in [Bayer::M2, Bayer::M4, Bayer::M8] {
        let (_, a) = run(0, Dither::Ordered(b));
        let (_, c) = run(b.side() as i32, Dither::Ordered(b));
        assert_eq!(a, c, "{b:?}: deslocar o cel um período não muda o padrão");
        let distinct: std::collections::BTreeSet<u8> = a.iter().copied().collect();
        assert!(
            distinct.len() >= 2,
            "{b:?}: um meio-tom tem de misturar cores"
        );
    }
    let (_, plain) = run(0, Dither::None);
    assert_eq!(
        plain
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        1,
        "controlo: sem dither é uma cor só"
    );
}

/// Floyd–Steinberg conserva a média: um cinza a 25 % numa paleta preto/branco dá ~25 % de branco.
#[test]
fn floyd_steinberg_preserves_the_average_tone() {
    let mut s = sprite(ColorMode::Rgba, Background::Black, 32, 32);
    let gray = [64, 64, 64, 255];
    paint(&mut s, Rect::new(0, 0, 32, 32), [gray; 1024].concat());
    apply(
        &mut s,
        Op::Palette {
            colors: vec![[0, 0, 0, 255], [255, 255, 255, 255]],
            transparent: 0,
        },
    )
    .expect("pal");
    let w = convert(
        &s,
        ColorMode::Indexed,
        Convert {
            dither: Dither::FloydSteinberg,
            ..Convert::default()
        },
    )
    .expect("muda");
    let white = w
        .cels
        .values()
        .next()
        .expect("cel")
        .data
        .iter()
        .filter(|&&v| v == 1)
        .count();
    assert!((230..=284).contains(&white), "~25 % de 1024: {white}");
}

#[test]
fn inks_follow_the_aseprite_rules() {
    let s = sprite(ColorMode::Rgba, Background::Transparent, 1, 1);
    let v = |c: [u8; 4]| u32::from_le_bytes(c);
    let red = v([255, 0, 0, 255]);
    let half_blue = v([0, 0, 255, 128]);
    let clear = v([0, 0, 0, 0]);
    let green = v([0, 255, 0, 255]);
    let none: &[u32] = &[];
    // Simple: opaco substitui, transparente apaga, meio alfa mistura.
    assert_eq!(ink(&s, Ink::Simple, red, green, none, 1), red);
    assert_eq!(ink(&s, Ink::Simple, clear, green, none, 1), clear);
    assert_eq!(
        ink(&s, Ink::Simple, half_blue, green, none, 1).to_le_bytes(),
        [0, 127, 128, 255]
    );
    // Alpha Compositing: a transparente não faz nada.
    assert_eq!(ink(&s, Ink::AlphaCompositing, clear, green, none, 1), green);
    // Copy Alpha+Color: substitui até o alfa.
    assert_eq!(
        ink(&s, Ink::CopyAlphaColor, half_blue, green, none, 1),
        half_blue
    );
    // Lock Alpha: não pinta onde não há tinta; guarda o alfa de destino.
    assert_eq!(ink(&s, Ink::LockAlpha, red, clear, none, 1), clear);
    let ghost = v([0, 255, 0, 90]);
    assert_eq!(
        ink(&s, Ink::LockAlpha, red, ghost, none, 1).to_le_bytes()[3],
        90
    );
    // Shading: um degrau, parado nas pontas, intocado fora da rampa.
    let ramp = [10, 20, 30];
    assert_eq!(shade(&ramp, 20, 1), 30);
    assert_eq!(shade(&ramp, 30, 1), 30);
    assert_eq!(shade(&ramp, 10, -1), 10);
    assert_eq!(shade(&ramp, 99, 1), 99, "fora da rampa fica");
    assert_eq!(ink(&s, Ink::Shading, red, 20, &ramp, -1), 10);
}

#[test]
fn palette_op_rejects_a_bad_palette_and_sort_is_a_stable_permutation() {
    let mut s = sprite(ColorMode::Indexed, Background::Transparent, 1, 1);
    assert_eq!(
        apply(
            &mut s,
            Op::Palette {
                colors: vec![],
                transparent: 0
            }
        ),
        Err(OpError::BadPalette)
    );
    assert_eq!(
        apply(&mut s, Op::Remap { perm: vec![0, 0] }),
        Err(OpError::BadPermutation)
    );
    let colors = s.palette().to_vec();
    for key in SortKey::ALL {
        let p = sort_permutation(&colors, 0..colors.len(), key, false, Some(0));
        assert!(is_permutation(&p), "{key:?}");
        assert_eq!(p[0], 0, "o transparente não se move");
    }
    let p = sort_permutation(&colors, 0..colors.len(), SortKey::Luminance, false, Some(0));
    let l: Vec<u8> = p[1..]
        .iter()
        .map(|&i| ph2d_pixel_core::color::luma(colors[i]))
        .collect();
    assert!(
        l.windows(2).all(|w| w[0] <= w[1]),
        "sobe em luminância: {l:?}"
    );
}

#[test]
fn whole_op_rejects_a_cel_of_the_wrong_size() {
    let mut s = sprite(ColorMode::Rgba, Background::Transparent, 2, 2);
    paint(&mut s, Rect::new(0, 0, 1, 1), vec![1, 2, 3, 255]);
    let mut w: Whole = convert(&s, ColorMode::Indexed, Convert::default()).expect("muda");
    let key = *w.cels.keys().next().expect("cel");
    w.cels.get_mut(&key).expect("cel").data.push(0);
    assert_eq!(
        apply(&mut s, Op::Whole(Box::new(w))),
        Err(OpError::BadCel(key))
    );
    assert_eq!(s.mode(), ColorMode::Rgba, "recusado sem mudar nada");
}
