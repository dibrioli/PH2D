//! **Prancha B49.1**: o lápis de dente cheio (Height + `Tooth fill`) passa DUAS vezes numa camada transparente sobre 3
//! fundos × 1080p/4K; a 2.ª passada lê o pó. «Antes» = o pó da tela inteira pela soma corrida
//! ([`super::COM_A_SOMA_CORRIDA`]), «depois» = o preguiçoso por ladrilho, no mesmo processo. PNG em
//! `docs/Painter/missao_4k/pranchas/B49-cont/`.

use crate::tool::PainterTool;
use crate::{LinhaDaPonta, PaintMedia};
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};

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
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// A imagem achatada depois de duas passadas do lápis (raio 24·h/1080) numa camada nova transparente.
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
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(24.0 * s);
    t.set_brush_falloff(8);
    t.set_brush_color_srgb8([40, 30, 120]);
    t.set_brush_texture_kind(27); // PaperRough
    t.set_brush_texture_mapping(1); // Tiled
    t.set_brush_texture_size(0, 6.0);
    t.set_brush_texture_size(1, 6.0);
    t.set_ponta(LinhaDaPonta::GrainMode, 4.0); // Height
    t.set_ponta(LinhaDaPonta::Softness, 0.3);
    t.set_ponta(LinhaDaPonta::DepthJitter, 1.0);
    t.set_ponta(LinhaDaPonta::DepthPressure, 1.0);
    t.set_ponta(LinhaDaPonta::Fill, 0.6);
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 4.0);
    let passos = 120;
    for volta in 0..2 {
        let ponto = |k: usize| {
            let f = k as f32 / passos as f32;
            let x = cx - l + 2.0 * l * f;
            [
                x,
                cy + (0.35 - 0.1 * volta as f32) * l * (std::f32::consts::TAU * f).sin(),
            ]
        };
        let cp = |pos, phase| CanvasPointer {
            pos,
            pressure: 0.5,
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
    t.on_tick(1000.0 / 60.0);
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// O quadrado `lado²` a partir de `(x0, y0)`, ampliado `z×` pelo vizinho mais próximo.
fn recorte(img: &[u8], w: u32, x0: u32, y0: u32, lado: u32, z: u32) -> Vec<[u8; 4]> {
    let mut out = Vec::with_capacity((lado * lado * z * z) as usize);
    for y in 0..lado * z {
        for x in 0..lado * z {
            let i = (((y0 + y / z) * w + x0 + x / z) * 4) as usize;
            out.push([img[i], img[i + 1], img[i + 2], img[i + 3]]);
        }
    }
    out
}

/// Antes | depois | o que muda (vermelho ∝ a diferença): o recorte central `h/2` e, embaixo, a lupa 4×.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let lupa = lado / 4;
    let (lx, ly) = (w / 2 - h / 8 - lupa / 2, h / 2 + h * 35 / 400 - lupa / 2);
    let gap = 8u32;
    let mut par = image::RgbaImage::from_pixel(
        3 * lado + 2 * gap,
        2 * lado + gap,
        image::Rgba([40, 40, 40, 255]),
    );
    for (linha, (ox, oy, l, z)) in [(x0, y0, lado, 1u32), (lx, ly, lupa, 4)]
        .into_iter()
        .enumerate()
    {
        let (ra, rd) = (recorte(a, w, ox, oy, l, z), recorte(d, w, ox, oy, l, z));
        let top = linha as u32 * (lado + gap);
        for (i, (pa, pd)) in ra.iter().zip(&rd).enumerate() {
            let (x, y) = (i as u32 % lado, top + i as u32 / lado);
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
#[ignore = "prancha B49: escreve PNGs em docs/Painter/missao_4k/pranchas/B49-cont/"]
fn prancha_b49_o_po_por_ladrilho() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B49-cont"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            super::COM_A_SOMA_CORRIDA.set(true);
            let a = cena(w, h, qual);
            super::COM_A_SOMA_CORRIDA.set(false);
            let d = cena(w, h, qual);
            let difs = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(d.as_chunks::<4>().0)
                .filter(|(x, y)| x != y)
                .count();
            let pior = a.iter().zip(&d).map(|(x, y)| x.abs_diff(*y)).max();
            eprintln!("PRANCHA {res} {nome}: {difs} píxeis diferentes, pior {pior:?}");
            monta(&a, &d, w, h)
                .save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
