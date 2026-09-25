//! **O NASCIMENTO da sessão não pinta a tela inteira** — filho de [`super`].
//!
//! O motor nasce com `Dirty::Full` (o `rebake_paper` do construtor marca a folha inteira: um papel novo
//! muda a granulação de tinta que JÁ exista). Numa sessão que acabou de nascer não existe tinta nenhuma,
//! e o composite da folha inteira escreve em todo pixel exactamente o que ele já tinha. Medido a 4096²
//! (2026-09-24): **`~20 ms`** no 1.º traço da sessão — e, desde que o composite DECLARA onde escreve
//! (`crate::undo::window`), a janela declarada do 1.º traço passava a ser a tela inteira e o commit dele
//! guardava o plano `Whole` (**dois** planos de 67 MB na história) onde antes guardava o traço.

use super::*;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;

const LADO: u32 = 256;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_media(PaintMedia::WetPaint);
    t
}

/// **O 1.º traço de uma sessão guarda o TRAÇO na história, não a tela inteira** — com o papel de
/// fábrica (o `Dirty::Full` do construtor) e com um papel AUTORADO (o re-cozido do `reconcile_facts`
/// no 1.º lote, que suja a folha outra vez).
#[test]
fn the_first_wet_stroke_of_a_session_keeps_the_stroke_not_the_whole_canvas() {
    for papel_autorado in [false, true] {
        primeiro_traco(papel_autorado);
    }
}

fn primeiro_traco(papel_autorado: bool) {
    let mut t = molhado();
    if papel_autorado {
        use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
        let d = &KNOB_DEFS[Knob::PaperContrast as usize];
        t.paint.wetpaint.knobs.set(Knob::PaperContrast, d.max);
    }
    let antes = t.canvas_rgba.as_ref().clone();
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    for k in 1..=4u8 {
        t.on_canvas_pointer(cp([30.0 + f32::from(k) * 6.0, 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, 30.0], PointerPhase::Up));
    assert_ne!(
        t.canvas_rgba.as_ref(),
        &antes,
        "a fixtura tem de pintar alguma coisa"
    );
    let plano = (LADO * LADO * 4) as usize;
    let guardado = t.undo.retained_bytes();
    assert!(
        guardado < plano / 4,
        "papel autorado {papel_autorado}: o 1.º traço guardou {guardado} bytes — um plano são \
         {plano}: o nascimento da sessão declarou a tela inteira e o commit guardou-a `Whole`"
    );
}

/// **A premissa: o composite da folha inteira no nascimento não muda um byte** — com e sem o véu do
/// Show Wet. É ela que torna honesto descartar o `Dirty::Full` do nascimento.
#[test]
fn the_birth_composite_would_not_change_a_pixel() {
    for veu in [false, true] {
        let mut t = molhado();
        assert!(t.ensure_wet_session(), "a sessão tem de nascer");
        let antes = t.canvas_rgba.as_ref().clone();
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("acabou de nascer")
            .engine
            .mark_dirty_full();
        t.wetpaint_composite_veiled(veu);
        assert_eq!(
            t.canvas_rgba.as_ref(),
            &antes,
            "véu {veu}: o composite da folha inteira no nascimento mudou pixels"
        );
    }
}
