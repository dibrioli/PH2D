//! **Prancha B22.2**: um Spray Can (`parado_s = 0,0035`) parado 1 s no centro, sobre 3 fundos × 1080p/4K, a 30, 60 e
//! 144 q/s. O «antes» a 30 q/s é o motor novo a `dt = 8 × 0,0035 + 1e-5` por tique: o relógio velho pousava 8 por
//! tique, e `ph2d-painter-brush` `o_antes_a_30_qs_e_o_motor_novo_a_8_periodos_por_tique` prova os mesmos dabs ao bit.

use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};
use ph2d_painter_brush::StrokeMethod;

const DT_ANTES_30_QS: f32 = 8.0 * 0.0035 + 1e-5;

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

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A imagem achatada depois de `ticks` quadros de `dt_s` com a lata parada no centro.
fn cena(w: u32, h: u32, qual: u8, dt_s: f32, ticks: usize) -> Vec<u8> {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_media(PaintMedia::Digital);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    t.set_brush_size_px(h as f32 / 40.0);
    t.set_brush_color_srgb8([30, 50, 140]);
    let b = &mut t.paint.brush;
    b.stroke_method = StrokeMethod::Space;
    b.hardness = 0.85;
    b.spray_count = 16;
    b.spacing = 0.015;
    b.jitter = 0.7;
    b.jitter_scale = 0.95;
    b.continuous_spacing = false;
    b.flow = 0.03;
    b.parado_s = 0.0035;
    let c = [w as f32 / 2.0, h as f32 / 2.0];
    t.on_canvas_pointer(ponto(c, PointerPhase::Down));
    for _ in 0..ticks {
        t.on_tick(dt_s * 1e3);
    }
    t.on_canvas_pointer(ponto(c, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// Antes 30 | depois 30 | depois 60 | depois 144 | o que muda a 30 (vermelho ∝ a diferença), recorte central `h/3`.
fn monta(v: [&[u8]; 4], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 3;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(5 * lado + 4 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = (((y0 + y) * w + x0 + x) * 4) as usize;
            for (k, img) in v.iter().enumerate() {
                let px = image::Rgba([img[i], img[i + 1], img[i + 2], 255]);
                par.put_pixel(x + k as u32 * (lado + gap), y, px);
            }
            let m = (0..4)
                .map(|c| v[0][i + c].abs_diff(v[1][i + c]))
                .max()
                .unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x + 4 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B22.2: grava as PNG em docs/Painter/missao_4k/pranchas/B22.2/"]
fn prancha_b22_2_spray_parado_a_30_60_e_144_qs() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B22.2"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let antes = cena(w, h, qual, DT_ANTES_30_QS, 30);
            let [d30, d60, d144] = [30usize, 60, 144].map(|f| cena(w, h, qual, 1.0 / f as f32, f));
            assert_ne!(
                antes, d30,
                "{res} {fundo_nome}: a 30 q/s a lata tem de pousar mais"
            );
            monta([&antes, &d30, &d60, &d144], w, h)
                .save(format!("{dir}/{res}_{fundo_nome}_30_60_144.png"))
                .expect("png");
        }
    }
}
