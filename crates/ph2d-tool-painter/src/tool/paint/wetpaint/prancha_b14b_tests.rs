//! **Prancha B14.2** (decisão 2 do dono): a imagem do GESTO com o Óleo Real e a luz rasante — antes = a luz `Cheia`
//! (a de sempre), depois = a `Simples` (a do pincel na tela). As duas compõem o MESMO motor no mesmo instante do
//! traço (o pincel ainda em baixo); o repouso continua a Cheia ao byte (`a_luz_simples_no_gesto_e_a_cheia_no_repouso`).
//! 3 fundos × 1080p/4K em `docs/Painter/missao_4k/pranchas/B14b/`.

use super::super::agua_sobre_oleo_tests::preset;
use super::super::offthread::otica::forca_no_teste;
use super::{ponto, tique};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase};
use std::time::Duration;

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.chunks_exact_mut(4).enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// Um traço de Dabber com Real Oil e a luz rasante (12°), parado A MEIO: o pincel continua na tela.
fn gesto(w: u32, h: u32, qual: u8) -> PainterTool {
    forca_no_teste(Some(false));
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_wet_oleo_real_ligado(true);
    t.paint.wetpaint.oleo_real_valores.altitude = 12f64.to_radians();
    let s = h as f32 / 1080.0;
    t.set_brush_size_px(56.0 * s);
    t.set_brush_color_srgb8([40, 90, 220]);
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 5.0);
    let passos = 24;
    let p = |k: usize| {
        let f = k as f32 / passos as f32;
        [
            cx - l + 2.0 * l * f,
            cy + 0.4 * l * (std::f32::consts::TAU * f).sin(),
        ]
    };
    let [x, y] = p(0);
    let _ = t.on_canvas_pointer(ponto(x, y, PointerPhase::Down));
    for k in 1..=passos {
        for sub in 0..3 {
            let f = (k - 1) as f32 + (sub + 1) as f32 / 3.0;
            let (a, b) = (p(k - 1), p(k));
            let g = f - (k - 1) as f32;
            let _ = t.on_canvas_pointer(ponto(
                a[0] + (b[0] - a[0]) * g,
                a[1] + (b[1] - a[1]) * g,
                PointerPhase::Move,
            ));
        }
        tique(&mut t);
        std::thread::sleep(Duration::from_millis(16));
    }
    assert!(t.wet_pincel_na_tela(), "premissa: o pincel na tela");
    t
}

/// A composição do motor INTEIRO deste instante, com a luz do gesto (`cheia = false`) ou a de repouso; devolve o
/// recorte central (lado `h/2`) da imagem final (papel e relevo incluídos).
fn imagem(t: &mut PainterTool, w: u32, h: u32, cheia: bool) -> Vec<u8> {
    let gesto = t.paint.wetpaint.live_gesture;
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    let aberto = s.stroke_open;
    s.bring_home();
    s.engine.mark_dirty_full();
    if cheia {
        s.stroke_open = false;
        t.paint.wetpaint.live_gesture = false;
    }
    t.wetpaint_composite();
    t.paint.wetpaint.live_gesture = gesto;
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("sessão")
        .stroke_open = aberto;
    t.invalidate_composite();
    let img = t.take_preview_arc().expect("o preview").0.to_vec();
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let mut out = Vec::with_capacity((lado * lado * 4) as usize);
    for y in y0..y0 + lado {
        let o = ((y * w + x0) * 4) as usize;
        out.extend_from_slice(&img[o..o + (lado * 4) as usize]);
    }
    out
}

#[test]
#[ignore = "prancha B14.2: escreve PNGs em docs/Painter/missao_4k/pranchas/B14b/ (--profile smoke)"]
fn prancha_b14b_a_luz_do_gesto() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B14b"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let mut t = gesto(w, h, qual);
            let d = imagem(&mut t, w, h, false);
            let a = imagem(&mut t, w, h, true);
            let lado = h / 2;
            let gap = 8u32;
            let mut tri = image::RgbaImage::from_pixel(
                lado * 3 + gap * 2,
                lado,
                image::Rgba([40, 40, 40, 255]),
            );
            let (mut difs, mut pior) = (0usize, 0u8);
            for y in 0..lado {
                for x in 0..lado {
                    let i = ((y * lado + x) * 4) as usize;
                    let px = |v: &[u8]| image::Rgba([v[i], v[i + 1], v[i + 2], 255]);
                    tri.put_pixel(x, y, px(&a));
                    tri.put_pixel(x + lado + gap, y, px(&d));
                    let m = (0..3)
                        .map(|c| a[i + c].abs_diff(d[i + c]))
                        .max()
                        .unwrap_or(0);
                    difs += usize::from(m > 0);
                    pior = pior.max(m);
                    let k = u8::try_from((u32::from(m) * 16).min(255)).unwrap_or(255);
                    tri.put_pixel(x + 2 * (lado + gap), y, image::Rgba([k, 0, 0, 255]));
                }
            }
            eprintln!("PRANCHA B14b {res} {nome}: {difs} píxeis diferem, pior {pior} níveis");
            tri.save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
    forca_no_teste(None);
}
