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
    pintados_com(media, |_| {})
}

fn pintados_com(media: PaintMedia, ajusta: impl Fn(&mut PainterTool)) -> Vec<NodeId> {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    ajusta(&mut t);
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

/// **Com o Solid o Accumulate e o Space Attenuation saem do painel** (decisão do dono, 2026-10-06): a
/// mancha obedece ao tecto do traço, e com o Accumulate ligado o traço passava dele (`0,744` contra a
/// mancha a `0,400`). No Digital, onde o meio os oferece; e voltam ao desmarcar o Solid.
#[test]
fn com_solid_o_accumulate_sai_do_painel() {
    for solid in [true, false] {
        let ids = pintados_com(PaintMedia::Digital, |t| {
            if solid {
                t.toggle_style_solid();
            }
        });
        for (id, nome) in [
            (PAINTER_BRUSH_ACCUMULATE, "Accumulate"),
            (PAINTER_BRUSH_SPACE_ATTEN, "Space Attenuation"),
        ] {
            assert_eq!(
                ids.contains(&id),
                !solid,
                "Digital com Solid {solid}: o {nome} {} pintado",
                if solid { "é" } else { "NÃO é" }
            );
        }
    }
}

/// **A Shape Color Ramp não aparece no Wet Paint** (doc 46 §1, recusada: a água leva UMA cor por
/// carimbo ao fluido, e a mistura K–M homogeneíza-o no primeiro passo — a rampa seria apagada pela
/// física). A lei é a `BrushSettings::shape_ramp_offered`.
#[test]
fn a_rampa_da_shape_so_onde_o_meio_a_oferece() {
    use ph2d_tool_painter::ids::{PAINTER_SHAPE_RAMP_ENABLE, PAINTER_SHAPE_RAMP_SECTION};
    for (media, oferece) in [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, true),
        (PaintMedia::Impasto, true),
        (PaintMedia::WetPaint, false),
    ] {
        let ids = pintados(media);
        for (id, nome) in [
            (PAINTER_SHAPE_RAMP_SECTION, "cabeçalho Shape Color"),
            (PAINTER_SHAPE_RAMP_ENABLE, "Use Color Ramp da Shape"),
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

/// **A Color Ramp do Grain não aparece no Wet Paint nem na Aquarela** — no Wet Paint pela mesma
/// física da da Shape (achada morta pela procura completa do censo, 2026-10-05: a secção inteira não
/// muda um texel); na Aquarela o Grain é o mapa de granulação. A lei é a
/// `BrushSettings::texture_ramp_offered`. Com um Grain escolhido (sem ele a rampa não se pinta).
#[test]
fn a_rampa_do_grain_so_onde_o_meio_a_oferece() {
    use ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE;
    for (media, oferece) in [
        (PaintMedia::Digital, true),
        (PaintMedia::Watercolor, false),
        (PaintMedia::Impasto, true),
        (PaintMedia::WetPaint, false),
    ] {
        let ids = pintados_com(media, |t| t.set_brush_texture_kind(1));
        assert_eq!(
            ids.contains(&PAINTER_BRUSH_TEXTURE_RAMP_ENABLE),
            oferece,
            "{media:?}: o Use Color Ramp do Grain {} pintado",
            if oferece { "NÃO é" } else { "é" }
        );
    }
}
