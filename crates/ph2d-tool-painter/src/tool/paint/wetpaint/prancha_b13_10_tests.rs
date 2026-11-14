//! **Prancha B13.10**: o freio da água nas bordas da folha. Dois traços Wet Paint (de fábrica) junto às bordas
//! ESQUERDA e DIREITA, deixados a correr, sobre 3 fundos × 1080p/4K. «Antes» = a sonda linear de antes
//! ([`ph2d_wet_paint::flow::FREIO_LINEAR`]), «depois» = a sonda fina com limites 2D, no mesmo processo. PNG em
//! `docs/Painter/missao_4k/pranchas/B13.10/`: a borda esquerda e a direita | antes | depois | o que muda.

use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};
use std::sync::atomic::Ordering::Relaxed;

#[path = "prancha_b13_4_tests.rs"]
mod prancha_b13_4;

pub(super) fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

fn cena(w: u32, h: u32, qual: u8) -> Vec<u8> {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.add_raster_layer("tinta").expect("a camada transparente");
    let s = h as f32 / 1080.0;
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(40.0 * s);
    t.set_brush_color_srgb8([40, 30, 120]);
    let passos = 120;
    for x in [6.0 * s, w as f32 - 6.0 * s] {
        let ponto = |k: usize| [x, h as f32 * (0.25 + 0.5 * k as f32 / passos as f32)];
        let cp = |pos, phase| CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(ponto(0), PointerPhase::Down));
        for k in 1..passos {
            t.on_canvas_pointer(cp(ponto(k), PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(ponto(passos), PointerPhase::Up));
    }
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// Linha de cima: a borda esquerda (`lado × lado`, ampliada 4×); de baixo: a direita. Antes | depois | diferença.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let (lado, z, gap) = (h / 16, 4u32, 8u32);
    let q = lado * z;
    let mut par =
        image::RgbaImage::from_pixel(3 * q + 2 * gap, 2 * q + gap, image::Rgba([40, 40, 40, 255]));
    let y0 = h / 2 - lado / 2;
    for (linha, x0) in [0u32, w - lado].into_iter().enumerate() {
        for y in 0..q {
            for x in 0..q {
                let i = (((y0 + y / z) * w + x0 + x / z) * 4) as usize;
                let (pa, pd) = (&a[i..i + 4], &d[i..i + 4]);
                let m = (0..4).map(|c| pa[c].abs_diff(pd[c])).max().unwrap_or(0);
                let k = 255 - (u16::from(m) * 32).min(255) as u8;
                let top = linha as u32 * (q + gap) + y;
                par.put_pixel(x, top, image::Rgba([pa[0], pa[1], pa[2], 255]));
                par.put_pixel(x + q + gap, top, image::Rgba([pd[0], pd[1], pd[2], 255]));
                par.put_pixel(x + 2 * (q + gap), top, image::Rgba([255, k, k, 255]));
            }
        }
    }
    par
}

#[test]
#[ignore = "prancha B13.10: escreve PNGs em docs/Painter/missao_4k/pranchas/B13.10/"]
fn prancha_b13_10_o_freio_na_borda() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B13.10"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            ph2d_wet_paint::flow::FREIO_LINEAR.store(true, Relaxed);
            let a = cena(w, h, qual);
            ph2d_wet_paint::flow::FREIO_LINEAR.store(false, Relaxed);
            let d = cena(w, h, qual);
            let difs = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(d.as_chunks::<4>().0)
                .filter(|(x, y)| x != y)
                .count();
            let longe = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(d.as_chunks::<4>().0)
                .enumerate()
                .filter(|(i, (x, y))| x != y && (16..w - 16).contains(&(*i as u32 % w)))
                .count();
            let pior = a.iter().zip(&d).map(|(x, y)| x.abs_diff(*y)).max();
            eprintln!(
                "PRANCHA {res} {nome}: {difs} píxeis diferentes ({longe} a mais de 16 px da borda), pior {pior:?}"
            );
            monta(&a, &d, w, h)
                .save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
