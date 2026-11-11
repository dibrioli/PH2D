//! **Prancha B02.5**: uma aguada a correr na faixa de cima, dois traços em baixo, e o Ctrl+Z para lá do tecto do undo
//! vivo. «Antes» = o enterro de todo o píxel que difere ([`super::casco_tests::sem_casco`]), «depois» = só no casco do
//! traço desfeito, no mesmo processo. 3 fundos × 1080p/4K em `docs/Painter/missao_4k/pranchas/B02-cont/`.

use super::super::agua_sobre_oleo_tests::{ponteiro, preset};
use super::super::undo_tinta_morre_tests::{AGUADA, teto};
use super::casco_tests::sem_casco;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase};

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

fn traco(t: &mut PainterTool, x0: f32, x1: f32, y: f32) {
    t.on_canvas_pointer(ponteiro(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        let x = x0 + (x1 - x0) * k as f32 / 40.0;
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(x1, y, PointerPhase::Up));
    t.ticks_em_blocos(2);
}

/// A imagem achatada 60 tiques depois do Ctrl+Z para lá do tecto (`sem` = o enterro antigo).
fn cena(w: u32, h: u32, qual: u8, sem: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset(AGUADA)).expect("o meio");
    let s = h as f32 / 1080.0;
    t.set_brush_size_px(80.0 * s);
    t.set_brush_color_srgb8([40, 90, 220]);
    let (x0, x1) = (0.15 * w as f32, 0.85 * w as f32);
    teto::set(Some(1));
    traco(&mut t, x0, x1, 0.2 * h as f32);
    t.ticks_em_blocos(4);
    traco(&mut t, x0, x1, 0.6 * h as f32);
    traco(&mut t, x0, x1, 0.8 * h as f32);
    assert!(t.undo_last(), "o passo do topo, com fotografia");
    sem_casco::set(sem);
    assert!(t.undo_last(), "para lá do tecto");
    sem_casco::set(false);
    teto::set(None);
    t.ticks_em_blocos(60);
    let sess = t.paint.wetpaint.session.as_mut().expect("a sessão");
    sess.bring_home();
    sess.engine.mark_dirty_full();
    t.wetpaint_composite();
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

#[test]
#[ignore = "prancha B02.5: escreve PNGs em docs/Painter/missao_4k/pranchas/B02-cont/ (--profile smoke)"]
fn prancha_b02_o_undo_alem_do_tecto() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B02-cont"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let a = cena(w, h, qual, true);
            let d = cena(w, h, qual, false);
            // Reduz para a largura de 1080p: as três colunas cabem num ecrã.
            let passo = w / 1920;
            let (lw, lh) = (w / passo, h / passo);
            let gap = 8u32;
            let mut tri =
                image::RgbaImage::from_pixel(lw * 3 + gap * 2, lh, image::Rgba([40, 40, 40, 255]));
            let (mut difs, mut pior) = (0usize, 0u8);
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    let m = (0..3)
                        .map(|c| a[i + c].abs_diff(d[i + c]))
                        .max()
                        .unwrap_or(0);
                    difs += usize::from(m > 0);
                    pior = pior.max(m);
                }
            }
            for y in 0..lh {
                for x in 0..lw {
                    let i = (((y * passo) * w + x * passo) * 4) as usize;
                    let px = |v: &[u8]| image::Rgba([v[i], v[i + 1], v[i + 2], 255]);
                    tri.put_pixel(x, y, px(&a));
                    tri.put_pixel(x + lw + gap, y, px(&d));
                    let m = (0..3)
                        .map(|c| a[i + c].abs_diff(d[i + c]))
                        .max()
                        .unwrap_or(0);
                    let k = u8::try_from((u32::from(m) * 16).min(255)).unwrap_or(255);
                    let k = 255 - k;
                    tri.put_pixel(x + 2 * (lw + gap), y, image::Rgba([255, k, k, 255]));
                }
            }
            eprintln!("PRANCHA B02 {res} {nome}: {difs} píxeis diferem, pior {pior} níveis");
            tri.save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
