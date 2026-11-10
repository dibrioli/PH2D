//! **Prancha B06b**: Impasto movido no Transform (Uniform) e dobrado no Warp em malha, antes (a malha semeada no
//! box movido, sem corpo, posse no centro do texel sem aro) e depois, em 3 fundos × 1080p/4K.
//! Escreve `docs/Painter/missao_4k/pranchas/B06b/`.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool as _, Tool as _,
};
use std::cell::Cell;

thread_local! {
    static ANTES: Cell<bool> = const { Cell::new(false) };
}

/// O caminho antigo da malha (B06.2 · B06.3 · B06.A5), só para a prancha «antes».
pub(super) fn antes() -> bool {
    ANTES.get()
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
        for (i, p) in v.chunks_exact_mut(4).enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// O recorte quadrado do centro (lado `h/2`) da imagem iluminada depois de mover e dobrar.
fn cena(w: u32, h: u32, qual: u8, antigo: bool) -> (Vec<u8>, u32) {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_MEDIA,
        crate::tool::PaintMedia::Impasto.to_u8().to_string(),
    ));
    let s = h as f32 / 1080.0;
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 6.0);
    t.paint.brush.radius_px = 26.0 * s;
    t.paint.brush.color = [0.75, 0.2, 0.15];
    risca(&mut t, [cx - l, cy], [cx + l, cy], 40);
    ANTES.set(antigo);
    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    t.set_deform_transform_mode(0);
    let mv = [40.0 * s, 24.0 * s];
    risca(&mut t, [cx, cy], [cx + mv[0], cy + mv[1]], 8);
    t.set_deform_transform_mode(3);
    let m = t.paint.deform.xform_mesh.clone().expect("a malha");
    // O ponto do meio da linha de cima, puxado para cima.
    let k = (m.cols / 2) as usize;
    let p = m.current[k];
    risca(&mut t, p, [p[0], p[1] - 70.0 * s], 12);
    ANTES.set(false);
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
#[ignore = "prancha B06b: escreve PNGs em docs/Painter/missao_4k/pranchas/B06b/"]
fn prancha_b06b_o_warp_em_malha_leva_o_corpo() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B06b"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let (a, lado) = cena(w, h, qual, true);
            let (d, _) = cena(w, h, qual, false);
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
