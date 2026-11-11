//! **Prancha B06.1**: Impasto e a Borracha do trilho, antes (apagar seguia o pincel) e depois (segue a
//! camada), em 3 fundos × 1080p/4K. Escreve `docs/Painter/missao_4k/pranchas/B06a/`.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool as _, Tool as _,
};
use std::cell::Cell;

thread_local! {
    static PELO_PINCEL: Cell<bool> = const { Cell::new(false) };
}

/// O predicado antigo de B06.1, só para a prancha «antes».
pub(super) fn pelo_pincel() -> bool {
    PELO_PINCEL.get()
}

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

/// O recorte quadrado do centro (lado `h/2`) da imagem iluminada, depois da Borracha (`wet`: a do Wet Paint, sobre a
/// tinta morta).
fn cena(w: u32, h: u32, qual: u8, antigo: bool, wet: bool) -> (Vec<u8>, u32) {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    let media = |t: &mut PainterTool, m: crate::tool::PaintMedia| {
        t.handle_panel_event(PanelEvent::SelectOption(
            crate::ids::PAINTER_BRUSH_MEDIA,
            m.to_u8().to_string(),
        ));
    };
    media(&mut t, crate::tool::PaintMedia::Impasto);
    let s = h as f32 / 1080.0;
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 4.0);
    t.paint.brush.radius_px = 28.0 * s;
    t.paint.brush.color = [0.75, 0.2, 0.15];
    risca(&mut t, [cx - l, cy], [cx + l, cy], 40);
    if wet {
        media(&mut t, crate::tool::PaintMedia::WetPaint);
    }
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "eraser".to_string(),
    ));
    t.paint.brush.radius_px = 34.0 * s;
    PELO_PINCEL.set(antigo);
    for _ in 0..3 {
        risca(&mut t, [cx, cy - l], [cx, cy + l], 40);
    }
    PELO_PINCEL.set(false);
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
#[ignore = "prancha B06.1: escreve PNGs em docs/Painter/missao_4k/pranchas/B06a/"]
fn prancha_b06a_a_borracha_leva_o_corpo() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B06a"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    let so_wet = std::env::var_os("PH2D_PRANCHA_SO_WET").is_some();
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome, wet) in [
            (0u8, "branco", false),
            (1, "blocos", false),
            (2, "papel", false),
            (0, "wet_branco", true),
            (1, "wet_blocos", true),
            (2, "wet_papel", true),
        ] {
            if so_wet && !wet {
                continue;
            }
            let (a, lado) = cena(w, h, qual, true, wet);
            let (d, _) = cena(w, h, qual, false, wet);
            let gap = 8u32;
            let larg = lado * 2 + gap;
            let mut par = image::RgbaImage::from_pixel(larg, lado, image::Rgba([40, 40, 40, 255]));
            // Sobre branco: onde a Borracha leva a camada a alfa 0, o que fica à vista é o corpo.
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
