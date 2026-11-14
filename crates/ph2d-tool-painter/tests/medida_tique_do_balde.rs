//! **O que um tique do slider do balde ALOCA** (B48.2, `docs/Painter/missao_4k/BACKLOG_AUDITORIA.md`):
//! Impasto, disco de 300 px, telas 2048² e 4096². Gate: os bytes por tique são função da REGIÃO
//! (`< 8 · região`), não da tela. Antes do B48.2 cada tique pedia ~0,3 GB a 4K.
//!
//! Um `#[test]` por binário: os contadores do dhat são do processo inteiro.
//!
//! ```text
//! bash scripts/cargo-test-narrow.sh ph2d-tool-painter o_tique_do_balde_aloca_pela_regiao --test medida_tique_do_balde
//! ```

use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::PainterTool;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const RAIO: f32 = 150.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Os bytes do tique mais barato de 5 (depois de 2 de aquecimento: o 1.º paga o journal) e a região.
fn mede(s: u32) -> (u64, u64) {
    let (su, c) = (s as usize, s as f32 * 0.5);
    let mut src = vec![255u8; su * su * 4];
    let mut regiao = 0u64;
    for y in 0..su {
        for x in 0..su {
            let d = ((x as f32 + 0.5 - c).powi(2) + (y as f32 + 0.5 - c).powi(2)).sqrt();
            if d <= RAIO {
                src[(y * su + x) * 4..(y * su + x) * 4 + 4].copy_from_slice(&[220, 20, 20, 255]);
                regiao += 1;
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, s, s);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "fill".to_string(),
    ));
    t.set_brush_color_srgb8([25, 180, 50]);
    t.sonda_raio(24.0);
    t.set_brush_impasto(true);
    t.set_brush_impasto_depth(1.0);
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c, c], PointerPhase::Up));
    assert!(t.has_active_fill(), "o balde não ficou pendente");
    let mut menor = u64::MAX;
    for k in 0..7 {
        let antes = dhat::HeapStats::get().total_bytes;
        t.set_fill_threshold(if k % 2 == 0 { 0.1 } else { 0.2 });
        let b = dhat::HeapStats::get().total_bytes - antes;
        if k >= 2 {
            menor = menor.min(b);
        }
    }
    t.fill_commit();
    (menor, regiao)
}

#[test]
fn o_tique_do_balde_aloca_pela_regiao() {
    let _p = dhat::Profiler::builder().testing().build();
    let (b2, r2) = mede(2048);
    let (b4, r4) = mede(4096);
    eprintln!("B48.2 tique: 2048² {b2} B (região {r2}) · 4096² {b4} B (região {r4})");
    assert_eq!(r2, r4, "a região tem de ser a mesma nas duas telas");
    assert!(b2 < 8 * r2, "2048²: {b2} B ≥ 8 · região ({r2})");
    assert!(b4 < 8 * r4, "4096²: {b4} B ≥ 8 · região ({r4})");
}
