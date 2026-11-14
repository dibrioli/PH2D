//! n.º 10 F5b, aberto 2b — o Ctrl+Z do traço do Wet na camada NOVA (com o papel): a janela que a placa redobra é
//! a do traço, nunca a tela inteira.

use super::super::*;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase, RasterEditTool};

const LADO: u32 = 128;

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(20.0, y, PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp(20.0 + 4.0 * k as f32, y, PointerPhase::Move));
        ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(60.0, y, PointerPhase::Up));
    ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
}

fn area(r: Option<(u32, u32, u32, u32)>) -> u64 {
    r.map_or(u64::from(LADO) * u64::from(LADO), |r| {
        u64::from(r.2) * u64::from(r.3)
    })
}

fn contem(fora: (u32, u32, u32, u32), dentro: (u32, u32, u32, u32)) -> bool {
    dentro.0 >= fora.0
        && dentro.1 >= fora.1
        && dentro.0 + dentro.2 <= fora.0 + fora.2
        && dentro.1 + dentro.3 <= fora.1 + fora.3
}

/// **O Ctrl+Z do traço na camada nova confina a janela (e a parte do relevo) à do traço.** Wet com papel: um traço
/// na base, «+», um traço na nova; a janela drenada depois do Ctrl+Z (e do Ctrl+Shift+Z) cabe na união das duas
/// janelas do traço, e é menor que a tela. A régua: a área de cada uma (`--nocapture`).
#[test]
fn o_desfazer_do_traco_na_camada_nova_redobra_so_a_janela_do_traco() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_wet_relogio_fixo(true);
    t.set_substrate_depth(1.0);
    t.set_paint_tool_mode("wetpaint");
    traco(&mut t, 30.0);
    let _ = t.take_preview_dirty();
    let nova = t.add_raster_layer("Nova").expect("a nova");
    t.select_layer(nova);
    let _ = t.take_preview_dirty();
    traco(&mut t, 80.0);
    assert!(t.take_preview_dirty(), "o traço marca");
    let do_traco = t.preview_gpu_region().expect("o traço confina");
    let relevo_do_traco = t.preview_gpu_relevo();
    eprintln!(
        "traço: janela {do_traco:?} ({} px) · relevo {relevo_do_traco:?}",
        area(Some(do_traco))
    );
    for (gesto, faz) in [
        (
            "undo",
            PainterTool::undo_last as fn(&mut PainterTool) -> bool,
        ),
        ("redo", PainterTool::redo_last),
    ] {
        assert!(faz(&mut t), "{gesto}");
        assert!(t.take_preview_dirty(), "{gesto}: marca");
        let janela = t.preview_gpu_region();
        let relevo = t.preview_gpu_relevo();
        eprintln!(
            "{gesto}: janela {janela:?} ({} px de {}) · relevo {relevo:?}",
            area(janela),
            LADO * LADO
        );
        let janela =
            janela.unwrap_or_else(|| panic!("{gesto}: a janela não confina (tela inteira)"));
        assert!(
            contem(do_traco, janela),
            "{gesto}: a janela {janela:?} é maior que a do traço {do_traco:?}"
        );
        if let Some(Some(r)) = relevo {
            assert!(contem(janela, r), "{gesto}: o relevo {r:?} sai da janela");
        }
    }
}
