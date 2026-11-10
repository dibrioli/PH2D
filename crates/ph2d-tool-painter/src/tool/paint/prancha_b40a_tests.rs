//! **Prancha B40.A1**: numa camada TRANSPARENTE com um disco pintado, (a) Color Fill com o alfa
//! travado e (b) Cut com a metade esquerda protegida pela máscara, sobre 3 fundos × 1080p/4K. O
//! «antes» é o caminho antigo (`CAMINHO_ANTIGO`, a guarda neutra), no mesmo processo.

use super::escrita_direta::CAMINHO_ANTIGO;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{RasterEditTool as _, Tool as _};
use std::sync::Arc;

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [[200u8, 60, 50], [60, 150, 210], [240, 200, 60], [90, 170, 90]];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.chunks_exact_mut(4).enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

/// O disco laranja de raio `h/6` no centro, numa camada transparente.
fn disco(w: u32, h: u32) -> Vec<u8> {
    let (cx, cy, r) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 6.0);
    let mut v = vec![0u8; (w * h * 4) as usize];
    for (i, p) in v.chunks_exact_mut(4).enumerate() {
        let (x, y) = ((i as u32 % w) as f32 + 0.5, (i as u32 / w) as f32 + 0.5);
        let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        let a = (r - d + 0.5).clamp(0.0, 1.0);
        if a > 0.0 {
            p.copy_from_slice(&[240, 140, 30, (a * 255.0).round() as u8]);
        }
    }
    v
}

/// A imagem achatada depois do caso (`0` Color Fill travado, `1` Cut protegido), pelo caminho `antigo`.
fn cena(w: u32, h: u32, qual: u8, caso: u8, antigo: bool) -> Vec<u8> {
    CAMINHO_ANTIGO.set(antigo);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    let id = t.add_raster_layer_with_pixels("tinta", disco(w, h)).expect("a camada");
    // Uma faixa central de meia altura: cobre o disco e o vazio à volta dele.
    t.set_rect_selection(w / 4, h / 4, w / 2, h / 2);
    if caso == 0 {
        t.set_layer_alpha_locked(id, true);
        t.set_brush_color_srgb8([40, 90, 230]);
        t.selection_color_fill();
    } else {
        t.ensure_mask_scratch();
        let mut m = vec![255u8; (w * h * 4) as usize];
        for (i, p) in m.chunks_exact_mut(4).enumerate() {
            if (i as u32 % w) < w / 2 {
                p[..3].copy_from_slice(&[0, 0, 0]);
            }
        }
        t.paint.mask_scratch_rgba = Arc::new(m);
        t.bump_mask_scratch_gen();
        t.selection_cut();
        // A película da máscara não entra na imagem: só a tinta.
        t.paint.mask_scratch_rgba = Arc::new(Vec::new());
    }
    CAMINHO_ANTIGO.set(false);
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
            let m = (0..4).map(|c| a[i + c].abs_diff(d[i + c])).max().unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([a[i], a[i + 1], a[i + 2], 255]));
            par.put_pixel(x + lado + gap, y, image::Rgba([d[i], d[i + 1], d[i + 2], 255]));
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B40a: grava as PNG em docs/Painter/missao_4k/pranchas/B40a/"]
fn prancha_b40a_fill_e_cut_respeitam_trinco_e_protecao() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/Painter/missao_4k/pranchas/B40a");
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            for (caso, caso_nome) in [(0u8, "colorfill_trinco"), (1, "cut_protegido")] {
                let a = cena(w, h, qual, caso, true);
                let d = cena(w, h, qual, caso, false);
                assert_ne!(a, d, "{res} {fundo_nome} {caso_nome}: o caso tem de estar hoje errado");
                monta(&a, &d, w, h)
                    .save(format!("{dir}/{res}_{fundo_nome}_{caso_nome}_antes_depois.png"))
                    .expect("png");
            }
        }
    }
}
