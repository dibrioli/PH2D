//! B19.10 empty-block summary: the composite with it is the composite without it, to the bit, over
//! the 22-mode stack of G1 made sparse, and it stays FRESH across every way a plane is written —
//! paint in place (`make_mut` moves it), paint over an undo snapshot (`make_mut` copies), undo, redo,
//! fill, clear, `try_unwrap` + rewrap, import of a new image.

use super::barreira::FIOS_DA_REGIAO;
use super::barreira_tests::{bits, pilha};
use super::resumo::{SEM_RESUMO, com_resumos};
use super::*;
use std::sync::atomic::Ordering::Relaxed;

fn compara(s: &LayerStack, src: &MapPixelSource, w: u32, h: u32, passo: &str) {
    let r = Region { x: 0, y: 0, w, h };
    let janela = Region {
        x: w / 3,
        y: h / 5,
        w: w / 2,
        h: h / 2,
    };
    for reg in [r, janela] {
        SEM_RESUMO.store(true, Relaxed);
        let sem = bits(&compor_regiao(s, src, w, h, reg, None));
        SEM_RESUMO.store(false, Relaxed);
        for fios in [1usize, 3, 16] {
            FIOS_DA_REGIAO.store(fios, Relaxed);
            let com = bits(&compor_regiao(s, src, w, h, reg, None));
            FIOS_DA_REGIAO.store(0, Relaxed);
            assert!(com == sem, "{passo}: {reg:?} {fios} fios");
        }
    }
}

/// Strokes only in some 64-px blocks: α = 0 elsewhere (whole empty blocks and partial ones).
fn esparsa(img: &mut LayerImage, w: u32, k: u32) {
    for (i, p) in img
        .rgba8_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let (x, y) = (i as u32 % w, i as u32 / w);
        if !(x / 64 + y / 9 + k).is_multiple_of(4) || x % 64 > 50 {
            p[3] = 0;
        }
    }
}

fn pinta(img: &mut LayerImage, w: u32, cx: u32, cy: u32, cor: [u8; 4]) {
    for (i, p) in img
        .rgba8_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let (x, y) = (i as u32 % w, i as u32 / w);
        if x.abs_diff(cx) < 23 && y.abs_diff(cy) < 11 {
            *p = cor;
        }
    }
}

#[test]
fn o_resumo_por_ladrilho_e_a_rota_sem_resumo_ao_bit_e_nunca_fica_velho() {
    let _trava = crate::compositor::compose::resumo::trava_das_chaves();
    let (w, h) = (333u32, 131u32);
    for espacial in [false, true] {
        let (s, mut src, _) = pilha(w, h, espacial);
        let base = s.root()[0];
        let rasters: Vec<LayerId> = src
            .images
            .keys()
            .copied()
            .filter(|&id| {
                id != base && matches!(s.get(id).map(|l| &l.kind), Some(LayerKind::Raster(_)))
            })
            .collect();
        for (k, id) in rasters.iter().enumerate() {
            let img = src.images.get_mut(id).unwrap();
            if k % 3 == 0 {
                *img = LayerImage::transparent(w, h);
            } else {
                esparsa(img, w, k as u32);
            }
        }
        // The probe below needs empty layers whose summary is really consulted.
        let l = rasters[0];
        let m = rasters[3];
        assert!(com_resumos(&s, &src, (w, h)).layer_vazios(l).is_some());
        compara(&s, &src, w, h, "vazia");
        // Paint in place: the plane is unique (+ the summary's Weak) → `make_mut` MOVES it.
        pinta(
            src.images.get_mut(&l).unwrap(),
            w,
            70,
            40,
            [200, 30, 90, 255],
        );
        compara(&s, &src, w, h, "pinta no lugar");
        // Paint over an undo snapshot → `make_mut` copies.
        let antes = src.images[&l].clone();
        pinta(
            src.images.get_mut(&l).unwrap(),
            w,
            200,
            90,
            [10, 220, 40, 180],
        );
        let depois = src.images[&l].clone();
        compara(&s, &src, w, h, "pinta sobre o instantâneo");
        src.insert(l, antes);
        compara(&s, &src, w, h, "desfaz");
        src.insert(l, depois);
        compara(&s, &src, w, h, "refaz");
        src.images.get_mut(&m).unwrap().rgba8_mut().fill(77);
        compara(&s, &src, w, h, "preenche");
        src.images.get_mut(&m).unwrap().rgba8_mut().fill(0);
        compara(&s, &src, w, h, "limpa");
        let img = src.images.get_mut(&l).unwrap();
        let plano = std::mem::take(&mut img.rgba8);
        let mut v = Arc::try_unwrap(plano).expect("único");
        v[(50 * w as usize + 300) * 4..][..4].copy_from_slice(&[1, 2, 3, 255]);
        img.rgba8 = Arc::new(v);
        compara(&s, &src, w, h, "tira, escreve e repõe");
        let mut nova = LayerImage::transparent(w, h);
        pinta(&mut nova, w, 10, 120, [255, 255, 0, 255]);
        src.insert(m, nova);
        compara(&s, &src, w, h, "importa");
    }
}
