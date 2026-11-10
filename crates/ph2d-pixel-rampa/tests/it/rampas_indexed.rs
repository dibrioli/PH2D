//! P13 R3: o modo RGB × ImageMagick, e `PreservaRampas` (a claridade nunca se inverte).

use ph2d_color::OklchColor;
use ph2d_pixel_core::convert::{Convert, Dither, convert};
use ph2d_pixel_core::{Background, ColorMode, DocOp, NewSprite, Op, Sprite, apply, rampa_de};
use ph2d_pixel_rampa::{
    PreservaRampas, converte_oklab, converte_preservando_rampas, oklab_de, srgb8_de_oklch,
};

use crate::indexed::{DITHERS, fixture, foto, indexed_data, indices, sprite};

fn opt(d: Dither) -> Convert {
    Convert {
        dither: d,
        ..Convert::default()
    }
}

/// O nosso RGB é a cor EXACTA mais próxima; o `magick +dither -remap` (7.1.2, Q16 HDRI) não é:
/// a procura dele na árvore de cubos poda e erra 124 de 4096 píxeis da `foto_p13` (ex.: (183,146,127)
/// → (186,218,120), d² 5242, quando (222,160,128) está a d² 1717; o Pillow 13 `quantize` erra 129).
/// Lei: onde diferimos, a nossa cor está ESTRITAMENTE mais perto; onde não, é o mesmo índice.
/// O `-ordered-dither o8x8` dele posteriza cada canal a 2 níveis ANTES do `-remap` e o FS difunde
/// com outra aritmética: diferem por desenho; os números medidos ficam fixos (mudou ⇒ reconfira).
#[test]
fn o_indexed_rgb_e_o_do_imagemagick() {
    let fx = fixture();
    let s = sprite(&fx, &foto(&fx), false);
    let rgb: Vec<[u8; 4]> = foto(&fx)
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2], 255])
        .collect();
    let d2 = |c: [u8; 4], i: u8| {
        let p = s.palette()[usize::from(i)];
        (0..3)
            .map(|k| (i32::from(c[k]) - i32::from(p[k])).pow(2))
            .sum::<i32>()
    };
    let mut medidos = Vec::new();
    for (nome, d) in [
        ("nenhum", Dither::None),
        ("o8x8", Dither::Ordered(ph2d_pixel_core::convert::Bayer::M8)),
        ("floyd_steinberg", Dither::FloydSteinberg),
    ] {
        let nosso = indexed_data(&convert(&s, ColorMode::Indexed, opt(d)).expect("converte"));
        let im = indices(&fx["saidas"][nome]["indices"]);
        if nome == "nenhum" {
            for (j, (&a, &b)) in nosso.iter().zip(&im).enumerate() {
                assert!(
                    a == b || d2(rgb[j], a) < d2(rgb[j], b),
                    "píxel {j}: {a} × IM {b}"
                );
            }
        }
        medidos.push((nome, nosso.iter().zip(&im).filter(|(a, b)| a != b).count()));
    }
    eprintln!("píxeis diferentes do ImageMagick (de 4096): {medidos:?}");
    assert_eq!(medidos, MEDIDO, "a diferença mudou: reconfira a nota");
}

/// Medido 09/11 (P13 R3) sobre a `foto_p13`, de 4096 píxeis.
const MEDIDO: [(&str, usize); 3] = [("nenhum", 124), ("o8x8", 3529), ("floyd_steinberg", 1089)];

/// Os pares (L da origem, degrau pela ordem de L) de cada rampa, e quantas inversões há.
fn inversoes(s: &Sprite, rgb: &[[u8; 4]], idx: &[u8]) -> usize {
    let p = PreservaRampas::new(s);
    let mut por_rampa: std::collections::BTreeMap<usize, Vec<(f32, usize)>> = Default::default();
    for (&c, &i) in rgb.iter().zip(idx) {
        if let Some((r, pos)) = p.degrau(i) {
            por_rampa.entry(r).or_default().push((oklab_de(c)[0], pos));
        }
    }
    let mut n = 0;
    for v in por_rampa.values_mut() {
        v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        n += v.windows(2).filter(|w| w[1].1 < w[0].1).count();
    }
    n
}

/// A paleta de duas rampas que a cor OKLab mais próxima INVERTE: `A` escura e saturada, `B`
/// clara e quase cinza; `c1` (mais escura, cinza) cai em `B`, `c2` (mais clara, saturada) em `A`.
fn armadilha() -> (Sprite, Vec<[u8; 4]>) {
    let lch = |l, c| srgb8_de_oklch(OklchColor::new(l, c, 30.0, 1.0));
    let (a, b) = (lch(0.5, 0.17), lch(0.62, 0.02));
    let pal = [a, b, [0, 0, 0, 255], [255, 255, 255, 255]];
    let spec = NewSprite {
        width: 2,
        height: 1,
        mode: ColorMode::Rgba,
        background: Background::White,
    };
    let mut s = Sprite::with_palette(spec, &pal).expect("sprite");
    let add = DocOp::AddRamp {
        name: "pele".into(),
        indices: vec![0, 1],
    };
    apply(&mut s, Op::Doc(Box::new(add))).expect("rampa");
    let (c1, c2) = (lch(0.54, 0.02), lch(0.58, 0.17));
    (s, vec![c1, c2])
}

fn pinta(mut s: Sprite, px: &[[u8; 4]]) -> Sprite {
    let key = (s.layers()[0].id, s.frames()[0].id);
    let mut w =
        convert(&s, ColorMode::Rgba, opt(Dither::None)).unwrap_or_else(|| ph2d_pixel_core::Whole {
            mode: ColorMode::Rgba,
            palette: s.palette().to_vec(),
            transparent: s.transparent_index(),
            cels: Default::default(),
            tilesets: Vec::new(),
        });
    let r = ph2d_pixel_core::Rect::new(0, 0, s.width(), s.height());
    w.cels
        .insert(key, ph2d_pixel_core::Cel::image(r, px.concat()));
    apply(&mut s, Op::Whole(Box::new(w))).expect("pinta");
    s
}

#[test]
fn manter_rampas_nunca_inverte_a_claridade() {
    // A armadilha: a OKLab pura inverte (a pré-condição), as rampas não.
    let (s, px) = armadilha();
    let s = pinta(s, &px);
    let ok = indexed_data(&converte_oklab(&s, opt(Dither::None)).expect("oklab"));
    assert_eq!(
        ok,
        vec![1, 0],
        "a armadilha deixou de armar: a OKLab pura já não inverte"
    );
    let pr = indexed_data(&converte_preservando_rampas(&s, opt(Dither::None)).expect("rampas"));
    assert_eq!(
        pr,
        vec![0, 1],
        "c1 (mais escura) tem de ficar no degrau mais escuro"
    );
    // A foto: zero inversões sem dither (medido 09/11: o RGB e a OKLab pura também dão 0 nela —
    // os dentes da régua são a armadilha acima).
    let fx = fixture();
    let s = sprite(&fx, &foto(&fx), true);
    let rgb: Vec<[u8; 4]> = foto(&fx)
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2], 255])
        .collect();
    let pr = indexed_data(&converte_preservando_rampas(&s, opt(Dither::None)).expect("rampas"));
    let ok = indexed_data(&converte_oklab(&s, opt(Dither::None)).expect("oklab"));
    let rg = indexed_data(&convert(&s, ColorMode::Indexed, opt(Dither::None)).expect("rgb"));
    let (n_pr, n_ok, n_rg) = (
        inversoes(&s, &rgb, &pr),
        inversoes(&s, &rgb, &ok),
        inversoes(&s, &rgb, &rg),
    );
    eprintln!("inversões na foto_p13: rampas {n_pr} · oklab {n_ok} · rgb {n_rg}");
    assert_eq!(n_pr, 0);
}

/// Com qualquer dither: toda célula fica na paleta e o `Op::Whole` aceita a conversão sem partir
/// as rampas (a paleta não muda, por isso nenhuma rampa precisa de remapear).
#[test]
fn manter_rampas_com_dither_fica_na_paleta_e_nas_rampas() {
    let fx = fixture();
    let s0 = sprite(&fx, &foto(&fx), true);
    for (nome, d) in DITHERS {
        let w = converte_preservando_rampas(&s0, opt(d)).expect("converte");
        assert!(
            indexed_data(&w)
                .iter()
                .all(|&i| usize::from(i) < s0.palette().len()),
            "{nome}"
        );
        let mut s = s0.clone();
        apply(&mut s, Op::Whole(Box::new(w))).expect(nome);
        assert_eq!(s.ramps(), s0.ramps(), "{nome}: as rampas ficam");
        assert_eq!(rampa_de(s.ramps(), 2).map(|r| r.1), Some(2));
    }
}

/// O teto perceptual: o ΔE OKLab médio da nossa OKLab ≤ o do pngquant sobre a `foto_p13`.
#[test]
#[ignore = "pngquant por instalar (sudo do dono)"]
fn o_indexed_perceptual_nao_e_pior_que_o_pngquant() {
    let fx = fixture();
    let pq = &fx["pngquant"];
    assert!(
        !pq.is_null(),
        "corra docs/PixelArt/ferramentas/oraculo_indexed_imagemagick.py com o pngquant instalado"
    );
    let s = sprite(&fx, &foto(&fx), false);
    let rgb: Vec<[u8; 4]> = foto(&fx)
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2], 255])
        .collect();
    let ok = indexed_data(&converte_oklab(&s, opt(Dither::None)).expect("oklab"));
    let de = |c: [u8; 4], i: u8| {
        let (p, q) = (oklab_de(c), oklab_de(s.palette()[usize::from(i)]));
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
    };
    let nosso: f32 = rgb.iter().zip(&ok).map(|(&c, &i)| de(c, i)).sum::<f32>() / rgb.len() as f32;
    let deles = pq["de_medio"].as_f64().expect("de_medio") as f32;
    assert!(nosso <= deles, "ΔE médio {nosso} > pngquant {deles}");
}
