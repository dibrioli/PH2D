//! **Prancha B35.9**: um traço Wet ainda molhado e a Borracha do Wet no máximo, antes (o máximo era a força 0,4:
//! `Eraser 0,4 × Erase Strength 1`) e depois (o Erase Strength é a força: 1), em 3 fundos × 1080p/4K. Escreve
//! `docs/Painter/missao_4k/pranchas/B35.9/`.

use crate::tool::{PaintMedia, PainterTool};
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool as _, Tool as _,
};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn risca(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2], passos: usize) {
    t.on_canvas_pointer(cp(de, PointerPhase::Down));
    for k in 1..=passos {
        let f = k as f32 / passos as f32;
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
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

/// O recorte quadrado do centro (lado `h/2`) depois da Borracha do Wet com a força `erase` (escala nova).
fn cena(w: u32, h: u32, qual: u8, erase: f64) -> (Vec<u8>, u32) {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_MEDIA,
        PaintMedia::WetPaint.to_u8().to_string(),
    ));
    let s = h as f32 / 1080.0;
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 4.0);
    t.paint.brush.radius_px = 28.0 * s;
    t.paint.brush.color = [0.2, 0.3, 0.75];
    risca(&mut t, [cx - l, cy], [cx + l, cy], 40);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "eraser".to_string(),
    ));
    t.paint.brush.radius_px = 34.0 * s;
    t.paint.wetpaint.knobs.erase = erase;
    risca(&mut t, [cx, cy - l], [cx, cy + l], 40);
    t.ticks_em_blocos(30);
    t.invalidate_composite();
    let img = t.take_preview_arc().expect("o preview").0.to_vec();
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let mut out = Vec::with_capacity((lado * lado * 4) as usize);
    for y in y0..y0 + lado {
        let o = ((y * w + x0) * 4) as usize;
        out.extend_from_slice(&img[o..o + (lado * 4) as usize]);
    }
    (out, lado)
}

#[test]
#[ignore = "prancha B35.9: escreve PNGs em docs/Painter/missao_4k/pranchas/B35.9/"]
fn prancha_b35_9_a_borracha_no_maximo_limpa_o_miolo() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B35.9"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    let antes = ph2d_wet_paint::tools::ERASER_DA_FABRICA;
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let (a, lado) = cena(w, h, qual, antes);
            let (d, _) = cena(w, h, qual, 1.0);
            let gap = 8u32;
            let mut par =
                image::RgbaImage::from_pixel(lado * 2 + gap, lado, image::Rgba([40, 40, 40, 255]));
            let sobre_branco = |p: &[u8], i: usize| {
                let k = u16::from(p[i + 3]);
                let c = |v: u8| ((u16::from(v) * k + 255 * (255 - k)) / 255) as u8;
                image::Rgba([c(p[i]), c(p[i + 1]), c(p[i + 2]), 255])
            };
            for y in 0..lado {
                for x in 0..lado {
                    let i = ((y * lado + x) * 4) as usize;
                    par.put_pixel(x, y, sobre_branco(&a, i));
                    par.put_pixel(x + lado + gap, y, sobre_branco(&d, i));
                }
            }
            let dif = a.iter().zip(&d).filter(|(x, y)| x != y).count();
            eprintln!("PRANCHA {res} {nome}: bytes diferentes antes×depois {dif}");
            par.save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
