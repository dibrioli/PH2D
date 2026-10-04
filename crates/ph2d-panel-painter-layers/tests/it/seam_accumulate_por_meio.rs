//! **O Accumulate e o Space Attenuation aparecem só onde o meio os oferece** (doc 46 §1).
//! A lei é a `BrushSettings::accumulate_offered`, a mesma que o traço pergunta; este gate prova que
//! a TELA a segue, meio a meio.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::ids::{PAINTER_BRUSH_ACCUMULATE, PAINTER_BRUSH_SPACE_ATTEN};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn pintados(media: PaintMedia) -> Vec<NodeId> {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    set_current_brush(Some(t.brush_settings()));
    set_current_dock_shows_layers(false);
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let vp = Rect::new(0.0, 0.0, 1600.0, 6000.0);
    let _ = host.paint::<PainterLayersPanel>(&mut st, vp);
    host.open_all_sections();
    host.paint::<PainterLayersPanel>(&mut st, vp)
        .into_iter()
        .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn o_accumulate_e_o_space_attenuation_so_onde_o_meio_os_oferece() {
    for (media, oferece) in [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, false),
        (PaintMedia::Impasto, false),
        (PaintMedia::WetPaint, false),
    ] {
        let ids = pintados(media);
        for (id, nome) in [
            (PAINTER_BRUSH_ACCUMULATE, "Accumulate"),
            (PAINTER_BRUSH_SPACE_ATTEN, "Space Attenuation"),
        ] {
            assert_eq!(
                ids.contains(&id),
                oferece,
                "{media:?}: o {nome} {} pintado",
                if oferece { "NÃO é" } else { "é" }
            );
        }
    }
}
