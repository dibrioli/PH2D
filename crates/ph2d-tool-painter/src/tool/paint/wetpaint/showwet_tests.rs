//! **O VÉU DO SHOW WET É VIVO** — o que ele mostra durante o traço é o que ele
//! mostraria se o artista o ligasse depois.
//!
//! ⚠️ **Report do Enio (2026-08-07):** *"Show Wet só aparece se a água for
//! colocada antes e depois se checar Show Wet. Mas se Show Wet estiver checado e
//! pintar com água pura (Pigment 0) não se vê a água."*
//!
//! **O ORÁCULO desta suíte é a diferença entre duas rotas**, nunca um número de
//! pixels escolhido a dedo: ligar o véu ANTES do traço tem de dar o MESMO canvas
//! que ligá-lo DEPOIS. Ligar depois passa por `wet_recomposite_full`, que
//! redesenha a folha inteira e não pode errar por retângulo — então ele é a
//! resposta de referência, e a rota viva é a que estava mentindo.
//!
//! Medido antes da correção (mesmo fixture): **Paint pigmento 0 divergia em
//! 2787 px** (o véu vivo mostrava 215 dos 3002 que o recompose mostra) e
//! **pigmento 600 divergia em 9 px**. A causa é uma só, e vive no motor —
//! `crates/ph2d-wet-paint/tests/the_accumulate_declares_what_it_wrote.rs`.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fixture() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
    let b = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_tool_mode("wetpaint");
    t
}

fn stroke(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    for k in 1..=20 {
        t.on_canvas_pointer(cp([30.0 + 6.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([150.0, 60.0], PointerPhase::Up));
}

fn click(t: &mut PainterTool, id: ph2d_a11y::NodeId) {
    assert!(
        t.route_brush_wetpaint_event(&PanelEvent::Click(id)),
        "o clique em {id:?} nao foi consumido pela rota wet"
    );
}

/// O que a seção Wet Paint oferece ao artista, nas três formas que molham papel.
#[derive(Clone, Copy)]
enum Media {
    /// A tinta comum (pigmento no default do produto).
    Paint,
    /// O pincel com o slider Pigment em ZERO — o caso do report.
    PaintNoPigment,
    /// A ferramenta Wet, cuja razão de existir é depositar água.
    WetTool,
}

impl Media {
    fn label(self) -> &'static str {
        match self {
            Media::Paint => "Paint",
            Media::PaintNoPigment => "Paint com Pigment 0",
            Media::WetTool => "ferramenta Wet",
        }
    }
}

/// Roda um traço e devolve o canvas final. `veil_first` liga o Show Wet ANTES do
/// traço (a rota VIVA); senão ele é ligado depois (a rota de REFERÊNCIA, que
/// recompõe a folha inteira).
fn painted(media: Media, veil_first: bool) -> Vec<u8> {
    let mut t = fixture();
    if veil_first {
        click(&mut t, core_ids::PAINTER_WETPAINT_SHOWWET);
    }
    match media {
        Media::PaintNoPigment => {
            assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(
                core_ids::PAINTER_WETPAINT_PIGMENT,
                0.0
            )));
        }
        Media::WetTool => click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[4]),
        Media::Paint => {}
    }
    stroke(&mut t);
    if !veil_first {
        click(&mut t, core_ids::PAINTER_WETPAINT_SHOWWET);
    }
    t.canvas_rgba.to_vec()
}

fn pixels_differing(a: &[u8], b: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0.iter())
        .filter(|(x, y)| x != y)
        .count()
}

/// **O véu vivo é o mesmo véu.** Para os três meios que molham papel, ligar o
/// Show Wet antes do traço tem de dar o canvas EXATO de ligá-lo depois.
///
/// **Mutação que sangra:** `Accumulated::wrote` devolvendo `None` (o defeito
/// original) — `Paint com Pigment 0` diverge em ~2787 px e `Paint` em ~9.
#[test]
fn the_live_veil_shows_what_a_full_recomposite_would_show() {
    for media in [Media::Paint, Media::PaintNoPigment, Media::WetTool] {
        let live = painted(media, true);
        let reference = painted(media, false);
        let n = pixels_differing(&live, &reference);
        assert_eq!(
            n,
            0,
            "o veu VIVO de {} discorda do recompose de folha inteira em {n} pixels",
            media.label()
        );
    }
}

/// **E o véu de fato DESENHA alguma coisa** — sem isto o gate acima seria verde
/// por vácuo (duas rotas que não pintam nada concordam perfeitamente).
///
/// O número não é um bar afinado: é a ordem de grandeza do traço, e existe só
/// para provar que a comparação acima não é entre dois nadas.
#[test]
fn the_veil_is_not_vacuously_equal_it_actually_draws() {
    for media in [Media::Paint, Media::PaintNoPigment, Media::WetTool] {
        let mut t = fixture();
        if let Media::PaintNoPigment = media {
            assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(
                core_ids::PAINTER_WETPAINT_PIGMENT,
                0.0
            )));
        }
        if let Media::WetTool = media {
            click(&mut t, core_ids::PAINTER_WETPAINT_TOOL_IDS[4]);
        }
        stroke(&mut t);
        let bare = t.canvas_rgba.to_vec();
        let veiled = painted(media, true);
        let n = pixels_differing(&bare, &veiled);
        assert!(
            n > 1000,
            "o veu de {} mudou so {n} pixels — ele nao esta sendo desenhado",
            media.label()
        );
    }
}
