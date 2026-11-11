//! **Prancha B40b**: (a) uma Line com Symmetry (Mirror X) e dabs espaçados — o 1.º dab ganha o seu
//! espelho (B40.1); (b) uma Ellipse no modo Mask e depois o pincel por cima — a área da figura fica
//! protegida (B40.3). 3 fundos × 1080p/4K. O «antes» é o caminho antigo pelos oráculos
//! `PRIMEIRO_DAB_ANTIGO` / `FORMA_NO_MASK_ANTIGA`, no mesmo processo.

use super::mask_probe::{cp, vstroke};
use super::mask_shape::FORMA_NO_MASK_ANTIGA;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::stroke::PRIMEIRO_DAB_ANTIGO;
use ph2d_painter_brush::symmetry::MirrorAxis;

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

fn click(t: &mut PainterTool, p: [f32; 2]) {
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    t.on_canvas_pointer(cp(p, PointerPhase::Up));
}

fn modo(t: &mut PainterTool, m: &str) {
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        m.to_string(),
    ));
}

/// A imagem depois do caso (`0` Line com Symmetry, `1` Ellipse no Mask + pincel), pelo caminho `antigo`.
fn cena(w: u32, h: u32, qual: u8, caso: u8, antigo: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    let (cx, cy, u) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 100.0);
    t.paint.brush.radius_px = 2.0 * u;
    if caso == 0 {
        PRIMEIRO_DAB_ANTIGO.set(antigo);
        t.paint.brush.spacing = 1.6;
        t.paint.brush.color = [0.15, 0.25, 0.85];
        t.paint.brush.symmetry.enabled = true;
        t.paint.brush.symmetry.axis = MirrorAxis::X;
        t.paint.brush.symmetry.center = [cx, cy];
        t.paint.brush.stroke_method = StrokeMethod::Line;
        click(&mut t, [cx - 20.0 * u, cy - 15.0 * u]);
        click(&mut t, [cx - 5.0 * u, cy + 15.0 * u]);
        assert!(t.line_commit(), "a linha confirma");
        PRIMEIRO_DAB_ANTIGO.set(false);
    } else {
        FORMA_NO_MASK_ANTIGA.set(antigo);
        modo(&mut t, "mask");
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([cx, cy], PointerPhase::Down));
        for i in 1..=8 {
            let k = i as f32 / 8.0;
            t.on_canvas_pointer(cp([cx + 12.0 * u * k, cy], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([cx + 12.0 * u, cy], PointerPhase::Up));
        assert!(t.ellipse_commit(), "a elipse confirma");
        FORMA_NO_MASK_ANTIGA.set(false);
        let _ = t.take_preview_arc();
        modo(&mut t, "paint");
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.radius_px = 5.0 * u;
        t.paint.brush.color = [0.1, 0.7, 0.2];
        vstroke(&mut t, cx - 12.0 * u, cy - 22.0 * u, cy + 22.0 * u, 24);
        vstroke(&mut t, cx + 4.0 * u, cy - 22.0 * u, cy + 22.0 * u, 24);
        // A película da máscara não entra na imagem: só a tinta (como na prancha B40a).
        t.paint.mask_scratch_rgba = std::sync::Arc::new(Vec::new());
    }
    t.on_tick(1000.0 / 60.0);
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// Antes | depois | o que muda (vermelho ∝ a diferença), no recorte central de lado `h/2`.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * lado + 2 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = (((y0 + y) * w + x0 + x) * 4) as usize;
            let m = (0..4)
                .map(|c| a[i + c].abs_diff(d[i + c]))
                .max()
                .unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([a[i], a[i + 1], a[i + 2], 255]));
            par.put_pixel(
                x + lado + gap,
                y,
                image::Rgba([d[i], d[i + 1], d[i + 2], 255]),
            );
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B40b: grava as PNG em docs/Painter/missao_4k/pranchas/B40b/"]
fn prancha_b40b_formas_espelham_e_protegem() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B40b"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            for (caso, caso_nome) in [(0u8, "linha_simetria"), (1, "forma_no_mask")] {
                let a = cena(w, h, qual, caso, true);
                let d = cena(w, h, qual, caso, false);
                assert_ne!(
                    a, d,
                    "{res} {fundo_nome} {caso_nome}: o caso tem de estar hoje errado"
                );
                monta(&a, &d, w, h)
                    .save(format!(
                        "{dir}/{res}_{fundo_nome}_{caso_nome}_antes_depois.png"
                    ))
                    .expect("png");
            }
        }
    }
}
