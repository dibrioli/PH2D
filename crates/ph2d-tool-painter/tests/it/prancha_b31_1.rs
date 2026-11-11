//! **Prancha B31.1** (missão 4K): o aro branco à volta do Impasto no papel. Uma crista de Impasto sobre papel com
//! relevo, 3 fundos (branco, blocos, papel Textured) × 1080p/4K; `lei antiga | lei nova | Δ×32`. Imprime, fora da
//! tinta, quantos píxeis mudaram e o pior clarão de antes.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --test it prancha_b31_1 -- --ignored --nocapture
//! ```

use crate::prancha_b34::{fundo, monta, ponto};
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase, RasterEditTool as _};
use ph2d_tool_painter::PainterTool;

fn cena(w: u32, h: u32, qual: u8, antiga: bool) -> Vec<u8> {
    PainterTool::prancha_b31_1_lei_antiga(antiga);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.set_substrate_depth(0.5);
    t.toggle_brush_impasto();
    t.set_brush_size_px(80.0 * h as f32 / 1080.0);
    t.set_brush_color_srgb8([30, 50, 140]);
    let (wf, hf) = (w as f32, h as f32);
    t.on_canvas_pointer(ponto([0.35 * wf, 0.5 * hf], PointerPhase::Down));
    for k in 1..=40 {
        let f = k as f32 / 40.0;
        t.on_canvas_pointer(ponto([(0.35 + 0.3 * f) * wf, 0.5 * hf], PointerPhase::Move));
    }
    t.on_canvas_pointer(ponto([0.65 * wf, 0.5 * hf], PointerPhase::Up));
    let img = t.take_preview_arc().expect("o preview").0.to_vec();
    PainterTool::prancha_b31_1_lei_antiga(false);
    img
}

#[test]
#[ignore = "prancha B31.1: grava as PNG em docs/Painter/missao_4k/pranchas/B31.1/"]
fn prancha_b31_1_o_aro_branco_do_impasto() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B31.1"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let antes = cena(w, h, qual, true);
            let depois = cena(w, h, qual, false);
            let (mut mudados, mut brancos_antes, mut brancos_depois) = (0usize, 0usize, 0usize);
            for (a, d) in antes
                .as_chunks::<4>()
                .0
                .iter()
                .zip(depois.as_chunks::<4>().0.iter())
            {
                mudados += usize::from(a[..3] != d[..3]);
                brancos_antes += usize::from(a[..3] == [255, 255, 255]);
                brancos_depois += usize::from(d[..3] == [255, 255, 255]);
            }
            println!(
                "[b31.1] {res:<5} {nome:<6} píxeis mudados {mudados} · brancos puros {brancos_antes} → {brancos_depois}"
            );
            monta(&antes, &depois, w, h)
                .save(format!("{dir}/{res}_{nome}_impasto_antes_depois.png"))
                .expect("png");
        }
    }
}
