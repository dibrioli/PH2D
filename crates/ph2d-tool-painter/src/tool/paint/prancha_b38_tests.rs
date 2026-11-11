//! **Prancha B38**: Clone com Grain Tiled e Heal numa camada TRANSPARENTE com tinta de alfa parcial (amarelo
//! macio), sobre 3 fundos × 1080p/4K. «Antes» = a mistura recta do Clone e o Heal cego ao alfa, postos de
//! volta no MESMO processo (`prancha_b38_*`); «depois» = o pré-multiplicado que leva o alfa.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --lib prancha_b38 -- --ignored --nocapture
//! ```

use crate::tool::PainterTool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool as _, Tool as _,
};
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};

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
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]) {
    t.on_canvas_pointer(cp(de, PointerPhase::Down));
    for k in 1..=60 {
        let f = k as f32 / 60.0;
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
}

fn modo(t: &mut PainterTool, m: &str) {
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        m.to_string(),
    ));
}

/// A imagem achatada: faixa amarela macia na metade esquerda da camada nova; `heal` = o Heal atravessa
/// a ponta da faixa; senão o Clone (Grain Checker Tiled) leva a faixa ao vazio da direita.
fn cena(w: u32, h: u32, qual: u8, heal: bool, antes: bool) -> Vec<u8> {
    ph2d_painter_brush::prancha_b38_mistura_recta(antes);
    ph2d_inpaint::prancha_b38_alfa_de_antes(antes);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.add_raster_layer("tinta").expect("a camada transparente");
    let s = h as f32 / 1080.0;
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 4.0);
    t.set_brush_color_srgb8([250, 225, 20]);
    t.paint.brush.radius_px = 50.0 * s;
    t.paint.brush.hardness = 0.1;
    traco(&mut t, [cx - l, cy], [cx, cy]);
    t.paint.brush.radius_px = 30.0 * s;
    t.paint.brush.hardness = 0.4;
    if heal {
        modo(&mut t, "inpaint");
        traco(&mut t, [cx, cy - l / 2.0], [cx, cy + l / 2.0]);
    } else {
        modo(&mut t, "clone");
        t.paint.brush.texture.kind = TextureKind::Checker;
        t.paint.brush.texture.mapping = TextureMapping::Tiled;
        t.paint.brush.strength = 0.7;
        t.handle_panel_event(PanelEvent::Click(
            crate::ids::PAINTER_BRUSH_CLONE_SET_SOURCE,
        ));
        t.on_canvas_pointer(cp([cx - l / 2.0, cy - l / 2.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([cx - l / 2.0, cy - l / 2.0], PointerPhase::Up));
        traco(
            &mut t,
            [cx + l / 2.0, cy - l / 2.0],
            [cx + l / 2.0, cy + l / 2.0],
        );
    }
    t.on_tick(1000.0 / 60.0);
    t.invalidate_composite();
    ph2d_painter_brush::prancha_b38_mistura_recta(false);
    ph2d_inpaint::prancha_b38_alfa_de_antes(false);
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// Antes | depois | o que muda (vermelho ∝ a diferença ×32), o recorte central `h/2`.
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
            let m = (0..3).map(|c| pa[c].abs_diff(pd[c])).max().unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([pa[0], pa[1], pa[2], 255]));
            par.put_pixel(x + lado + gap, y, image::Rgba([pd[0], pd[1], pd[2], 255]));
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B38: escreve PNGs em docs/Painter/missao_4k/pranchas/B38/"]
fn prancha_b38_clone_e_heal_na_camada_transparente() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B38"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            for (heal, oque) in [(false, "clone"), (true, "heal")] {
                let a = cena(w, h, qual, heal, true);
                let d = cena(w, h, qual, heal, false);
                let (mut difs, mut mais_claros, mut pior) = (0usize, 0usize, 0u8);
                for (x, y) in a.as_chunks::<4>().0.iter().zip(d.as_chunks::<4>().0) {
                    if x[..3] != y[..3] {
                        difs += 1;
                        let (sa, sd): (u32, u32) = (
                            x[..3].iter().map(|&v| u32::from(v)).sum(),
                            y[..3].iter().map(|&v| u32::from(v)).sum(),
                        );
                        mais_claros += usize::from(sd > sa);
                        pior = pior.max((0..3).map(|c| x[c].abs_diff(y[c])).max().unwrap_or(0));
                    }
                }
                eprintln!(
                    "PRANCHA B38 {oque} {res} {nome}: {difs} píxeis diferentes ({mais_claros} mais claros), pior {pior}"
                );
                monta(&a, &d, w, h)
                    .save(format!("{dir}/{res}_{nome}_{oque}_antes_depois.png"))
                    .expect("png");
            }
        }
    }
}
