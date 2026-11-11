//! **Prancha B16.2**: uma sessão molhada de Watercolor Basic com o RAIO a mudar a cada traço, sobre 3
//! fundos × 1080p/4K. «Antes» = o halo pelo pincel vivo e o pen-up pela união (o caminho antigo, os
//! oráculos [`HALO_PELO_PINCEL_VIVO`] e [`COMMIT_PELA_UNIAO`]); «depois» = o dono do halo e o pen-up
//! pelo traço. Grava em `docs/Painter/missao_4k/pranchas/B16-cont3/`.

use super::super::dono::HALO_PELO_PINCEL_VIVO;
use super::COMMIT_PELA_UNIAO;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
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
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

/// Seis traços verticais lado a lado, raio 10/16/24 × h/1080 a alternar, na mesma sessão molhada.
fn cena(w: u32, h: u32, qual: u8, antes: bool) -> Vec<u8> {
    HALO_PELO_PINCEL_VIVO.set(antes);
    COMMIT_PELA_UNIAO.set(antes);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.set_dry_time_s(60.0);
    let s = h as f32 / 1080.0;
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    for k in 0..6usize {
        t.set_brush_size_px([10.0, 16.0, 24.0][k % 3] * s);
        let x = cx + (k as f32 - 2.5) * 40.0 * s;
        t.on_canvas_pointer(cp([x, cy - 120.0 * s], PointerPhase::Down));
        for j in 1..=20 {
            t.on_canvas_pointer(cp(
                [x, cy - 120.0 * s + 12.0 * s * j as f32],
                PointerPhase::Move,
            ));
            t.paint_tick(1.0 / 60.0);
        }
        t.on_canvas_pointer(cp([x, cy + 120.0 * s], PointerPhase::Up));
        t.paint_tick(0.25);
    }
    HALO_PELO_PINCEL_VIVO.set(false);
    COMMIT_PELA_UNIAO.set(false);
    t.canvas_rgba.to_vec()
}

/// Antes | depois | o que muda (vermelho ∝ a diferença), o recorte central `h/2`.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * lado + 2 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = (((y0 + y) * w + x0 + x) * 4) as usize;
            let (pa, pd) = (&a[i..i + 4], &d[i..i + 4]);
            let m = (0..4).map(|c| pa[c].abs_diff(pd[c])).max().unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([pa[0], pa[1], pa[2], 255]));
            par.put_pixel(x + lado + gap, y, image::Rgba([pd[0], pd[1], pd[2], 255]));
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B16.2: escreve PNGs em docs/Painter/missao_4k/pranchas/B16-cont3/"]
fn prancha_b16_2_o_dono_do_halo() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B16-cont3"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let a = cena(w, h, qual, true);
            let d = cena(w, h, qual, false);
            let difs = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(d.as_chunks::<4>().0)
                .filter(|(x, y)| x != y)
                .count();
            let pior = a
                .iter()
                .zip(&d)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            eprintln!("PRANCHA {res} {nome}: {difs} píxeis diferentes, pior {pior}");
            monta(&a, &d, w, h)
                .save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
