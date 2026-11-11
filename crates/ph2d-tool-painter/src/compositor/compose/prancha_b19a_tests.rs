//! **Prancha B19a** (B19.1 recorte + B19.3 Shadows/Highlights), 3 fundos × 1080p/4K.
//!
//! O «antes» é a lei antiga, no mesmo processo:
//! - B19.1: a lei antiga SALTAVA a camada escondida / a opacidade 0 sem tocar no elo do recorte, o que é
//!   exatamente a pilha SEM essa camada — o antes é `composite` da pilha sem a base.
//! - B19.3: `sh_lei_antiga` é o `apply_shadows_highlights` de 87f4b847a0 verbatim (o ramo `l > 1e-4`,
//!   cinza abaixo), aplicado ao acumulador debaixo do ajuste; o mesmo caminho com a lei de hoje tem de
//!   dar o `composite` com o ajuste AO BIT (prova de que a montagem é a do compositor).

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::Tool as _;
use ph2d_painter_effects::adjustments::{
    AdjustWindow, AdjustmentKind, AdjustmentParams, ShadowsHighlightsParams,
    apply_shadows_highlights, separable_blur_scalar,
};

pub(super) fn fundo(w: u32, h: u32, qual: u8) -> Vec<u8> {
    if qual == 2 {
        let mut t = PainterTool::default();
        t.agenda_papel_texturizado(7);
        t.bind_document(7, vec![255u8; (w * h * 4) as usize], w, h);
        t.on_tick(1000.0 / 60.0);
        t.invalidate_composite();
        return t.take_preview_arc().expect("o papel").0.to_vec();
    }
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if qual == 1 {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

pub(super) fn camada(w: u32, h: u32, f: impl Fn(u32, u32) -> Option<[u8; 4]>) -> LayerImage {
    let mut img = LayerImage::transparent(w, h);
    for (i, p) in img.rgba8.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        if let Some(c) = f(i as u32 % w, i as u32 / w) {
            p.copy_from_slice(&c);
        }
    }
    img
}

/// B19.1: `fundo` · `base` (disco verde de raio h/5) · `recorte` (riscas, recortado). `caso` 0 = base
/// escondida, 1 = base a opacidade 0. Devolve (antes, depois).
fn recorte(w: u32, h: u32, bg: &[u8], caso: u8) -> (Vec<u8>, Vec<u8>) {
    let (cx, cy, r) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 5.0);
    let disco = camada(w, h, |x, y| {
        let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
        (d <= r).then_some([60, 170, 90, 255])
    });
    let faixa = (h / 36).max(2);
    let riscas = camada(w, h, |x, y| {
        ((x + y) / faixa)
            .is_multiple_of(2)
            .then_some([210, 40, 60, 255])
    });
    let monta_pilha = |com_base: bool| {
        let mut s = LayerStack::new();
        let mut src = MapPixelSource::default();
        let f = s.add_raster("fundo", w, h).unwrap();
        src.insert(
            f,
            LayerImage {
                width: w,
                height: h,
                rgba8: bg.to_vec(),
            },
        );
        let base = com_base.then(|| {
            let b = s.add_raster("base", w, h).unwrap();
            src.insert(b, disco.clone());
            b
        });
        let rc = s.add_raster("recorte", w, h).unwrap();
        s.set_clipping(rc, true);
        src.insert(rc, riscas.clone());
        (s, src, base)
    };
    let (s0, src0, _) = monta_pilha(false);
    let antes = composite(&s0, &src0, w, h);
    let (mut s, src, base) = monta_pilha(true);
    let base = base.unwrap();
    if caso == 0 {
        s.set_visible(base, false);
    } else {
        s.get_mut(base).unwrap().opacity = 0.0;
    }
    (antes, composite(&s, &src, w, h))
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn luma(px: &[f32; 4]) -> f32 {
    0.2126 * px[0].clamp(0.0, 1.0) + 0.7152 * px[1].clamp(0.0, 1.0) + 0.0722 * px[2].clamp(0.0, 1.0)
}

/// O ORÁCULO do antes: `apply_shadows_highlights` de 87f4b847a0, verbatim (serial).
fn sh_lei_antiga(p: &ShadowsHighlightsParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    let mut local_lo: Vec<f32> = acc.iter().map(luma).collect();
    let mut local_hi = local_lo.clone();
    separable_blur_scalar(p.shadows_radius, &mut local_lo, win);
    separable_blur_scalar(p.highlights_radius, &mut local_hi, win);
    let tw_s = p.shadows_tonal_width.max(1e-3);
    let tw_h = p.highlights_tonal_width.max(1e-3);
    for (i, px) in acc.iter_mut().enumerate() {
        let l = luma(px);
        let ws = 1.0 - smoothstep(0.0, tw_s, local_lo[i]);
        let wh = smoothstep(1.0 - tw_h, 1.0, local_hi[i]);
        let mut new_l = l + p.shadows_amount * ws - p.highlights_amount * wh;
        new_l = (0.5 + (new_l - 0.5) * (1.0 + p.midtone_contrast)).clamp(0.0, 1.0);
        let mut d = [
            px[0].clamp(0.0, 1.0),
            px[1].clamp(0.0, 1.0),
            px[2].clamp(0.0, 1.0),
        ];
        if l > 1e-4 {
            let ratio = new_l / l;
            for c in &mut d {
                *c = (*c * ratio).clamp(0.0, 1.0);
            }
        } else {
            d = [new_l, new_l, new_l];
        }
        let cc = p.color_correction * (ws + wh).min(1.0);
        if cc != 0.0 {
            for c in &mut d {
                *c = (new_l + (*c - new_l) * (1.0 + cc)).clamp(0.0, 1.0);
            }
        }
        px[0] = d[0];
        px[1] = d[1];
        px[2] = d[2];
    }
}

/// B19.3: `fundo` · `ruido` (quadrado de lado h/3 de ruído quase preto, 0..=6 por canal, opaco) ·
/// Shadows/Highlights (Shadows 0,5). Devolve (antes, depois).
fn sombras(w: u32, h: u32, bg: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let lado = h / 3;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let ruido = camada(w, h, |x, y| {
        if x < x0 || y < y0 || x >= x0 + lado || y >= y0 + lado {
            return None;
        }
        let mut s = (y * w + x).wrapping_mul(2_654_435_761) ^ 0x9e37_79b9;
        let mut c = [0u8, 0, 0, 255];
        for k in &mut c[..3] {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *k = ((s >> 24) % 7) as u8;
        }
        Some(c)
    });
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let f = s.add_raster("fundo", w, h).unwrap();
    src.insert(
        f,
        LayerImage {
            width: w,
            height: h,
            rgba8: bg.to_vec(),
        },
    );
    let rd = s.add_raster("ruido", w, h).unwrap();
    src.insert(rd, ruido);
    let por_baixo = composite(&s, &src, w, h);
    let p = ShadowsHighlightsParams {
        shadows_amount: 0.5,
        ..ShadowsHighlightsParams::default()
    };
    let adj = s.add_adjustment(AdjustmentKind::ShadowsHighlights).unwrap();
    s.adjustment_mut(adj).unwrap().params = AdjustmentParams::ShadowsHighlights(p);
    let depois = composite(&s, &src, w, h);
    let win = AdjustWindow::full(w, h);
    let acc0: Vec<[f32; 4]> = por_baixo
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| [0, 1, 2, 3].map(|k| decode_byte(c[k])))
        .collect();
    let mut hoje = acc0.clone();
    apply_shadows_highlights(&p, &mut hoje, win);
    assert_eq!(
        encode(&hoje),
        depois,
        "a montagem do oráculo tem de ser a do compositor, ao bit"
    );
    let mut antes = acc0;
    sh_lei_antiga(&p, &mut antes, win);
    (encode(&antes), depois)
}

/// Antes | depois | o que muda (vermelho ∝ a diferença), no recorte central de lado `h/2`.
pub(super) fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * lado + 2 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = (((y0 + y) * w + x0 + x) * 4) as usize;
            let m = (0..3)
                .map(|c| a[i + c].abs_diff(d[i + c]))
                .max()
                .unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([a[i], a[i + 1], a[i + 2], 255]));
            par.put_pixel(
                x + lado + gap,
                y,
                image::Rgba([d[i], d[i + 1], d[i + 2], 255]),
            );
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B19a: grava as PNG em docs/Painter/missao_4k/pranchas/B19a/"]
fn prancha_b19a_recorte_e_sombras() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B19a"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let bg = fundo(w, h, qual);
            let casos = [
                ("recorte_base_escondida", recorte(w, h, &bg, 0)),
                ("recorte_base_opacidade0", recorte(w, h, &bg, 1)),
                ("sombras_ruido_quase_preto", sombras(w, h, &bg)),
            ];
            for (caso, (a, d)) in casos {
                assert_ne!(
                    a, d,
                    "{res} {nome} {caso}: a lei antiga tem de dar outra imagem"
                );
                monta(&a, &d, w, h)
                    .save(format!("{dir}/{res}_{nome}_{caso}_antes_depois.png"))
                    .expect("png");
            }
        }
    }
}
